// M1-02j: per dual pivot (8b473 called from 28c93), the two-RHS FTRAN 09d112 and its L / row-eta kernels: every
// 21b9de (sparse L) and 21f83a (sparse row etas) call made from 09d112 with its input and output vectors, and the
// L factor (LU+0x20 struct: column starts +0x38, rows +0x48, values +0x40, levels +0xf8, level starts +0x100,
// level count +0x108, LU+0xe4 = L+0xc4) plus the row -> L index map LU+0x30, dumped whenever it changes.
// Read-only hooks (docs/m1/m1_02j_lsolve.md section 2). Addresses are never stored.
// usage: tools/gurun.sh node tools/m1_lsolve.js MODEL OUTPUT.json.gz
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
 let k = 0, cur = null, ft = null, nl = 0, lsig = '', lupd = -1;
 const ldump = (LU, m) => {  // the L factor as 21b9de reads it, once per change
   const L = LU.add(0x20), cp = L.add(0x38).readPointer(), cpm = cp.add(8 * m).readS64().toNumber();
   const sig = [cp, L.add(0x48).readPointer(), L.add(0x40).readPointer(), cpm].join(':');
   const upd = LU.add(0x18c).readS32();
   if (sig !== lsig || upd < lupd) {
     const beg = longs(cp, m + 1), c0 = beg[0], nnz = cpm - c0, x128 = L.add(0x108).readS32();
     send({type: 'L', id: nl, k, m, beg, row: ints(L.add(0x48).readPointer().add(4 * c0), nnz),
       val: dbls(L.add(0x40).readPointer().add(8 * c0), nnz), x128, xe4: L.add(0xc4).readS32(),
       lev: x128 >= 0 ? ints(L.add(0xf8).readPointer(), m) : null,
       lst: x128 >= 0 ? ints(L.add(0x100).readPointer(), x128 + 2) : null, perm: ints(LU.add(0x30).readPointer(), m)});
     lsig = sig; nl++;
   }
   lupd = upd; return nl - 1; };
 Interceptor.attach(b.add(0x11f620), {
  onEnter(a) { this.on = this.returnAddress.sub(b).toUInt32() === 0xc58d5; if (!this.on) return;
    cur = {type: 'P', k: k++, ft: []}; },
  onLeave() { if (!this.on) return; send(cur); cur = null; }});
 Interceptor.attach(b.add(0x125f50), {
  onEnter(a) { this.on = cur !== null && this.returnAddress.sub(b).toUInt32() === 0x120200; if (!this.on) return;
    const LU = a[0], m = LU.add(8).readS32(); this.a = [a[1], a[3]]; this.m = m;
    ft = {m, r: a[6].toInt32(), lid: ldump(LU, m), upd: LU.add(0x18c).readS32(), x180: LU.add(0x180).readS32(),
      aq_in: vec(a[2]), rho_in: vec(a[4]), calls: []}; },
  onLeave() { if (!this.on) return; ft.aq_out = vec(this.a[0]); ft.tau_out_nnz = this.a[1].readS32();
    cur.ft.push(ft); ft = null; }});
 // 21b9de(tol, LU, m, L, in, out, mark, work, buckets, &work): in and out in L index space
 const lsites = {0x1261af: 1, 0x126210: 1, 0x126284: 1, 0x1263ac: 1};
 Interceptor.attach(b.add(0x12a990), {
  onEnter(a) { this.on = ft !== null && lsites[this.returnAddress.sub(b).toUInt32() - 5] === 1; if (!this.on) return;
    this.c = {f: '21b9de', site: (this.returnAddress.sub(b).toUInt32() - 5).toString(16), m: a[1].toInt32(),
      x128: a[2].add(0x108).readS32(), xe4: a[2].add(0xc4).readS32(), in: vec(a[3])}; this.o = a[4]; },
  onLeave() { if (!this.on) return; this.c.out = vec(this.o); ft.calls.push(this.c); }});
 // 21f83a(tol, LU, n_eta, ?, ?, v1, v2, v3, v4, ...): 09d112's own vectors: the four vectors before and after
 const rsites = {0x1279e5: 1, 0x127a7e: 1};
 Interceptor.attach(b.add(0x129aa0), {
  onEnter(a) { this.on = ft !== null && rsites[this.returnAddress.sub(b).toUInt32() - 5] === 1; if (!this.on) return;
    this.v = [a[4], a[5], a[6], a[7]];
    this.c = {f: '21f83a', site: (this.returnAddress.sub(b).toUInt32() - 5).toString(16), neta: a[1].toInt32(),
      before: this.v.map(vec)}; },
  onLeave() { if (!this.on) return; this.c.after = this.v.map(vec); ft.calls.push(this.c); }});
 // the remaining 09d112 callees by call site, for the path record
 const sites = {0x1262c2: '214344', 0x126347: '214344', 0x129547: '21841f', 0x12959f: '2126cd', 0x1295e0: '21686f',
   0x129634: '21574d'};
 const addr = {'214344': 0x12bd30, '21841f': 0x12c300, '2126cd': 0x121ef0, '21686f': 0x1223e0, '21574d': 0x123690};
 for (const f of Object.keys(addr)) Interceptor.attach(b.add(addr[f]), {onEnter() { if (!ft) return;
   const s = this.returnAddress.sub(b).toUInt32() - 5; if (sites[s] === f) ft.calls.push({f, site: s.toString(16)}); }});
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'm1_lsolve', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason, events}) + '\n'));
   console.log(path.basename(model), outputText.trim(), 'events', events.length, 'ok', ok, errors.join(' ')); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
