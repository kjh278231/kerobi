// M1-02p: every write to the dual simplex work double (06268b param_2) during one dual run, by a hardware watchpoint
// set at 06268b entry and cleared at its return (docs/m1/m1_02p_work.md section 2). Per write: pivot k (8b473 calls from
// 28c93, as tools/m1_update.js), trap pc (instruction after the store), the innermost hooked writer function and its call
// site, and the new value bits. Also the 05b257 entries/returns from 06268b and the refactor decisions at LAB_001f6eb5.
// The watchpoint only traps; no state is changed. Addresses are stored as offsets in libgurobi130.so.
// usage: tools/gurun.sh node tools/m1_work.js MODEL OUTPUT.json.gz
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
// functions that hold a work store (watchpoint exploration, section 2) + 1c090a (receives the pointer, no store seen)
const WRITERS = [0x11f620, 0x120740, 0x125f50, 0x12de10, 0x131710, 0x132460, 0x134d90, 0xbdc10, 0xbe6b0, 0xbf5b0, 0xc0050,
  0xc3f80, 0xc4fe0, 0xc74a0, 0xccbc0, 0xcd720, 0xd4190, 0xe45c0, 0xe7c50, 0xf15c0, 0xcc6a0];
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return; const b = m.base, end = b.add(m.size);
 const hx8 = p => { const v = new DataView(p.readByteArray(8)); return v.getUint32(4, true).toString(16).padStart(8, '0') + v.getUint32(0, true).toString(16).padStart(8, '0'); };
 const off = p => (p.compare(b) >= 0 && p.compare(end) < 0) ? p.sub(b).toUInt32() : -1;
 let k = 0, W = null, SPX = null, armed = false, depth = 0; const stack = [];
 Interceptor.attach(b.add(0x11f620), {onEnter(a) { if (this.returnAddress.sub(b).toUInt32() === 0xc58d5) k++; }});
 for (const f of WRITERS) Interceptor.attach(b.add(f), {onEnter(a) { stack.push([f, off(this.returnAddress)]); }, onLeave(r) { stack.pop(); }});
 Process.setExceptionHandler(d => {
   if (!armed || (d.type !== 'single-step' && d.type !== 'breakpoint')) return false;
   const top = stack.length ? stack[stack.length - 1] : [-1, -1], pc = off(d.context.pc);
   const e = {type: 'W', k, pc, f: top[0], c: top[1], v: hx8(W), x: d.type};
   if ((pc === 0xbf981 || pc === 0xbfed2) && SPX) e.x338 = SPX.add(0x338).readS32();  // 1ca57 amount reads spx+0x338
   send(e); return true; });
 Interceptor.attach(b.add(0xf6a00), {
  onEnter(a) { if (depth++ > 0) return; W = a[1]; if (W.isNull()) { send({type: 'E', k, null: true}); return; }
    send({type: 'E', k, v: hx8(W)});
    for (const t of Process.enumerateThreads()) t.setHardwareWatchpoint(0, W, 8, 'w'); armed = true; },
  onLeave(r) { if (--depth > 0) return; send({type: 'L', k, v: W.isNull() ? null : hx8(W), ret: r.toInt32()});
    if (armed) { armed = false; for (const t of Process.enumerateThreads()) t.unsetHardwareWatchpoint(0); } }});
 // 05b257(spx, &work) from 06268b C:183 (0xf6c82) and C:188 (0xf6c96)
 Interceptor.attach(b.add(0xf15c0), {
  onEnter(a) { const site = this.returnAddress.sub(b).toUInt32(); this.on = site === 0xf6c87 || site === 0xf6c9b; if (!this.on) return;
    this.spx = a[0]; SPX = a[0]; const env = a[0].add(0x4b8).readPointer().add(0x100).readPointer();
    send({type: 'U', k, site: site === 0xf6c87 ? 0 : 1, reason: a[0].add(0x294).readS32(), x290: a[0].add(0x290).readS32(),
      wp: a[1].equals(W), work: a[1].isNull() ? null : hx8(a[1]), w150: hx8(a[0].add(0x150)), w158: hx8(a[0].add(0x158)),
      env2454: env.add(0x2454).readS32(), env2728: env.add(0x2728).readS32(), x338: a[0].add(0x338).readS32()}); },
  onLeave(r) { if (!this.on) return; send({type: 'V', k, ret: r.toInt32(), reason: this.spx.add(0x294).readS32(),
    w150: hx8(this.spx.add(0x150)), w158: hx8(this.spx.add(0x158)), x338: this.spx.add(0x338).readS32()}); }});
 // refactor decision LAB_001f6eb5: r14 = spx, [rsp+0xe8] = LU, r12 = &work
 Interceptor.attach(b.add(0xf6eb5), function () { const c = this.context, sx = c.r14, LU = c.rsp.add(0xe8).readPointer();
   send({type: 'D', k, upd: LU.add(0x18c).readS32(), per: LU.add(0x188).readS32(), reason: sx.add(0x294).readS32(),
     w150: hx8(sx.add(0x150)), w158: hx8(sx.add(0x158)), c3d0: sx.add(0x3d0).readS64().toNumber(),
     work: c.r12.isNull() ? null : hx8(c.r12), wp: c.r12.equals(W), x148: hx8(sx.add(0x148)), x1f8: hx8(LU.add(0x1f8)), x338: sx.add(0x338).readS32()}); });
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_work', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason, events}) + '\n'));
   const c = {}; for (const e of events) c[e.type] = (c[e.type] || 0) + 1;
   console.log(path.basename(model), outputText.trim(), JSON.stringify(c), 'ok', ok, errors.join(' ').slice(0, 300)); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
