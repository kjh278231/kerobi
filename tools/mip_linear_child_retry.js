// Read-only natural capture of the numeric-retry block in 5ce67b and of node-status-3 setter callers.
// One invocation spawns exactly one original process. Nothing is written into the target.
// Usage: tools/gurun.sh node tools/mip_linear_child_retry.js retry|set3 MODEL OUTPUT 'JSON_PARAMS'
//  retry: for every 5ce67b solve that reaches the retry block (ELF 0x4f53a8), emit that solve's ordered events.
//  set3:  for every 5ff754(node, 3) call, emit the return address and an accurate backtrace (file offsets).
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [mode, model, output, extra] = process.argv.slice(2);
if (!['retry', 'set3'].includes(mode) || !model || !output) throw Error('usage: retry|set3 MODEL OUTPUT [JSON_PARAMS]');
const root = path.resolve(__dirname, '../..');
const py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const expected = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c';
if (hash(binary) !== expected) throw Error('wrong Gurobi binary');
const params = Object.assign({Threads: 1, Seed: 0, LogToConsole: 0}, JSON.parse(extra || '{}'));
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\nm.optimize()\nprint('DONE',m.Status,int(m.NodeCount),flush=True)`;
const agent = `
const mode = ${JSON.stringify(mode)};
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return;
 const b = m.base, off = p => p.sub(b).toUInt32();
 // Same field offsets as mip_linear_child_experiment.js (verified there against 627f3e/5ff358 C).
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
 if (mode === 'set3') {
  Interceptor.attach(b.add(0x5180f0), {onEnter(a) { if (a[1].toInt32() !== 3) return;
   (buf ? (e => buf.push(e)) : send)({event: 'set3', ret: off(this.returnAddress), backtrace: Thread.backtrace(this.context, Backtracer.ACCURATE).slice(0, 8).map(p => p.compare(b) >= 0 && p.sub(b).compare(ptr(0x4000000)) < 0 ? off(p) : null), node: node(a[0])}); }});
  let inFallback = 0, seq = 0, buf = null; const marked = new Set();
  Interceptor.attach(b.add(0x566030), {onEnter(a) { inFallback++; this.n = a[0]; this.k = ++seq; buf = [{event: 'fallback:enter', seq: this.k, node: node(a[0])}]; },
   onLeave(r) { inFallback--; const nd = node(this.n); buf.push({event: 'fallback:leave', seq: this.k, ret: r.toInt32(), node: nd});
    if (nd && nd.status === 3) { marked.add(nd.id); for (const e of buf) send(e); }
    else send({event: 'fallback_count', node_status: nd ? nd.status : null, lp_status: buf.filter(e => e.event === 'fallback_simplex:leave').map(e => e.lp ? e.lp.status : null)});
    buf = null; }});
  Interceptor.attach(b.add(0x153cc0), {onEnter(a) { this.on = inFallback > 0; if (!this.on) return; this.k = ++seq; this.lp = a[0]; const st = lpstate(this.lp);
    this.dims = st && st.ws_cols !== null ? [st.ws_cols, st.ws_rows] : null; this.vb = a[2]; this.cb = a[3];
    buf.push({event: 'fallback_simplex:enter', seq: this.k, iteration_limit: this.lp.add(0x100).readPointer().add(0x2960).readDouble(), lp: st,
      vbasis: this.dims && !a[2].isNull() ? ints(a[2], this.dims[0]) : null, cbasis: this.dims && !a[3].isNull() ? ints(a[3], this.dims[1]) : null}); },
   onLeave(r) { if (!this.on) return; buf.push({event: 'fallback_simplex:leave', seq: this.k, ret: r.toInt32(), iteration_limit: this.lp.add(0x100).readPointer().add(0x2960).readDouble(), lp: lpstate(this.lp),
      vbasis_out: this.dims && !this.vb.isNull() ? ints(this.vb, this.dims[0]) : null, cbasis_out: this.dims && !this.cb.isNull() ? ints(this.cb, this.dims[1]) : null}); }});
  let branching = null;
  Interceptor.attach(b.add(0x5320c0), {onEnter(a) { const nd = node(a[0].readPointer()); this.on = !!(nd && nd.status === 3 && marked.has(nd.id)); if (!this.on) return;
   // node ids are reused: only the status-3 branch of the marked node counts
   marked.delete(nd.id); this.k = ++seq; branching = nd.id; send({event: 'branch:enter', seq: this.k, node: nd}); },
   onLeave(r) { if (!this.on) return; branching = null; send({event: 'branch:leave', seq: this.k, ret: r.toInt32()}); }});
  Interceptor.attach(b.add(0x5177f0), {onEnter(a) { this.on = branching !== null; this.out = a[2]; },
   onLeave(r) { if (this.on) send({event: 'create:leave', ret: r.toInt32(), branching_node: branching, child: r.toInt32() === 0 ? node(this.out.readPointer()) : null}); }});
  Interceptor.attach(b.add(0x5180f0), {onEnter(a) { if (a[1].toInt32() === 10) send({event: 'set10', ret: off(this.returnAddress), node: node(a[0])}); }});
  return;
 }
 const solves = [];   // stack of {call, lp, buf, retry}
 let nextCall = 0;
 const top = () => solves.length ? solves[solves.length - 1] : null;
 const rec = e => { const s = top(); if (s) s.buf.push(e); };
 Interceptor.attach(b.add(0x4f0e10), {
  onEnter(a) { this.h = a[0]; this.n = a[0].readPointer(); this.call = ++nextCall; this.lp = a[2].add(8).readPointer();
   solves.push({call: this.call, lp: this.lp, buf: [], retry: false});
   rec({event: 'solve:enter', call: this.call, node: node(this.n), lp: lpstate(this.lp)}); },
  onLeave(r) { const after = this.h.readPointer(), s = solves.pop();
   if (!s || s.call !== this.call) throw Error('solve nesting mismatch');
   // Holder may be replaced; read the node only through the current holder.
   s.buf.push({event: 'solve:leave', call: this.call, ret: r.toInt32(), holder_changed: !after.equals(this.n), node: node(after), lp: lpstate(this.lp)});
   if (s.retry) send({event: 'retry_solve', events: s.buf}); }});
 for (const [name, o] of [['lpdriver', 0x14acb0], ['simplex', 0x153cc0], ['reopt', 0x1cb7e0]]) {
  Interceptor.attach(b.add(o), {
   onEnter(a) { const s = top(); this.on = !!(s && s.retry); if (!this.on) return; this.call = ++nextCall; this.lp = a[0];
    const st = lpstate(this.lp); this.dims = name === 'lpdriver' || !st || st.ws_cols === null ? null : [st.ws_cols, st.ws_rows]; this.vb = a[2]; this.cb = a[3];
    rec({event: name + ':enter', call: this.call, lp: st, vbasis: this.dims ? ints(a[2], this.dims[0]) : null, cbasis: this.dims ? ints(a[3], this.dims[1]) : null}); },
   onLeave(r) { if (this.on) rec({event: name + ':leave', call: this.call, ret: r.toInt32(), lp: lpstate(this.lp),
    vbasis_out: this.dims ? ints(this.vb, this.dims[0]) : null, cbasis_out: this.dims ? ints(this.cb, this.dims[1]) : null}); }});
 }
 for (const [name, o] of [['normalize_before', 0x4f1a98], ['normalize_after', 0x4f1aca], ['retry', 0x4f53a8]]) {
  Interceptor.attach(b.add(o), {onEnter() { const s = top(); if (!s) return; const c = this.context;
   if (name === 'retry') s.retry = true;
   rec({event: name, node: node(c.rsp.add(0x3b0).readPointer()), lp_status: c.r13.add(0x38).readS32(), lp: lpstate(s.lp)}); }});
 }
 Interceptor.attach(b.add(0x5180f0), {onEnter(a) { rec({event: 'set', value: a[1].toInt32(), ret: off(this.returnAddress), node: node(a[0])}); }});
 Interceptor.attach(b.add(0x518170), {onEnter(a) { this.on = !!top(); if (this.on) rec({event: 'finish:enter', node: node(a[0])}); },
  onLeave(r) { if (this.on) rec({event: 'finish:leave', ret: r.toInt32()}); }});
}});
`;
(async () => {
 const device = await frida.getLocalDevice(); const events = []; let outputText = ''; const errors = [];
 const pid = await device.spawn([py, '-c', code], {stdio: 'pipe'}); const session = await device.attach(pid);
 const script = await session.createScript(agent);
 const counts = {};
 script.message.connect(msg => { if (msg.type !== 'send') { errors.push(msg.stack || JSON.stringify(msg)); return; }
   const e = msg.payload; if (e.event !== 'fallback_count') { events.push(e); return; }
   const k = 'node' + e.node_status + ':lp' + e.lp_status.join(','); counts[k] = (counts[k] || 0) + 1; });
 device.output.connect((p, fd, data) => { if (p === pid) outputText += data.toString(); });
 session.detached.connect((reason, crash) => { const ok = outputText.includes('DONE') && !errors.length;
   fs.mkdirSync(path.dirname(output), {recursive: true});
   fs.writeFileSync(output, JSON.stringify({schema: 1, kind: 'natural_' + mode, model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, params, output: outputText.trim(), errors, ok, detach_reason: reason, fallback_counts: counts, events}, null, 1) + '\n');
   console.log(mode, path.basename(model), outputText.trim(), 'events', events.length, 'ok', ok, errors.join(' ')); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
