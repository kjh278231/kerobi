// M2-03d-7: where the empty rows of the B&B node LP simplex ws (M2-03d-6 C2) come from. Read-only hooks.
// For every top-level cda5b inside a depth-1 5ce67b solve after the first restart, at 02f7e1 return:
//  ws created here (lp+0xe0 null at entry), D (lp+0xe8) and ws dimensions/nnz, pointer aliases ws CSC/b <-> D fields,
//  08b2fe source field. A 'b:state' event is sent when that state changes; every call in the target window is sent.
// At the target solve (TARGET call) also the 03d-6 hashes (dhash, wsHash) and the owner scan of ws+0x78/ws+0xa0.
// Usage: tools/gurun.sh node tools/mip_blank_rows.js MODEL OUTPUT 'JSON_PARAMS' [TARGET_CALL] [WINDOW]
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [model, output, extra, targetArg, windowArg] = process.argv.slice(2);
if (!model || !output) throw Error('usage: MODEL OUTPUT [JSON_PARAMS] [TARGET_CALL] [WINDOW]');
if (fs.existsSync(output)) throw Error('output exists: ' + output);
const target = parseInt(targetArg || '19901'), win = parseInt(windowArg || '60');
const root = path.resolve(__dirname, '../..');
const py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const expected = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c';
if (hash(binary) !== expected) throw Error('wrong Gurobi binary');
const params = Object.assign({Threads: 1, Seed: 0, LogToConsole: 0}, JSON.parse(extra || '{}'));
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\nm.optimize()\nprint('DONE',m.Status,int(m.NodeCount),flush=True)`;
const agent = `
const TARGET = ${target}, WIN = ${win};
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
 // ---- new ----
 const sumLen = (len, n) => { if (len.isNull() || n <= 0) return null; const a = new Int32Array(len.readByteArray(4 * n)); let s = 0; for (const x of a) s += x; return s; };
 function fnvBytes(p, bytes) { if (p.isNull() || bytes <= 0) return null; const u = new Uint8Array(p.readByteArray(bytes)); let h = 0x811c9dc5;
   for (const x of u) h = Math.imul(h ^ x, 0x01000193) >>> 0; return h.toString(16).padStart(8, '0'); }
 // per-row entry counts of a CSC view (beg, len, ind) with n columns and m rows
 function rowCounts(beg, len, ind, n, m) { const c = new Int32Array(m), L = new Int32Array(len.readByteArray(4 * n));
   for (let j = 0; j < n; j++) { if (L[j] <= 0) continue; const s = beg.add(8 * j).readS64().toNumber(), I = new Int32Array(ind.add(4 * s).readByteArray(4 * L[j]));
     for (const i of I) if (i >= 0 && i < m) c[i]++; } return c; }
 // name a pointer by the D / lp / ws slots that hold it (first 0x480 bytes of D, 0x300 of lp, 0x600 of ws)
 function owners(p, lp) { if (p.isNull()) return null; const r = [], D = lp.add(0xe8).readPointer(), ws = lp.add(0xe0).readPointer();
   for (const [name, base, size] of [['D', D, 0x480], ['lp', lp, 0x300], ['ws', ws, 0x600]]) { if (base.isNull()) continue;
     for (let o = 0; o < size; o += 8) if (base.add(o).readPointer().equals(p)) r.push(name + '+0x' + o.toString(16)); }
   // inside D arrays (pointer into the middle of a D block) is reported as an offset from the D field
   return r; }
 const mains = []; let mipDepth = 0;
 Interceptor.attach(b.add(0x4e36e0), {onEnter() { mains.push(++mipDepth); }, onLeave() { mains.pop(); mipDepth--; }});
 const solves = []; let nextCall = 0, lastKey = null, lastDiv = null;
 const top = () => solves.length ? solves[solves.length - 1] : null;
 Interceptor.attach(b.add(0x4f0e10), {
  onEnter(a) { const n = a[0].readPointer(), I = n.add(0x18).readPointer(), mm = I.readPointer(), r = mm.add(0x620).readPointer();
   solves.push({call: ++nextCall, lp: a[2].add(8).readPointer(), node: n.readDouble(), depth: mipDepth, restarts: r.add(0x808).readS32(),
     self: r.equals(mm), cda: 0, k: 0}); },
  onLeave() { solves.pop(); }});
 const on = s => s && s.depth === 1 && s.restarts >= 1 && s.cda === 1;
 Interceptor.attach(b.add(0x153cc0), {onEnter(a) { const s = top(); this.s = s; if (s) { s.cda++; if (s.cda === 1) s.k++; } },
  onLeave() { if (this.s) this.s.cda--; }});
 let src = null;
 Interceptor.attach(b.add(0x11f530), {onLeave(r) { const s = top(); if (!on(s) || !s.in2f) return;
   const lp = s.lp2f; src = r.isNull() ? 'null' : r.equals(lp.add(0xf8).readPointer()) ? 'lp+0xf8' : r.equals(lp.add(0xf0).readPointer()) ? 'lp+0xf0' : r.equals(lp.add(0xe8).readPointer()) ? 'lp+0xe8' : 'other'; }});
 Interceptor.attach(b.add(0xd06f0), {
  onEnter(a) { const s = top(); this.on = on(s); if (!this.on) return; this.lp = a[0]; s.in2f = true; s.lp2f = a[0]; src = null;
   this.wsNew = a[0].add(0xe0).readPointer().isNull(); this.ws0 = a[0].add(0xe0).readPointer(); },
  onLeave(r) { if (!this.on) return; const s = top(); s.in2f = false; const lp = this.lp, D = lp.add(0xe8).readPointer(), ws = lp.add(0xe0).readPointer();
   const m = D.add(8).readS32(), n = D.add(0xc).readS32(), Dlen = D.add(0x100).readPointer();
   const st = {lp_is_node_lp: lp.equals(s.lp), ws_new: this.wsNew, ws_same: !this.wsNew && ws.equals(this.ws0), src, lpf8_null: lp.add(0xf8).readPointer().isNull(),
     lpf0_null: lp.add(0xf0).readPointer().isNull(), D_rows: m, D_cols: n, D_nnz: sumLen(Dlen, n), D_len_h: fnvBytes(Dlen, 4 * n)};
   if (!ws.isNull()) { const wm = ws.add(0x64).readS32(), wn = ws.add(0x68).readS32(), wl = ws.add(0x78).readPointer();
     Object.assign(st, {ws_rows: wm, ws_cols: wn, ws_nnz: sumLen(wl, wn), ws_len_h: fnvBytes(wl, 4 * wn),
       alias: [[0x70, 0xf8], [0x78, 0x100], [0x80, 0x108], [0x88, 0x110], [0xa0, 0x380]].map(([x, y]) => ws.add(x).readPointer().equals(D.add(y).readPointer()) ? 1 : 0).join('')}); }
   const div = st.ws_nnz !== undefined && st.ws_nnz !== st.D_nnz;
   const key = JSON.stringify(st), inWin = s.call > TARGET - WIN && s.call <= TARGET;
   if (key !== lastKey || div !== lastDiv || inWin || st.ws_new) {
     const e = Object.assign({event: 'b:state', call: s.call, k: s.k, node: s.node, rc: r.toInt32(), changed: key !== lastKey}, st);
     if ((div || s.call === TARGET) && !ws.isNull()) {
       const wm = ws.add(0x64).readS32(), wn = ws.add(0x68).readS32();
       const cD = rowCounts(D.add(0xf8).readPointer(), Dlen, D.add(0x108).readPointer(), n, m), cW = rowCounts(ws.add(0x70).readPointer(), ws.add(0x78).readPointer(), ws.add(0x80).readPointer(), wn, wm);
       const blank = []; for (let i = 0; i < Math.min(m, wm); i++) if (cW[i] === 0 && cD[i] > 0) blank.push(i);
       const bW = new Float64Array(ws.add(0xa0).readPointer().readByteArray(8 * wm)), bD = new Float64Array(D.add(0x380).readPointer().readByteArray(8 * m));
       let bdiff = 0; for (let i = 0; i < Math.min(m, wm); i++) if (bW[i] !== bD[i]) bdiff++;
       Object.assign(e, {blank_n: blank.length, blank_h: fnvBytes(Memory.alloc(4 * Math.max(1, blank.length)).writeByteArray(new Int32Array(blank).buffer), 4 * blank.length),
         b_diff: bdiff, len_owner: owners(ws.add(0x78).readPointer(), lp), b_owner: owners(ws.add(0xa0).readPointer(), lp), beg_owner: owners(ws.add(0x70).readPointer(), lp)});
       if (s.call === TARGET) Object.assign(e, {blank, dhash: dhash(lp), ws_hash: wsHash(lp)}); }
     send(e); lastKey = key; lastDiv = div; } }});
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
   fs.writeFileSync(output, JSON.stringify({schema: 1, kind: 'blank_rows_probe', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, target, window: win, output: outputText.trim(), errors, ok, detach_reason: reason, events}) + '\n');
   console.log(path.basename(model), outputText.trim(), 'events', events.length, 'ok', ok, errors.join(' ')); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
