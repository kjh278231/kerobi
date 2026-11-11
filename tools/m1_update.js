// M1-02n: per dual run, every 05b257 call made by 06268b (enter/leave) and every refactor decision at 06268b
// LAB_001f6eb5 (0xf6eb5), with the inputs of the new prefix bound (05b257 C:463-512) and of the three refactor
// predicates (06268b C:1273-1303): docs/m1/m1_02n_update.md section 2. Pivots are numbered by the 8b473 calls from
// 28c93 as in tools/m1_usolve.js. Read-only hooks; addresses are never stored.
// usage: tools/gurun.sh node tools/m1_update.js MODEL OUTPUT.json.gz
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
 const ints = (p, n) => n > 0 ? Array.from(new Int32Array(p.readByteArray(4 * n))) : [];
 const longs = (p, n) => { const r = []; for (let k = 0; k < n; k++) r.push(p.add(8 * k).readS64().toNumber()); return r; };
 // LU state the prefix rule reads/writes: t +0x100, e8 +0xe8, ec +0xec, LU+0xf0 list, pos +0x40, U positions with an
 // empty U column (ord +0x48, column start +0x88 / end +0x90), update count +0x18c, LU+0x190/0x184/0x180/0x1e0/0x1e8
 const lu = (LU, full) => { const mm = LU.add(8).readS32(), ec = LU.add(0xec).readS32();
   const s = {m: mm, t: LU.add(0x100).readS32(), e8: LU.add(0xe8).readS32(), ec, upd: LU.add(0x18c).readS32(),
     per: LU.add(0x188).readS32(), f0: ints(LU.add(0xf0).readPointer(), ec), c190: LU.add(0x190).readS64().toNumber(),
     c184: LU.add(0x184).readS32(), c180: LU.add(0x180).readS32(), c1e0: LU.add(0x1e0).readS64().toNumber(),
     c1e8: LU.add(0x1e8).readS64().toNumber()};
   if (full) { const ord = ints(LU.add(0x48).readPointer(), mm), beg = longs(LU.add(0x88).readPointer(), mm),
       end = longs(LU.add(0x90).readPointer(), mm);
     s.pos = ints(LU.add(0x40).readPointer(), mm); s.emp = [];
     for (let p = 0; p < mm; p++) if (end[ord[p]] === beg[ord[p]]) s.emp.push(p); }
   return s; };
 let k = 0, spx = null;
 Interceptor.attach(b.add(0x11f620), {onEnter(a) { if (this.returnAddress.sub(b).toUInt32() === 0xc58d5) k++; }});
 // 05b257(spx, &work) from 06268b C:183 (0xf6c82) and C:188 (0xf6c96)
 Interceptor.attach(b.add(0xf15c0), {
  onEnter(a) { const site = this.returnAddress.sub(b).toUInt32(); this.on = site === 0xf6c87 || site === 0xf6c9b;
    if (!this.on) return; spx = a[0]; const LU = a[0].add(0x470).readPointer(); this.LU = LU;
    const r = a[0].add(0x2f8).readS32(), sp = LU.add(0x1c8).readPointer(), n = sp.readU32();
    const idx = ints(sp.add(8).readPointer(), n), vp = sp.add(0x10).readPointer();
    const rb = LU.add(0xb0).readPointer(), re = LU.add(0xb8).readPointer();
    this.e = Object.assign({type: 'U', k, site: site === 0xf6c87 ? 0 : 1, reason: a[0].add(0x294).readS32(),
      x290: a[0].add(0x290).readS32(), r, spike: idx, spv: idx.map(i => hx8(vp.add(8 * i))), d8: LU.add(0x1d8).readS32(),
      urow_r: r >= 0 ? re.add(8 * r).readS64().toNumber() - rb.add(8 * r).readS64().toNumber() : null}, {in: lu(LU, true)}); },
  onLeave(ret) { if (!this.on) return; this.e.ret = ret.toInt32(); this.e.out = lu(this.LU, true);
    this.e.reason_out = spx.add(0x294).readS32(); send(this.e); }});
 // refactor decision LAB_001f6eb5: r14 = spx, [rsp+0xe8] = LU, r12 = &work, [rsp+0x78] m, [rsp+0x80] 1.5m, [rsp+0x88] 4m^2
 let envd = false;
 Interceptor.attach(b.add(0xf6eb5), function () { const c = this.context, sx = c.r14, LU = c.rsp.add(0xe8).readPointer();
   const e = {type: 'D', k, c46c: sx.add(0x46c).readS32(), w150: hx8(sx.add(0x150)), w158: hx8(sx.add(0x158)),
     c3d0: sx.add(0x3d0).readS64().toNumber(), work: c.r12.isNull() ? null : hx8(c.r12), s78: hx8(c.rsp.add(0x78)),
     s80: hx8(c.rsp.add(0x80)), s88: hx8(c.rsp.add(0x88)), reason: sx.add(0x294).readS32(), lu: lu(LU, false)};
   if (!envd) { const env = sx.add(0x4b8).readPointer().add(0x100).readPointer(); envd = true;
     e.env2454 = env.add(0x2454).readS32(); e.env2728 = env.add(0x2728).readS32(); }
   send(e); });
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_update', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason, events}) + '\n'));
   const c = {}; for (const e of events) c[e.type] = (c[e.type] || 0) + 1;
   console.log(path.basename(model), outputText.trim(), JSON.stringify(c), 'ok', ok, errors.join(' ').slice(0, 300)); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
