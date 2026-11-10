// M2-03d-11: raw clause inputs of the two DynamicRows gates of 5fb8ec at every evaluation (passes and failures), and their outcome
// through separate positive hooks (docs/m2/mip_dive_gate.md section 2). Read-only hooks. New events (all MIP depths):
//  g:526   at 0x510940 (C:526 react_all gate)  } fb (5fb8ec call serial), k (evaluation serial in that call), node, mip_depth, dyn,
//  g:1581  at 0x512468 (C:1581 selection gate) } I+0x2c, W+0x34, I+0x28, I+0xa0 / I+8 null, param_3+0x18, slot / rbx identity
//  g:hit   positive hooks: 510961 (C:526 true), 51305f, 513078, 51319d (a7f0ad eax), 5131af (a7f923 eax), 5131b7 (param_3+0x18, r15 = param_3), 5131c2
// Everything below the g:* additions is tools/mip_dive_rows.js (M2-03d-8) unchanged; its b:* / d:* projection must equal the M2-03d-8 capture.
// M2-03d-8: inputs and result of the DynamicRows dive row selection in 5fb8ec (C:1581-1856). Read-only hooks.
// New events (all MIP depths; docs/m2/mip_dive_rows.md section 2):
//  d:gate        at 0x512468 when the C:1581 gate predicts a pass (env+0x2b48 != 0, I+0x2c == param_2+0x34, I+0x28 == 2, I+0xa0, I+8)
//  d:gate_counts first failing gate clause per MIP depth (sent at the end of each top-level MIP)
//  d:model       first time a model row set (content hash of D row CSR, rhs, sense, vtype) is seen
//  d:sel         at 0x513951 (selection end): inputs (bounds as diffs to the previous event of the same model, flags, 7889a5 array,
//                cut attributes and slack, rc (always, also when DualReductions is 0), tolerances, cutoff) and the selected list (0x48(%rsp), count 0x30(%rsp)); 66661c calls
//  d:blocked     a 0x513078 pass that did not reach 0x513951 (a7f0ad / a7f923 / +0x18 clause)
// The M2-03d-7 hooks of tools/mip_blank_rows.js are kept unchanged below (b:* events; their projection must equal the M2-03d-7 capture):
// Mechanism (C, docs/m2/mip_blank_rows.md §2): 10e5fc/10e62f/10e662 flag rows in (lp+0x230)+0xb0 (0x1000 off / 0x2000 on),
// edd71 (model update) folds the flags into X = D+0x140 (+0 count, +0x08 list, +0x10 row->position or -1),
// cda5b C:120 10f448(D) swaps D+0x100/D+0x380 with X's reduced length / rhs arrays, C:2110 10f5ae(D) swaps them back.
// Events (top-level MIP, 5bbf7f depth 1, only; nested MIP ops are counted):
//  b:lpnew  first time an LP with an X is seen (lp id = order of appearance; no addresses)
//  b:op     every 10e5fc / 10e62f / 10e662 call: rows, return offset, last 5ce67b call/node, D rows, env+0x2b48
//  b:xlist  at edd71 return when the X list (count, rows) of an LP changed
//  b:t:*    inside the TARGET 5ce67b call: X at top-level cda5b entry, D hash before the swap, 10f448/10f5ae swap, ws at 02f7e1 return
// Usage: tools/gurun.sh node tools/mip_dive_gate.js MODEL OUTPUT.json.gz 'JSON_PARAMS' TARGET_CALL
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const zlib = require('zlib');
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
 Interceptor.attach(b.add(0x4e36e0), {onEnter() { mipDepth++; }, onLeave() { mipDepth--; if (mipDepth === 0) { send({event: 'b:nested_ops', counts: nested}); send({event: 'd:gate_counts', counts: gateCounts}); } }});
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
 // ==== M2-03d-8 hooks (read-only; they never call lpId or touch the M2-03d-7 state above) ====
 const gateCounts = {};
 const hexs = (p, n) => { if (p.isNull() || n <= 0) return ''; const dv = new DataView(p.readByteArray(8 * n)); let s = ''; for (let k = 0; k < n; k++) s += hx(dv, 8 * k); return s; };
 const i64s = (p, n) => { const r = []; for (let k = 0; k < n; k++) r.push(p.add(8 * k).readS64().toNumber()); return r; };
 const bytes = (p, n) => (p.isNull() || n <= 0) ? [] : Array.from(new Uint8Array(p.readByteArray(n)));
 const bits = p => hx(new DataView(p.readByteArray(8)), 0);
 const p2stack = [];   // param_2 of the open 5fb8ec calls
 // M2-03d-11: one frame per open 5fb8ec call (serial, param_1 node, param_2, param_3)
 const fbs = []; let fbSerial = 0;
 Interceptor.attach(b.add(0x5101d0), {onEnter(a) { p2stack.push(a[1]); fbs.push({fb: ++fbSerial, p1: a[0], node: a[0].readPointer(), p2: a[1], p3: a[2], k: 0}); },
   onLeave() { p2stack.pop(); fbs.pop(); }});
 const fbTop = () => fbs.length ? fbs[fbs.length - 1] : null;
 // raw clause values, read even when an earlier clause fails (I, W and param_3 stay valid for the whole 5fb8ec call)
 function gateRaw(kind, c) { const f = fbTop(), n = c.rsp.add(0x130).readPointer(), I = n.add(0x18).readPointer(), W = c.rbx;
   send({event: kind, fb: f ? f.fb : null, k: f ? ++f.k : null, mip_depth: mipDepth, call: nextCall, node: n.readDouble(),
     dyn: c.r12.add(0x2b48).readS32(), depth: I.add(0x2c).readS32(), w34: W.add(0x34).readS32(), state: I.add(0x28).readS32(),
     a0_null: I.add(0xa0).readPointer().isNull(), sol_null: I.add(8).readPointer().isNull(), p3_18: f ? f.p3.add(0x18).readS32() : null,
     // the slot is reloaded from *param_1 (C:1577), which 5fb8ec may replace during the call
     slot_is_node: !!(f && n.equals(f.p1.readPointer())), node_is_entry: !!(f && n.equals(f.node)), rbx_is_param2: !!(f && W.equals(f.p2))}); }
 // instruction probes (function form): the object form treats the address as a function entry and rewrites [rsp]
 // (its return-address slot); at 51319d..5131c2 that slot holds the mip pointer read at 5131ef (docs/lessons/tooling.md)
 const hit = (at, extra) => { const f = fbTop(); send(Object.assign({event: 'g:hit', at, fb: f ? f.fb : null}, extra || {})); };
 Interceptor.attach(b.add(0x510940), function () { gateRaw('g:526', this.context); });
 Interceptor.attach(b.add(0x510961), function () { hit('510961'); });
 Interceptor.attach(b.add(0x51305f), function () { hit('51305f'); });
 Interceptor.attach(b.add(0x51319d), function () { hit('51319d', {eax: this.context.rax.toInt32()}); });
 Interceptor.attach(b.add(0x5131af), function () { hit('5131af', {eax: this.context.rax.toInt32()}); });
 Interceptor.attach(b.add(0x5131b7), function () { const f = fbTop(), r15 = this.context.r15;
   hit('5131b7', {r15_18: r15.add(0x18).readS32(), r15_is_param3: !!(f && r15.equals(f.p3))}); });
 Interceptor.attach(b.add(0x5131c2), function () { hit('5131c2'); });
 // C:1581 gate; 511b22 / 511b93 jump straight here, so I is read through the 0x130(%rsp) slot exactly like 0x512473
 Interceptor.attach(b.add(0x512468), {onEnter() {
   gateRaw('g:1581', this.context);
   const c = this.context, env = c.r12, n = c.rsp.add(0x130).readPointer(), I = n.add(0x18).readPointer(), p2 = c.rbx;
   const dyn = env.add(0x2b48).readS32();
   let why = null, d = null, p34 = null;
   if (dyn === 0) why = 'dyn0';
   else { d = I.add(0x2c).readS32(); p34 = p2.add(0x34).readS32();
     why = d !== p34 ? 'depth' : I.add(0x28).readS32() !== 2 ? 'state' : I.add(0xa0).readPointer().isNull() ? 'a0' : I.add(8).readPointer().isNull() ? 'sol' : null; }
   const gc = gateCounts[mipDepth] || (gateCounts[mipDepth] = {}), k = why || 'pass';
   gc[k] = (gc[k] || 0) + 1;
   if (why === null) send({event: 'd:gate', mip_depth: mipDepth, call: nextCall, node: n.readDouble(), node_depth: d, p34, dyn,
     rbx_is_param2: p2stack.length > 0 && p2.equals(p2stack[p2stack.length - 1])}); }});
 const open = [];      // selections between 0x513078 and 0x513951, one per MIP nesting level
 Interceptor.attach(b.add(0x513078), {onEnter() {
   hit('513078');
   const n = this.context.rsp.add(0x140).readPointer().readPointer();
   while (open.length && open[open.length - 1].mip_depth >= mipDepth) { const o = open.pop(); send({event: 'd:blocked', mip_depth: o.mip_depth, call: o.call, node: o.node}); }
   open.push({mip_depth: mipDepth, call: nextCall, node: n.readDouble(), calls: []}); }});
 // 66661c: only caller is 5fb8ec 0x5133df; the caller's row index is in r15d at entry
 Interceptor.attach(b.add(0x554a30), {onEnter() { this.o = open.length ? open[open.length - 1] : null; this.row = this.context.r15.toInt32(); },
   onLeave(r) { if (this.o) this.o.calls.push([this.row, r.toInt32()]); else send({event: 'd:66661c_outside'}); }});
 const models = new Map(); let modelSerial = 0;
 Interceptor.attach(b.add(0x513951), {onEnter() {
   const c = this.context, sp = c.rsp, env = c.r12;
   const o = open.length && open[open.length - 1].mip_depth === mipDepth ? open.pop() : null;
   const n = sp.add(0x68).readPointer(), I = n.add(0x18).readPointer(), mip = I.readPointer(), lp = sp.add(0x28).readPointer();
   const mlp = mip.add(8).readPointer(), D = mlp.add(0xe8).readPointer(), sol = I.add(8).readPointer();
   const nrows = sp.add(0x78).readS32(), ncuts = sp.add(0x50).readS32(), ncols = D.add(0xc).readS32();
   const cutObj = mip.add(0x3240).readPointer();
   // stack slots against the pointers derived from I / mip / D (same loads as 0x513078..0x513185)
   const slots = {flags: [0x60, I.add(0xa0)], lb: [0xb8, I.add(0x80)], ub: [0x88, I.add(0x88)], slack: [0x58, sol.add(0x20)], rc: [0x90, sol.add(0x28)],
     vtype: [0x80, mip.add(0x650)], rbeg: [0xc8, D.add(0x118)], rend: [0xc0, D.add(0x120)], rind: [0xb0, D.add(0x130)], rval: [0xa0, D.add(0x138)],
     rhs: [0xa8, D.add(0x380)], sense: [0x98, D.add(0x388)]};
   const P = {}, bad = [];
   for (const k in slots) { const [so, q] = slots[k]; P[k] = q.readPointer(); if (!sp.add(so).readPointer().equals(P[k])) bad.push(k); }
   if (!sp.add(0x70).readPointer().equals(cutObj)) bad.push('cutobj');
   if (nrows !== D.add(8).readS32()) bad.push('nrows');
   if (ncuts !== cutObj.add(0x60).readS32()) bad.push('ncuts');
   // model rows, identified by content (heap addresses are reused)
   const beg = i64s(P.rbeg, nrows), end = i64s(P.rend, nrows), st = [0x811c9dc5, 0x12345678];
   H(st, P.rbeg, 8 * nrows); H(st, P.rend, 8 * nrows);
   for (let i = 0; i < nrows; i++) { H(st, P.rind.add(4 * beg[i]), 4 * (end[i] - beg[i])); H(st, P.rval.add(8 * beg[i]), 8 * (end[i] - beg[i])); }
   H(st, P.rhs, 8 * nrows); H(st, P.sense, nrows); H(st, P.vtype, ncols);
   const mkey = hs(st) + ':' + nrows + ':' + ncols;
   let m = models.get(mkey);
   if (!m) { m = {id: ++modelSerial, lb: null, ub: null}; models.set(mkey, m);
     const ind = []; let val = '';
     for (let i = 0; i < nrows; i++) { for (const x of ints(P.rind.add(4 * beg[i]), end[i] - beg[i])) ind.push(x); val += hexs(P.rval.add(8 * beg[i]), end[i] - beg[i]); }
     send({event: 'd:model', model: m.id, hash: mkey, mip_depth: mipDepth, call: nextCall, nrows, ncols, row_len: beg.map((x, i) => end[i] - x), ind, val,
       rhs: hexs(P.rhs, nrows), sense: String.fromCharCode.apply(null, bytes(P.sense, nrows)), vtype: String.fromCharCode.apply(null, bytes(P.vtype, ncols))}); }
   const diff = (prev, cur) => { const r = []; for (let j = 0; j < cur.length; j++) if (prev === null || prev[j] !== cur[j]) r.push([j, cur[j]]); return r; };
   const split = s => { const r = []; for (let k = 0; k < s.length; k += 16) r.push(s.substr(k, 16)); return r; };
   const lb = split(hexs(P.lb, ncols)), ub = split(hexs(P.ub, ncols));
   const lbd = diff(m.lb, lb), ubd = diff(m.ub, ub); m.lb = lb; m.ub = ub;
   const flags = []; ints(P.flags, nrows + ncuts).forEach((v, i) => { if (v !== 0) flags.push([i, v]); });
   const bobj = mip.add(0x32e8).readPointer(); let b7 = null;
   if (!bobj.isNull()) { const cnt = bobj.add(0xc).readS32(), nn = [];
     ints(bobj.add(0x10).readPointer(), Math.min(cnt, nrows)).forEach((v, i) => { if (v >= 0) nn.push([i, v]); }); b7 = {count: cnt, nonneg: nn}; }
   const cutType = [], cut40 = [];
   if (ncuts > 0) { const arr = cutObj.add(0x78).readPointer();
     for (let j = 0; j < ncuts; j++) { const q = arr.add(8 * j).readPointer(); cutType.push(q.add(0x2c).readS32()); if (!q.add(0x40).readPointer().isNull()) cut40.push(j); } }
   const ic = mip.add(0x620).readPointer().add(0x3220).readPointer();
   const dual = env.add(0x2430).readS32(), cutRaw = ic.isNull() ? null : bits(ic.add(0x40));
   const cnt = sp.add(0x30).readS64().toNumber(), list = ints(sp.add(0x48).readPointer(), cnt);
   send({event: 'd:sel', mip_depth: mipDepth, call: nextCall, node: n.readDouble(), node_depth: I.add(0x2c).readS32(), entered: !!o,
     calls_66661c: o ? o.calls : null, bad_slots: bad, model: m.id, mlp_is_node_lp: mlp.equals(lp), node_lp_rows: lp.add(0xe8).readPointer().add(8).readS32(),
     nrows, ncols, ncuts, dyn: env.add(0x2b48).readS32(), dual, feastol: bits(env.add(0x2460)),
     cutoff_raw: cutRaw, i10: bits(I.add(0x10)), cutoff_stack: nrows > 0 ? bits(sp.add(0x10)) : null,
     lb_diff: lbd, ub_diff: ubd, flags, b7889a5: b7, cut_type: cutType, cut_40: cut40, cut_slack: hexs(P.slack.add(8 * nrows), ncuts),
     rc: hexs(P.rc, ncols), count: cnt, list}); }});
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'dive_gate', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, target, output: outputText.trim(), errors, ok, detach_reason: reason, events}) + '\n'));
   console.log(path.basename(model), outputText.trim(), 'events', events.length, 'ok', ok, errors.join(' ')); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
