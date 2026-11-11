// M1-02l: tools/m1_tau_path.js (M1-02a-1, unchanged fields) plus, at each 09d112 entry, the row-eta pool the R phase
// reads (docs/m1/m1_02l_eta_peel.md section 2): LU+0x148 (idx pool) and LU+0x150 (value pool) mod 16, LU+0x158,
// LU+0x160 (used), LU+0x168 (capacity), LU+0x184, and per eta record LU+0x198 {r, len, idx*, val*} the pool offsets
// (idx - LU+0x148)/4, (val - LU+0x150)/8 and idx, val mod 16. Read-only; no address is stored, only offsets and mod 16.
// usage: tools/gurun.sh node tools/m1_eta_pool.js MODEL OUTPUT.json.gz
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
 // vector {int nnz; int* idx @+8; double* val @+0x10}; nnz < 0 = dense, m values
 const vec = (p, m) => { const nnz = p.readS32(); return {nnz, idx: nnz >= 0 ? ints(p.add(8).readPointer(), nnz) : null,
   val: dbls(p.add(0x10).readPointer(), nnz < 0 ? m : nnz)}; };
 let k = 0, cur = null, ft = null;  // cur: open 8b473 call from 28c93, ft: open 09d112 call inside it
 Interceptor.attach(b.add(0x11f620), {
  onEnter(a) { this.on = this.returnAddress.sub(b).toUInt32() === 0xc58d5; if (!this.on) return;
    this.spx = a[0]; cur = {k: k++, ft: null, n9d: 0}; },
  onLeave() { if (!this.on) return; const spx = this.spx, m = spx.add(0x64).readS32(), LU = spx.add(0x470).readPointer();
    const P = spx.add(0x478).readPointer();
    Object.assign(cur, {m, mode: spx.add(0x3ec).readS32(), r: spx.add(0x2f8).readS32(), s290: spx.add(0x290).readS32(),
      thr: hx8(spx.add(0x3a8)), ar: hx8(spx.add(0x300)), beta: hx8(spx.add(0x310)), lu_upd: LU.add(0x18c).readS32(),
      aq: vec(spx.add(0x210).readPointer(), m), tau_nnz: spx.add(0x218).readPointer().readS32(),
      w: dbls(P.add(0x08).readPointer(), m)});
    const n = cur.aq.nnz < 0 ? m : cur.aq.nnz;
    cur.tau = dbls(spx.add(0x218).readPointer().add(0x10).readPointer(), n);  // 1bae9c C:451 reads tau val[k]
    send(cur); cur = null; }});
 Interceptor.attach(b.add(0x125f50), {
  onEnter(a) { this.on = cur !== null && this.returnAddress.sub(b).toUInt32() === 0x120200; if (!this.on) return;
    const LU = a[0], m = LU.add(8).readS32();
    this.a = [a[1], a[3], a[5]]; this.m = m;
    ft = {m, r: a[6].toInt32(), calls: [],
      lu: {ec: LU.add(0xec).readS32(), x100: LU.add(0x100).readS32(), x104: LU.add(0x104).readS32(),
           x128: LU.add(0x128).readS32(), x180: LU.add(0x180).readS32()},
      aq_in: vec(a[2], m), rho_in: vec(a[4], m)};
    // U rows read by the tau gather (09d112 C:2302-2402): begin/end LU+0xb0/0xb8 (long), idx LU+0xd0, val LU+0xc8,
    // row flag LU+0xf8 (-1 divide, -2 tail), prefix t = LU+0x100, diagonal LU+0x50, U order LU+0x48
    const lp = o => LU.add(o).readPointer(), b0 = lp(0xb0), b8 = lp(0xb8), ui = lp(0xd0), uv = lp(0xc8);
    ft.t = LU.add(0x100).readS32(); ft.f8 = ints(lp(0xf8), m); ft.ord = ints(lp(0x48), m); ft.diag = dbls(lp(0x50), m);
    const p148 = lp(0x148), p150 = lp(0x150), ne = LU.add(0x180).readS32(), rec = lp(0x198);
    const off = (p, b, w) => p.sub(b).toInt32() / w, lo = p => p.and(0xf).toInt32();
    ft.pool = {b148: lo(p148), b150: lo(p150), x158: LU.add(0x158).readS64().toNumber(), x160: LU.add(0x160).readS64().toNumber(),
      x168: LU.add(0x168).readS64().toNumber(), x184: LU.add(0x184).readS32(), etas: []};
    for (let q = 0; q < ne; q++) { const e = rec.add(0x18 * q), ip = e.add(8).readPointer(), vp = e.add(0x10).readPointer();
      ft.pool.etas.push({r: e.readS32(), len: e.add(4).readS32(), io: off(ip, p148, 4), vo: off(vp, p150, 8), ia: lo(ip), va: lo(vp)}); }
    ft.urow = [];
    for (let i = 0; i < m; i++) { const s = b0.add(8 * i).readS64().toNumber(), e = b8.add(8 * i).readS64().toNumber();
      ft.urow.push({idx: ints(ui.add(4 * s), e - s), val: dbls(uv.add(8 * s), e - s)}); } },
  onLeave() { if (!this.on) return; const [o1, o2, pn] = this.a, m = this.m;
    // tau val is aligned with the alpha_q output list (its nnz field is not that count): read as many values
    const aq_out = vec(o1, m), tn = o2.readS32(), na = aq_out.nnz < 0 ? m : aq_out.nnz;
    const tau_out = {nnz: tn, idx: tn >= 0 ? ints(o2.add(8).readPointer(), tn) : null,
      val: dbls(o2.add(0x10).readPointer(), Math.max(na, tn < 0 ? 0 : tn))};
    Object.assign(ft, {aq_out, tau_out, nnz6: pn.readS32()});
    cur.n9d++; cur.ft = ft; ft = null; }});
 // callees of 09d112, named by call site (return address - 5)
 const sites = {0x1261af: '21b9de', 0x126210: '21b9de', 0x126284: '21b9de', 0x1263ac: '21b9de', 0x1262c2: '214344',
   0x126347: '214344', 0x1279e5: '21f83a', 0x127a7e: '21f83a', 0x128618: 'memcpy', 0x129547: '21841f', 0x12959f: '2126cd',
   0x1295e0: '21686f', 0x129634: '21574d'};
 const fns = {};
 for (const [s, f] of Object.entries(sites)) if (f !== 'memcpy') fns[f] = true;
 const addr = {'21b9de': 0x12a990, '214344': 0x12bd30, '21f83a': 0x129aa0, '21841f': 0x12c300, '2126cd': 0x121ef0,
   '21686f': 0x1223e0, '21574d': 0x123690};
 for (const f of Object.keys(fns)) Interceptor.attach(b.add(addr[f]), {onEnter() { if (!ft) return;
   const s = this.returnAddress.sub(b).toUInt32() - 5; if (sites[s] === f) ft.calls.push(f + '@' + s.toString(16)); }});
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_eta_pool', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason, events}) + '\n'));
   console.log(path.basename(model), outputText.trim(), 'events', events.length, 'ok', ok, errors.join(' ')); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
