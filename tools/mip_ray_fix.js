// M2-06f-5 (docs/m2/mip_infeas_ray_fix.md): read-only trace of 6684cd, the in-place fix of the infeasible-node ray.
//   fk    95da98 leave at the 5ce67b site (0x4f46cb): rc, fail, L+0x58/68, and the return address of the enclosing
//         5ce67b call (gate classification, contract section 3)
//   fx    6684cd entry (only caller 5fb8ec C:2217): the gate fields and every replay input (L fields, ray, work, the
//         ancestor chain records, node bounds, types, scales, senses, the chain columns of the node LP, the rows those
//         columns touch, cut ages, 95ce25 inputs); fk = index of the fk event that last filled this L (-1 if none)
//   q     788a84 leave inside fx: row argument and return
//   ce    95ce25 entry inside fx: ray (nonzero bits), rho, g, s, work
//   ce_   95ce25 leave: fail code, rho, L5c, g, work
//   fx_   6684cd leave: rc, ray (nonzero bits), L+0x5c/60/68, work, entries that differ from the entry ray
// Arrays are hex bit strings (doubles) or ints; repeated large arrays are stored once by content hash.
// Nothing is written into the target. One invocation spawns one process.
// Usage: tools/gurun.sh node tools/mip_ray_fix.js MODEL OUTPUT.json.gz 'JSON_PARAMS'
const fs = require('fs'), path = require('path'), crypto = require('crypto'), zlib = require('zlib');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [model, output, extra] = process.argv.slice(2);
if (!model || !output) throw Error('usage: MODEL OUTPUT [JSON_PARAMS]');
if (fs.existsSync(output)) throw Error('output exists: ' + output);
const root = path.resolve(__dirname, '../..');
const py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const expected = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c';
if (hash(binary) !== expected) throw Error('wrong Gurobi binary');
const params = Object.assign({Threads: 1, Seed: 0, LogToConsole: 0}, JSON.parse(extra || '{}'));
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\n` +
  `m.optimize()\nprint('DONE',m.Status,int(m.NodeCount),flush=True)`;
const agent = `
const hx = p => p.readU64().toString(16).padStart(16, '0');
const dv = (p, n) => { if (p.isNull() || n <= 0) return null; const d = new BigUint64Array(p.readByteArray(8 * n));
  const o = new Array(n); for (let i = 0; i < n; i++) o[i] = d[i].toString(16).padStart(16, '0'); return o; };
const iv = (p, n) => p.isNull() || n <= 0 ? [] : Array.from(new Int32Array(p.readByteArray(4 * n)));
const bv = (p, n) => p.isNull() || n <= 0 ? null : Array.from(new Uint8Array(p.readByteArray(n)));
const nz = (p, n) => { const d = new BigUint64Array(p.readByteArray(8 * n)), o = {};
  for (let i = 0; i < n; i++) if (d[i] !== 0n) o[i] = d[i].toString(16).padStart(16, '0'); return o; };
let seq = 0; const emit = ev => { send(ev); return seq++; };
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return;
 const b = m.base, off = p => p.sub(b).toUInt32();
 const st5 = [];                          // return addresses of the active 5ce67b calls
 const last = new Map();                  // L address -> index of the fk event that filled it
 let fx = null;                           // the active 6684cd call
 Interceptor.attach(b.add(0x4f0e10), {onEnter() { st5.push(off(this.returnAddress)); }, onLeave() { st5.pop(); }});
 Interceptor.attach(b.add(0x781170), {onEnter(a) { this.ra = off(this.returnAddress); if (this.ra !== 0x4f46cb) return;
   this.L = a[4].sub(0x58); this.fail = a[12]; },
  onLeave(r) { if (this.ra !== 0x4f46cb) return; const L = this.L;
   const ev = {e: 'fk', rc: r.toInt32(), f: this.fail.readS32(), l58: L.add(0x58).readS32(), l68: hx(L.add(0x68)),
     v70: L.add(0x70).readPointer().isNull() ? 0 : 1, ra5: st5.length ? st5[st5.length - 1] : null, d5: st5.length};
   const k = emit(ev); last.set(L.toString(), k); }});
 Interceptor.attach(b.add(0x554ee0), {onEnter(a) {
   const P = a[0], N = a[1], w = a[2], M = P.readPointer(), S = P.add(8).readPointer();
   const I = N.add(0x18).readPointer(), L = I.add(8).readPointer(), lp = S.add(0xe8).readPointer();
   const lp0 = M.add(8).readPointer().add(0xe8).readPointer(), env = S.add(0x100).readPointer();
   const n = lp.add(0xc).readS32(), mm = lp.add(8).readS32(), n0 = lp0.add(0xc).readS32(), m0 = lp0.add(8).readS32();
   const ray = L.add(0x70).readPointer(), fm = lp.add(0x3c).readS32();
   const ev = {e: 'fx', fk: last.has(L.toString()) ? last.get(L.toString()) : -1, n, m: mm, n0, m0, fm,
     l38: L.add(0x38).readS32(), l54: L.add(0x54).readS32(), l58: L.add(0x58).readS32(), l5c: L.add(0x5c).readS32(),
     l60: hx(L.add(0x60)), l68: hx(L.add(0x68)), v70: ray.isNull() ? 0 : 1, g2a48: env.add(0x2a48).readS32(),
     w: w.isNull() ? null : hx(w), depth: I.add(0x2c).readS32(), I0M: I.readPointer().equals(M) ? 1 : 0,
     inf: hx(env.add(0x2458))};
   fx = {L, ray, w, n, m: mm, d0: null};
   if (ray.isNull() || n + mm <= 0) { emit(ev); return; }
   ev.ray = nz(ray, n + mm); fx.d0 = ray.readByteArray(8 * (n + mm));
   // ancestor chain (the root, whose parent is null, is not read by 6684cd)
   const chain = [], vars = new Set();
   for (let node = N, k = 0; !node.add(0x10).readPointer().isNull() && k < 100000; node = node.add(0x10).readPointer(), k++) {
     const c1 = node.add(0x24).readS32(), c2 = node.add(0x28).readS32(), c3 = node.add(0x2c).readS32(), rec = [];
     for (let t = 0; t < c3; t++) { const v = node.add(0x34 + 16 * t).readS32(); rec.push([node.add(0x30 + 16 * t).readU8(), v]); vars.add(v); }
     const l1 = iv(node.add(0x30 + 16 * c3), c1), l2 = iv(node.add(0x30 + 16 * c3 + 4 * c1), c2);
     l1.forEach(v => vars.add(v)); l2.forEach(v => vars.add(v));
     chain.push({c1, c2, c3, rec, l1, l2}); }
   ev.chain = chain;
   const cb = lp.add(0xf8).readPointer(), cc = lp.add(0x100).readPointer(), ci = lp.add(0x108).readPointer(), cv = lp.add(0x110).readPointer();
   const cols = {}, rset = new Set();
   for (const v of Array.from(vars).sort((x, y) => x - y)) { if (v < 0 || v >= n) { cols[v] = null; continue; }
     const beg = cb.add(8 * v).readS64().toNumber(), cnt = cc.add(4 * v).readS32();
     const idx = iv(ci.add(4 * beg), cnt); idx.forEach(i => rset.add(i)); cols[v] = [idx, dv(cv.add(8 * beg), cnt)]; }
   ev.cols = cols;
   const rb = lp0.add(0x118).readPointer(), re = lp0.add(0x120).readPointer(), ri = lp0.add(0x130).readPointer(), rv = lp0.add(0x138).readPointer();
   const rhs0 = lp0.add(0x380).readPointer(), cuts = M.add(0x3240).readPointer().add(0x78).readPointer(), rows = {};
   for (const i of Array.from(rset).sort((x, y) => x - y)) {
     if (i < 0 || i >= mm) { rows[i] = null; continue; }
     if (i < m0) { const beg = rb.add(8 * i).readS64().toNumber(), cnt = re.add(8 * i).readS64().toNumber() - beg, vp = rv.add(8 * beg);
       rows[i] = {idx: iv(ri.add(4 * beg), cnt), val: dv(vp, cnt), rhs: hx(rhs0.add(8 * i)), al: vp.and(0xf).toInt32()}; }
     else { const c = cuts.add(8 * (i - m0)).readPointer(), cnt = c.add(0x10).readS32(), vp = c.add(8).readPointer();
       rows[i] = {idx: iv(c.readPointer(), cnt), val: dv(vp, cnt), rhs: hx(c.add(0x18)), age: c.add(0x38).readS32(), al: vp.and(0xf).toInt32()}; } }
   ev.rows = rows;
   if (mm > m0) { const age = []; for (let t = 0; t < mm - m0; t++) { let x = null; try { x = cuts.add(8 * t).readPointer().add(0x38).readS32(); } catch (e) {} age.push(x); } ev.age = age; }
   ev.lb0 = dv(I.add(0x80).readPointer(), n); ev.ub0 = dv(I.add(0x88).readPointer(), n);
   ev.vt = bv(M.add(0x650).readPointer(), n); ev.sense = bv(lp.add(0x388).readPointer(), mm);
   ev.C = dv(lp.add(0x3e0).readPointer(), n); ev.R = dv(lp.add(0x3d8).readPointer(), mm);
   ev.slb = dv(lp.add(0x390).readPointer(), n); ev.sub = dv(lp.add(0x398).readPointer(), n);
   ev.flip = fm > 0 ? bv(lp.add(0x408).readPointer(), n) : null;
   ev.x32e8 = M.add(0x32e8).readPointer().isNull() ? 0 : 1;
   emit(ev); },
  onLeave(r) { const c = fx; fx = null; if (!c) return; const L = c.L, N = c.n + c.m;
   const ev = {e: 'fx_', rc: r.toInt32(), l5c: L.add(0x5c).readS32(), l60: hx(L.add(0x60)), l68: hx(L.add(0x68)),
     w: c.w.isNull() ? null : hx(c.w), same: L.add(0x70).readPointer().equals(c.ray) ? 1 : 0};
   if (c.d0) { ev.ray = nz(c.ray, N); const now = new BigUint64Array(c.ray.readByteArray(8 * N)), was = new BigUint64Array(c.d0);
     let d = 0; for (let i = 0; i < N; i++) if (now[i] !== was[i]) d++; ev.d = d; }
   emit(ev); }});
 Interceptor.attach(b.add(0x61a1f0), {onEnter(a) { this.c = fx; if (fx) this.row = a[2].toInt32(); },
  onLeave(rv) { if (this.c) emit({e: 'q', row: this.row, ret: rv.toInt32() & 0xff}); }});
 Interceptor.attach(b.add(0x7807f0), {onEnter(a) { this.c = fx; if (!fx) return;
   this.rho = a[3]; this.l5c = a[4]; this.g = a[5]; this.fail = a[7]; this.w = a[8];
   emit({e: 'ce', s: a[2].toInt32(), rho: hx(a[3]), g: hx(a[5]), w: a[8].isNull() ? null : hx(a[8]),
     same: a[6].equals(fx.ray) ? 1 : 0, ray: nz(a[6], fx.n + fx.m)}); },
  onLeave() { if (!this.c) return;
   emit({e: 'ce_', fail: this.fail.readS32(), rho: hx(this.rho), l5c: this.l5c.readS32(), g: hx(this.g),
     w: this.w.isNull() ? null : hx(this.w)}); }});
}});
`;
(async () => {
 const device = await frida.getLocalDevice(); const events = []; let outputText = ''; const errors = [];
 const pid = await device.spawn([py, '-c', code], {stdio: 'pipe'}); const session = await device.attach(pid);
 const script = await session.createScript(agent);
 script.message.connect(msg => { if (msg.type !== 'send') { errors.push(msg.stack || JSON.stringify(msg)); return; } events.push(msg.payload); });
 device.output.connect((p, fd, data) => { if (p === pid) outputText += data.toString(); });
 session.detached.connect((reason) => {
   if (outputText.includes('Gurobi is already running')) { console.log(outputText.trim()); process.exit(1); }
   const ok = outputText.includes('DONE') && !errors.length;
   // large arrays repeat across calls: store each distinct one once, by content hash
   const arrays = {};
   for (const ev of events) if (ev.e === 'fx') for (const f of ['chain', 'cols', 'rows', 'age', 'lb0', 'ub0', 'vt', 'sense', 'C', 'R', 'slb', 'sub', 'flip']) {
     if (ev[f] == null) continue; const s = JSON.stringify(ev[f]), h = crypto.createHash('sha256').update(s).digest('hex').slice(0, 24);
     arrays[h] = ev[f]; ev[f] = h; }
   fs.mkdirSync(path.dirname(output), {recursive: true});
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'mip_ray_fix', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok,
     detach_reason: reason, events, arrays})));
   const c = {}; for (const e of events) c[e.e] = (c[e.e] || 0) + 1;
   console.log(path.basename(model), outputText.trim(), JSON.stringify(c), 'ok', ok, errors.join(' ').slice(0, 300));
   process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
