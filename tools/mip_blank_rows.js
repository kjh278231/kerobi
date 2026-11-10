// M2-03d-7: provenance of the empty rows of the B&B node LP simplex ws (M2-03d-6 C2). Read-only hooks.
// Mechanism (C, docs/m2/mip_blank_rows.md §2): 10e5fc/10e62f/10e662 flag rows in (lp+0x230)+0xb0 (0x1000 off / 0x2000 on),
// edd71 (model update) folds the flags into X = D+0x140 (+0 count, +0x08 list, +0x10 row->position or -1),
// cda5b C:120 10f448(D) swaps D+0x100/D+0x380 with X's reduced length / rhs arrays, C:2110 10f5ae(D) swaps them back.
// Events (top-level MIP, 5bbf7f depth 1, only; nested MIP ops are counted):
//  b:lpnew  first time an LP with an X is seen (lp id = order of appearance; no addresses)
//  b:op     every 10e5fc / 10e62f / 10e662 call: rows, return offset, last 5ce67b call/node, D rows, env+0x2b48
//  b:xlist  at edd71 return when the X list (count, rows) of an LP changed
//  b:t:*    inside the TARGET 5ce67b call: X at top-level cda5b entry, D hash before the swap, 10f448/10f5ae swap, ws at 02f7e1 return
// Usage: tools/gurun.sh node tools/mip_blank_rows.js MODEL OUTPUT 'JSON_PARAMS' TARGET_CALL
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [model, output, extra, targetArg] = process.argv.slice(2);
if (!model || !output || !targetArg) throw Error('usage: MODEL OUTPUT JSON_PARAMS TARGET_CALL');
if (fs.existsSync(output)) throw Error('output exists: ' + output);
const target = parseInt(targetArg);
const root = path.resolve(__dirname, '../..');
const py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const expected = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c';
if (hash(binary) !== expected) throw Error('wrong Gurobi binary');
const params = Object.assign({Threads: 1, Seed: 0, LogToConsole: 0}, JSON.parse(extra || '{}'));
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\nm.optimize()\nprint('DONE',m.Status,int(m.NodeCount),flush=True)`;
const agent = `
const TARGET = ${target};
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return;
 const b = m.base, off = p => p.sub(b).toUInt32();
 // ---- 03d-6 hash helpers, unchanged (tools/mip_retry_node_lp.js) ----
 function H(st, p, bytes) { if (p.isNull()) { st[0] = Math.imul(st[0] ^ 0xdead, 0x01000193) >>> 0; return; } if (bytes === 0) return;
   const buf = p.readByteArray(bytes), w = bytes >> 2, u32 = new Uint32Array(buf, 0, w), u8 = new Uint8Array(buf);
   let a = st[0], c = st[1];
   for (let k = 0; k < w; k++) { a = Math.imul(a ^ u32[k], 0x01000193) >>> 0; c = Math.imul(c ^ u32[k], 0x5bd1e995) >>> 0; c ^= c >>> 15; }
   for (let k = w * 4; k < bytes; k++) { a = Math.imul(a ^ u8[k], 0x01000193) >>> 0; c = Math.imul(c ^ u8[k], 0x5bd1e995) >>> 0; }
   st[0] = a; st[1] = c; }
 const hs = st => st[0].toString(16).padStart(8, '0') + st[1].toString(16).padStart(8, '0');
 const nnzEnd = (beg, len, n) => { let e = 0; for (let j = 0; j < n; j++) e = Math.max(e, beg.add(8 * j).readS64().toNumber() + len.add(4 * j).readS32()); return e; };
 function dhash(lp) { const D = lp.add(0xe8).readPointer(), m = D.add(8).readS32(), n = D.add(0xc).readS32(), st = [0x811c9dc5, 0x12345678];
   const beg = D.add(0xf8).readPointer(), len = D.add(0x100).readPointer(), e = nnzEnd(beg, len, n);
   H(st, beg, 8 * n); H(st, len, 4 * n); H(st, D.add(0x108).readPointer(), 4 * e); H(st, D.add(0x110).readPointer(), 8 * e);
   for (const o of [0x50, 0x390, 0x398]) H(st, D.add(o).readPointer(), 8 * n);
   H(st, D.add(0x380).readPointer(), 8 * m); H(st, D.add(0x388).readPointer(), m); H(st, D.add(0x60), 8);
   return {rows: m, cols: n, nnz_end: e, hash: hs(st)}; }
 function wsArrays(ws) { const m = ws.add(0x64).readS32(), n = ws.add(0x68).readS32(), beg = ws.add(0x70).readPointer(), len = ws.add(0x78).readPointer();
   return {m, n, beg, len, e: nnzEnd(beg, len, n), ind: ws.add(0x80).readPointer(), val: ws.add(0x88).readPointer()}; }
 function wsHash(lp) { const ws = lp.add(0xe0).readPointer(), w = wsArrays(ws), st = [0x811c9dc5, 0x12345678];
   H(st, w.beg, 8 * w.n); H(st, w.len, 4 * w.n); H(st, w.ind, 4 * w.e); H(st, w.val, 8 * w.e); H(st, ws.add(0x90).readPointer(), w.m);
   for (const o of [0x98, 0xa8, 0xb0]) H(st, ws.add(o).readPointer(), 8 * (w.n + w.m));
   H(st, ws.add(0xa0).readPointer(), 8 * w.m); H(st, ws.add(0x3b8), 16);
   return {rows: w.m, cols: w.n, hash: hs(st)}; }
 // ---- helpers ----
 const ints = (p, n) => (p.isNull() || n <= 0) ? [] : Array.from(new Int32Array(p.readByteArray(4 * n)));
 const sum = a => a.reduce((s, x) => s + x, 0);
 const hx = (dv, k) => dv.getUint32(k + 4, true).toString(16).padStart(8, '0') + dv.getUint32(k, true).toString(16).padStart(8, '0');
 function dbls(p, n) { if (p.isNull() || n <= 0) return []; const dv = new DataView(p.readByteArray(8 * n)), v = []; for (let k = 0; k < n; k++) v.push(hx(dv, 8 * k)); return v; }
 const Dof = lp => lp.add(0xe8).readPointer();
 const Xof = lp => { const D = Dof(lp); return D.isNull() ? NULL : D.add(0x140).readPointer(); };
 function xlist(lp) { const X = Xof(lp); if (X.isNull()) return null; const n = X.readS32(); return {count: n, rows: ints(X.add(8).readPointer(), n)}; }
 // ---- context: MIP nesting, 5ce67b call counter (same numbering as tools/mip_retry_node_lp.js before its retry), last node ----
 let mipDepth = 0;
 const nested = {};
 // one listener per address (docs/lessons/tooling.md); the end-of-run summary of nested-MIP ops (not replayed) is sent here
 Interceptor.attach(b.add(0x4e36e0), {onEnter() { mipDepth++; }, onLeave() { mipDepth--; if (mipDepth === 0) send({event: 'b:nested_ops', counts: nested}); }});
 const solves = []; let nextCall = 0, last = null;
 const top = () => solves.length ? solves[solves.length - 1] : null;
 Interceptor.attach(b.add(0x4f0e10), {
  onEnter(a) { const n = a[0].readPointer(), I = n.add(0x18).readPointer(), mm = I.readPointer();
   const s = {call: ++nextCall, lp: a[2].add(8).readPointer(), node: n.readDouble(), node_depth: I.add(0x2c).readS32(), depth: mipDepth,
     restarts: mm.add(0x620).readPointer().add(0x808).readS32(), cda: 0, k: 0};
   solves.push(s); if (mipDepth === 1) last = s; },
  onLeave() { solves.pop(); }});
 // ---- LP identity (no addresses): new id when an LP pointer is first seen with an X, or its X pointer changes ----
 const lps = new Map(); let lpSerial = 0;
 function lpId(lp) { const X = Xof(lp), key = lp.toString(), r = lps.get(key);
   if (r && (X.equals(r.x) || r.x.isNull())) { r.x = X; return r.id; }
   const id = ++lpSerial; lps.set(key, {id, x: X, last: null});
   const D = Dof(lp);
   send({event: 'b:lpnew', lp: id, call: nextCall, mip_depth: mipDepth, x_null: X.isNull(), D_rows: D.add(8).readS32(), D_cols: D.add(0xc).readS32(),
     is_node_lp: !!(last && last.lp.equals(lp))}); return id; }
 const ctx = () => last ? {call: last.call, node: last.node, node_depth: last.node_depth, restarts: last.restarts} : {call: nextCall};
 for (const [name, o, hasRows] of [['deact', 0x1a2c20, true], ['react', 0x1a2df0, true], ['react_all', 0x1a2fc0, false]]) {
  Interceptor.attach(b.add(o), {
   onEnter(a) { this.on = mipDepth === 1; if (!this.on) { nested[name] = (nested[name] || 0) + 1; return; }
    this.lp = a[0]; const D = Dof(a[0]), xl = xlist(a[0]);
    this.e = Object.assign({event: 'b:op', op: name, ret: off(this.returnAddress), lp: lpId(a[0])}, ctx(), {
      D_rows: D.add(8).readS32(), dynamic_rows: a[0].add(0x100).readPointer().add(0x2b48).readS32(),
      rows: hasRows ? ints(a[2], a[1].toInt32()) : null, x_count_before: xl ? xl.count : null}); send(this.e); },
   // sent at entry: react_all runs edd71 itself, so its b:xlist follows this event
   onLeave(r) { if (this.on && r.toInt32() !== 0) send({event: 'b:op_fail', op: name, rc: r.toInt32()}); }});
 }
 Interceptor.attach(b.add(0x184e60), {onEnter(a) { this.on = mipDepth === 1; this.lp = a[0]; },
  onLeave(r) { if (!this.on || Xof(this.lp).isNull()) return; const id = lpId(this.lp), rec = lps.get(this.lp.toString()), xl = xlist(this.lp);
   const D = Dof(this.lp), key = D.add(8).readS32() + ':' + xl.count + ':' + xl.rows.join(',');
   if (key === rec.last) return; rec.last = key;
   send(Object.assign({event: 'b:xlist', lp: id, rc: r.toInt32()}, ctx(), {D_rows: D.add(8).readS32(), count: xl.count, rows: xl.rows})); }});
 // ---- target solve ----
 const inT = () => { const s = top(); return s && s.call === TARGET ? s : null; };
 Interceptor.attach(b.add(0x153cc0), {
  onEnter(a) { const s = inT(); this.s = s; if (!s) return; s.cda++; if (s.cda !== 1) return; s.k++;
   const lp = a[0], D = Dof(lp), X = Xof(lp), m = D.add(8).readS32(), xl = xlist(lp);
   const e = {event: 'b:t:cda5b', k: s.k, lp: lpId(lp), is_node_lp: lp.equals(s.lp), node: s.node, method: a[1].toInt32(), dhash: dhash(lp), x_null: X.isNull(),
     dynamic_rows: lp.add(0x100).readPointer().add(0x2b48).readS32()};
   if (!X.isNull()) { const pos = ints(X.add(0x10).readPointer(), m), marked = [];
     for (let i = 0; i < m; i++) if (pos[i] >= 0) marked.push([i, pos[i]]);
     Object.assign(e, {x_count: xl.count, x_rows: xl.rows, x_marked: marked, x_built: X.add(0x18).readS32(), x_ref: X.add(0x1c).readS32()}); }
   send(e); },
  onLeave() { if (this.s) this.s.cda--; }});
 const swap = (name, o) => Interceptor.attach(b.add(o), {
  onEnter(a) { const s = inT(); this.on = !!(s && s.cda === 1); if (!this.on) return; this.D = a[0]; const X = a[0].add(0x140).readPointer(); this.X = X;
   this.before = {len: a[0].add(0x100).readPointer(), rhs: a[0].add(0x380).readPointer(), xlen: X.isNull() ? NULL : X.add(0x28).readPointer(), xrhs: X.isNull() ? NULL : X.add(0x20).readPointer()};
   this.e = {event: 'b:t:' + name, k: s.k, x_null: X.isNull(), built_before: X.isNull() ? null : X.add(0x18).readS32(), ref_before: X.isNull() ? null : X.add(0x1c).readS32()}; },
  onLeave() { if (!this.on) return; const D = this.D, X = this.X, bf = this.before, n = D.add(0xc).readS32(), m = D.add(8).readS32();
   if (!X.isNull()) { const len = D.add(0x100).readPointer(), rhs = D.add(0x380).readPointer();
     Object.assign(this.e, {built_after: X.add(0x18).readS32(), ref_after: X.add(0x1c).readS32(),
       D_len_was_xlen: len.equals(bf.xlen), D_rhs_was_xrhs: rhs.equals(bf.xrhs), xlen_was_D_len: X.add(0x28).readPointer().equals(bf.len), xrhs_was_D_rhs: X.add(0x20).readPointer().equals(bf.rhs),
       D_len_unchanged: len.equals(bf.len), D_nnz: sum(ints(len, n)), was_nnz: sum(ints(bf.len, n)),
       // rows whose rhs bits differ between the two arrays, and the values now in D at those rows
       rhs_diff: (() => { const A = new Uint32Array(rhs.readByteArray(8 * m)), B = new Uint32Array(bf.rhs.readByteArray(8 * m)), z = [];
         for (let i = 0; i < m; i++) if (A[2 * i] !== B[2 * i] || A[2 * i + 1] !== B[2 * i + 1]) z.push(i); return z; })(),
       rhs_now: null}); this.e.rhs_now = dbls(rhs.add(0), m).filter((v, i) => this.e.rhs_diff.includes(i)); }
   send(this.e); }});
 swap('swap_in', 0x1a31c0); swap('swap_out', 0x1a33d0);
 Interceptor.attach(b.add(0xd06f0), {onEnter(a) { const s = inT(); this.on = !!(s && s.cda === 1); this.lp = a[0]; },
  onLeave(r) { if (!this.on) return; const s = inT(), lp = this.lp, ws = lp.add(0xe0).readPointer(), D = Dof(lp);
   const w = ws.isNull() ? null : wsArrays(ws);
   send({event: 'b:t:2f7e1', k: s.k, rc: r.toInt32(), ws_null: ws.isNull(), ws_hash: ws.isNull() ? null : wsHash(lp), ws_nnz: w ? sum(ints(w.len, w.n)) : null,
     ws_len_is_D_len: w ? w.len.equals(D.add(0x100).readPointer()) : null, ws_b_is_D_rhs: ws.isNull() ? null : ws.add(0xa0).readPointer().equals(D.add(0x380).readPointer())}); }});
}});
`;
(async () => {
 fs.mkdirSync(path.dirname(path.resolve(output)), {recursive: true});
 const device = await frida.getLocalDevice(); const events = []; let outputText = ''; const errors = [];
 const pid = await device.spawn([py, '-c', code], {stdio: 'pipe'}); const session = await device.attach(pid);
 const script = await session.createScript(agent);
 script.message.connect(msg => { if (msg.type !== 'send') { errors.push(msg.stack || JSON.stringify(msg)); return; } events.push(msg.payload); });
 device.output.connect((p, fd, data) => { if (p === pid) outputText += data.toString(); });
 session.detached.connect(reason => {
   const ok = outputText.includes('DONE') && !errors.length;
   fs.writeFileSync(output, JSON.stringify({schema: 1, kind: 'blank_rows', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, target, output: outputText.trim(), errors, ok, detach_reason: reason, events}) + '\n');
   console.log(path.basename(model), outputText.trim(), 'events', events.length, 'ok', ok, errors.join(' ')); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
