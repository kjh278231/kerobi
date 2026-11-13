// M1-02y: at every FT update 05b257 (entry 0xf15c0, rdi = spx, LU = spx+0x470) of one Gurobi LP solve, the U^T e_r copy
// LU+0x1d0 that the new diagonal eta.S reads (C:321-372): [nnz, value pointer mod 16, pointer changed since the previous
// update (0/1), LU+0x100 prefix t, idx pointer mod 16, val - idx, val - struct (bytes), LU+8 m,
// W = *(LU+0x178 + 0x10) mod 16 (045a76 workspace), idx - W, spx+0x64, spx+0x68, spx+0x3c8, LU+0x188,
// r = spx+0x2f8 at entry, the pool value base LU+0x150 mod 16, the 036ade (refactor) entries so far, the return value and the diagonal LU+0x50[r] at exit (bits; null when r < 0)]. A value array 8 but not 16-byte aligned peels one term into lane 0 (C:332-337).
// Read-only hook; no addresses are stored.
// usage: tools/gurun.sh node tools/m1_eta_align.js MODEL OUTPUT.json.gz [Method]
const fs = require('fs'), path = require('path'), crypto = require('crypto'), zlib = require('zlib');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [model, output, method] = process.argv.slice(2);
if (!model || !output) throw Error('usage: MODEL OUTPUT [Method]');
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
 let prev = null, batch = [], nref = 0;
 Interceptor.attach(b.add(0xd4190), {onEnter() { nref++; }});
 const hx8 = p => { const v = new DataView(p.readByteArray(8)); return v.getUint32(4, true).toString(16).padStart(8, '0') + v.getUint32(0, true).toString(16).padStart(8, '0'); };
 Interceptor.attach(b.add(0xf15c0), {onEnter(a) { const LU = a[0].add(0x470).readPointer(), s = LU.add(0x1d0).readPointer();
   const v = s.add(0x10).readPointer(); const ix = s.add(8).readPointer(); const W = LU.add(0x178).readPointer().add(0x10).readPointer();
   batch.push([s.readS32(), v.and(0xf).toUInt32(), prev === null || !v.equals(prev) ? 1 : 0, LU.add(0x100).readS32(), ix.and(0xf).toUInt32(),
     v.sub(ix).toInt32(), v.sub(s).toInt32(), LU.add(8).readS32(), W.and(0xf).toUInt32(), ix.sub(W).toInt32(), a[0].add(0x64).readS32(),
     a[0].add(0x68).readS32(), a[0].add(0x3c8).readS32(), LU.add(0x188).readS32()]);
   this.rec = batch[batch.length - 1]; this.LU = LU; this.r = a[0].add(0x2f8).readS32();
   this.rec.push(this.r, LU.add(0x150).readPointer().and(0xf).toUInt32(), nref);
   prev = v; },
  onLeave(ret) { const d = this.r >= 0 ? hx8(this.LU.add(0x50).readPointer().add(8 * this.r)) : null;
   this.rec.push(ret.toInt32(), d); if (batch.length >= 2000) { send(batch); batch = []; } }});
 Interceptor.attach(Module.getGlobalExportByName('getppid'), {onEnter() { if (batch.length) send(batch); batch = []; }});
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_eta_align', fields: ['nnz_1d0', 'val_mod16', 'ptr_changed', 'x100', 'idx_mod16', 'val-idx', 'val-struct', 'm', 'W_mod16', 'idx-W', 'spx64', 'spx68', 'spx3c8', 'LU188', 'r(spx+0x2f8)', 'LU150_mod16', 'refactors(036ade entries) so far', 'ret', 'LU+0x50[r] at exit'], model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason,
     events}) + '\n'));
   console.log(path.basename(model), outputText.trim(), events.length, 'ok', ok, errors.join(' ').slice(0, 300)); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
