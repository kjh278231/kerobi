// M1-02w: the element pool size around every LU factor 036ade (0xd4190, only caller 05b257) of one Method=1 run,
// read-only. Per call: the pivot count k (8b473 calls from 0xc58d5), the return address, m, n, the LU F = S+0x470 and its
// F+0x158 (pool size, param_15[2]) / F+0x160 / F+0x168 at entry, F+0x158 / F+0x168 and the return value at exit, and
// the dense-switch markers inside the call: dense:enter (0xd88bf, active n), local_70 mode (0xd85b4), MKL dgetrf
// (0x9fef00, n). Hook points as tools/push_dense_agent.js (M3-04c-21i/21j).
// usage: tools/gurun.sh node tools/m1_pool_size.js MODEL OUTPUT.json.gz [Method]
const fs = require('fs'), path = require('path'), crypto = require('crypto'), zlib = require('zlib');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [model, output, method] = process.argv.slice(2);
if (!model || !output) throw Error('usage: MODEL OUTPUT');
if (fs.existsSync(output)) throw Error('output exists: ' + output);
const root = path.resolve(__dirname, '../..');
const py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const expected = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c';
if (hash(binary) !== expected) throw Error('wrong Gurobi binary');
const params = {Threads: 1, Method: Number(method ?? 1), Presolve: 0, OutputFlag: 0};
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\nm.optimize()\nprint('DONE',m.Status,int(m.IterCount),flush=True)`;
const agent = `Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return; const b = m.base;
 let k = 0, cur = null;
 const L = (F, o) => F.add(o).readS64().toNumber();
 Interceptor.attach(b.add(0x11f620), {onEnter() { if (this.returnAddress.sub(b).toUInt32() === 0xc58d5) k++; }});
 Interceptor.attach(b.add(0xd4190), {
  onEnter(a) { const F = a[0].add(0x470).readPointer(); this.F = F;
    cur = {k, ret: this.returnAddress.sub(b).toUInt32(), m: a[2].toInt32(), n: a[3].toInt32(),
      enter: [L(F, 0x158), L(F, 0x160), L(F, 0x168)], dense: [], mode70: 0, dgetrf: []}; this.e = cur; },
  onLeave(r) { const e = this.e; cur = null;
    send([Object.assign(e, {leave: [L(this.F, 0x158), L(this.F, 0x168)], rc: r.toInt32()})]); }});
 Interceptor.attach(b.add(0xd88bf), function () { if (cur) cur.dense.push(this.context.rsp.add(0x100).readS32()); });
 Interceptor.attach(b.add(0xd85b4), function () { if (cur) cur.mode70++; });
 Interceptor.attach(b.add(0x9fef00), {onEnter(a) { if (cur) cur.dgetrf.push(a[0].readS32()); }});
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_pool_size', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason,
     events}) + '\n'));
   console.log(path.basename(model), outputText.trim(), events.length, 'ok', ok, errors.join(' ').slice(0, 300)); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
