// M1-02y: the row-wise U of the original at the 21a941 (sparse U^T) entry of the CALL-th dual e_r BTRAN (221173 via 9e376
// back to 0xc51dd, counted from 0): for every U row k < m the [idx, value bits] of LU+0xb0[k]..LU+0xb8[k] in the pools
// LU+0xd0 / LU+0xc8, and the diagonal LU+0x50 (bits). 21a941 gets rdi = LU (221173 asm 0x12e56b). Read-only hooks.
// usage: tools/gurun.sh node tools/m1_urow_dump.js MODEL OUTPUT.json.gz [Method] [CALL]
const fs = require('fs'), path = require('path'), crypto = require('crypto'), zlib = require('zlib');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [model, output, method, callArg] = process.argv.slice(2);
const CALL = Number(callArg ?? 31);
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
 const hexa = (p, n) => { if (n <= 0) return []; const v = new DataView(p.readByteArray(8 * n)), r = [];
   for (let j = 0; j < n; j++) r.push(v.getUint32(8 * j + 4, true).toString(16).padStart(8, '0') + v.getUint32(8 * j, true).toString(16).padStart(8, '0'));
   return r; };
 let wrap = null, n = 0, er = false;
 for (const [a, w] of [[0x12de10, 288], [0x131710, 376]])
  Interceptor.attach(b.add(a), {onEnter(x) { wrap = [w, this.returnAddress.sub(b).toUInt32()]; }});
 Interceptor.attach(b.add(0x12de60), {onEnter() { er = wrap[0] === 376 && wrap[1] === 0xc51dd; if (er) n++; }});
 Interceptor.attach(b.add(0x130b70), {onEnter(a) { if (!er || n - 1 !== ${CALL}) return; const LU = a[0], mm = LU.add(8).readS32();
   const st = LU.add(0xb0).readPointer(), en = LU.add(0xb8).readPointer(), ix = LU.add(0xd0).readPointer(), vv = LU.add(0xc8).readPointer();
   const rows = [];
   for (let k = 0; k < mm; k++) { const s = st.add(8 * k).readS64().toNumber(), e = en.add(8 * k).readS64().toNumber();
     rows.push([e > s ? Array.from(new Int32Array(ix.add(4 * s).readByteArray(4 * (e - s)))) : [], hexa(vv.add(8 * s), e - s)]); }
   send([{call: ${CALL}, m: mm, diag: hexa(LU.add(0x50).readPointer(), mm), rows}]); }});
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_urow_dump', call: CALL, model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason,
     events}) + '\n'));
   console.log(path.basename(model), outputText.trim(), events.length, 'ok', ok, errors.join(' ').slice(0, 300)); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
