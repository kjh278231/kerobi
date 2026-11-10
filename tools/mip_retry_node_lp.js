// M2-03d-3: the node LP of the natural numeric retry in 5ce67b. Copy of tools/mip_retry_basis.js natural mode
// (its events are kept unchanged so they can be compared with the M2-03c-2 captures) plus 'y:' events:
//  y:dims   at every 5ce67b entry, when (MIP struct, LP rows, LP cols) changes
//  y:dhash  at every top-level cda5b entry of a solve: hash of the raw model data D (memory reads only)
//  y:lp     at the first cda5b on the node LP after the retry decision: the node LP through the public API (instrumented: GRBcheckmodel clears the env error buffer)
//  y:ws     at 02f7e1 return inside the retry cda5b: the simplex work arrays; y:ws_hash at the first cda5b
//  y:ws_leave, y:where (backtrace, MIP identity) at the retry
// Doubles are stored as 16 hex digits of their u64 bit pattern. After optimize() the python side dumps the model.
// Usage: tools/gurun.sh node tools/mip_retry_node_lp.js MODEL OUTPUT 'JSON_PARAMS'
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const mode = 'natural';
const [model, output, extra] = process.argv.slice(2);
if (!model || !output) throw Error('usage: MODEL OUTPUT [JSON_PARAMS]');
const root = path.resolve(__dirname, '../..');
const ptab = path.join(root, 're/notes/env_param_offsets.tsv');
const envParams = fs.readFileSync(ptab, 'utf8').trim().split('\n').slice(1).map(l => l.split('\t'))
  .filter(r => r[0] === 'int' || r[0] === 'double').map(r => [r[0], r[1], parseInt(r[2], 16)]);
envParams.push(['double', 'env+0x24b8', 0x24b8]);
const modelDump = output + '.model.tmp.json';
const py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const expected = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c';
if (hash(binary) !== expected) throw Error('wrong Gurobi binary');
const params = Object.assign({Threads: 1, Seed: 0, LogToConsole: 0}, JSON.parse(extra || '{}'));
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\nm.optimize()\nprint('DONE',m.Status,int(m.NodeCount),flush=True)
import json, struct
h = lambda x: '%016x' % struct.unpack('<Q', struct.pack('<d', x))[0]
vs, cs = m.getVars(), m.getConstrs()
rows = []
for c in cs:
    r = m.getRow(c); rows.append([[r.getVar(k).index for k in range(r.size())], [h(r.getCoeff(k)) for k in range(r.size())]])
json.dump({'cols': len(vs), 'rows': len(cs), 'model_sense': m.ModelSense, 'obj_con': h(m.ObjCon),
  'obj': [h(v.Obj) for v in vs], 'lb': [h(v.LB) for v in vs], 'ub': [h(v.UB) for v in vs], 'vtype': ''.join(v.VType for v in vs),
  'rhs': [h(c.RHS) for c in cs], 'sense': ''.join(c.Sense for c in cs), 'row_major': rows}, open(${JSON.stringify(path.resolve(modelDump))}, 'w'))`;
const agent = `
const mode = ${JSON.stringify(mode)};
const ENV_PARAMS = ${JSON.stringify(envParams)};
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return;
 const b = m.base, off = p => p.sub(b).toUInt32();
 // ---- 03c-1 helpers, unchanged (the projection compares their output byte for byte) ----
 function node(n) {
  if (n.isNull()) return null;
  const i = n.add(0x18).readPointer(), mm = i.readPointer(), p = n.add(0x10).readPointer();
  return {id: n.readDouble(), parent: p.isNull() ? null : p.readDouble(), depth: i.add(0x2c).readS32(),
    root: n.equals(mm.add(0x790).readPointer()), status: i.add(0x28).readS32(), cache: mm.add(0xa8).readS32(), mip: mm.readS32(),
    counter_d4: mm.add(0x620).readPointer().add(0xd4).readS32()};
 }
 function ints(p, n) { if (p.isNull()) return null; const v = []; for (let k = 0; k < n; k++) v.push(p.add(4 * k).readS32()); return v; }
 function lpstate(lp) {
  if (lp.isNull()) return null;
  const env = lp.add(0x100).readPointer(), ws = lp.add(0xe0).readPointer();
  let vstat = null, rows = null, cols = null;
  if (!ws.isNull()) { rows = ws.add(0x64).readS32(); cols = ws.add(0x68).readS32();
    if (rows < 0 || cols < 0 || rows + cols > 200000) throw Error('basis dimensions'); vstat = ints(ws.add(0x168).readPointer(), rows + cols); }
  return {status: lp.add(0x64).readS32(), presolve: env.add(0x2424).readS32(), method: env.add(0x2438).readS32(),
    concurrent_method: env.add(0x233c).readS32(), threads: env.add(0x2708).readS32(), ws_rows: rows, ws_cols: cols, vstat};
 }
 // ---- new helpers ----
 function fnv(v) { let h = 0x811c9dc5; for (const x of v) { h ^= x & 0xff; h = Math.imul(h, 0x1000193); h ^= (x >>> 8) & 0xff; h = Math.imul(h, 0x1000193);
   h ^= (x >>> 16) & 0xff; h = Math.imul(h, 0x1000193); h ^= (x >>> 24) & 0xff; h = Math.imul(h, 0x1000193); } return (h >>> 0).toString(16); }
 const dims = lp => { const d = lp.add(0xe8).readPointer(); return {rows: d.add(8).readS32(), cols: d.add(0xc).readS32()}; };
 // 69d7ea/69d237: columns come from the MIP's LP model M+8, rows stored at I+0x94
 const mipCols = I => I.readPointer().add(8).readPointer().add(0xe8).readPointer().add(0xc).readS32();
 const packedWords = I => (I.add(0x94).readS32() + 2 * mipCols(I) + 31) >> 5;
 function nodeBasis(I) { return {I94: I.add(0x94).readS32(), vb_null: I.add(0x98).readPointer().isNull(), cb_null: I.add(0xa0).readPointer().isNull(),
   dual_null: I.add(0xa8).readPointer().isNull(), packed_null: I.add(0xb0).readPointer().isNull()}; }
 const rel = (p, I) => p.isNull() ? 'null' : p.equals(I.add(0x98).readPointer()) ? 'I98' : p.equals(I.add(0xa0).readPointer()) ? 'Ia0' : 'other';
 function wsinfo(lp) { const ws = lp.add(0xe0).readPointer(); if (ws.isNull()) return {ws: false, lp_status: lp.add(0x64).readS32(), lp238_null: lp.add(0x238).readPointer().isNull()};
   return {ws: true, lp_status: lp.add(0x64).readS32(), ws_f0: ws.add(0xf0).readS32(), ws_1d8: ws.add(0x1d8).readS32(), lp238_null: lp.add(0x238).readPointer().isNull()}; }
 // ---- M2-03d-3 y: helpers ----
 const hx = (dv, k) => dv.getUint32(k + 4, true).toString(16).padStart(8, '0') + dv.getUint32(k, true).toString(16).padStart(8, '0');
 function dbls(p, n) { if (p.isNull()) return null; if (n === 0) return []; const dv = new DataView(p.readByteArray(8 * n)), v = [];
   for (let k = 0; k < n; k++) v.push(hx(dv, 8 * k)); return v; }
 const dbl1 = p => dbls(p, 1)[0];
 function chars(p, n) { if (p.isNull()) return null; let s = ''; for (let k = 0; k < n; k++) s += String.fromCharCode(p.add(k).readU8()); return s; }
 function longs(p, n) { if (p.isNull()) return null; const v = []; for (let k = 0; k < n; k++) v.push(p.add(8 * k).readS64().toNumber()); return v; }
 // two 32-bit lanes over the raw bytes; equality evidence only
 function H(st, p, bytes) { if (p.isNull()) { st[0] = Math.imul(st[0] ^ 0xdead, 0x01000193) >>> 0; return; } if (bytes === 0) return;
   const buf = p.readByteArray(bytes), w = bytes >> 2, u32 = new Uint32Array(buf, 0, w), u8 = new Uint8Array(buf);
   let a = st[0], c = st[1];
   for (let k = 0; k < w; k++) { a = Math.imul(a ^ u32[k], 0x01000193) >>> 0; c = Math.imul(c ^ u32[k], 0x5bd1e995) >>> 0; c ^= c >>> 15; }
   for (let k = w * 4; k < bytes; k++) { a = Math.imul(a ^ u8[k], 0x01000193) >>> 0; c = Math.imul(c ^ u8[k], 0x5bd1e995) >>> 0; }
   st[0] = a; st[1] = c; }
 const hs = st => st[0].toString(16).padStart(8, '0') + st[1].toString(16).padStart(8, '0');
 const nnzEnd = (beg, len, n) => { let e = 0; for (let j = 0; j < n; j++) e = Math.max(e, beg.add(8 * j).readS64().toNumber() + len.add(4 * j).readS32()); return e; };
 // raw model data D = lp+0xe8 (GRBXgetvars layout): CSC +0xf8 long beg, +0x100 int len, +0x108 ind, +0x110 val
 function dhash(lp) { const D = lp.add(0xe8).readPointer(), m = D.add(8).readS32(), n = D.add(0xc).readS32(), st = [0x811c9dc5, 0x12345678];
   const beg = D.add(0xf8).readPointer(), len = D.add(0x100).readPointer(), e = nnzEnd(beg, len, n);
   H(st, beg, 8 * n); H(st, len, 4 * n); H(st, D.add(0x108).readPointer(), 4 * e); H(st, D.add(0x110).readPointer(), 8 * e);
   for (const o of [0x50, 0x390, 0x398]) H(st, D.add(o).readPointer(), 8 * n);
   H(st, D.add(0x380).readPointer(), 8 * m); H(st, D.add(0x388).readPointer(), m); H(st, D.add(0x60), 8);
   return {rows: m, cols: n, nnz_end: e, hash: hs(st), D3c: D.add(0x3c).readS32(),
     null_D3d8: D.add(0x3d8).readPointer().isNull(), null_D3e0: D.add(0x3e0).readPointer().isNull(), null_D408: D.add(0x408).readPointer().isNull()}; }
 // simplex work struct ws = lp+0xe0 (re/notes/13_simplex.md 4.1)
 function wsArrays(ws) { const m = ws.add(0x64).readS32(), n = ws.add(0x68).readS32(), beg = ws.add(0x70).readPointer(), len = ws.add(0x78).readPointer();
   return {m, n, beg, len, e: nnzEnd(beg, len, n), ind: ws.add(0x80).readPointer(), val: ws.add(0x88).readPointer()}; }
 function wsHash(lp) { const ws = lp.add(0xe0).readPointer(), w = wsArrays(ws), st = [0x811c9dc5, 0x12345678];
   H(st, w.beg, 8 * w.n); H(st, w.len, 4 * w.n); H(st, w.ind, 4 * w.e); H(st, w.val, 8 * w.e); H(st, ws.add(0x90).readPointer(), w.m);
   for (const o of [0x98, 0xa8, 0xb0]) H(st, ws.add(o).readPointer(), 8 * (w.n + w.m));
   H(st, ws.add(0xa0).readPointer(), 8 * w.m); H(st, ws.add(0x3b8), 16);
   return {rows: w.m, cols: w.n, hash: hs(st)}; }
 function wsDump(lp) { const ws = lp.add(0xe0).readPointer(), w = wsArrays(ws), cbeg = [], ind = [], val = [];
   for (let j = 0; j < w.n; j++) { const s0 = w.beg.add(8 * j).readS64().toNumber(), l = w.len.add(4 * j).readS32(); cbeg.push(ind.length);
     for (let k = s0; k < s0 + l; k++) { ind.push(w.ind.add(4 * k).readS32()); val.push(dbl1(w.val.add(8 * k))); } }
   cbeg.push(ind.length);
   return Object.assign(wsHash(lp), {status_f0: ws.add(0xf0).readS32(), sense_60: dbl1(ws.add(0x60)), obj_scale_d8: dbl1(ws.add(0xd8)),
     null_colscale_e0: ws.add(0xe0).readPointer().isNull(), null_rowscale_e8: ws.add(0xe8).readPointer().isNull(),
     infbound_370: dbl1(ws.add(0x370)), cutoff_3b8: dbl1(ws.add(0x3b8)), cutoff2_3c0: dbl1(ws.add(0x3c0)),
     cbeg, ind, val, sense: chars(ws.add(0x90).readPointer(), w.m), c: dbls(ws.add(0x98).readPointer(), w.n + w.m), b: dbls(ws.add(0xa0).readPointer(), w.m),
     l: dbls(ws.add(0xa8).readPointer(), w.n + w.m), u: dbls(ws.add(0xb0).readPointer(), w.n + w.m)}); }
 // public API (instrumented: GRBcheckmodel clears the env error buffer; GRBgetvars allocates)
 const api = {getvars: new NativeFunction(b.add(0x7a1150), 'int', ['pointer', 'pointer', 'pointer', 'pointer', 'pointer', 'int', 'int']),
   intattr: new NativeFunction(b.add(0x7a34b0), 'int', ['pointer', 'pointer', 'pointer']),
   dblattr: new NativeFunction(b.add(0x7a3820), 'int', ['pointer', 'pointer', 'pointer']),
   dblarr: new NativeFunction(b.add(0x7a4cf0), 'int', ['pointer', 'pointer', 'int', 'int', 'pointer']),
   chararr: new NativeFunction(b.add(0x7a4930), 'int', ['pointer', 'pointer', 'int', 'int', 'pointer'])};
 function apiDump(lp) { const rcs = {}, ck = (k, rc) => { rcs[k] = rc; if (rc !== 0) throw Error('api ' + k + ' rc ' + rc); };
   const ia = name => { const o = Memory.alloc(4); ck(name, api.intattr(lp, Memory.allocUtf8String(name), o)); return o.readS32(); };
   const n = ia('NumVars'), m = ia('NumConstrs'), sense = ia('ModelSense');
   const oc = Memory.alloc(8); ck('ObjCon', api.dblattr(lp, Memory.allocUtf8String('ObjCon'), oc));
   const da = (name, k) => { const o = Memory.alloc(8 * Math.max(1, k)); ck(name, api.dblarr(lp, Memory.allocUtf8String(name), 0, k, o)); return dbls(o, k); };
   const ca = (name, k) => { const o = Memory.alloc(Math.max(1, k)); ck(name, api.chararr(lp, Memory.allocUtf8String(name), 0, k, o)); return chars(o, k); };
   const nz = Memory.alloc(4); ck('getvars0', api.getvars(lp, nz, NULL, NULL, NULL, 0, n)); const e = nz.readS32();
   const vb = Memory.alloc(4 * (n + 1)), vi = Memory.alloc(4 * Math.max(1, e)), vv = Memory.alloc(8 * Math.max(1, e));
   ck('getvars', api.getvars(lp, nz, vb, vi, vv, 0, n)); if (nz.readS32() !== e) throw Error('getvars nnz changed');
   const cbeg = ints(vb, n); cbeg.push(e);
   const env = lp.add(0x100).readPointer(), params = {};
   for (const [t, name, o] of ENV_PARAMS) params[name] = t === 'int' ? env.add(o).readS32() : dbl1(env.add(o));
   return {cols: n, rows: m, model_sense: sense, obj_con: dbl1(oc), obj: da('Obj', n), lb: da('LB', n), ub: da('UB', n), vtype: ca('VType', n),
     rhs: da('RHS', m), sense: ca('Sense', m), cbeg, ind: ints(vi, e), val: dbls(vv, e), lp_f0_null: lp.add(0xf0).readPointer().isNull(), env_params: params, api_rc: rcs}; }
 const mipIndex = new Map(); let lastDims = null;
 const mipIdx = mm => { const k = mm.toString(); if (!mipIndex.has(k)) mipIndex.set(k, mipIndex.size); return mipIndex.get(k); };
 const inLib = p => p.compare(b) >= 0 && p.compare(b.add(m.size)) < 0;
 const bt = (c, kind) => Thread.backtrace(c, kind).slice(0, 16).map(p => inLib(p) ? off(p) : null);
 // MIP identity: M+0x620 root(master) MIP, M+0x628 == M at top level (5daa82); root M+0x808 restart count
 const mipId = mm => { const r = mm.add(0x620).readPointer();
   return {mip: mipIdx(mm), mips_seen: mipIndex.size, root_mip: mipIdx(r), self_628: mm.add(0x628).readPointer().equals(mm), restarts: r.add(0x808).readS32()}; };
 // 5bbf7f(model) MIP main: nesting depth tells top-level search from a MIP solved inside another one
 let mipDepth = 0, callNow = () => nextCall;
 Interceptor.attach(b.add(0x4e36e0), {onEnter(a) { this.p = a[0].add(0x1d8).readPointer(); this.dp = ++mipDepth; const d = a[0].add(0xe8).readPointer();
   const pd = this.p.isNull() ? null : dims(this.p);
   send({event: 'y:mipmain', phase: 'enter', call: callNow(), depth: this.dp, ret: off(this.returnAddress), bt: bt(this.context, Backtracer.FUZZY),
     model_rows: d.add(8).readS32(), model_cols: d.add(0xc).readS32(), presolved: pd, env_2728: a[0].add(0x100).readPointer().add(0x2728).readS32()}); },
  onLeave(r) { const mm = this.p.isNull() ? NULL : this.p.add(0x2b8).readPointer();
   send({event: 'y:mipmain', phase: 'leave', call: callNow(), depth: this.dp, rc: r.toInt32(), mip: mm.isNull() ? null : mipIdx(mm)}); mipDepth--; }});
 Interceptor.attach(b.add(0x5805a0), {onEnter(a) { this.mm = a[0]; send(Object.assign({event: 'y:restart', phase: 'enter', call: callNow(), depth: mipDepth}, mipId(a[0]))); },
  onLeave(r) { send(Object.assign({event: 'y:restart', phase: 'leave', call: callNow(), rc: r.toInt32()}, mipId(this.mm))); }});
 // ---- provenance of I+0xb0 across the whole run ----
 const lastPack = new Map();     // I ptr string -> record of last write of I+0xb0
 const lastExport = new Map();   // vbasis dest ptr string -> {seq, ret, hash}
 let gseq = 0;
 // A record is keyed by the raw I pointer, which the heap reuses: accept it only while I+0xb0 still holds the recorded content.
 function validPack(I) { const r = lastPack.get(I.toString()), p = I.add(0xb0).readPointer(); if (!r || p.isNull()) return null;
   return r.hash === fnv(ints(p, packedWords(I))) ? r : {fn: r.fn, stale: true}; }
 const trim = (r, d) => r === undefined ? null : r === null ? null : Object.assign({}, r, {from: d > 0 && r.from ? trim(r.from, d - 1) : (r.from ? 'truncated' : null)});
 Interceptor.attach(b.add(0x1a6300), {onEnter(a) { this.lp = a[0]; this.vb = a[1]; this.cb = a[2]; this.ret = off(this.returnAddress); },
  onLeave(r) { const d = dims(this.lp), rc = r.toInt32();
   const vb = rc === 0 ? ints(this.vb, d.cols) : null, cb = rc === 0 ? ints(this.cb, d.rows) : null;
   const rec = {seq: ++gseq, ret: this.ret, rc, hash: vb && cb ? fnv(vb.concat(cb)) : null};
   if (!this.vb.isNull() && !(top() && top().synth)) lastExport.set(this.vb.toString(), rec);
   const s = top(); if (s) s.buf.push({event: 'x:export', synthetic: !!s.synth, ret: this.ret, rc, cols: d.cols, rows: d.rows, after: wsinfo(this.lp), vbasis: vb, cbasis: cb}); }});
 Interceptor.attach(b.add(0x579300), {onEnter(a) { this.n = a[0]; this.I = a[0].add(0x18).readPointer(); this.ret = off(this.returnAddress);
   const src = this.I.add(0x98).readPointer(), cols = mipCols(this.I), rows = this.I.add(0x94).readS32();
   this.src = src.isNull() ? null : {export: lastExport.get(src.toString()) || null, cols, rows, hash: fnv(ints(src, cols).concat(ints(this.I.add(0xa0).readPointer(), rows)))};
   const prev = lastPack.get(this.I.toString()); this.prev = !prev ? null : prev.node === this.n.readDouble() ? prev : {fn: prev.fn, stale: true}; },
  onLeave(r) { const p = this.I.add(0xb0).readPointer();
   lastPack.set(this.I.toString(), {fn: 'pack', seq: ++gseq, ret: this.ret, rc: r.toInt32(), node: this.n.readDouble(), I94: this.I.add(0x94).readS32(),
     hash: p.isNull() ? null : fnv(ints(p, packedWords(this.I))), src: this.src, from: this.prev}); }});
 Interceptor.attach(b.add(0x57a210), {onEnter(a) { this.n = a[0]; this.I = a[0].add(0x18).readPointer(); this.pI = a[2].add(0x18).readPointer();
   this.ret = off(this.returnAddress); this.parent = a[2].readDouble(); this.from = validPack(this.pI); this.parentUnpacked = !this.pI.add(0x98).readPointer().isNull();
   if (this.parentUnpacked) { const pv = this.pI.add(0x98).readPointer(), cols = mipCols(this.pI), rows = this.pI.add(0x94).readS32();
    this.parentSrc = {export: lastExport.get(pv.toString()) || null, cols, rows, hash: fnv(ints(pv, cols).concat(ints(this.pI.add(0xa0).readPointer(), rows)))}; } else this.parentSrc = null; },
  onLeave(r) { const p = this.I.add(0xb0).readPointer();
   lastPack.set(this.I.toString(), {fn: 'copy', seq: ++gseq, ret: this.ret, rc: r.toInt32(), node: this.n.readDouble(), parent: this.parent, parent_unpacked: this.parentUnpacked, parent_src: this.parentSrc,
     child_unpacked: (() => { const v = this.I.add(0x98).readPointer(); if (v.isNull()) return null; const cols = mipCols(this.I), rows = this.I.add(0x94).readS32();
       return {cols, rows, hash: fnv(ints(v, cols).concat(ints(this.I.add(0xa0).readPointer(), rows)))}; })(),
     I94: this.I.add(0x94).readS32(), hash: p.isNull() ? null : fnv(ints(p, packedWords(this.I))), from: this.from}); }});
 // ---- 03c-1 retry capture, unchanged except the x: events ----
 const solves = [];
 let nextCall = 0;
 const top = () => solves.length ? solves[solves.length - 1] : null;
 const rec = e => { const s = top(); if (s) s.buf.push(e); };
 Interceptor.attach(b.add(0x4f0e10), {
  onEnter(a) { this.h = a[0]; this.n = a[0].readPointer(); this.call = ++nextCall; this.lp = a[2].add(8).readPointer();
   solves.push({call: this.call, lp: this.lp, buf: [], retry: false, I: this.n.add(0x18).readPointer(), inner: 0, cda: 0, cdaStack: []});
   rec({event: 'solve:enter', call: this.call, node: node(this.n), lp: lpstate(this.lp)});
   const I = this.n.add(0x18).readPointer();
   rec({event: 'x:solve_basis', node_basis: nodeBasis(I), provenance: trim(validPack(I), 8), lp: wsinfo(this.lp)});
   const mm = I.readPointer(), d = dims(this.lp), key = [mipIdx(mm), d.rows, d.cols].join();
   if (key !== lastDims) { lastDims = key;
    send(Object.assign({event: 'y:dims', call: this.call, depth: mipDepth}, mipId(mm), {rows: d.rows, cols: d.cols, root: node(this.n).root,
     lp_is_mip_lp: this.lp.equals(mm.add(8).readPointer()), env_2728: this.lp.add(0x100).readPointer().add(0x2728).readS32()})); } },
  onLeave(r) { const after = this.h.readPointer(), s = solves.pop();
   if (!s || s.call !== this.call) throw Error('solve nesting mismatch');
   s.buf.push({event: 'solve:leave', call: this.call, ret: r.toInt32(), holder_changed: !after.equals(this.n), node: node(after), lp: lpstate(this.lp)});
   if (s.retry) send({event: 'retry_solve', events: s.buf}); }});
 // One listener per address: a second listener on the same function lost its onLeave in the probe run.
 const exportFn = new NativeFunction(b.add(0x1a6300), 'int', ['pointer', 'pointer', 'pointer']);
 for (const [name, o] of [['lpdriver', 0x14acb0], ['simplex', 0x153cc0], ['reopt', 0x1cb7e0]]) {
  Interceptor.attach(b.add(o), {
   onEnter(a) { const s = top(); this.on = !!(s && s.retry); this.x = !!s;
    if (this.on) { this.call = ++nextCall; this.lp = a[0];
     const st = lpstate(this.lp); this.dims = name === 'lpdriver' || !st || st.ws_cols === null ? null : [st.ws_cols, st.ws_rows]; this.vb = a[2]; this.cb = a[3];
     rec({event: name + ':enter', call: this.call, lp: st, vbasis: this.dims ? ints(a[2], this.dims[0]) : null, cbasis: this.dims ? ints(a[3], this.dims[1]) : null});
     this.top = s.inner++ === 0; }
    if (this.x && name === 'simplex') xCdaEnter(this, a, s); },
   onLeave(r) {
    if (this.on) rec({event: name + ':leave', call: this.call, ret: r.toInt32(), lp: lpstate(this.lp),
     vbasis_out: this.dims ? ints(this.vb, this.dims[0]) : null, cbasis_out: this.dims ? ints(this.cb, this.dims[1]) : null});
    if (this.x && name === 'simplex') xCdaLeave(this, r);
    if (this.on) { const s = top(); s.inner--; if (this.top) afterRetryCall(this.lp); } }});
 }
 for (const [name, o] of [['normalize_before', 0x4f1a98], ['normalize_after', 0x4f1aca], ['retry', 0x4f53a8]]) {
  Interceptor.attach(b.add(o), {onEnter() { const s = top(); if (!s) return; const c = this.context;
   if (name === 'retry') s.retry = true;
   rec({event: name, node: node(c.rsp.add(0x3b0).readPointer()), lp_status: c.r13.add(0x38).readS32(), lp: lpstate(s.lp)});
   if (name === 'retry') rec({event: 'x:retry_basis', node_basis: nodeBasis(s.I), lp: wsinfo(s.lp)});
   if (name === 'retry') { const mm = s.I.readPointer(), mlp = mm.add(8).readPointer();
    rec(Object.assign({event: 'y:where', depth: mipDepth, bt_accurate: bt(c, Backtracer.ACCURATE), bt_fuzzy: bt(c, Backtracer.FUZZY)}, mipId(mm), {
     lp_is_mip_lp: s.lp.equals(mlp), mip_lp: dims(mlp), lp_env_2728: s.lp.add(0x100).readPointer().add(0x2728).readS32(),
     mip_env_2728: mlp.add(0x100).readPointer().add(0x2728).readS32(), mip_status: mm.readS32()}));
    rec(Object.assign({event: 'y:decision_dhash'}, dhash(s.lp))); } }});
 }
 Interceptor.attach(b.add(0x5180f0), {onEnter(a) { rec({event: 'set', value: a[1].toInt32(), ret: off(this.returnAddress), node: node(a[0])}); }});
 Interceptor.attach(b.add(0x518170), {onEnter(a) { this.on = !!top(); if (this.on) rec({event: 'finish:enter', node: node(a[0])}); },
  onLeave(r) { if (this.on) rec({event: 'finish:leave', ret: r.toInt32()}); }});
 // ---- new x: hooks inside the current 5ce67b solve ----
 Interceptor.attach(b.add(0x579610), {onEnter(a) { const s = top(); this.on = !!s; if (!this.on) return; this.I = a[0].add(0x18).readPointer();
   const p = this.I.add(0xb0).readPointer(), w = p.isNull() ? null : ints(p, packedWords(this.I));
   rec({event: 'x:decode:enter', node_basis: nodeBasis(this.I), cols: mipCols(this.I), rows: dims(a[1].add(8).readPointer()).rows, packed: w, packed_hash: w ? fnv(w) : null,
     provenance: trim(validPack(this.I), 8)}); },
  onLeave(r) { if (!this.on) return; const I = this.I, vb = I.add(0x98).readPointer(), cb = I.add(0xa0).readPointer(), cols = mipCols(I), rows = I.add(0x94).readS32();
   rec({event: 'x:decode:leave', rc: r.toInt32(), node_basis: nodeBasis(I), vbasis: ints(vb, cols), cbasis: ints(cb, rows)}); }});
 function xCdaEnter(t, a, s) { t.k = ++s.cda; s.cdaStack.push(t.k); t.xlp = a[0]; const d = dims(a[0]);
   const z = [4, 5, 6, 7].map(k => a[k].isNull());
   rec({event: 'x:cda5b:enter', k: t.k, depth: s.cdaStack.length, retry: s.retry, ret: off(t.returnAddress), method: a[1].toInt32(), vb_rel: rel(a[2], s.I), cb_rel: rel(a[3], s.I),
     starts_null: z, cols: d.cols, rows: d.rows, vbasis: a[2].isNull() ? null : ints(a[2], d.cols), cbasis: a[3].isNull() ? null : ints(a[3], d.rows), node_basis: nodeBasis(s.I), lp: wsinfo(a[0])});
   if (s.cdaStack.length !== 1) return;
   rec(Object.assign({event: 'y:dhash', k: t.k, lp_is_node_lp: a[0].equals(s.lp)}, dhash(a[0])));
   // first LP call on the node LP itself after the retry decision: D is unscaled here (at the decision gmu's D is scaled in place)
   if (s.retry && !s.dumped && a[0].equals(s.lp)) { s.dumped = true; rec(Object.assign({event: 'y:lp', k: t.k, instrumented: true, dhash: dhash(a[0])}, apiDump(a[0]))); } }
 function xCdaLeave(t, r) { const s = top(); s.cdaStack.pop(); rec({event: 'x:cda5b:leave', k: t.k, rc: r.toInt32(), lp: wsinfo(t.xlp)});
   if (s.retry && !s.cdaStack.length) { const ws = t.xlp.add(0xe0).readPointer();
    rec({event: 'y:ws_leave', k: t.k, status_f0: ws.isNull() ? null : ws.add(0xf0).readS32(), obj_118: ws.isNull() ? null : dbl1(ws.add(0x118))}); } }
 // top-level LP call inside the retry block: post-retry export predicates; in synth mode also the export itself
 function afterRetryCall(lp) {
  rec({event: 'x:after_retry_call', lp: wsinfo(lp)});
  if (mode !== 'synth') return;
  const d = dims(lp), vb = Memory.alloc(4 * Math.max(1, d.cols)), cb = Memory.alloc(4 * Math.max(1, d.rows));
  const s = top(); s.synth = true; const rc = exportFn(lp, vb, cb); s.synth = false;
  rec({event: 'x:synth_export', synthetic: true, rc, cols: d.cols, rows: d.rows, vbasis: rc === 0 ? ints(vb, d.cols) : null, cbasis: rc === 0 ? ints(cb, d.rows) : null}); }
 const inCda = () => { const s = top(); return !!(s && s.cdaStack.length); };
 Interceptor.attach(b.add(0xd06f0), {onEnter(a) { this.on = inCda(); this.lp = a[0]; },
  onLeave(r) { if (!this.on) return; const s = top(), k = s.cdaStack[s.cdaStack.length - 1];
   rec({event: 'x:2f7e1:leave', k, rc: r.toInt32(), lp: wsinfo(this.lp)});
   if (s.cdaStack.length !== 1 || this.lp.add(0xe0).readPointer().isNull()) return;
   rec(Object.assign({event: s.retry ? 'y:ws' : 'y:ws_hash', k}, s.retry ? wsDump(this.lp) : wsHash(this.lp))); }});
 Interceptor.attach(b.add(0xd3150), {onEnter(a) { this.on = inCda(); if (!this.on) return; this.ws = a[0]; const s = top();
   rec({event: 'x:load:enter', k: s.cdaStack[s.cdaStack.length - 1], vb_rel: rel(a[1], s.I), cb_rel: rel(a[2], s.I)}); },
  onLeave(r) { if (!this.on) return; const ws = this.ws, n = ws.add(0x64).readS32() + ws.add(0x68).readS32(), h = ints(ws.add(0x168).readPointer(), n);
   rec({event: 'x:load:leave', rc: r.toInt32(), ws_rows: ws.add(0x64).readS32(), ws_cols: ws.add(0x68).readS32(), header_nonneg: h.filter(x => x >= 0).length, header_hash: fnv(h)}); }});
 Interceptor.attach(b.add(0xd4190), {onEnter() { this.on = inCda(); if (this.on) rec({event: 'x:factor', k: top().cdaStack[top().cdaStack.length - 1]}); }});
 Interceptor.attach(b.add(0x19ed40), {onEnter(a) { const s = top(); if (!s) return; const d = dims(a[0]);
   const vb = a[1].isNull() ? null : ints(a[1], d.cols), cb = a[2].isNull() ? null : ints(a[2], d.rows);
   rec({event: 'x:setbasis', ret: off(this.returnAddress), vb_rel: rel(a[1], s.I), cb_rel: rel(a[2], s.I), cols: d.cols, rows: d.rows, hash: vb && cb ? fnv(vb.concat(cb)) : null}); }});
 for (const [name, o] of [['save_lp9', 0x4f8470], ['save_lp2', 0x4f6a50], ['degen', 0x566f50]]) {
  Interceptor.attach(b.add(o), {onEnter(a) { this.on = !!top(); if (this.on) rec({event: 'x:' + name + ':enter', ret: off(this.returnAddress)}); },
   onLeave(r) { if (this.on) rec({event: 'x:' + name + ':leave', rc: r.toInt32(), node_basis: nodeBasis(top().I)}); }});
 }
}});
`;
(async () => {
 fs.mkdirSync(path.dirname(path.resolve(output)), {recursive: true}); if (fs.existsSync(modelDump)) fs.unlinkSync(modelDump);
 const device = await frida.getLocalDevice(); const events = []; let outputText = ''; const errors = [];
 const pid = await device.spawn([py, '-c', code], {stdio: 'pipe'}); const session = await device.attach(pid);
 const script = await session.createScript(agent);
 script.message.connect(msg => { if (msg.type !== 'send') { errors.push(msg.stack || JSON.stringify(msg)); return; } events.push(msg.payload); });
 device.output.connect((p, fd, data) => { if (p === pid) outputText += data.toString(); });
 session.detached.connect((reason, crash) => {
   const dump = fs.existsSync(modelDump) ? JSON.parse(fs.readFileSync(modelDump, 'utf8')) : null; if (dump) fs.unlinkSync(modelDump);
   const ok = outputText.includes('DONE') && !errors.length && !!dump;
   fs.mkdirSync(path.dirname(output), {recursive: true});
   fs.writeFileSync(output, JSON.stringify({schema: 1, kind: 'natural_retry_node_lp', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), env_param_table_sha256: hash(ptab), params, output: outputText.trim(), errors, ok,
     detach_reason: reason, model_dump: dump, events}) + '\n');
   console.log(mode, path.basename(model), outputText.trim(), 'events', events.length, 'ok', ok, errors.join(' ')); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
