// M1-02k: every BTRAN 221173 call (via 9e288 / 9e376) of one Gurobi LP solve: the wrapper and its caller, m, the
// input nnz, LU+0x108 / LU+0x110 bits, LU+0x180 (row etas), the 9e376 output slot, whether the sparse U^T kernel
// 21a941 ran, and the output nnz. Read-only hooks; addresses are stored only as offsets into the library.
// With SPEC (lines `k<TAB>pos,bits;pos,bits;...`, bits = u64 hex), after the k-th e_r BTRAN of 28c93 (9e376 back to
// 0xc51dd) it calls 9e288(LU, in, out, NULL) on each RHS of k three times: natural, LU+0x110 := -LU+0x108 (sparse U^T
// forced, (int)0*nnz <= m) and LU+0x110 := m + 2 (dense forced); LU+0x110 is read only by the C:89 test and 21a941
// reads LU+0x108 only. Each output and LU+0x200 is recorded; LU+0x110 and LU+0x200 get their bits back afterwards.
// NativeFunction calls from a hook are not intercepted, so the natural event stream only sees state effects.
// usage: tools/gurun.sh node tools/m1_btran_path.js MODEL OUTPUT.json.gz [Method] [SPEC]
const fs = require('fs'), path = require('path'), crypto = require('crypto'), zlib = require('zlib');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [model, output, method, specFile] = process.argv.slice(2);
if (!model || !output) throw Error('usage: MODEL OUTPUT [Method]');
if (fs.existsSync(output)) throw Error('output exists: ' + output);
const root = path.resolve(__dirname, '../..');
const py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const expected = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c';
if (hash(binary) !== expected) throw Error('wrong Gurobi binary');
const spec = {};
if (specFile) for (const l of fs.readFileSync(specFile, 'utf8').split('\n').filter(Boolean)) {
  const [k, xs] = l.split('\t'); (spec[k] = spec[k] || []).push(xs.split(';').filter(Boolean).map(e => e.split(','))); }
const params = {Threads: 1, Method: Number(method ?? 1), Presolve: 0, OutputFlag: 0};
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\nm.optimize()\nprint('DONE',m.Status,int(m.IterCount),flush=True)`;
const agent = `const spec = ${JSON.stringify(spec)};
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return; const b = m.base;
 const hx8 = p => { const v = new DataView(p.readByteArray(8)); return v.getUint32(4, true).toString(16).padStart(8, '0') + v.getUint32(0, true).toString(16).padStart(8, '0'); };
 const off = p => p.sub(b).toUInt32();
 let wrap = null, k941 = 0, batch = [], ker = 0;
 const bt = new NativeFunction(b.add(0x12de10), 'void', ['pointer', 'pointer', 'pointer', 'pointer']);
 const vec = (m, xs) => { const v = Memory.alloc(24), idx = Memory.alloc(4 * (m + 8)), val = Memory.alloc(8 * (m + 8));
   idx.writeByteArray(new ArrayBuffer(4 * (m + 8))); val.writeByteArray(new ArrayBuffer(8 * (m + 8)));
   xs.forEach(([p, h], i) => { idx.add(4 * i).writeS32(Number(p)); val.add(8 * i).writeU64(uint64('0x' + h)); });
   v.writeS32(xs.length); v.add(8).writePointer(idx); v.add(16).writePointer(val); return [v, idx, val]; };
 const synth = (LU, k) => { const m = LU.add(8).readS32(), x110 = LU.add(0x110).readU64(), x200 = LU.add(0x200).readU64();
   const neg = new DataView(new ArrayBuffer(8)); neg.setFloat64(0, -LU.add(0x108).readDouble(), true);
   const modes = [x110, uint64('0x' + neg.getUint32(4, true).toString(16).padStart(8, '0') + neg.getUint32(0, true).toString(16).padStart(8, '0')), null];
   spec[k].forEach((xs, i) => { const o = [];
     for (const md of modes) {
       if (md === null) LU.add(0x110).writeDouble(m + 2); else LU.add(0x110).writeU64(md);
       const a = vec(m, xs), c = vec(m, []); bt(LU, a[0], c[0], ptr(0));
       const n = c[0].readS32(), ip = c[0].add(8).readPointer(), vp = c[0].add(16).readPointer(), cnt = n < 0 ? m : n, val = [];
       for (let j = 0; j < cnt; j++) val.push(hx8(vp.add(8 * j)));
       o.push({nnz: n, idx: n < 0 ? [] : Array.from(new Int32Array(ip.readByteArray(4 * n || 1))).slice(0, n), val, w200: hx8(LU.add(0x200))}); }
     LU.add(0x110).writeU64(x110); LU.add(0x200).writeU64(x200);
     batch.push({k, i, m, x108: hx8(LU.add(0x108)), x110: hx8(LU.add(0x110)), x104: LU.add(0x104).readS32(), x180: LU.add(0x180).readS32(), o}); }); };
 const flush = () => { if (batch.length) send(batch); batch = []; };
 // 9e288(LU, in, out, work) and 9e376(LU, in, out, work): the caller of the wrapper names the BTRAN site
 for (const [a, w] of [[0x12de10, 288], [0x131710, 376]])
  Interceptor.attach(b.add(a), {onEnter(x) { wrap = [w, off(this.returnAddress), x[1].readS32()]; this.LU = x[0];
    this.er = w === 376 && wrap[1] === 0xc51dd; },
   onLeave() { if (!this.er) return; const k = ker++; if (spec[k]) synth(this.LU, k); }});
 Interceptor.attach(b.add(0x130b70), {onEnter() { k941++; }});
 Interceptor.attach(b.add(0x12de60), {
  onEnter(a) { const LU = a[0]; this.o = a[2]; k941 = 0;
   let p4 = null, r = null;
   if (!a[3].isNull()) { r = a[1].add(8).readPointer().readS32(); const rb = LU.add(0xb0).readPointer(), re = LU.add(0xb8).readPointer();
     p4 = re.add(8 * r).readS64().toNumber() - rb.add(8 * r).readS64().toNumber(); }  // U row r length (0: general test)
   this.e = [wrap, LU.add(8).readS32(), a[1].readS32(), hx8(LU.add(0x108)), hx8(LU.add(0x110)), LU.add(0x180).readS32(),
     LU.add(0x104).readS32(), p4, r]; },
  onLeave() { this.e.push(k941, this.o.readS32()); batch.push(this.e); if (batch.length >= 2000) flush(); }});
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_btran_path', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason,
     spec_sha256: specFile ? hash(specFile) : null,
     fields: ['wrap[w,ret,in_nnz]', 'm', 'nnz', 'x108', 'x110', 'x180', 'x104', 'p4_urow', 'r', 'k941', 'out_nnz'],
     events: events.filter(Array.isArray), synth: events.filter(e => !Array.isArray(e))}) + '\n'));
   console.log(path.basename(model), outputText.trim(), events.length, 'ok', ok, errors.join(' ').slice(0, 300)); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
