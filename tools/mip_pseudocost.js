// M2-06e (docs/m2/mip_pseudocost.md): read-only trace of the pseudocost tables (0x80 B, notes 15 §4.6).
//   new   6ec3da (create, 5daa82) and 6eca1a (thread copy): full nonzero snapshot of the new table
//   free  6edb26: full nonzero snapshot before the free (end-of-life check of the chain)
//   upd   every 6edfcf call: caller, table, j, cnt/sum of both directions before and after;
//         at the two 5fb8ec table-A sites the caller-frame slots w [rsp+0x18] and cutoff [rsp+0x20]
//   gate  5fb8ec 0x5112a7 (entry of the node-update block, reached only from 0x511277): the inputs of the rule
//   dgate 5fe9cd 0x51679f (the decay gate): the inputs of the gate
//   dec   6ee1d3 entry/leave: LP info fields, the ancestor path (j, dir, v[j]) and the touched entries before/after
//   ins   the M2-06a heap insert (same fields) for the non-interference check against bench/oracle/m2_06b2
//   bulk  6ecf59 / 6ed2ad / 6ed6a1 / 6ed22f calls (parallel merge and reset paths), counted
//   sym   6ef22e entry/leave (symmetry-class propagation, found with a hardware watchpoint): class map and the
//         nonzero entries before and after
// Tables are named by creation order, never by address. xmm registers are not readable (lessons/tooling.md), so the
// 6edfcf arguments f and delta are not read; the checker recomputes them. Doubles are u64 hex. Nothing is written into
// the target. One invocation spawns exactly one original process.
// Usage: tools/gurun.sh node tools/mip_pseudocost.js MODEL OUTPUT.json.gz 'JSON_PARAMS'
const fs = require('fs'), path = require('path'), crypto = require('crypto'), zlib = require('zlib');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [model, output, extra] = process.argv.slice(2);
if (!model || !output) throw Error('usage: MODEL OUTPUT [JSON_PARAMS]');
if (fs.existsSync(output)) throw Error('output exists: ' + output);
const root = path.resolve(__dirname, '../..');
const py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const expected = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c';
if (hash(binary) !== expected) throw Error('wrong Gurobi binary');
const funcs = fs.readFileSync(path.join(root, 're/decomp/index.tsv'), 'utf8').trim().split('\n').slice(1).map(l => l.split('\t'))
  .map(r => [parseInt(r[0], 16), parseInt(r[2], 10), r[1].replace(/^PRIVATE0+/, '')]).sort((a, b) => a[0] - b[0]);
const params = Object.assign({Threads: 1, Seed: 0, LogToConsole: 0}, JSON.parse(extra || '{}'));
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\n` +
  `m.optimize()\nprint('DONE',m.Status,int(m.NodeCount),flush=True)`;
const agent = `
const FUNCS = ${JSON.stringify(funcs)};
function fname(o) { let lo = 0, hi = FUNCS.length - 1;
  while (lo < hi) { const mid = (lo + hi + 1) >> 1; if (FUNCS[mid][0] <= o) lo = mid; else hi = mid - 1; }
  const f = FUNCS[lo]; return f[0] <= o && o < f[0] + f[1] ? f[2] : 'raw:' + o.toString(16); }
const hx = p => p.readU64().toString(16).padStart(16, '0');
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return;
 const b = m.base, off = p => p.sub(b).toUInt32();
 let depth = 0, inSel = 0;
 // ---- tables by creation order
 const tabs = new Map(); let tabSerial = 0;
 const tser = t => t.isNull() ? -2 : (tabs.has(t.toString()) ? tabs.get(t.toString()) : -1);
 const tsize = t => { const d = t.add(0x70).readPointer().add(8).readPointer().add(0xe8).readPointer();
   return [d.add(0xc).readS32(), d.add(8).readS32()]; };
 const ent = (t, j) => [t.readPointer().add(4 * j).readS32(), t.add(8).readPointer().add(4 * j).readS32(),
   hx(t.add(0x10).readPointer().add(8 * j)), hx(t.add(0x18).readPointer().add(8 * j))];
 const snap = t => { const [nc, nr] = tsize(t), n = nc + nr, out = [];
   if (n > 0) { const cd = new Int32Array(t.readPointer().readByteArray(4 * n)), cu = new Int32Array(t.add(8).readPointer().readByteArray(4 * n));
     const sd = new BigUint64Array(t.add(0x10).readPointer().readByteArray(8 * n)), su = new BigUint64Array(t.add(0x18).readPointer().readByteArray(8 * n));
     for (let j = 0; j < n; j++) if (cd[j] || cu[j] || sd[j] || su[j])
       out.push([j, cd[j], cu[j], sd[j].toString(16).padStart(16, '0'), su[j].toString(16).padStart(16, '0')]); }
   return {ncols: nc, nrows: nr, eta: t.add(0x60).readS32(), sbit: t.add(0x64).readS32(), sbcost: hx(t.add(0x68)), nz: out}; };
 Interceptor.attach(b.add(0x4e36e0), {onEnter() { depth++; }, onLeave() { depth--; }});
 // 6ec3da(M, src, map, &out) and 6eca1a(M, src, &out)
 for (const [fa, fn, oi] of [[0x5b6690, '6ec3da', 3], [0x5b6a30, '6eca1a', 2]])
 Interceptor.attach(b.add(fa), {onEnter(a) { this.M = a[0]; this.src = a[1]; this.out = a[oi]; this.from = fname(off(this.returnAddress));
   this.slot = this.out.sub(a[0]).toInt32();
   this.map = fn === '6ec3da' && !a[2].isNull() ? a[2] : null;
   this.flag = a[0].add(0x620).readPointer().add(0x850).readS32(); },
  onLeave(r) { const t = this.out.readPointer(), rc = r.toInt32();
   if (t.isNull()) { send({e: 'new', f: fn, nest: depth, from: this.from, rc, t: -2}); return; }
   const k = tabSerial++; tabs.set(t.toString(), k); const s = snap(t);
   let map = null; if (this.map && s.ncols > 0) map = Array.from(new Int32Array(this.map.readByteArray(4 * s.ncols)));
   send(Object.assign({e: 'new', f: fn, nest: depth, from: this.from, rc, t: k, slot: this.slot, src: this.src.isNull() ? -2 : tser(this.src),
     flag850: this.flag, map}, s)); }});
 Interceptor.attach(b.add(0x5b70c0), {onEnter(a) { const t = a[1].readPointer(); if (t.isNull()) return; const k = tser(t);
   send(Object.assign({e: 'free', nest: depth, from: fname(off(this.returnAddress)), t: k}, k >= 0 ? snap(t) : {}));
   tabs.delete(t.toString()); }});
 for (const [fa, fn] of [[0x5b6dd0, '6ecf59'], [0x5b6ef0, '6ed2ad'], [0x5b6f80, '6ed6a1'], [0x5b6ea0, '6ed22f']])
   Interceptor.attach(b.add(fa), {onEnter() { send(['bulk', fn, fname(off(this.returnAddress))]); }});
 // ---- 6ef22e(alloc, M, T): symmetry-class propagation at the end of the root (64da9f C:1869-1873)
 Interceptor.attach(b.add(0x5b7b20), {onEnter(a) { const M = a[1], t = a[2], S = M.add(0x32a8).readPointer(); this.t = t;
   const ev = {e: 'sym', nest: depth, from: fname(off(this.returnAddress)), t: tser(t), tA: tser(M.add(0x3250).readPointer())};
   this.ev = ev; ev.S = S.isNull() ? 0 : 1;
   if (!S.isNull()) { ev.norb = S.add(4).readS32(); const nc = tsize(t)[0];
     ev.cls = nc > 0 && ev.norb > 0 ? Array.from(new Int32Array(S.add(0x40).readPointer().readByteArray(4 * nc))) : null; }
   ev.before = snap(t).nz; },
  onLeave(r) { this.ev.rc = r.toInt32(); this.ev.after = snap(this.t).nz; send(this.ev); }});
 // ---- 6edfcf(f, delta, T, j)
 const SITE = {0x511407: 'n12', 0x514e7d: 'n45'};
 Interceptor.attach(b.add(0x5b7330), {onEnter(a) { const t = a[0], j = a[1].toInt32(), ra = off(this.returnAddress);
   this.t = t; this.j = j;
   this.ev = {e: 'upd', nest: depth, from: fname(ra), ra, t: tser(t), j, before: ent(t, j), chg: t.add(0x48).readS32()};
   if (SITE[ra]) { const crsp = this.context.rsp.add(8); this.ev.site = SITE[ra]; this.ev.w = hx(crsp.add(0x18)); this.ev.cut = hx(crsp.add(0x20)); } },
  onLeave() { this.ev.after = ent(this.t, this.j); this.ev.chg_after = this.t.add(0x48).readS32(); send(this.ev); }});
 const cutoff = M => { const p = M.add(0x620).readPointer().add(0x3220).readPointer(); return p.isNull() ? null : hx(p.add(0x40)); };
 // ---- 5fb8ec node-update gate (rsp and rbp are the 5fb8ec frame; r10 = N loaded at 0x51129f)
 Interceptor.attach(b.add(0x5112a7), function () { const x = this.context, rsp = x.rsp, N = x.r10, M = x.rbp;
   const I = N.add(0x18).readPointer(), J = rsp.add(0x60).readPointer(), j = J.add(0x34).readS32();
   send({e: 'gate', nest: depth, id: N.readDouble(), parent: N.add(0x10).readPointer().isNull() ? 0 : 1, ij: I.equals(J) ? 1 : 0,
     st: I.add(0x28).readS32(), depth: I.add(0x2c).readS32(), j, x: hx(J.add(0x38)), dir: J.add(0x40).readU8(),
     vt: j >= 0 ? rsp.add(0x68).readPointer().add(j).readU8() : null, lp: hx(J.add(0x10)), p: hx(I.add(0x78)), cut: cutoff(M),
     tA: tser(M.add(0x3250).readPointer())}); });
 // ---- 5fe9cd decay gate
 Interceptor.attach(b.add(0x51679f), function () { const x = this.context, N = x.r15, K = x.rbx;
   const I = N.add(0x18).readPointer();
   send({e: 'dgate', nest: depth, id: N.readDouble(), parent: N.add(0x10).readPointer().isNull() ? 0 : 1, ik: I.equals(K) ? 1 : 0,
     st: I.add(0x28).readS32(), j: K.add(0x34).readS32(), lp: hx(K.add(0x10)), p: hx(I.add(0x78)),
     tA: tser(K.readPointer().add(0x3250).readPointer())}); });
 // ---- 6ee1d3(N, T)
 Interceptor.attach(b.add(0x5b73d0), {onEnter(a) { const N = a[0], t = a[1]; this.t = t;
   const ev = {e: 'dec', nest: depth, id: N.readDouble(), t: tser(t)}; this.ev = ev;
   const I = N.add(0x18).readPointer(); if (I.isNull()) { ev.noI = 1; return; }
   const L = I.add(8).readPointer(); if (L.isNull()) { ev.noL = 1; return; }
   ev.l38 = L.add(0x38).readS32(); ev.l58 = L.add(0x58).readS32(); ev.l54 = L.add(0x54).readS32();
   ev.l18 = L.add(0x18).readPointer().isNull() ? 0 : 1; ev.l28 = L.add(0x28).readPointer().isNull() ? 0 : 1;
   ev.l70 = L.add(0x70).readPointer().isNull() ? 0 : 1;
   const v = (ev.l38 === 6 || ev.l58 >= 0) ? L.add(0x70).readPointer() : (ev.l18 ? L.add(0x28).readPointer() : ptr(0));
   const anc = [], keys = new Map();
   for (let A = N.add(0x10).readPointer(), k = 0; !A.isNull() && k < 40; A = A.add(0x10).readPointer(), k++) {
     const AI = A.add(0x18).readPointer(); if (AI.isNull()) { anc.push(null); break; }
     const j = AI.add(0x34).readS32(), dir = AI.add(0x40).readU8();
     anc.push([j, dir, j >= 0 && !v.isNull() ? hx(v.add(8 * j)) : null]);
     if (j >= 0) keys.set(j, 1); }
   ev.anc = anc; this.js = Array.from(keys.keys()); ev.before = this.js.map(j => [j].concat(ent(t, j))); },
  onLeave() { const ev = this.ev; if (this.js) ev.after = this.js.map(j => [j].concat(ent(this.t, j))); send(ev); }});
 // ---- M2-06a heap insert (tools/mip_node_select.js rule, same fields as bench/oracle/m2_06b2)
 const nent = n => { const i = n.add(0x18).readPointer();
   return [n.readDouble(), hx(i.add(0x18)), hx(i.add(0x50)), hx(i.add(0x20)), i.add(0x2c).readS32(), i.add(0x30).readS32(),
     i.add(0x28).readS32(), i.add(0x41).readU8()]; };
 const tree = mm => mm.add(0x620).readPointer().add(0x780).readPointer();
 const trees = new Map(); let treeSerial = 0;
 const top = t => { const k = t.toString(); if (!trees.has(k)) { trees.set(k, treeSerial++); send({e: 'tnew', tree: trees.get(k)}); }
   return trees.get(k); };
 const tid = t => trees.has(t.toString()) ? trees.get(t.toString()) : -1;
 Interceptor.attach(b.add(0x5b29d0), {onEnter(a) { this.top = depth === 1; if (!this.top) return; top(tree(a[0])); inSel = 1; },
   onLeave() { if (this.top) inSel = 0; }});
 Interceptor.attach(b.add(0x5b1e70), {onEnter(a) {
   const t = tree(a[0]), from = fname(off(this.returnAddress)); if (depth === 1 && from === '5daa82') top(t);
   if (tid(t) < 0) { send(['ins_other', depth]); return; }
   send({e: 'ins', in_sel: inSel, from, tree: tid(t), node: nent(a[1]), heap_n: t.add(0x120).readU64().toNumber()}); }});
 Interceptor.attach(b.add(0x5adfd0), {onEnter(a) { const t = a[1].readPointer(); if (t.isNull()) return;
   const k = t.toString(); if (trees.has(k)) trees.delete(k); }});
}});
`;
(async () => {
 const device = await frida.getLocalDevice(); const counts = {}; const events = []; let outputText = ''; const errors = [];
 const pid = await device.spawn([py, '-c', code], {stdio: 'pipe'}); const session = await device.attach(pid);
 const script = await session.createScript(agent);
 script.message.connect(msg => { if (msg.type !== 'send') { errors.push(msg.stack || JSON.stringify(msg)); return; }
   const p = msg.payload;
   if (Array.isArray(p)) { const k = p.join(':'); counts[k] = (counts[k] || 0) + 1; } else events.push(p); });
 device.output.connect((p, fd, data) => { if (p === pid) outputText += data.toString(); });
 session.detached.connect((reason) => {
   if (outputText.includes('Gurobi is already running')) { console.log(outputText.trim()); process.exit(1); }
   const ok = outputText.includes('DONE') && !errors.length;
   fs.mkdirSync(path.dirname(output), {recursive: true});
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'mip_pseudocost', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok,
     detach_reason: reason, counts, events})));
   const c = {}; for (const e of events) c[e.e] = (c[e.e] || 0) + 1;
   const u = {}; for (const e of events) if (e.e === 'upd') { const k = e.from + (e.site ? '/' + e.site : ''); u[k] = (u[k] || 0) + 1; }
   console.log(path.basename(model), outputText.trim(), JSON.stringify(c), JSON.stringify(u), JSON.stringify(counts), 'ok', ok,
     errors.join(' ').slice(0, 300));
   process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
