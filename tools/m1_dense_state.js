// M1-02w-3: every LU factor 036ade (0xd4190, only caller 05b257) of one Method=1 run, read-only: the union of
// tools/m1_block_state.js and tools/m1_pool_size.js plus the Markowitz tol and the factor's diagonal.
// Per call i (events in call order): pivot count k (8b473 calls from 0xc58d5), return address, m, n, S index, block state
// S+0x2c0/0x4e0/0x4e4 and basis head (args[8]) at entry, S+0x390 (tol, 05b257 passes it as 036ade's 2nd double) bits,
// F = S+0x470 +0x158/+0x160/+0x168 at entry; at exit S+0x4e0/0x4e4, F+0x158/+0x168, the return value, LU fields
// +0xd8/+0xdc/+0xe0/+0xe4/+0x100; inside: dense:enter (0xd88bf, active n), local_70 mode (0xd85b4), MKL dgetrf (0x9fef00).
// Diagonal: at the first 21a941 (0x130b70, rdi = LU = F) after a call that returned 0 the diagonal LU+0x50 (m bits) when
// the update count F+0x18c is still 0 (a fresh factor), sent as {diag_of: i, upd, diag}.
// usage: tools/gurun.sh node tools/m1_dense_state.js MODEL OUTPUT.json.gz
const fs = require('fs'), path = require('path'), crypto = require('crypto'), zlib = require('zlib');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [model, output] = process.argv.slice(2);
if (!model || !output) throw Error('usage: MODEL OUTPUT');
if (fs.existsSync(output)) throw Error('output exists: ' + output);
const root = path.resolve(__dirname, '../..');
const py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const expected = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c';
if (hash(binary) !== expected) throw Error('wrong Gurobi binary');
const params = {Threads: 1, Method: 1, Presolve: 0, OutputFlag: 0};
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\nm.optimize()\nprint('DONE',m.Status,int(m.IterCount),flush=True)`;
const agent = `Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return; const b = m.base;
 const hexa = (p, n) => { if (n <= 0) return []; const v = new DataView(p.readByteArray(8 * n)), r = [];
   for (let j = 0; j < n; j++) r.push(v.getUint32(8 * j + 4, true).toString(16).padStart(8, '0') + v.getUint32(8 * j, true).toString(16).padStart(8, '0'));
   return r; };
 let k = 0, i = 0, cur = null, pend = null; const ss = [];
 const st = s => [s.add(0x2c0).readS32(), s.add(0x4e0).readS32(), s.add(0x4e4).readS32()];
 const L = (F, o) => F.add(o).readS64().toNumber();
 Interceptor.attach(b.add(0x11f620), {onEnter() { if (this.returnAddress.sub(b).toUInt32() === 0xc58d5) k++; }});
 Interceptor.attach(b.add(0xd4190), {
  onEnter(a) { const S = a[0], F = S.add(0x470).readPointer(); this.S = S; this.F = F; pend = null;
    let s = ss.findIndex(p => p.equals(S)); if (s < 0) { ss.push(S); s = ss.length - 1; }
    const nb = a[2].toInt32(), h = []; for (let j = 0; j < nb; j++) h.push(a[8].add(4 * j).readS32());
    cur = {i: i++, k, ret: this.returnAddress.sub(b).toUInt32(), m: nb, n: a[3].toInt32(), s, enter: st(S), head: h,
      tol: hexa(S.add(0x390), 1)[0], pool: [L(F, 0x158), L(F, 0x160), L(F, 0x168)], dense: [], mode70: 0, dgetrf: []};
    this.e = cur; },
  onLeave(r) { const e = this.e, x = st(this.S), F = this.F; cur = null;
    Object.assign(e, {leave: [x[1], x[2]], pool_leave: [L(F, 0x158), L(F, 0x168)], rc: r.toInt32(),
      lu: [0xd8, 0xdc, 0xe0, 0xe4, 0x100].map(o => F.add(o).readS32())});
    send([e]); if (e.rc === 0) pend = {F, i: e.i}; }});
 Interceptor.attach(b.add(0xd88bf), function () { if (cur) cur.dense.push(this.context.rsp.add(0x100).readS32()); });
 Interceptor.attach(b.add(0xd85b4), function () { if (cur) cur.mode70++; });
 Interceptor.attach(b.add(0x9fef00), {onEnter(a) { if (cur) cur.dgetrf.push(a[0].readS32()); }});
 Interceptor.attach(b.add(0x130b70), {onEnter(a) { if (!pend || !a[0].equals(pend.F)) return; const LU = a[0], p = pend; pend = null;
   const upd = LU.add(0x18c).readS32(), mm = LU.add(8).readS32();
   send([{diag_of: p.i, upd, m: mm, diag: upd === 0 ? hexa(LU.add(0x50).readPointer(), mm) : []}]); }});
 Interceptor.attach(Module.getGlobalExportByName('getppid'), {onEnter() {}});
}});`;
(async () => {
 fs.mkdirSync(path.dirname(path.resolve(output)), {recursive: true});
 const device = await frida.getLocalDevice(); const events = []; let outputText = ''; const errors = [];
 const pid = await device.spawn([py, '-c', code + '\nimport os\nos.getppid()'], {stdio: 'pipe'}); const session = await device.attach(pid);
 const script = await session.createScript(agent);
 script.message.connect(msg => { if (msg.type !== 'send') { errors.push(msg.stack || JSON.stringify(msg)); return; } events.push(...msg.payload); });
 device.output.connect((p, fd, data) => { if (p === pid) outputText += data.toString(); });
 session.detached.connect(reason => {
   if (outputText.includes('Gurobi is already running')) { console.log(outputText.trim()); process.exit(1); }
   const ok = outputText.includes('DONE') && !errors.length;
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_dense_state', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason,
     events}) + '\n'));
   console.log(path.basename(model), outputText.trim(), events.length, 'ok', ok, errors.join(' ').slice(0, 300)); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
