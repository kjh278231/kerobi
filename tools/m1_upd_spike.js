// M1-02w-4: FT updates 05b257 (entry 0xf15c0, rdi = spx, LU = spx+0x470, r = spx+0x2f8) made while the dual e_r count
// (9e376 back to 0xc51dd, 0-based) is in K0..K1: [e_r count, r, spike LU+0x1c8 (nnz, idx, m values by position),
// eta cache LU+0x1d0 (nnz, idx, value bits, value pointer mod 16), LU+0x1d8, new diagonal LU+0x50[r] at exit (bits)].
// The new diagonal is eta.S over the cache (C:321-372, S = LU+0x1c8 values). Read-only hooks; no addresses are stored.
// usage: tools/gurun.sh node tools/m1_upd_spike.js MODEL OUTPUT.json.gz K0 K1 [Method]
const fs = require('fs'), path = require('path'), crypto = require('crypto'), zlib = require('zlib');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [model, output, k0, k1, method] = process.argv.slice(2);
if (!model || !output || k0 === undefined || k1 === undefined) throw Error('usage: MODEL OUTPUT K0 K1 [Method]');
if (fs.existsSync(output)) throw Error('output exists: ' + output);
const root = path.resolve(__dirname, '../..');
const py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const expected = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c';
if (hash(binary) !== expected) throw Error('wrong Gurobi binary');
const params = {Threads: 1, Method: Number(method ?? 1), Presolve: 0, OutputFlag: 0};
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\nm.optimize()\nprint('DONE',m.Status,int(m.IterCount),flush=True)`;
const agent = `const K0 = ${Number(k0)}, K1 = ${Number(k1)};
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return; const b = m.base;
 const hexa = (p, n) => { if (n <= 0) return []; const v = new DataView(p.readByteArray(8 * n)), r = [];
   for (let j = 0; j < n; j++) r.push(v.getUint32(8 * j + 4, true).toString(16).padStart(8, '0') + v.getUint32(8 * j, true).toString(16).padStart(8, '0'));
   return r; };
 const ints = (p, n) => n > 0 ? Array.from(new Int32Array(p.readByteArray(4 * n))) : [];
 let wrap = null, ker = 0, batch = [];
 const flush = () => { if (batch.length) send(batch); batch = []; };
 for (const [a, w] of [[0x12de10, 288], [0x131710, 376]])
  Interceptor.attach(b.add(a), {onEnter(x) { wrap = [w, this.returnAddress.sub(b).toUInt32()]; }});
 Interceptor.attach(b.add(0x12de60), {onEnter() { if (wrap[0] === 376 && wrap[1] === 0xc51dd) ker++; }});
 Interceptor.attach(b.add(0xf15c0), {
  onEnter(a) { this.keep = false; const k = ker - 1; if (k < K0 || k > K1) return;
   const LU = a[0].add(0x470).readPointer(), m = LU.add(8).readS32(), r = a[0].add(0x2f8).readS32();
   const sp = LU.add(0x1c8).readPointer(), sn = sp.readS32(), et = LU.add(0x1d0).readPointer(), en = et.readS32();
   this.keep = true; this.LU = LU; this.r = r;
   this.e = [k, r, [sn, ints(sp.add(8).readPointer(), sn), hexa(sp.add(16).readPointer(), m)],
     [en, ints(et.add(8).readPointer(), en), hexa(et.add(16).readPointer(), en), et.add(16).readPointer().and(0xf).toUInt32()],
     LU.add(0x1d8).readS32()]; },
  onLeave() { if (!this.keep) return;
   this.e.push(this.r >= 0 ? hexa(this.LU.add(0x50).readPointer().add(8 * this.r), 1)[0] : null);
   batch.push(this.e); if (batch.length >= 100) flush(); }});
 Interceptor.attach(Module.getGlobalExportByName('getppid'), {onEnter() { flush(); }});
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_upd_spike', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, range: [Number(k0), Number(k1)], output: outputText.trim(), errors, ok,
     detach_reason: reason, fields: ['er_count', 'r', 'spike [nnz, idx, m values]', 'eta [nnz, idx, val, val mod 16]', 'x1d8', 'diag_exit'],
     events}) + '\n'));
   console.log(path.basename(model), outputText.trim(), events.length, 'ok', ok, errors.join(' ').slice(0, 300)); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
