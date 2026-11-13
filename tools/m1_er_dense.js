// M1-02w-4: stage values of dual e_r BTRAN calls (9e376 back to 0xc51dd) that take the sparse U^T kernel and the dense
// row-eta path (221173 C:1235-1376, no 0x12ec77 hit). For e_r calls K0..K1 (0-based e_r count) it keeps:
//   etas    the row etas at 221173 entry (LU+0x198, LU+0x180 records of 0x18 bytes: r, len, idx*, val*)
//   ut      the U^T output list after 21a941 returns (param_3: nnz, idx, value bits)
//   eta_out the dense vector at 0x12e912 (r13 = param_2, values at +0x10, m by row): the L^T input after the etas
//   out     the final output (as tools/m1_lt_path.js)
// plus [r, LU+0x180, out_nnz] for every e_r call. Read-only hooks; addresses are library offsets.
// usage: tools/gurun.sh node tools/m1_er_dense.js MODEL OUTPUT.json.gz K0 K1 [Method]
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
 const lst = q => { const n = q.readS32(); return [n, n > 0 ? ints(q.add(8).readPointer(), n) : [], hexa(q.add(16).readPointer(), n)]; };
 let wrap = null, ker = 0, cur = null, batch = [];
 const flush = () => { if (batch.length) send(batch); batch = []; };
 for (const [a, w] of [[0x12de10, 288], [0x131710, 376]])
  Interceptor.attach(b.add(a), {onEnter(x) { wrap = [w, this.returnAddress.sub(b).toUInt32()]; }});
 Interceptor.attach(b.add(0x130b70), {onLeave() { if (cur && cur.keep) { cur.k941++; cur.ut = lst(cur.p3); } }});
 Interceptor.attach(b.add(0x12e912), {onEnter() { if (cur && cur.keep) { const p2 = this.context.r13;
   cur.hit912 = p2.equals(cur.p2) ? 1 : -1; cur.eo = hexa(p2.add(16).readPointer(), cur.m); } }});
 Interceptor.attach(b.add(0x12de60), {
  onEnter(a) { const LU = a[0]; const er = wrap[0] === 376 && wrap[1] === 0xc51dd;
   cur = null; if (!er) return;
   const k = ker++; const n180 = LU.add(0x180).readS32();
   cur = {k, keep: K0 <= k && k <= K1, p2: a[1], p3: a[2], m: LU.add(8).readS32(), r: a[1].add(8).readPointer().readS32(), n180, k941: 0, ut: null, hit912: 0, eo: null};
   if (cur.keep) { const base = LU.add(0x198).readPointer(), et = [];
     for (let i = 0; i < n180; i++) { const e = base.add(0x18 * i), n = e.add(4).readS32();
       et.push([e.readS32(), ints(e.add(8).readPointer(), n), hexa(e.add(16).readPointer(), n)]); }
     cur.etas = et; } },
  onLeave() { if (!cur) return; const n = cur.p3.readS32(), c = cur;
   const ev = [c.k, c.r, c.n180, n];
   if (c.keep) ev.push({etas: c.etas, k941: c.k941, ut: c.ut, hit912: c.hit912, eta_out: c.eo,
     out_idx: n < 0 ? [] : ints(c.p3.add(8).readPointer(), n), out_val: hexa(c.p3.add(16).readPointer(), n < 0 ? c.m : n)});
   cur = null; batch.push(ev); if (batch.length >= 500) flush(); }});
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_er_dense', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, range: [Number(k0), Number(k1)], output: outputText.trim(), errors, ok,
     detach_reason: reason, fields: ['k', 'r', 'x180', 'out_nnz', 'in range: {etas [r, idx, val], k941, ut [nnz, idx, val], hit912, eta_out, out_idx, out_val}'],
     events}) + '\n'));
   console.log(path.basename(model), outputText.trim(), events.length, 'ok', ok, errors.join(' ').slice(0, 300)); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
