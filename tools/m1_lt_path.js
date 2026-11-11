// M1-02x: every BTRAN 221173 call (via 9e288 / 9e376) of one Gurobi LP solve with the M1-02k event fields (wrapper and
// its caller, m, input nnz, LU+0x108 / LU+0x110 bits, LU+0x180, LU+0x104, 9e376 U row r length, r, 21a941 runs,
// output nnz) followed by LU+0x140 at entry, the sparse L^T test hit and the e_r output. The hit is 0x12ec77
// `mov 0x140(%rbx),%esi`, reached only when nnz >= 0 and LU+0x104*nnz <= m (asm 0x12ec5a-0x12ec6d): [r11d = L^T
// input nnz, ebp = m, LU+0x140]. For dual e_r calls (9e376 back to 0xc51dd) the output idx (sparse) and value bits
// (nnz values, or m by row when dense) are kept. Read-only hooks; addresses are stored only as library offsets.
// usage: tools/gurun.sh node tools/m1_lt_path.js MODEL OUTPUT.json.gz [Method]
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
 const hx8 = p => { const v = new DataView(p.readByteArray(8)); return v.getUint32(4, true).toString(16).padStart(8, '0') + v.getUint32(0, true).toString(16).padStart(8, '0'); };
 const off = p => p.sub(b).toUInt32();
 const hexa = (p, n) => { if (n <= 0) return []; const v = new DataView(p.readByteArray(8 * n)), r = [];
   for (let j = 0; j < n; j++) r.push(v.getUint32(8 * j + 4, true).toString(16).padStart(8, '0') + v.getUint32(8 * j, true).toString(16).padStart(8, '0'));
   return r; };
 let wrap = null, k941 = 0, hit = null, batch = [];
 const flush = () => { if (batch.length) send(batch); batch = []; };
 for (const [a, w] of [[0x12de10, 288], [0x131710, 376]])
  Interceptor.attach(b.add(a), {onEnter(x) { wrap = [w, off(this.returnAddress), x[1].readS32()]; }});
 Interceptor.attach(b.add(0x130b70), {onEnter() { k941++; }});
 Interceptor.attach(b.add(0x12ec77), {onEnter() { const c = this.context;
   hit = [c.r11.toUInt32() | 0, c.rbp.toUInt32() | 0, c.rbx.add(0x140).readS32()]; }});
 Interceptor.attach(b.add(0x12de60), {
  onEnter(a) { const LU = a[0]; this.o = a[2]; k941 = 0; hit = null;
   this.m = LU.add(8).readS32(); this.er = wrap[0] === 376 && wrap[1] === 0xc51dd;
   let p4 = null, r = null;
   if (!a[3].isNull()) { r = a[1].add(8).readPointer().readS32(); const rb = LU.add(0xb0).readPointer(), re = LU.add(0xb8).readPointer();
     p4 = re.add(8 * r).readS64().toNumber() - rb.add(8 * r).readS64().toNumber(); }
   this.e = [wrap, this.m, a[1].readS32(), hx8(LU.add(0x108)), hx8(LU.add(0x110)), LU.add(0x180).readS32(),
     LU.add(0x104).readS32(), p4, r];
   this.x140 = LU.add(0x140).readS32(); },
  onLeave() { const n = this.o.readS32(); this.e.push(k941, n, this.x140, hit);
   if (this.er) this.e.push(n < 0 ? [] : Array.from(new Int32Array(this.o.add(8).readPointer().readByteArray(4 * n || 1))).slice(0, n),
     hexa(this.o.add(16).readPointer(), n < 0 ? this.m : n));
   batch.push(this.e); if (batch.length >= 2000) flush(); }});
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_lt_path', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason,
     fields: ['wrap[w,ret,in_nnz]', 'm', 'nnz', 'x108', 'x110', 'x180', 'x104', 'p4_urow', 'r', 'k941', 'out_nnz', 'x140',
              'hit[lt_nnz,m,x140]|null', 'e_r only: out_idx', 'e_r only: out_val'],
     events}) + '\n'));
   console.log(path.basename(model), outputText.trim(), events.length, 'ok', ok, errors.join(' ').slice(0, 300)); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
