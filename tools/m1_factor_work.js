// M1-02q: the Markowitz-loop work terms of the LU factor 036ade (0xd4190) during one dual run (06268b), read-only.
// Per 036ade call inside 06268b: the basis (param_5 = nb, param_6 = n, param_11 = basis heads by position), whether
// a work pointer is passed (param_19); param_1/2 are doubles, so param_3 is args[0], and every accumulator term of the pivot loop in order (docs/m1/m1_02q_work.md
// section 2): column singleton 0xd6077 (cnt + 1)*10 with cnt = r15, row singleton 0xd647b with cnt =
// *(int*)(rdx + 4*r13), general pivot 0xd6ea4 ns*6*bound with ns = *(int*)(rsp+0x70) (bound is xmm4: not readable),
// then 10*min(ecx, r10d) at 0xd7157 (ecx = sum of the row counts over the pivot column, r10d the other count).
// Also the fold point 0xd85c7 (store 0xd85e6), the acc > 1e8 early fold 0xd5e16, spx+0x390 (Markowitz tol) per call,
// and the 05b257 refactor entries from 06268b with k, reason and the work bits (non-interference against M1-02p).
// No state is changed. Addresses are offsets in libgurobi130.so.
// usage: tools/gurun.sh node tools/m1_factor_work.js MODEL OUTPUT.json.gz
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
 let k = 0, depth = 0, inf = false;
 const s32 = r => r.toInt32();
 const hx8 = p => { const v = new DataView(p.readByteArray(8)); return v.getUint32(4, true).toString(16).padStart(8, '0') + v.getUint32(0, true).toString(16).padStart(8, '0'); };
 Interceptor.attach(b.add(0x11f620), {onEnter(a) { if (this.returnAddress.sub(b).toUInt32() === 0xc58d5) k++; }});
 Interceptor.attach(b.add(0xf6a00), {onEnter(a) { depth++; }, onLeave(r) { depth--; }});
 Interceptor.attach(b.add(0xf15c0), {onEnter(a) { const site = this.returnAddress.sub(b).toUInt32();
   if (depth > 0 && (site === 0xf6c87 || site === 0xf6c9b)) send({type: 'U', k, reason: a[0].add(0x294).readS32(), work: a[1].isNull() ? null : hx8(a[1])}); }});
 Interceptor.attach(b.add(0xd4190), {
  onEnter(a) { this.on = depth > 0; if (!this.on) return; inf = true; const nb = a[2].toInt32();  // param_1/2 are doubles: param_3 = a[0]
    const h = []; for (let i = 0; i < nb; i++) h.push(a[8].add(4 * i).readS32());
    send({type: 'F', k, nb, n: a[3].toInt32(), work: !a[16].isNull(), x390: hx8(a[0].add(0x390)), ret: this.returnAddress.sub(b).toUInt32(), head: h}); },
  onLeave(r) { if (!this.on) return; inf = false; send({type: 'X', k, ret: r.toInt32()}); }});
 const probe = (o, f) => Interceptor.attach(b.add(o), function () { if (inf) send(Object.assign({type: 'T', pc: o}, f(this.context))); });
 probe(0xd6077, c => ({cnt: s32(c.r15)}));
 probe(0xd647b, c => ({cnt: c.rdx.add(4 * s32(c.r13)).readS32()}));
 probe(0xd6ea4, c => ({ns: c.rsp.add(0x70).readS32()}));
 probe(0xd7157, c => ({ecx: s32(c.rcx), r10: s32(c.r10)}));
 probe(0xd85c7, c => ({}));
 probe(0xd5e16, c => ({}));  // acc > 1e8: early fold (store at 0xd5e35)
}});`;
(async () => {
 fs.mkdirSync(path.dirname(path.resolve(output)), {recursive: true});
 const device = await frida.getLocalDevice(); const events = []; let outputText = ''; const errors = [];
 const pid = await device.spawn([py, '-c', code], {stdio: 'pipe'}); const session = await device.attach(pid);
 const script = await session.createScript(agent);
 script.message.connect(msg => { if (msg.type !== 'send') { errors.push(msg.stack || JSON.stringify(msg)); return; } events.push(msg.payload); });
 device.output.connect((p, fd, data) => { if (p === pid) outputText += data.toString(); });
 session.detached.connect(reason => {
   if (outputText.includes('Gurobi is already running')) { console.log(outputText.trim()); process.exit(1); }
   const ok = outputText.includes('DONE') && !errors.length;
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_factor_work', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason, events}) + '\n'));
   const c = {}; for (const e of events) c[e.type + (e.pc ? e.pc.toString(16) : '')] = (c[e.type + (e.pc ? e.pc.toString(16) : '')] || 0) + 1;
   console.log(path.basename(model), outputText.trim(), JSON.stringify(c), 'ok', ok, errors.join(' ').slice(0, 300)); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
