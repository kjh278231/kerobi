// M1-02w-2: the LU pool size F+0x158 written by 045a76 (0xe3130, reached by a tail jump from 12f54b), read-only.
// Per 045a76 call: return address, spx+0x28 (quad flag), m = spx+0x64, n = spx+0x68, nnz = spx+0x3d0, k = spx+0x50
// (GURO_PAR_SIMPLEXALLOCS), c = *(*(spx+0x4b8)+0xe8)+0x44 at entry; F = spx+0x470 and F+0x158/0x160/0x168 and the
// return value at exit. Per 036ade (0xd4190) call: F+0x158 at entry (only the first after each 045a76 is kept).
// EXPLORATION (M1-02w-2 §6): also F+0x170 at every 036ade exit and the c cell at entry/exit of 0cda5b (0x153cc0)
// and 14abfb (0x1cb7e0), the two writers of c. usage: tools/gurun.sh node bench/oracle/m1_02w2/explore_c.js MODEL OUTPUT.json.gz [PARAMS_JSON] [twice]
//   twice: optimize, reset(), optimize again in the same process (the second solve reads c from the first).
const fs = require('fs'), path = require('path'), crypto = require('crypto'), zlib = require('zlib');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [model, output, extra, twice] = process.argv.slice(2);
if (!model || !output) throw Error('usage: MODEL OUTPUT [PARAMS_JSON] [twice]');
if (fs.existsSync(output)) throw Error('output exists: ' + output);
const root = path.resolve(__dirname, '../../../..');
const py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const expected = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c';
if (hash(binary) !== expected) throw Error('wrong Gurobi binary');
const params = Object.assign({Threads: 1, Method: 1, Presolve: 0, OutputFlag: 0}, JSON.parse(extra || '{}'));
const solves = twice === 'twice' ? 2 : 1;
let code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\n`;
for (let i = 0; i < solves; i++) code += `${i ? 'm.reset()\n' : ''}m.optimize()\nprint('DONE',m.Status,int(m.IterCount),flush=True)\n`;
const agent = `Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return; const b = m.base;
 let fresh = false;
 const L = (p, o) => p.add(o).readS64().toNumber(), I = (p, o) => p.add(o).readS32();
 Interceptor.attach(b.add(0xe3130), {
  onEnter(a) { const s = a[0]; this.s = s;
    this.e = {ev: 'alloc', ret: this.returnAddress.sub(b).toUInt32(), quad: I(s, 0x28), m: I(s, 0x64), n: I(s, 0x68),
      nnz: L(s, 0x3d0), k: I(s, 0x50), c: I(s.add(0x4b8).readPointer().add(0xe8).readPointer(), 0x44)}; },
  onLeave(r) { const F = this.s.add(0x470).readPointer(); fresh = true;
    send([Object.assign(this.e, {x158: L(F, 0x158), x160: L(F, 0x160), x168: L(F, 0x168), rc: r.toInt32()})]); }});
 Interceptor.attach(b.add(0xd4190), {onEnter(a) { if (!fresh) return; fresh = false;
    send([{ev: 'factor', x158: L(a[0].add(0x470).readPointer(), 0x158)}]); }});
 Interceptor.attach(b.add(0xd4190), {onEnter(a) { this.F = a[0].add(0x470).readPointer(); },
   onLeave(r) { send([{ev: 'f170', x170: I(this.F, 0x170), x158: L(this.F, 0x158), rc: r.toInt32()}]); }});
 for (const [nm, off] of [['cda5b', 0x153cc0], ['14abfb', 0x1cb7e0]]) Interceptor.attach(b.add(off), {
   onEnter(a) { this.p = a[0]; send([{ev: nm + ':in', c: I(a[0].add(0xe8).readPointer(), 0x44)}]); },
   onLeave() { send([{ev: nm + ':out', c: I(this.p.add(0xe8).readPointer(), 0x44)}]); }});
 Interceptor.attach(Module.getGlobalExportByName('getppid'), {onEnter() {}});
}});`;
(async () => {
 fs.mkdirSync(path.dirname(path.resolve(output)), {recursive: true});
 const device = await frida.getLocalDevice(); const events = []; let outputText = ''; const errors = [];
 const pid = await device.spawn([py, '-c', code + 'import os\nos.getppid()'], {stdio: 'pipe'}); const session = await device.attach(pid);
 const script = await session.createScript(agent);
 script.message.connect(msg => { if (msg.type !== 'send') { errors.push(msg.stack || JSON.stringify(msg)); return; } events.push(...msg.payload); });
 device.output.connect((p, fd, data) => { if (p === pid) outputText += data.toString(); });
 session.detached.connect(reason => {
   if (outputText.includes('Gurobi is already running')) { console.log(outputText.trim()); process.exit(1); }
   const ok = (outputText.match(/DONE/g) || []).length === solves && !errors.length;
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_factspace', model: path.basename(model),
     model_sha256: hash(model), binary_sha256: expected, tool_sha256: hash(__filename), params, solves,
     output: outputText.trim(), errors, ok, detach_reason: reason, events}) + '\n'));
   console.log(path.basename(model), outputText.trim().replace(/\n/g, ' | '), events.length, 'ok', ok, errors.join(' ').slice(0, 300));
   process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
