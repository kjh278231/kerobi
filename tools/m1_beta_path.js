// M1-02a: per dual pivot, the BTRAN output of 28c93 (09e376 return 0xc51dd) and the vector 8b473 sums into the DSE
// beta spx+0x310 (0x11f7ba..0x11fa4c), with the summation path and the stored bits. Read-only hooks
// (docs/m1/m1_02a_beta.md section 2). Addresses are never stored; only alignment / alias predicates.
// usage: tools/gurun.sh node tools/m1_beta_path.js MODEL OUTPUT.json.gz
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
const agent = `
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return; const b = m.base;
 const hx8 = p => { const v = new DataView(p.readByteArray(8)); return v.getUint32(4, true).toString(16).padStart(8, '0') + v.getUint32(0, true).toString(16).padStart(8, '0'); };
 const dbls = (p, n) => { const r = []; for (let k = 0; k < n; k++) r.push(hx8(p.add(8 * k))); return r; };
 const ints = (p, n) => n > 0 ? Array.from(new Int32Array(p.readByteArray(4 * n))) : [];
 let bt = null, cur = null, k = 0;  // bt: last 28c93 BTRAN output, cur: the open 8b473 sum
 // 28c93 (param_1 = spx) -> BTRAN 09e376(LU, in, out); out = {int nnz; int* idx; double* val}, nnz < 0 = dense (m values).
 // 28c93 C:149-163 compacts a dense output in place, so the dense values are read at the 09e376 return.
 let spx28 = null;
 Interceptor.attach(b.add(0xc4fe0), {onEnter(a) { spx28 = a[0]; }});
 Interceptor.attach(b.add(0x131710), {onEnter(a) { this.on = this.returnAddress.sub(b).toUInt32() === 0xc51dd; this.out = a[2]; },
  onLeave() { if (!this.on) return; const o = this.out, nnz = o.readS32(), m = spx28.add(0x64).readS32(), val = o.add(0x10).readPointer();
    bt = {out: o.toString(), nnz, idx: nnz >= 0 ? ints(o.add(8).readPointer(), nnz) : null, val: dbls(val, nnz < 0 ? m : nnz)}; }});
 Interceptor.attach(b.add(0x11f7ba), {onEnter() { const c = this.context, spx = c.r14, v = c.rbx, m = spx.add(0x64).readS32();
   const nnz = v.readS32(), n = nnz < 0 ? m : nnz, val = v.add(0x10).readPointer();
   const e = {k: k++, m, v_is_218: v.equals(spx.add(0x218).readPointer()), nnz, n,
     thr: hx8(spx.add(0x3a8)), idx: nnz >= 0 ? ints(v.add(8).readPointer(), nnz) : null, val: dbls(val, n),
     mod16: val.and(0xf).toInt32(), alias: (() => { const d = BigInt(spx.toString()) - BigInt(val.toString()); return !(d >= BigInt(8 * n) || -d >= 0xf20n); })(),
     path: null, beta: null, bt: null};
   if (bt) e.bt = {same_vector: bt.out === v.toString(), nnz: bt.nnz, idx: bt.idx, val: bt.val};
   bt = null; cur = e; }});
 Interceptor.attach(b.add(0x11f98b), {onEnter() { if (cur) cur.path = 'scalar'; }});
 for (const a of [0x11f97d, 0x11fa55]) Interceptor.attach(b.add(a), {onEnter() { if (!cur) return;
   if (cur.path === null) cur.path = 'simd'; cur.beta = hx8(this.context.r14.add(0x310)); send(cur); cur = null; }});
}});`;
(async () => {
 fs.mkdirSync(path.dirname(path.resolve(output)), {recursive: true});
 const device = await frida.getLocalDevice(); const events = []; let outputText = ''; const errors = [];
 const pid = await device.spawn([py, '-c', code], {stdio: 'pipe'}); const session = await device.attach(pid);
 const script = await session.createScript(agent);
 script.message.connect(msg => { if (msg.type !== 'send') { errors.push(msg.stack || JSON.stringify(msg)); return; } events.push(msg.payload); });
 device.output.connect((p, fd, data) => { if (p === pid) outputText += data.toString(); });
 session.detached.connect(reason => {
   // lock held by another Gurobi process: write nothing, so tools/gurun.sh can retry into the same path
   if (outputText.includes('Gurobi is already running')) { console.log(outputText.trim()); process.exit(1); }
   const ok = outputText.includes('DONE') && !errors.length;
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_beta_path', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason, events}) + '\n'));
   console.log(path.basename(model), outputText.trim(), 'events', events.length, 'ok', ok, errors.join(' ')); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
