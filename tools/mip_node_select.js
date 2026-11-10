// M2-06a (docs/m2/mip_node_select_spec.md): read-only trace of the node containers around node selection.
// Top-level trees only: a tree is top when its root is inserted by 5daa82 at 5bbf7f nesting 1 or a 6e8eb3 call at
// nesting 1 selects from it. For those trees, at any nesting (a sub-MIP can prune the parent tree): every 6e8eb3 call
// (enter/leave: plunge list, deferred list, counters, and the global heap when that call pops it), 6e460c return (heap),
// every 6e5802 heap insert, 6e460c prune with the nodes it frees (606036), 6e3763 free; 68bdcf deferred adds at nesting 1.
// Nodes are named by their id N+0x00 (M+0x58 counter); doubles are u64 hex (big-endian digits). No pointers are stored.
// Nothing is written into the target. One invocation spawns exactly one original process.
// Usage: tools/gurun.sh node tools/mip_node_select.js MODEL OUTPUT.json.gz 'JSON_PARAMS'
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
 let depth = 0, inSel = 0, sel = 0, touched = 0;
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
   const pl = a[1].add(0x3fa8).readS32(); this.p0 = a[0].add(0x620).readPointer().add(0xd0).readS32();
   send({e: 'enter', sel: this.s, mode: a[3].toInt32(), ret: fname(off(this.returnAddress)), st: state(a[0], a[1], pl === 0)}); },
  onLeave(r) { if (!this.top) return; inSel = 0;
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'mip_node_select', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason, counts, events})));
   const c = {}; for (const e of events) c[e.e] = (c[e.e] || 0) + 1;
   console.log(path.basename(model), outputText.trim(), JSON.stringify(c), JSON.stringify(counts), 'ok', ok, errors.join(' ').slice(0, 300));
   process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
