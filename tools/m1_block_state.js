// M1-01e: the 1da0b2 2x2 block-phase state on the simplex struct S around every LU factor 036ade (0xd4190, only caller
// 05b257) of one Method=1 run, read-only. S is args[0] (036ade's param_1/2 are doubles). Per call: the pivot count k
// (8b473 calls from 0xc58d5, as M1-02q), the return address, m (args[2]), an index of the distinct S pointer, and
// S+0x2c0/0x4e0/0x4e4 and the basis head (args[8], m = args[2]) at entry; S+0x4e0/0x4e4, the return value and the
// LU (S+0x470) fields +0xd8/+0xdc/+0xe0/+0xe4/+0x100 (1da0b2 C:1514-1534 writes them via param_21 = LU+0x20) at exit.
// usage: tools/gurun.sh node tools/m1_block_state.js MODEL OUTPUT.json.gz
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
 let k = 0; const ss = [];
 const st = s => [s.add(0x2c0).readS32(), s.add(0x4e0).readS32(), s.add(0x4e4).readS32()];
 Interceptor.attach(b.add(0x11f620), {onEnter() { if (this.returnAddress.sub(b).toUInt32() === 0xc58d5) k++; }});
 Interceptor.attach(b.add(0xd4190), {
  onEnter(a) { this.S = a[0]; let i = ss.findIndex(p => p.equals(a[0])); if (i < 0) { ss.push(a[0]); i = ss.length - 1; }
    const nb = a[2].toInt32(), h = []; for (let j = 0; j < nb; j++) h.push(a[8].add(4 * j).readS32());
    this.e = {k, ret: this.returnAddress.sub(b).toUInt32(), m: nb, n: a[3].toInt32(), s: i, enter: st(a[0]), head: h}; },
  onLeave(r) { const x = st(this.S), F = this.S.add(0x470).readPointer();
    send([Object.assign(this.e, {leave: [x[1], x[2]], rc: r.toInt32(), lu: [0xd8, 0xdc, 0xe0, 0xe4, 0x100].map(o => F.add(o).readS32())})]); }});
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_block_state', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason,
     events}) + '\n'));
   console.log(path.basename(model), outputText.trim(), events.length, 'ok', ok, errors.join(' ').slice(0, 300)); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
