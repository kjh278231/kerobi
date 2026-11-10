// M2-06f-2 (docs/m2/mip_node_life.md): read-only trace of node lifetime and of the node LP result vectors.
//   nn    5ff358(parent, M, &out) leave: node serial, parent serial, root flag (out == &M+0x790), depth, id
//   rel   5f9d4e(env, N, T, s) entry: node, stage, old stage I+0x68 (null without I); stage 6 also on leave (L made)
//   fr    5fa560(env, X, s) entry/leave: node, stage, I/L present, root flag; I present after (stage != 0)
//   mv    ac6569 (realloc wrapper) on a node N: the node keeps its serial at the new address
//   sol   12ef7f(lp, a, b) entry/leave: call site, path, and for the 1121ad main path the basic-column check
//   fk    95da98 at the 5ce67b call (0x4f46c6) entry/leave: r9 == r8 - 4, rc, L+0x58, L+0x54, L+0x70 present
//   dgate 5fe9cd 0x51679f and dec 6ee1d3: the M2-06e fields (tools/mip_pseudocost.js) plus node serial, depth, the
//         ancestor serials and end reason, and the provenance (event index, site) of the vector the decay reads
// Nodes are named by creation order and vectors by the index of the event that last wrote them, never by address.
// Every sent object gets the next event index. Nothing is written into the target. One invocation spawns one process.
// Usage: tools/gurun.sh node tools/mip_node_life.js MODEL OUTPUT.json.gz 'JSON_PARAMS'
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
const funcs = fs.readFileSync(path.join(root, 're/decomp/index.tsv'), 'utf8').trim().split('\n').slice(1).map(l => l.split('\t'))
  .map(r => [parseInt(r[0], 16), parseInt(r[2], 10), r[1].replace(/^PRIVATE0+/, '')]).sort((a, b) => a[0] - b[0]);
const params = Object.assign({Threads: 1, Seed: 0, LogToConsole: 0}, JSON.parse(extra || '{}'));
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\n` +
  `m.optimize()\nprint('DONE',m.Status,int(m.NodeCount),flush=True)`;
const agent = `
const FUNCS = ${JSON.stringify(funcs)};
function fname(o) { let lo = 0, hi = FUNCS.length - 1;
  while (lo < hi) { const mid = (lo + hi + 1) >> 1; if (FUNCS[mid][0] <= o) lo = mid; else hi = mid - 1; }
  const f = FUNCS[lo]; return f[0] <= o && o < f[0] + f[1] ? f[2] : 'raw:' + o.toString(16); }
const hx = p => p.readU64().toString(16).padStart(16, '0');
let seq = 0; const emit = ev => { send(ev); return seq++; };
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return;
 const b = m.base, off = p => p.sub(b).toUInt32();
 let depth = 0;
 Interceptor.attach(b.add(0x4e36e0), {onEnter() { depth++; }, onLeave() { depth--; }});
 // ---- node serials (creation order); -1 = null pointer, -3 = pointer never created by 5ff358
 const nodes = new Map(); let nodeSerial = 0;
 const nser = n => n.isNull() ? -1 : (nodes.has(n.toString()) ? nodes.get(n.toString()) : -3);
 const prov = new Map();   // vector address -> [event index, site]
 const pv = p => p.isNull() ? null : (prov.has(p.toString()) ? prov.get(p.toString()) : [-1, 'none']);
 Interceptor.attach(b.add(0x5177f0), {onEnter(a) { this.par = a[0]; this.M = a[1]; this.out = a[2]; this.from = fname(off(this.returnAddress)); },
  onLeave(r) { const rc = r.toInt32(), N = this.out.readPointer();
   if (rc !== 0 || N.isNull()) { emit({e: 'nn', nest: depth, rc, from: this.from}); return; }
   const s = nodeSerial++; nodes.set(N.toString(), s); const I = N.add(0x18).readPointer();
   emit({e: 'nn', nest: depth, rc, from: this.from, s, par: nser(this.par), root: this.out.equals(this.M.add(0x790)) ? 1 : 0,
     depth: I.add(0x2c).readS32(), id: N.readDouble()}); }});
 // ac6569(env, p, size) is the realloc wrapper: a node N grows with its bound-change list and may move
 Interceptor.attach(b.add(0x8d1eb0), {onEnter(a) { this.k = a[1].isNull() ? undefined : nodes.get(a[1].toString());
   if (this.k !== undefined) { this.p = a[1]; this.ra = off(this.returnAddress); } },
  onLeave(r) { if (this.k === undefined) return; const moved = r.equals(this.p) ? 0 : 1;
   if (moved && !r.isNull()) { nodes.delete(this.p.toString()); nodes.set(r.toString(), this.k); }
   emit({e: 'mv', n: this.k, moved, from: fname(this.ra)}); }});
 Interceptor.attach(b.add(0x516bc0), {onEnter(a) { const N = a[1], s = a[3].toInt32(), I = N.add(0x18).readPointer();
   this.N = N; this.s = s;
   const ev = {e: 'rel', nest: depth, from: fname(off(this.returnAddress)), n: nser(N), s, T: a[2].isNull() ? 0 : 1,
     old: I.isNull() ? null : I.add(0x68).readS32()};
   if (s === 6) ev.L = I.isNull() ? null : (I.add(8).readPointer().isNull() ? 0 : 1);
   emit(ev); },
  onLeave(r) { if (this.s !== 6) return; const I = this.N.add(0x18).readPointer();
   emit({e: 'rel6', n: nser(this.N), rc: r.toInt32(), L: I.isNull() ? null : (I.add(8).readPointer().isNull() ? 0 : 1)}); }});
 Interceptor.attach(b.add(0x516f10), {onEnter(a) { const X = a[1], s = a[2].toInt32(), I = X.add(0x18).readPointer();
   this.X = X; this.s = s; this.k = nser(X);
   emit({e: 'fr', n: this.k, s, I: I.isNull() ? 0 : 1, L: I.isNull() ? null : (I.add(8).readPointer().isNull() ? 0 : 1),
     root: I.isNull() ? null : (X.equals(I.readPointer().add(0x790).readPointer()) ? 1 : 0)}); },
  onLeave() { if (this.s === 0) { nodes.delete(this.X.toString()); emit({e: 'fr_', n: this.k, gone: 1}); return; }
   const I = this.X.add(0x18).readPointer();
   emit({e: 'fr_', n: this.k, I: I.isNull() ? 0 : 1, L: I.isNull() ? null : (I.add(8).readPointer().isNull() ? 0 : 1)}); }});
 // ---- 12ef7f(lp, rc n, pi m)
 Interceptor.attach(b.add(0x1bbe40), {onEnter(a) { const lp = a[0]; this.o1 = a[1]; this.ra = off(this.returnAddress);
   let path = 'err', spx = ptr(0);
   if (a[1].isNull() && a[2].isNull()) path = 'none';
   else if (!lp.isNull() && !lp.add(0xe8).readPointer().isNull()) { spx = lp.add(0xe0).readPointer();
     if (!spx.isNull() && spx.add(0x28).readS32() !== 0) path = '23ac2b';
     else if (spx.isNull()) path = 'copy';
     else path = spx.add(0x488).readPointer().isNull() ? (spx.add(0x480).readPointer().isNull() ? 'main' : 'main480') : 'main488'; }
   this.path = path; this.spx = spx; },
  onLeave(r) { const ev = {e: 'sol', ra: this.ra, from: fname(this.ra), path: this.path, rc: r.toInt32()};
   if ((this.path === 'main' || this.path === 'main480') && !this.o1.isNull() && ev.rc === 0) {
     const n = this.spx.add(0x68).readS32(); ev.ncol = n;
     if (n > 0) { const vs = new Int32Array(this.spx.add(0x168).readPointer().readByteArray(4 * n));
       const d = new BigUint64Array(this.o1.readByteArray(8 * n)); let bas = 0, basnz = 0, nb = 0, nbz = 0;
       for (let j = 0; j < n; j++) { const z = (d[j] & 0x7fffffffffffffffn) === 0n;
         if (vs[j] >= 0) { bas++; if (!z) basnz++; } else { nb++; if (z) nbz++; } }
       Object.assign(ev, {bas, basnz, nb, nbz}); } }
   const k = emit(ev); if (!this.o1.isNull() && ev.rc === 0) prov.set(this.o1.toString(), [k, this.ra]); }});
 // ---- 95da98 at the 5ce67b infeasible site
 Interceptor.attach(b.add(0x781170), {onEnter(a) { this.ra = off(this.returnAddress); if (this.ra !== 0x4f46cb) return;
   this.L = a[4].sub(0x58); this.p354 = a[12];
   emit({e: 'fk', r9: a[5].equals(a[4].sub(4)) ? 1 : 0, v70: this.L.add(0x70).readPointer().isNull() ? 0 : 1}); },
  onLeave(r) { if (this.ra !== 0x4f46cb) return; const L = this.L, v = L.add(0x70).readPointer();
   const k = emit({e: 'fk_', rc: r.toInt32(), f354: this.p354.readS32(), l58: L.add(0x58).readS32(), l54: L.add(0x54).readS32(),
     v70: v.isNull() ? 0 : 1});
   if (!v.isNull() && r.toInt32() === 0) prov.set(v.toString(), [k, 0x781170]); }});
 // ---- 5fe9cd decay gate (M2-06e fields + node serial)
 Interceptor.attach(b.add(0x51679f), function () { const x = this.context, N = x.r15, K = x.rbx;
   const I = N.add(0x18).readPointer();
   emit({e: 'dgate', nest: depth, id: N.readDouble(), parent: N.add(0x10).readPointer().isNull() ? 0 : 1, ik: I.equals(K) ? 1 : 0,
     st: I.add(0x28).readS32(), j: K.add(0x34).readS32(), lp: hx(K.add(0x10)), p: hx(I.add(0x78)), n: nser(N)}); });
 // ---- 6ee1d3(N, T) (M2-06e fields without the table entries + serials, depth, provenance)
 Interceptor.attach(b.add(0x5b73d0), {onEnter(a) { const N = a[0];
   const ev = {e: 'dec', nest: depth, id: N.readDouble(), n: nser(N)}; this.ev = ev;
   const I = N.add(0x18).readPointer(); if (I.isNull()) { ev.noI = 1; return; }
   ev.depth = I.add(0x2c).readS32();
   const L = I.add(8).readPointer(); if (L.isNull()) { ev.noL = 1; return; }
   ev.l38 = L.add(0x38).readS32(); ev.l58 = L.add(0x58).readS32(); ev.l54 = L.add(0x54).readS32();
   ev.l18 = L.add(0x18).readPointer().isNull() ? 0 : 1; ev.l28 = L.add(0x28).readPointer().isNull() ? 0 : 1;
   ev.l70 = L.add(0x70).readPointer().isNull() ? 0 : 1;
   ev.p28 = pv(L.add(0x28).readPointer()); ev.p70 = pv(L.add(0x70).readPointer());
   const v = (ev.l38 === 6 || ev.l58 >= 0) ? L.add(0x70).readPointer() : (ev.l18 ? L.add(0x28).readPointer() : ptr(0));
   const anc = [], as = [];
   let A = N.add(0x10).readPointer(), k = 0;
   for (; !A.isNull() && k < 40; A = A.add(0x10).readPointer(), k++) {
     const AI = A.add(0x18).readPointer(); as.push(nser(A)); if (AI.isNull()) { anc.push(null); break; }
     const j = AI.add(0x34).readS32(), dir = AI.add(0x40).readU8();
     anc.push([j, dir, j >= 0 && !v.isNull() ? hx(v.add(8 * j)) : null]); }
   ev.anc = anc; ev.as = as; ev.end = A.isNull() ? 'nop' : (k >= 40 ? 'cap' : 'noI'); },
  onLeave() { emit(this.ev); }});
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
   fs.mkdirSync(path.dirname(output), {recursive: true});
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'mip_node_life', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok,
     detach_reason: reason, events})));
   const c = {}; for (const e of events) c[e.e] = (c[e.e] || 0) + 1;
   console.log(path.basename(model), outputText.trim(), JSON.stringify(c), 'ok', ok, errors.join(' ').slice(0, 300));
   process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
