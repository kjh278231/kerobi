// M2-03d-5: read-only scan for natural numeric retries (5ce67b retry block 0x4f53a8) and the context they run in.
// Counts like tools/mip_linear_child_scan.js (normalize before/after, retry, node status setter) with two more keys:
// the direct caller of the active 5ce67b (return address mapped through ../re/decomp/index.tsv) and the 5bbf7f nesting depth.
// Each retry also sends one detailed record (node id/parent/depth, MIP identity, LP dims, node basis pointers).
// Nothing is written into the target. One invocation spawns exactly one original process.
// Usage: tools/gurun.sh node tools/mip_retry_bnb_scan.js MODEL OUTPUT 'JSON_PARAMS'
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [model, output, extra] = process.argv.slice(2);
if (!model || !output) throw Error('usage: MODEL OUTPUT [JSON_PARAMS]');
const root = path.resolve(__dirname, '../..');
const py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const expected = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c';
if (hash(binary) !== expected) throw Error('wrong Gurobi binary');
// function table: [file offset, size, short name]; names are the 6-hex suffix of PRIVATE symbols
const funcs = fs.readFileSync(path.join(root, 're/decomp/index.tsv'), 'utf8').trim().split('\n').slice(1).map(l => l.split('\t'))
  .map(r => [parseInt(r[0], 16), parseInt(r[2], 10), r[1].replace(/^PRIVATE0+/, '')]).sort((a, b) => a[0] - b[0]);
const params = Object.assign({Threads: 1, Seed: 0, LogToConsole: 0}, JSON.parse(extra || '{}'));
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\nm.optimize()\nprint('DONE',m.Status,int(m.NodeCount),flush=True)`;
const agent = `
const FUNCS = ${JSON.stringify(funcs)};
function fname(o) { let lo = 0, hi = FUNCS.length - 1;
  while (lo < hi) { const mid = (lo + hi + 1) >> 1; if (FUNCS[mid][0] <= o) lo = mid; else hi = mid - 1; }
  const f = FUNCS[lo]; return f[0] <= o && o < f[0] + f[1] ? f[2] : 'raw:' + o.toString(16); }
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return;
 const b = m.base, off = p => p.sub(b).toUInt32();
 // node(), dims(), mipId() and the 5bbf7f depth counter are copied from tools/mip_retry_node_lp.js
 function node(n) {
  if (n.isNull()) return null;
  const i = n.add(0x18).readPointer(), mm = i.readPointer(), p = n.add(0x10).readPointer();
  return {id: n.readDouble(), parent: p.isNull() ? null : p.readDouble(), depth: i.add(0x2c).readS32(),
    root: n.equals(mm.add(0x790).readPointer()), status: i.add(0x28).readS32(), cache: mm.add(0xa8).readS32(), mip: mm.readS32(),
    counter_d4: mm.add(0x620).readPointer().add(0xd4).readS32()};
 }
 const dims = lp => { const d = lp.add(0xe8).readPointer(); return {rows: d.add(8).readS32(), cols: d.add(0xc).readS32()}; };
 function nodeBasis(I) { return {I94: I.add(0x94).readS32(), vb_null: I.add(0x98).readPointer().isNull(), cb_null: I.add(0xa0).readPointer().isNull(),
   dual_null: I.add(0xa8).readPointer().isNull(), packed_null: I.add(0xb0).readPointer().isNull()}; }
 const mains = [];
 let mainSerial = 0, mipDepth = 0, solveSerial = 0;
 const mipId = mm => { const r = mm.add(0x620).readPointer();
   return {mipmain: mains.length ? mains[mains.length - 1] : null, root_is_self: r.equals(mm), self_628: mm.add(0x628).readPointer().equals(mm), restarts: r.add(0x808).readS32()}; };
 Interceptor.attach(b.add(0x4e36e0), {onEnter(a) { this.p = a[0].add(0x1d8).readPointer(); this.dp = ++mipDepth; this.serial = mainSerial++; mains.push(this.serial);
   const d = a[0].add(0xe8).readPointer();
   send({event: 'mipmain', serial: this.serial, depth: this.dp, ret: fname(off(this.returnAddress)),
     model_rows: d.add(8).readS32(), model_cols: d.add(0xc).readS32(), presolved: this.p.isNull() ? null : dims(this.p)}); },
  onLeave() { if (mains.pop() !== this.serial) throw Error('mipmain nesting'); mipDepth--; }});
 // 5ce67b caller stack (it nests through sub-MIPs)
 const callers = [];
 Interceptor.attach(b.add(0x4f0e10), {onEnter() { this.c = fname(off(this.returnAddress)); this.s = solveSerial++; callers.push([this.c, this.s]); },
  onLeave() { const t = callers.pop(); if (!t || t[1] !== this.s) throw Error('5ce67b nesting'); }});
 const top = () => callers.length ? callers[callers.length - 1] : ['none', -1];
 for (const [name, o] of [['before', 0x4f1a98], ['after', 0x4f1aca]]) {
  Interceptor.attach(b.add(o), {onEnter() {
   const c = this.context, n = c.rsp.add(0x3b0).readPointer(), mm = n.add(0x18).readPointer().readPointer();
   send(['n', name, top()[0], mipDepth, n.equals(mm.add(0x790).readPointer()) ? 1 : 0, n.add(0x10).readPointer().isNull() ? 0 : 1,
     mm.add(0xa8).readS32(), c.r13.add(0x38).readS32()]);
  }});
 }
 Interceptor.attach(b.add(0x4f53a8), {onEnter() {
  const c = this.context, n = c.rsp.add(0x3b0).readPointer(), I = n.add(0x18).readPointer(), mm = I.readPointer(), t = top();
  send(['retry', t[0], mipDepth]);
  send(Object.assign({event: 'retry', caller: t[0], solve_serial: t[1], depth: mipDepth, lp_status: c.r13.add(0x38).readS32(),
    node: node(n), node_basis: nodeBasis(I), mip_lp: dims(mm.add(8).readPointer()), mip_status: mm.readS32()}, mipId(mm)));
 }});
 Interceptor.attach(b.add(0x5180f0), {onEnter(a) { send(['set', a[1].toInt32()]); }});
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
 session.detached.connect((reason) => { const ok = outputText.includes('DONE') && !errors.length;
   fs.mkdirSync(path.dirname(output), {recursive: true});
   fs.writeFileSync(output, JSON.stringify({schema: 1, kind: 'retry_bnb_scan', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason, counts, events}, null, 1) + '\n');
   const r = events.filter(e => e.event === 'retry').map(e => [e.caller, e.depth, e.node && e.node.parent !== null ? 'P' : '-'].join('/'));
   console.log(path.basename(model), outputText.trim(), 'retry', JSON.stringify(r), 'ok', ok, errors.join(' ').slice(0, 300)); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
