// M1-02m: per dual pivot (8b473 called from 28c93), the sparse-U kernels of the two-RHS FTRAN 09d112: every 21686f
// and 21841f call made from 09d112 with its alpha_q-side input (copied on entry: 21686f compacts it in place) and
// output, and the U factor as the kernel reads it (U = LU+0x20: row -> U position +0x20, U position -> row +0x28,
// diagonal +0x30, column start / tail-prefix split / end +0x68 / +0x78 / +0x70, values +0x80, rows +0x88, LU+0xf0
// list +0xd0, LU+0xf8 marks +0xd8, list length +0xcc, prefix t +0xe0, D = +0xe8 + +0xf0).
// Read-only hooks (docs/m1/m1_02m_usolve.md section 2). Addresses are never stored.
// usage: tools/gurun.sh node tools/m1_usolve.js MODEL OUTPUT.json.gz
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
 const longs = (p, n) => { const r = []; for (let k = 0; k < n; k++) r.push(p.add(8 * k).readS64().toNumber()); return r; };
 // vector {int nnz; int* idx @+8; double* val @+0x10}; nnz < 0 = dense (values not read)
 const vec = p => { const nnz = p.readS32(); return nnz >= 0 ? {nnz, idx: ints(p.add(8).readPointer(), nnz),
   val: dbls(p.add(0x10).readPointer(), nnz)} : {nnz}; };
 // the U factor as 21686f / 21841f read it; cols by U position >= t: [row, tail-part rows, prefix-part rows,
 // tail-part values, prefix-part values]; pre_nonempty counts columns below t that are not empty
 const udump = (U, m) => {
   const t = U.add(0xe0).readS32(), ec = U.add(0xcc).readS32(), ord = ints(U.add(0x28).readPointer(), m);
   const beg = longs(U.add(0x68).readPointer(), m), mid = longs(U.add(0x78).readPointer(), m),
     end = longs(U.add(0x70).readPointer(), m), rp = U.add(0x88).readPointer(), vp = U.add(0x80).readPointer();
   const cols = [];
   for (let p = t; p < m; p++) { const i = ord[p];
     cols.push([i, ints(rp.add(4 * beg[i]), mid[i] - beg[i]), ints(rp.add(4 * mid[i]), end[i] - mid[i]),
       dbls(vp.add(8 * beg[i]), mid[i] - beg[i]), dbls(vp.add(8 * mid[i]), end[i] - mid[i])]); }
   let pre_nonempty = 0;
   for (let p = 0; p < t; p++) if (end[ord[p]] !== beg[ord[p]]) pre_nonempty++;
   return {t, ec, D: [hx8(U.add(0xe8)), hx8(U.add(0xf0))], ord, f0: ints(U.add(0xd0).readPointer(), ec),
     f8: ints(U.add(0xd8).readPointer(), m), diag: dbls(U.add(0x30).readPointer(), m), cols, pre_nonempty}; };
 let k = 0, cur = null, ft = null, LU = null;
 Interceptor.attach(b.add(0x11f620), {
  onEnter(a) { this.on = this.returnAddress.sub(b).toUInt32() === 0xc58d5; if (!this.on) return;
    cur = {type: 'P', k: k++, ft: []}; },
  onLeave() { if (!this.on) return; send(cur); cur = null; }});
 Interceptor.attach(b.add(0x125f50), {
  onEnter(a) { this.on = cur !== null && this.returnAddress.sub(b).toUInt32() === 0x120200; if (!this.on) return;
    LU = a[0]; this.a = a[1];
    ft = {m: LU.add(8).readS32(), upd: LU.add(0x18c).readS32(), calls: []}; },
  onLeave() { if (!this.on) return; const o = this.a.readS32(); ft.aq_out = o >= 0 ? {nnz: o, idx: ints(this.a.add(8).readPointer(), o)} : {nnz: o};
    cur.ft.push(ft); ft = null; }});
 // 21686f(m, U, in, out, mark, work, &work) from 0x1295e0; 21841f(LU, m, U, in, out, rho_in, rho_out, mark, work,
 // &work) from 0x129547 (09d112 C:1804, C:1883; tol in xmm0)
 const ks = {'21686f': [0x1223e0, 0x1295e0, 0], '21841f': [0x12c300, 0x129547, 1]};
 for (const f of Object.keys(ks)) { const [fa, site, s] = ks[f];
  Interceptor.attach(b.add(fa), {
   onEnter(a) { this.on = ft !== null && this.returnAddress.sub(b).toUInt32() - 5 === site; if (!this.on) return;
     const m = a[s].toInt32(), U = a[s + 1];
     this.c = Object.assign({f, m, uoff: U.sub(LU).toInt32(), in: vec(a[s + 2])}, udump(U, m));
     this.o = a[s + 3]; },
   onLeave() { if (!this.on) return; this.c.out = vec(this.o); ft.calls.push(this.c); }}); }
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_usolve', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason, events}) + '\n'));
   console.log(path.basename(model), outputText.trim(), 'events', events.length, 'ok', ok, errors.join(' ')); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
