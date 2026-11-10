// M2-06b-2 (docs/m2/mip_node_key.md): read-only trace of the heap key I+0x50 that 627f3e writes for each child,
// with the inputs the asm reads, plus the M2-06a heap-insert events (same fields) for the join and the
// non-interference check. Three write groups (section 2 of the doc):
//   A 0x533b2c (before the M+0x40 test of 0x533be5/0x533bed): b = I+0x18, n = double at rsp+0x10, M = rbp,
//     parent I = rdi (depth +0x2c, LP +0x08 -> +0x80), ncols = [rsp+0x40] -> +0xe8 -> +0xc, child N = [rsp+0xc0][r14]
//   B 0x5334fe (before 0x533560/0x5335c8): children I = r10, r8, b = rdx+0x18, n = int rsp+0x48, M = rcx,
//     -1e100 side: child +0x90 int / int rsp+0x20, child N = [rsp+0xc0][0,1]
//   C 0x53244d (before 0x532451/0x53245b): child I = r9, b = I+0x18, M = r12, child N = rbp[r13]
//   D 6406da 0x534bf1 (before 0x534ca5/0x534cad, the A formula): child I = r10, b = I+0x18, n = double at rsp+0x80,
//     M = r13, parent I = rdi, ncols = [rsp+0x88] -> +0xe8 -> +0xc, child N = [rsp+0x68][rbp]
//   E 63c7ae 0x5356ae (before 0x5356b2/0x5356c2 and 0x5356f5/0x535705): children N = [rsp+0x50][0,1], source I = r8,
//     b = r8+0x18, -1e100 side: int rsp+0x28, M = r14
// 6406da and 63c7ae are called only by 627f3e. Node creation 5ff358 (out pointer a[2]) is logged with its caller so
// every inserted node can be traced to a creator.
// Probes are function-type (docs/lessons/tooling.md); xmm registers are not readable, so the written key is read from
// I+0x50 when 627f3e returns. Nodes are named by their id N+0x00; doubles are u64 hex. Nothing is written into the
// target. One invocation spawns exactly one original process.
// Usage: tools/gurun.sh node tools/mip_node_key.js MODEL OUTPUT.json.gz 'JSON_PARAMS' ['{"zero_obj":1}']
const fs = require('fs'), path = require('path'), crypto = require('crypto'), zlib = require('zlib');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [model, output, extra, optText] = process.argv.slice(2);
if (!model || !output) throw Error('usage: MODEL OUTPUT [JSON_PARAMS] [OPTIONS]');
if (fs.existsSync(output)) throw Error('output exists: ' + output);
const opts = JSON.parse(optText || '{}');
const root = path.resolve(__dirname, '../..');
const py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const expected = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c';
if (hash(binary) !== expected) throw Error('wrong Gurobi binary');
const funcs = fs.readFileSync(path.join(root, 're/decomp/index.tsv'), 'utf8').trim().split('\n').slice(1).map(l => l.split('\t'))
  .map(r => [parseInt(r[0], 16), parseInt(r[2], 10), r[1].replace(/^PRIVATE0+/, '')]).sort((a, b) => a[0] - b[0]);
const params = Object.assign({Threads: 1, Seed: 0, LogToConsole: 0}, JSON.parse(extra || '{}'));
// zero_obj: the objective is cleared after reading (a synthetic input for the M+0x40 == -1e100 side)
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\n` +
  (opts.zero_obj ? `m.setObjective(gp.LinExpr(), m.ModelSense)\n` : '') +
  `m.optimize()\nprint('DONE',m.Status,int(m.NodeCount),flush=True)`;
const agent = `
const FUNCS = ${JSON.stringify(funcs)};
function fname(o) { let lo = 0, hi = FUNCS.length - 1;
  while (lo < hi) { const mid = (lo + hi + 1) >> 1; if (FUNCS[mid][0] <= o) lo = mid; else hi = mid - 1; }
  const f = FUNCS[lo]; return f[0] <= o && o < f[0] + f[1] ? f[2] : 'raw:' + o.toString(16); }
const hx = p => p.readU64().toString(16).padStart(16, '0');
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return;
 const b = m.base, off = p => p.sub(b).toUInt32();
 let depth = 0, inSel = 0, calls = 0;
 // the M2-06a insert event, same fields and top-tree rule (tools/mip_node_select.js)
 const ent = n => { const i = n.add(0x18).readPointer();
   return [n.readDouble(), hx(i.add(0x18)), hx(i.add(0x50)), hx(i.add(0x20)), i.add(0x2c).readS32(), i.add(0x30).readS32(),
     i.add(0x28).readS32(), i.add(0x41).readU8()]; };
 const tree = mm => mm.add(0x620).readPointer().add(0x780).readPointer();
 const trees = new Map(); let treeSerial = 0;
 const top = t => { const k = t.toString(); if (!trees.has(k)) { trees.set(k, treeSerial++); send({e: 'tnew', tree: trees.get(k)}); }
   return trees.get(k); };
 const tid = t => trees.has(t.toString()) ? trees.get(t.toString()) : -1;
 Interceptor.attach(b.add(0x4e36e0), {onEnter() { depth++; }, onLeave() { depth--; }});
 Interceptor.attach(b.add(0x5b29d0), {onEnter(a) { this.top = depth === 1; if (!this.top) return; top(tree(a[0])); inSel = 1; },
   onLeave() { if (this.top) inSel = 0; }});
 Interceptor.attach(b.add(0x5b1e70), {onEnter(a) {
   const t = tree(a[0]), from = fname(off(this.returnAddress)); if (depth === 1 && from === '5daa82') top(t);
   if (tid(t) < 0) { send(['ins_other', depth]); return; }
   send({e: 'ins', in_sel: inSel, from, tree: tid(t), node: ent(a[1]), heap_n: t.add(0x120).readU64().toNumber()}); }});
 Interceptor.attach(b.add(0x5adfd0), {onEnter(a) { const t = a[1].readPointer(); if (t.isNull()) return;
   const k = t.toString(); if (trees.has(k)) trees.delete(k); }});
 // 627f3e / 6406da / 63c7ae calls: probes add child records to the innermost open call; keys are read when it returns
 const open = [];
 for (const [fa, fn] of [[0x5320c0, '627f3e'], [0x5341d0, '6406da'], [0x5350b0, '63c7ae']])
 Interceptor.attach(b.add(fa), {onEnter() { open.push({c: calls++, f: fn, nest: depth, from: fname(off(this.returnAddress)), w: []}); },
  onLeave(r) { const c = open.pop(), rc = r.toInt32();
   for (const w of c.w) for (const k of w.kids) {
     // the id is read at the probe; N can be freed before the return (fhnw-binpack4-4), so the key is read through I
     // and moved = 1 when N no longer holds the same id and I
     k.moved = k.n.readDouble() === k.id && k.n.add(0x18).readPointer().equals(k.i) ? 0 : 1; k.key = hx(k.i.add(0x50));
     k.bound = hx(k.i.add(0x18)); delete k.n; delete k.i; }
   if (c.w.length) send({e: 'call', c: c.c, f: c.f, nest: c.nest, from: c.from, rc, w: c.w}); }});
 Interceptor.attach(b.add(0x5177f0), {onEnter(a) { this.out = a[2]; this.from = fname(off(this.returnAddress)); },
  onLeave(r) { const n = this.out.readPointer();
   send({e: 'new', nest: depth, from: this.from, rc: r.toInt32(), id: n.isNull() || r.toInt32() ? null : n.readDouble()}); }});
 const rec = w => { if (!open.length) { send(['probe_outside', w.p]); return; } open[open.length - 1].w.push(w); };
 const kid = (n, i, more) => Object.assign({n, i, id: n.readDouble()}, more || {});
 Interceptor.attach(b.add(0x533b2c), function () { const x = this.context, rsp = x.rsp, M = x.rbp, P = x.rdi, i = x.r9;
   const n = rsp.add(0xc0).readPointer().add(8 * x.r14.toInt32()).readPointer();
   rec({p: 'A', m40: hx(M.add(0x40)), a4: M.add(0xa4).readS32(), nf: hx(rsp.add(0x10)), nf_lp: P.add(8).readPointer().add(0x80).readS32(),
     pdepth: P.add(0x2c).readS32(), ncols: rsp.add(0x40).readPointer().add(0xe8).readPointer().add(0xc).readS32(),
     parent: rsp.add(0x78).readPointer().readDouble(), parent_same: rsp.add(0x78).readPointer().add(0x18).readPointer().equals(P) ? 1 : 0,
     kids: [kid(n, i, {b: hx(i.add(0x18))})]}); });
 Interceptor.attach(b.add(0x5334fe), function () { const x = this.context, rsp = x.rsp, M = x.rcx, arr = rsp.add(0xc0).readPointer();
   const src = x.rdx, i0 = x.r10, i1 = x.r8;
   rec({p: 'B', m40: hx(M.add(0x40)), a4: M.add(0xa4).readS32(), n48: rsp.add(0x48).readS32(), n20: rsp.add(0x20).readS32(),
     b: hx(src.add(0x18)), src: src.equals(i0) ? 'kid0' : src.equals(i1) ? 'kid1' : 'other',
     kids: [kid(arr.readPointer(), i0, {i90: i0.add(0x90).readS32()}), kid(arr.add(8).readPointer(), i1, {i90: i1.add(0x90).readS32()})]}); });
 Interceptor.attach(b.add(0x53244d), function () { const x = this.context, i = x.r9;
   const n = x.rbp.add(8 * x.r13.toInt32()).readPointer();
   rec({p: 'C', m40: hx(x.r12.add(0x40)), kids: [kid(n, i, {b: hx(i.add(0x18))})]}); });
 Interceptor.attach(b.add(0x534bf1), function () { const x = this.context, rsp = x.rsp, M = x.r13, P = x.rdi, i = x.r10;
   const n = rsp.add(0x68).readPointer().add(8 * x.rbp.toInt32()).readPointer();
   rec({p: 'D', m40: hx(M.add(0x40)), a4: M.add(0xa4).readS32(), nf: hx(rsp.add(0x80)), pdepth: P.add(0x2c).readS32(),
     ncols: rsp.add(0x88).readPointer().add(0xe8).readPointer().add(0xc).readS32(), kids: [kid(n, i, {b: hx(i.add(0x18))})]}); });
 Interceptor.attach(b.add(0x5356ae), function () { const x = this.context, rsp = x.rsp, arr = rsp.add(0x50).readPointer();
   const n0 = arr.readPointer(), n1 = arr.add(8).readPointer();
   rec({p: 'E', m40: hx(x.r14.add(0x40)), b: hx(x.r8.add(0x18)), n28: rsp.add(0x28).readS32(),
     kids: [kid(n0, n0.add(0x18).readPointer()), kid(n1, n1.add(0x18).readPointer())]}); });
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'mip_node_key', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, options: opts, output: outputText.trim(), errors, ok,
     detach_reason: reason, counts, events})));
   const c = {}; for (const e of events) c[e.e] = (c[e.e] || 0) + 1;
   const w = {}; for (const e of events) if (e.e === 'call') for (const x of e.w) w[x.p] = (w[x.p] || 0) + 1;
   console.log(path.basename(model), outputText.trim(), JSON.stringify(c), JSON.stringify(w), JSON.stringify(counts), 'ok', ok,
     errors.join(' ').slice(0, 300));
   process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
