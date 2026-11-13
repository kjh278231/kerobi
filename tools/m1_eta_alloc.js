// M1-02y (explore): the allocation holding the LU+0x1d0 arrays. Before: at every FT update 05b257 (entry 0xf15c0, rdi = spx, LU = spx+0x470) of one Gurobi LP solve, the U^T e_r copy
// LU+0x1d0 that the new diagonal eta.S reads (C:321-372): [nnz, value pointer mod 16, pointer changed since the previous
// update (0/1), LU+0x100 prefix t, idx pointer mod 16, val - idx, val - struct (bytes), LU+8 m]. A value array 8 but not 16-byte aligned peels one term into lane 0 (C:332-337).
// Read-only hook; no addresses are stored.
// Records libc malloc/calloc/realloc results (freed blocks dropped) with the return offset of the ac645d/ac64e6 wrapper call and, at the first 05b257,
// the block containing the LU+0x1d0 idx pointer.
// usage: tools/gurun.sh node tools/m1_eta_alloc.js MODEL OUTPUT.json.gz [Method]
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
const agent = `Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return; const b = m.base, e = b.add(m.size);
 const blocks = new Map(); let done = false;
 const caller = c => Thread.backtrace(c, Backtracer.ACCURATE).filter(r => r.compare(b) >= 0 && r.compare(e) < 0).slice(0, 6).map(r => r.sub(b).toUInt32());
 for (const [name, szf] of [['malloc', a => a[0].toUInt32()], ['calloc', a => a[0].toUInt32() * a[1].toUInt32()], ['realloc', a => a[1].toUInt32()]]) {
  Interceptor.attach(Module.getGlobalExportByName(name), {onEnter(a) { if (done) return; this.sz = szf(a); this.ra = this.returnAddress; this.c = this.sz >= 1 << 30 ? caller(this.context) : null; },
   onLeave(r) { if (done || this.sz < 16 || r.isNull()) return; blocks.set(r.toString(), [this.sz, wrapRet, name]); }});
 }
 let wrapRet = -1, wrapSz = 0;
 for (const w of [0x8d1880, 0x8d1bb0]) Interceptor.attach(b.add(w), {onEnter(a) { if (!done) { this.prev = wrapRet; wrapRet = this.returnAddress.sub(b).toUInt32(); } }, onLeave() { if (!done) wrapRet = this.prev; }});
 Interceptor.attach(Module.getGlobalExportByName('free'), {onEnter(a) { if (!done) blocks.delete(a[0].toString()); }});
 Interceptor.attach(Module.getGlobalExportByName('realloc'), {onEnter(a) { if (!done && !a[0].isNull()) blocks.delete(a[0].toString()); }});
 Interceptor.attach(b.add(0xf15c0), {onEnter(a) { if (done) return; done = true; const LU = a[0].add(0x470).readPointer(), s = LU.add(0x1d0).readPointer();
   const ix = s.add(8).readPointer(), v = s.add(0x10).readPointer(), mm = LU.add(8).readS32(); const hit = [];
   for (const [k, [sz, c, nm]] of blocks) { const p = ptr(k); for (const [tag, q] of [['struct', s], ['idx', ix], ['val', v]])
     if (q.compare(p) >= 0 && q.compare(p.add(sz)) < 0) hit.push([tag, nm, sz, c, q.sub(p).toInt32(), p.and(0xf).toUInt32()]); }
   send([{m: mm, idx16: ix.and(0xf).toUInt32(), hit}]); }});
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_eta_alloc', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason,
     events}) + '\n'));
   console.log(path.basename(model), outputText.trim(), events.length, 'ok', ok, errors.join(' ').slice(0, 300)); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
