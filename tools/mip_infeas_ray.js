// M2-06f-4 (docs/m2/mip_infeas_ray.md): read-only trace of 95da98, the infeasible-node vector, at the 5ce67b call.
//   fk    95da98 entry (return 0x4f46cb): the M2-06f-2 fields (r9 == r8 - 4, L+0x70 present) plus the loop gate state
//   row   12ea3f leave inside fk: the replay inputs read at that moment (row alpha, basic var, x_B[r], bounds, scales,
//         flip, status, rhs, cut values, tolerances, work); arrays are hex bit strings (doubles) or ints
//   q     7889ec leave inside fk: row argument and return
//   rs    cda5b entry/leave inside fk (re-solve): LP status after
//   fk_   95da98 leave: the M2-06f-2 fields plus every output (L+0x58/54/5c/60/68, ray and bnd nonzero bits, work)
//   x2a   5d80d8 entry (5ce67b only calls it after a successful fk when env+0x2a40 != 0)
//   pp    6684cd leave (5fb8ec C:2214, gate env+0x2a48) on an L that fk_ filled: changed ray entries (and their new
//         bits), L+0x5c/60/68 after; it rewrites the ray in place (C:736-767, 981) and L+0x5c/60/68 (C:976-978)
//   dec   6ee1d3 entry on a node whose L has L+0x58 >= 0: L+0x38/54/58 and whether L+0x70 equals the bytes of its
//         last known writer (fk_ or pp; src names which)
//   oth   95da98 called from elsewhere (687f96): count only
// Nothing is written into the target. One invocation spawns one process.
// Usage: tools/gurun.sh node tools/mip_infeas_ray.js MODEL OUTPUT.json.gz 'JSON_PARAMS'
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
const iv = (p, n) => p.isNull() || n <= 0 ? null : Array.from(new Int32Array(p.readByteArray(4 * n)));
const bv = (p, n) => p.isNull() || n <= 0 ? null : Array.from(new Uint8Array(p.readByteArray(n)));
const nz = (p, n) => { const d = new BigUint64Array(p.readByteArray(8 * n)), o = {};
  for (let i = 0; i < n; i++) if (d[i] !== 0n) o[i] = d[i].toString(16).padStart(16, '0'); return o; };
let seq = 0; const emit = ev => { send(ev); return seq++; };
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return;
 const b = m.base, off = p => p.sub(b).toUInt32();
 let cur = null;                          // the active 95da98 call at the 5ce67b site
 const last = new Map();                  // L address -> {k, bytes of L+0x70 at fk_, length}
 Interceptor.attach(b.add(0x781170), {onEnter(a) { this.ra = off(this.returnAddress);
   if (this.ra !== 0x4f46cb) { emit({e: 'oth', ra: this.ra}); return; }
   const S = a[0], L = a[4].sub(0x58), lp = S.add(0xe8).readPointer(), spx = S.add(0xe0).readPointer();
   const w = a[13];
   cur = {S, M: a[1], ctx: a[2], X: a[3], L, ray: a[9], bnd: a[10], fail: a[12], w, lp,
     arrs: [0x380, 0x390, 0x398, 0x3d8, 0x3e0, 0x408].map(o => lp.add(o).readPointer().toString())};
   this.cur = cur;
   const ev = {e: 'fk', r9: a[5].equals(a[4].sub(4)) ? 1 : 0, v70: L.add(0x70).readPointer().isNull() ? 0 : 1,
     lim: a[11].toInt32(), st: S.add(100).readS32(), spx: spx.isNull() ? 0 : 1,
     ctx: a[2].isNull() ? 0 : 1, w: w.isNull() ? null : hx(w), delta: hx(S.add(0x100).readPointer().add(0x2460))};
   if (!spx.isNull()) { const q = spx.add(0xf4).readS32(); ev.q = q;
     if (q >= 0) { ev.qa8 = hx(spx.add(0xa8).readPointer().add(8 * q)); ev.qb0 = hx(spx.add(0xb0).readPointer().add(8 * q)); }
     ev.fac = (spx.add(0x4a0).readPointer().isNull() && spx.add(0x478).readPointer().isNull()) ? 0 : 1;
     ev.r = spx.add(0x2f8).readS32(); ev.quad = spx.add(0x28).readS32(); }
   emit(ev); },
  onLeave(r) { if (this.ra !== 0x4f46cb) return; const c = this.cur, L = c.L; cur = null;
   const lp = c.lp, n = lp.add(0xc).readS32(), m = lp.add(8).readS32(), v = L.add(0x70).readPointer();
   const ev = {e: 'fk_', rc: r.toInt32(), f354: c.fail.readS32(), l58: L.add(0x58).readS32(), l54: L.add(0x54).readS32(),
     v70: v.isNull() ? 0 : 1, l5c: L.add(0x5c).readS32(), l60: hx(L.add(0x60)), l68: hx(L.add(0x68)), n, m,
     w: c.w.isNull() ? null : hx(c.w), same: [0x380, 0x390, 0x398, 0x3d8, 0x3e0, 0x408]
       .every((o, i) => lp.add(o).readPointer().toString() === c.arrs[i]) && c.S.add(0xe8).readPointer().equals(lp) ? 1 : 0};
   if (n + m > 0) { ev.ray = nz(c.ray, n + m); ev.bnd = nz(c.bnd, n + m); }
   const k = emit(ev);
   if (!v.isNull() && ev.rc === 0 && ev.f354 === 0) last.set(L.toString(), {k, src: 'fk_', len: n + m, v: v.toString(), bytes: v.readByteArray(8 * (n + m))});
   else last.delete(L.toString()); }});
 // 12ea3f(S, r, vec, out, work) inside the call: inputs at the moment 95da98 starts using them
 Interceptor.attach(b.add(0x1bbc70), {onEnter(a) { this.c = cur; if (!cur) return;
   this.S = a[0]; this.r = a[1].toInt32(); this.vec = a[2]; this.w0 = cur.w.isNull() ? null : hx(cur.w); },
  onLeave(rv) { const c = this.c; if (!c) return; const S = this.S, lp = S.add(0xe8).readPointer(), spx = S.add(0xe0).readPointer();
   const n = lp.add(0xc).readS32(), m = lp.add(8).readS32(), quad = spx.add(0x28).readS32();
   const ev = {e: 'row', rc: rv.toInt32(), r: this.r, n, m, quad, w0: this.w0, w: c.w.isNull() ? null : hx(c.w),
     lpsame: lp.equals(c.lp) ? 1 : 0, spxr: spx.add(0x2f8).readS32()};
   if (ev.rc !== 0 || quad !== 0) { emit(ev); return; }
   const cnt = this.vec.readS32(); ev.cnt = cnt;
   if (cnt >= 0) { ev.idx = iv(this.vec.add(8).readPointer(), cnt); ev.val = dv(this.vec.add(0x10).readPointer(), cnt); }
   else ev.val = dv(this.vec.add(0x10).readPointer(), n + m);
   const h1 = spx.add(0x470).readPointer(), h2 = spx.add(0x498).readPointer();
   const head = !h1.isNull() ? h1.add(0x18).readPointer() : (!h2.isNull() ? h2.add(0x18).readPointer() : ptr(0));
   ev.j = head.add(4 * this.r).readS32(); ev.x = hx(spx.add(0x1e8).readPointer().add(8 * this.r));
   ev.status = iv(spx.add(0x168).readPointer(), n + m);
   ev.rhs = dv(lp.add(0x380).readPointer(), m); ev.lb = dv(lp.add(0x390).readPointer(), n + m); ev.ub = dv(lp.add(0x398).readPointer(), n + m);
   ev.R = dv(lp.add(0x3d8).readPointer(), m); ev.C = dv(lp.add(0x3e0).readPointer(), n);
   ev.fm = lp.add(0x3c).readS32(); ev.flip = ev.fm > 0 ? bv(lp.add(0x408).readPointer(), n) : null;
   const env = S.add(0x100).readPointer(); ev.delta = hx(env.add(0x2460)); ev.inf = hx(env.add(0x2458)); ev.g2a40 = env.add(0x2a40).readS32(); ev.g2a48 = env.add(0x2a48).readS32();
   ev.ctxS = c.ctx.isNull() ? null : (c.ctx.add(8).readPointer().equals(S) ? 1 : 0);
   const m0 = c.M.add(8).readPointer().add(0xe8).readPointer().add(8).readS32(); ev.m0 = m0;
   if (m > m0) { const arr = c.M.add(0x3240).readPointer().add(0x78).readPointer(), cut = [];
     for (let t = 0; t < m - m0; t++) { let x = null; try { x = arr.add(8 * t).readPointer().add(0x38).readS32(); } catch (e) {} cut.push(x); }
     ev.cut = cut; }
   emit(ev); }});
 Interceptor.attach(b.add(0x61a1c0), {onEnter(a) { this.c = cur; if (cur) this.row = a[2].toInt32(); },
  onLeave(rv) { if (this.c) emit({e: 'q', row: this.row, ret: rv.toInt32() & 0xff}); }});
 Interceptor.attach(b.add(0x153cc0), {onEnter(a) { this.c = cur; if (cur) this.S = a[0]; },
  onLeave(rv) { if (this.c) emit({e: 'rs', rc: rv.toInt32(), st: this.S.add(100).readS32()}); }});
 Interceptor.attach(b.add(0x4f6650), {onEnter() { emit({e: 'x2a'}); }});
 Interceptor.attach(b.add(0x554ee0), {onEnter(a) { const I = a[1].add(0x18).readPointer(); this.L = I.isNull() ? null : I.add(8).readPointer(); },
  onLeave(r) { if (!this.L || this.L.isNull()) return; const L = this.L, p = last.get(L.toString()); if (!p) return;
   const v = L.add(0x70).readPointer(); if (v.toString() !== p.v) { emit({e: 'pp', k: p.k, moved: 1}); last.delete(L.toString()); return; }
   const now = new BigUint64Array(v.readByteArray(8 * p.len)), was = new BigUint64Array(p.bytes), ch = {};
   for (let i = 0; i < p.len; i++) if (now[i] !== was[i]) ch[i] = now[i].toString(16).padStart(16, '0');
   const ev = {e: 'pp', k: p.k, rc: r.toInt32(), d: Object.keys(ch).length, ch, l5c: L.add(0x5c).readS32(), l60: hx(L.add(0x60)), l68: hx(L.add(0x68))};
   const k = emit(ev); if (ev.d) last.set(L.toString(), {k, src: 'pp', len: p.len, v: p.v, bytes: v.readByteArray(8 * p.len)}); }});
 Interceptor.attach(b.add(0x5b73d0), {onEnter(a) { const I = a[0].add(0x18).readPointer(); if (I.isNull()) return;
   const L = I.add(8).readPointer(); if (L.isNull()) return; const l58 = L.add(0x58).readS32(); if (l58 < 0) return;
   const v = L.add(0x70).readPointer(), p = last.get(L.toString());
   const ev = {e: 'dec', l38: L.add(0x38).readS32(), l58, l54: L.add(0x54).readS32(), k: p ? p.k : -1, src: p ? p.src : null};
   if (p && !v.isNull()) { const now = new Uint8Array(v.readByteArray(8 * p.len)), was = new Uint8Array(p.bytes);
     let d = 0; for (let i = 0; i < now.length; i++) if (now[i] !== was[i]) d++; ev.vsame = v.toString() === p.v ? 1 : 0; ev.diff = d; }
   emit(ev); }});
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
   for (const ev of events) if (ev.e === 'row') for (const f of ['status', 'rhs', 'lb', 'ub', 'R', 'C', 'flip', 'cut']) {
     if (ev[f] == null) continue; const s = JSON.stringify(ev[f]), h = crypto.createHash('sha256').update(s).digest('hex').slice(0, 24);
     arrays[h] = ev[f]; ev[f] = h; }
   fs.mkdirSync(path.dirname(output), {recursive: true});
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'mip_infeas_ray', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok,
     detach_reason: reason, events, arrays})));
   const c = {}; for (const e of events) c[e.e] = (c[e.e] || 0) + 1;
   console.log(path.basename(model), outputText.trim(), JSON.stringify(c), 'ok', ok, errors.join(' ').slice(0, 300));
   process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
