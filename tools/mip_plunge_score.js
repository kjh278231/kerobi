// M2-06c-3 (docs/m2/mip_plunge_r47.md): tools/mip_plunge_rule.js (06c, hash-pinned) with new events only; every 06c event is
// unchanged. ents: [I+0x34, I+0x38, I+0x40] per pick-list entry at the pick probe; x4: 5e6f57 return in rule 4 (x[I+0x34] of the
// entries rule 4 reads); ranc: rule 1/6 body 5fb13c calls; r5: a7b4d6 return in rule 5 (eax, slot 0x70(%rsp)).
// M2-06c (docs/m2/mip_plunge_rule.md): the M2-06a trace (tools/mip_node_select.js, events unchanged) plus, inside top-level
// 6e8eb3 calls only, the plunge rule number, its RNG slot, the rule 3/12 pick and the tail cleanup (events pin s70 drw rpre
// rule pick tail anc thr rm tend). Mid-function probes are function-form (docs/lessons/tooling.md). 5f9a41/5e8239 are called
// read-only at the threshold probe.
// M2-06a (docs/m2/mip_node_select_spec.md): read-only trace of the node containers around node selection.
// Top-level trees only: a tree is top when its root is inserted by 5daa82 at 5bbf7f nesting 1 or a 6e8eb3 call at
// nesting 1 selects from it. For those trees, at any nesting (a sub-MIP can prune the parent tree): every 6e8eb3 call
// (enter/leave: plunge list, deferred list, counters, and the global heap when that call pops it), 6e460c return (heap),
// every 6e5802 heap insert, 6e460c prune with the nodes it frees (606036), 6e3763 free; 68bdcf deferred adds at nesting 1.
// Nodes are named by their id N+0x00 (M+0x58 counter); doubles are u64 hex (big-endian digits). No pointers are stored.
// Nothing is written into the target. One invocation spawns exactly one original process.
// Usage: tools/gurun.sh node tools/mip_plunge_score.js MODEL OUTPUT.json.gz 'JSON_PARAMS'
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
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\nm.optimize()\nprint('DONE',m.Status,int(m.NodeCount),flush=True)`;
const agent = `
const FUNCS = ${JSON.stringify(funcs)};
function fname(o) { let lo = 0, hi = FUNCS.length - 1;
  while (lo < hi) { const mid = (lo + hi + 1) >> 1; if (FUNCS[mid][0] <= o) lo = mid; else hi = mid - 1; }
  const f = FUNCS[lo]; return f[0] <= o && o < f[0] + f[1] ? f[2] : 'raw:' + o.toString(16); }
const hx = p => p.readU64().toString(16).padStart(16, '0');
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return;
 const b = m.base, off = p => p.sub(b).toUInt32();
 let depth = 0, inSel = 0, sel = 0, touched = 0, cur = null;
 const f2h = v => { const d = new DataView(new ArrayBuffer(8)); d.setFloat64(0, v); return d.getBigUint64(0).toString(16).padStart(16, '0'); };
 const nid = n => n.isNull() ? null : n.readDouble();
 const ids = w => { const k = w.add(0x3fa8).readS32(), a = w.add(0x3fb0).readPointer(), r = [];
   for (let j = 0; j < k; j++) r.push(a.add(8 * j).readPointer().readDouble()); return r; };
 const wk = c => c.r14.isNull() ? null : hx(c.r14);
 const u32 = p => p.and(ptr('0xffffffff')).toUInt32(), s32 = p => u32(p) | 0;
 // plunge / deferred entries: [id, bound, key, est, depth, plunge start (I+0x30), status, excluded (I+0x41)]
 const ent = n => { const i = n.add(0x18).readPointer();
   return [n.readDouble(), hx(i.add(0x18)), hx(i.add(0x50)), hx(i.add(0x20)), i.add(0x2c).readS32(), i.add(0x30).readS32(),
     i.add(0x28).readS32(), i.add(0x41).readU8()]; };
 const list = (cnt, arr) => { const k = cnt.readS32(), a = arr.readPointer(), r = [];
   for (let j = 0; j < k; j++) r.push(ent(a.add(8 * j).readPointer())); return r; };
 const tree = mm => mm.add(0x620).readPointer().add(0x780).readPointer();
 // top tree serial in order of becoming top; a tree freed by 6e3763 loses its serial, so a reused address gets a new one
 const trees = new Map(); let treeSerial = 0;
 const top = t => { const k = t.toString(); if (!trees.has(k)) { trees.set(k, treeSerial++); send({e: 'tnew', tree: trees.get(k)}); }
   return trees.get(k); };
 const tid = t => trees.has(t.toString()) ? trees.get(t.toString()) : -1;
 const heap = t => { const k = t.add(0x120).readU64().toNumber(), a = t.add(0x130).readPointer(), r = [];
   for (let j = 0; j < k; j++) { const n = a.add(8 * j).readPointer(), i = n.add(0x18).readPointer();
     r.push([n.readDouble(), hx(i.add(0x50)), hx(i.add(0x18))]); }
   return r; };
 const state = (mm, w, withHeap) => { const t = tree(mm);
   const s = {tree: top(t), k: w.add(0x30).readS32(), wdepth: w.add(0x34).readS32(), pops: mm.add(0x620).readPointer().add(0xd0).readS32(),
     plunge_min: hx(w.add(0x3f98)), heap_n: t.add(0x120).readU64().toNumber(), heap_min: hx(t.add(0x170)),
     heap10_n: t.add(0x150).readU64().toNumber(), mem: hx(t.add(0x18)),
     plunge: list(w.add(0x3fa8), w.add(0x3fb0)), deferred: list(w.add(0x3fb8), w.add(0x3fc0))};
   if (withHeap) s.heap = heap(t); return s; };
 Interceptor.attach(b.add(0x4e36e0), {onEnter() { depth++; }, onLeave() { depth--; }});
 Interceptor.attach(b.add(0x5b29d0), {onEnter(a) {
   this.top = depth === 1; if (!this.top) { send(['sel_sub', depth]); return; }
   this.mm = a[0]; this.w = a[1]; this.out = a[2]; this.s = sel++; inSel = 1; touched = 0;
   cur = {mm: a[0], w: a[1], s: this.s};
   { const mm = a[0], w = a[1], mx = mm.add(0x620).readPointer(), env = mm.add(8).readPointer().add(0x100).readPointer(),
       lp0 = mm.add(8).readPointer().add(0xe8).readPointer();
     send({e: 'pin', sel: this.s, k: w.add(0x30).readS32(), w28: hx(w.add(0x28)), ign_hint: env.add(0x2e2c).readS32(),
       hint: mm.add(0x3560).readPointer().isNull() ? 0 : 1, m848: mm.add(0x848).readS32(), mx808: mx.add(0x808).readS32(),
       mx40: hx(mx.add(0x40)), ncols: lp0.add(0xc).readS32(), nint: lp0.add(0x2c).readS32(), bdir: env.add(0x27e0).readS32(),
       focus: env.add(0x2710).readS32(), m2c: mm.add(0x2c).readS32()}); }
   const pl = a[1].add(0x3fa8).readS32(); this.p0 = a[0].add(0x620).readPointer().add(0xd0).readS32();
   send({e: 'enter', sel: this.s, mode: a[3].toInt32(), ret: fname(off(this.returnAddress)), st: state(a[0], a[1], pl === 0)}); },
  onLeave(r) { if (!this.top) return; inSel = 0; cur = null;
   const n = this.out.readPointer(), popped = this.mm.add(0x620).readPointer().add(0xd0).readS32();
   send({e: 'leave', sel: this.s, rc: r.toInt32(), node: n.isNull() ? null : ent(n), st: state(this.mm, this.w, popped !== this.p0)}); }});
 // heap insert 6e5802(M, N)
 Interceptor.attach(b.add(0x5b1e70), {onEnter(a) {
   const t = tree(a[0]), from = fname(off(this.returnAddress)); if (depth === 1 && from === '5daa82') top(t);
   if (tid(t) < 0) { send(['ins_other', depth]); return; }
   if (depth !== 1) send(['ins_top_nested', depth, from]);
   touched++; const n = a[1];
   send({e: 'ins', in_sel: inSel, from, tree: tid(t), node: ent(n), heap_n: t.add(0x120).readU64().toNumber()}); }});
 // heap prune 6e460c(cutoff in xmm0, M): drop bound >= cutoff (stable), re-heapify. Frida's x64 context has no xmm0;
 // the dropped nodes are the following acct events from 6e460c
 Interceptor.attach(b.add(0x5aed60), {onEnter(a) { const t = tree(a[0]);
   if (tid(t) < 0) { send(['prune_other', depth]); return; }
   this.t = t;
   send({e: 'prune', depth, in_sel: inSel, from: fname(off(this.returnAddress)), tree: tid(t), heap_n: t.add(0x120).readU64().toNumber(),
     heap10_n: t.add(0x150).readU64().toNumber()}); },
  onLeave() { if (this.t) send({e: 'pruned', tree: tid(this.t), heap: heap(this.t), heap_min: hx(this.t.add(0x170))}); }});
 // tree free 6e3763(env, &T)
 Interceptor.attach(b.add(0x5adfd0), {onEnter(a) { const t = a[1].readPointer(); if (t.isNull()) return;
   const k = t.toString(); if (!trees.has(k)) return;
   send({e: 'tfree', from: fname(off(this.returnAddress)), tree: trees.get(k), heap_n: t.add(0x120).readU64().toNumber()}); trees.delete(k); }});
 // deferred add 68bdcf(W, N)
 Interceptor.attach(b.add(0x56e490), {onEnter(a) { if (depth !== 1) return;
   send({e: 'defer', in_sel: inSel, from: fname(off(this.returnAddress)), node: ent(a[1])}); }});
 // node memory accounting 606036(N): every heap removal outside 6e8eb3/6e5802 shows up here with its caller
 Interceptor.attach(b.add(0x51ba80), {onEnter(a) { const c = fname(off(this.returnAddress));
   if (c === '6e5802' || c === '6e8eb3' || c === '6e3763') return;
   const t = tid(tree(a[0].add(0x18).readPointer().readPointer())); if (t < 0) return;
   send({e: 'acct', depth, in_sel: inSel, from: c, tree: t, node: a[0].readDouble()}); }});
 // deferred flush 68c1fd(W)
 Interceptor.attach(b.add(0x56e800), {onEnter(a) { if (depth !== 1) return;
   send({e: 'flush', in_sel: inSel, from: fname(off(this.returnAddress)), n: a[0].add(0x3fb8).readS32()}); }});
 // ---- M2-06c ----
 const at = (o, f) => Interceptor.attach(b.add(o), function () { if (cur) f(this.context); });
 const ret = (o, sites, f) => Interceptor.attach(b.add(o), {onEnter(a) { this.on = cur && sites.includes(off(this.returnAddress));
     if (this.on) { this.site = off(this.returnAddress); if (f.enter) f.enter.call(this, a); } },
   onLeave(r) { if (this.on && f.leave) f.leave.call(this, r); }});
 ret(0x509bf0, [0x5b3341], {leave(r) { send({e: 's70', f: '5e70c1', r: s32(r)}); }});
 ret(0x509b20, [0x5b342b], {leave(r) { send({e: 's70', f: '5e6efd', r: s32(r)}); }});
 at(0x5b34f9, c => send({e: 'drw', st: hx(c.rsp.add(0x70))}));
 at(0x5b3539, c => send({e: 'rpre', ecx: s32(c.rcx), esi: s32(c.rsi), focus: c.r15.add(0x2710).readS32(), work: wk(c)}));
 at(0x5b361d, c => send({e: 'rule', ecx: s32(c.rcx), esi: s32(c.rsi), r8b: u32(c.r8) & 0xff, r10d: s32(c.r10), n: s32(c.r11),
   st: hx(c.rsp.add(0x70)), work: wk(c)}));
 // pick list entries: [id, est, depth, excluded (I+0x41), bound]
 const pents = w => { const k = w.add(0x3fa8).readS32(), a = w.add(0x3fb0).readPointer(), r = [];
   for (let j = 0; j < k; j++) { const n = a.add(8 * j).readPointer(), i = n.add(0x18).readPointer();
     r.push([n.readDouble(), hx(i.add(0x20)), i.add(0x2c).readS32(), i.add(0x41).readU8(), hx(i.add(0x18))]); }
   return r; };
 at(0x5b44a9, c => { const k = cur.w.add(0x3fa8).readS32(), a = cur.w.add(0x3fb0).readPointer(), r = [];
   for (let j = 0; j < k; j++) { const i = a.add(8 * j).readPointer().add(0x18).readPointer();
     r.push([i.add(0x34).readS32(), hx(i.add(0x38)), i.add(0x40).readU8()]); }
   send({e: 'ents', ents: r});
   send({e: 'pick', idx: c.rsp.add(0x40).readS64().toNumber(), list: pents(cur.w), work: wk(c)}); });
 at(0x5b4506, c => { const sn = c.rsp.add(0x18).readPointer(), i = sn.add(0x18).readPointer(), t = cur.mm.add(0x620).readPointer().add(0x780).readPointer();
   send({e: 'tail', esi: s32(c.rsi), rbp_m: c.rbp.equals(cur.mm) ? 1 : 0, m2c: c.rbp.add(0x2c).readS32(), k: cur.w.add(0x30).readS32(),
     t138: t.add(0x138).readU64().toNumber(), t120: t.add(0x120).readU64().toNumber(), n: s32(c.r11), sel: sn.readDouble(),
     depth: i.add(0x2c).readS32(), x41: i.add(0x41).readU8(), focus: c.r15.add(0x2710).readS32()}); });
 const BODY = {0x5b3c2c: '6a', 0x5b3d1f: '6b', 0x5b42e6: '1a', 0x5b4402: '1b'};
 ret(0x5101a0, [0x5b465b, 0x5b4766, 0x5b3c2c, 0x5b3d1f, 0x5b42e6, 0x5b4402], {enter(a) { const dp = a[1].toInt32(); let n = a[0]; const chain = [];
     if (BODY[this.site]) { this.ev = {e: 'ranc', at: BODY[this.site], node: n.readDouble(), d: dp, depth: n.add(0x18).readPointer().add(0x2c).readS32()}; return; }
     const steps = Math.max(0, n.add(0x18).readPointer().add(0x2c).readS32() - dp);
     // only the start node's I is read by 5fb13c; ancestors may have released I, so the chain holds ids only
     for (let j = 0; j <= steps && j < 64 && !n.isNull(); j++) { chain.push(n.readDouble()); n = n.add(0x10).readPointer(); }
     const i0 = a[0].add(0x18).readPointer();
     this.ev = {e: 'anc', w: this.site === 0x5b465b ? 'sel' : 'it', d: dp, depth: i0.add(0x2c).readS32(), chain, bound: hx(i0.add(0x18))}; },
   leave(r) { this.ev.res = nid(r); send(this.ev); }});
 // rule 4: x = 5e6f57(M) at 0x5b3e35; record x[I+0x34] for the entries the loop reads (I+0x41 == 0, I+0x34 >= 0)
 ret(0x509b60, [0x5b3e3a], {leave(r) { const ev = {e: 'x4', null: r.isNull() ? 1 : 0, x: []};
     const k = cur.w.add(0x3fa8).readS32(), a = cur.w.add(0x3fb0).readPointer();
     for (let j = 0; j < k && !r.isNull(); j++) { const i = a.add(8 * j).readPointer().add(0x18).readPointer(), v = i.add(0x34).readS32();
       if (i.add(0x41).readU8() === 0 && v >= 0) ev.x.push([j, v, hx(r.add(8 * v))]); }
     send(ev); }});
 // rule 5: a7b4d6(&slot) at 0x5b3dc0
 ret(0x88a870, [0x5b3dc5], {enter(a) { this.slot = a[0]; }, leave(r) { send({e: 'r5', eax: s32(r), st: hx(this.slot)}); }});
 const f5f9a41 = new NativeFunction(b.add(0x50fa10), 'int', ['pointer']), f5e8239 = new NativeFunction(b.add(0x50abc0), 'double', ['pointer', 'int']);
 at(0x5b46f4, c => send({e: 'thr', t20: hx(c.rsp.add(0x20)), t28: hx(c.rsp.add(0x28)), t00: hx(c.rsp), d: c.rsp.add(0x30).readS32(),
   anc: nid(c.rsp.add(0x40).readPointer()), f9a41: f5f9a41(cur.mm) & 0xff, v8239: f2h(f5e8239(cur.mm, 2))}));
 ret(0x516bc0, [0x5b47aa, 0x5b47d5], {enter(a) { send({e: 'rm', node: a[1].readDouble(), stage: a[3].toInt32()}); }});
 at(0x5b49d1, c => send({e: 'tend', n: cur.w.add(0x3fa8).readS32(), list: ids(cur.w), work: wk(c)}));
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'mip_plunge_score', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason, counts, events})));
   const c = {}; for (const e of events) c[e.e] = (c[e.e] || 0) + 1;
   console.log(path.basename(model), outputText.trim(), JSON.stringify(c), JSON.stringify(counts), 'ok', ok, errors.join(' ').slice(0, 300));
   process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
