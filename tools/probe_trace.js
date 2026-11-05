// Probing oracle (docs/m2_red_probe.md): run Gurobi Model.presolve() on one model under Frida and dump, for every pass
// of every `4d3a1d` call, what tools/probe_check.py needs to replay the pass in the port bit for bit:
//   pass   (`5b9aad` call at 0x458ae9, the pass start): state (bounds, types, rows, counts, activities, row lists in
//          list order with dead nodes, work), conflict graph *(st+0x3f0), env values, 4d3a1d arguments and timer;
//   merged (same call, return): conflict graph and work;
//   ctx_in / ctx_out (`4e48f7` entry / return): the probe context, candidates and work;
//   imp    (`5b86ce` entry): bounds, counts, conflict graph and work before the implication export;
//   end    (`5b9aad` call at 0x460ed5, return): conflict graph, *out and work.
// Doubles are hex strings of their bits (JSON would drop -0.0). Gurobi: one process at a time (wrap in tools/gurun.sh).
// usage: node tools/probe_trace.js <model> <out.json> [Param=Value ...]
const FRIDA = process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida';
const frida = require(FRIDA);
const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');

const [model, outPath, ...kv] = process.argv.slice(2);
const root = path.resolve(__dirname, '..', '..');
const shortOf = n => { const m = /^PRIVATE0*([0-9a-f]+)$/.exec(n); return m ? m[1].slice(-6).replace(/^0+/, '') : n; };
const idx = fs.readFileSync(path.join(root, 're/decomp/index.tsv'), 'utf8').trim().split('\n').slice(1).map(l => l.split('\t'));
const off = s => { const r = idx.find(r => shortOf(r[1]) === s); if (!r) throw new Error('no fn ' + s); return parseInt(r[0], 16); };
const mps = execFileSync(path.join(root, '.venv/bin/python'), ['-I', '-c',
  `import sys; sys.path.insert(0, ${JSON.stringify(__dirname)}); from parity import instances; print(instances()[${JSON.stringify(model)}])`]).toString().trim();
const params = { LogToConsole: 0, Threads: 1, WorkLimit: 60 };
for (const p of kv) { const [k, v] = p.split('='); params[k] = Number(v); }
const pyCode = `
import gurobipy as gp
env = gp.Env(params=${JSON.stringify(params)})
m = gp.read(${JSON.stringify(mps)}, env)
q = m.presolve()
print("DONE", q.NumConstrs, q.NumVars, flush=True)
`;
const fn = { probe: off('4d3a1d'), merge: off('5b9aad'), engine: off('4e48f7'), limit: off('5b86ce'), state: off('5b84b1') };
const agent = `
const FN = ${JSON.stringify(fn)};
let base = null, st = null, env = null, depth = 0, call = 0, pass = 0, out = [];
function emit(x) { x.call = call; x.pass = pass; out.push(x); if (out.length > 4) { send(out); out = []; } }
function H(p, n) { if (p.isNull() || n <= 0) return null; return Array.from(new BigUint64Array(p.readByteArray(8 * n)), x => x.toString(16)); }
function I32(p, n) { if (p.isNull() || n <= 0) return null; return Array.from(new Int32Array(p.readByteArray(4 * n))); }
function I64(p, n) { if (p.isNull() || n <= 0) return null; return Array.from(new BigInt64Array(p.readByteArray(8 * n)), x => Number(x)); }
function U8(p, n) { if (p.isNull() || n <= 0) return null; return Array.from(new Uint8Array(p.readByteArray(n))); }
function CH(p, n) { if (p.isNull() || n <= 0) return null; return String.fromCharCode.apply(null, new Uint8Array(p.readByteArray(n))); }
const P = (b, o) => b.add(o).readPointer(), I = (b, o) => b.add(o).readS32(), D = (b, o) => H(b.add(o), 1)[0];
function work() { const w = P(st, 0x458); return w.isNull() ? null : H(w, 1)[0]; }
function graph() {
  const g = P(st, 0x3f0); if (g.isNull()) return null;
  const n0 = I(g, 0), npend = g.add(0x18).readS64().toNumber(), nflag = I(g, 0x38);
  const o = { n0, rc0: I(g, 4), npend, cap: g.add(0x20).readS64().toNumber(), nflag };
  o.ptr = n0 > 0 ? I64(P(g, 8), 2 * n0 + 1) : null;
  o.adj = o.ptr ? I32(P(g, 0x10), o.ptr[2 * n0]) : null;
  o.pa = I32(P(g, 0x28), npend); o.pb = I32(P(g, 0x30), npend);
  o.flag = U8(P(g, 0x40), nflag);
  return o;
}
function state() {
  const n = I(st, 0xc), m = I(st, 0x10), s = { n, m };
  s.lb = H(P(st, 0xa0), n); s.ub = H(P(st, 0xa8), n); s.obj = H(P(st, 0xb0), n); s.vtype = CH(P(st, 0x150), n);
  s.rhs = H(P(st, 0xb8), m); s.sense = CH(P(st, 0xc0), m);
  s.row_cnt = I32(P(st, 0x60), m); s.col_cnt = I32(P(st, 0x68), n);
  s.inf_max = I32(P(st, 0x70), m); s.inf_min = I32(P(st, 0x78), m); s.act_max = H(P(st, 0x90), m); s.act_min = H(P(st, 0x98), m);
  s.mult_lo = H(P(st, 0x80), n); s.mult_hi = H(P(st, 0x88), n);
  s.objcon = D(st, 0xc8); s.nnz = st.add(0xd8).readS64().toNumber(); s.cutoff = D(st, 0x490);
  s.mu = D(st, 0x448); s.work = work(); s.work_base = D(st, 0x450); s.budget = D(st, 0x1f0); s.prelimit = I(st, 0x1dc);
  s.eq_rows = I(st, 0xf8); s.cols_removed = I(st, 0x408); s.rows_removed = I(st, 0x404);
  s.x1f8 = I(st, 0x1f8); s.x240 = I(st, 0x240); s.x27c = I(st, 0x27c); s.x2f8 = I(st, 0x2f8); s.x330 = I(st, 0x330);
  s.p250 = P(st, 0x250).toString(); s.p3c0 = P(st, 0x3c0).toString(); s.p3c8 = P(st, 0x3c8).toString();
  // row lists in list order, dead nodes (col -1) included: beg (m+1), col, val
  const rh = P(st, 0x178), beg = [0], col = [], val = [];
  for (let i = 0; i < m; i++) {
    let q = rh.add(8 * i).readPointer();
    while (!q.isNull()) { col.push(q.add(12).readS32()); val.push(H(q, 1)[0]); q = q.add(0x10).readPointer(); }
    beg.push(col.length);
  }
  s.rbeg = beg; s.rcol = col; s.rval = val;
  s.env = { dual: I(env, 0x2430), aggmult: D(env, 0x26c0), seed: I(env, 0x2cd0), feastol: D(env, 0x2460), intfeastol: D(env, 0x2468),
            preclique: I(env, 0x254c), presolve: I(env, 0x2424), infbound: D(env, 0x2458), pregubprobe: I(env, 0x2830), x2728: I(env, 0x2728) };
  return s;
}
function ctx(c, full) {
  const nr = I(c, 0x20), nrow0 = I(c, 0x24), nc = I(c, 0x28), n = I(c, 0x2c), k6 = I(c, 0x30);
  const o = { nr, nrow0, nc, n, k6, mu: D(c, 0x18), itol: D(c, 0xd0), budget: D(c, 0xd8), fac: D(c, 0xe0), stop: I(c, 0xe8), p7: I(c, 0xf0),
    nchg: I(c, 0xf4), nbin: I(c, 0xf8), neq: I(c, 0xfc), nagg: I(c, 0x140), rec: I(c, 0x1b0), n37: I(c, 0x1b8), lim: I(c, 0x1e0), imp: I(c, 0x1e4),
    groups: I(c, 0x1e8), used: I(c, 0x1ec), cap: I(c, 0x1f0) };
  o.lb = H(P(c, 0x100), nc); o.ub = H(P(c, 0x108), nc); o.amin = H(P(c, 0x128), nrow0 + k6); o.amax = H(P(c, 0x130), nrow0 + k6);
  o.sense = CH(P(c, 0x120), nr); o.aggto = I32(P(c, 0x148), nc); o.aggmul = H(P(c, 0x150), nc); o.aggoff = H(P(c, 0x158), nc);
  o.cnt0 = I32(P(c, 0x160), nc); o.cnt1 = I32(P(c, 0x168), nc);
  if (full) {
    const rs = I32(P(c, 0x50), nr + 1); o.rbeg = rs; o.rnb = I32(P(c, 0x58), nr); o.rcol = I32(P(c, 0x60), rs[nr]); o.rval = H(P(c, 0x68), rs[nr]);
    const cs = I32(P(c, 0x38), nc + 1); o.cbeg = cs; o.crow = I32(P(c, 0x40), cs[nc]); o.cval = H(P(c, 0x48), cs[nc]);
    o.rhs = H(P(c, 0x80), nr); o.range = H(P(c, 0x88), nrow0 + k6); o.vt = CH(P(c, 0x98), nc); o.lb0 = H(P(c, 0x110), nc); o.ub0 = H(P(c, 0x118), nc);
  } else {
    const g = I(c, 0x1e8); if (o.imp || g) { o.gbeg = I32(P(c, 0x1f8), 2 * g + 1); o.imps = I32(P(c, 0x200), I(c, 0x1ec)); }
    if (o.rec) { o.rhead = I32(P(c, 0x1c0), nrow0); o.rnext = I32(P(c, 0x1c8), o.n37); o.rlit = I32(P(c, 0x1d0), o.n37); o.rslack = H(P(c, 0x1d8), o.n37); }
  }
  return o;
}
function install() {
  Interceptor.attach(base.add(FN.state), { onEnter(a) { st = a[1].sub(0x3f0); env = a[0]; } });
  Interceptor.attach(base.add(FN.probe), {
    onEnter(a) {
      depth++; call++; pass = 0; env = a[3];
      const t = a[4], args = []; for (let i = 5; i < 10; i++) args.push(a[i].toInt32());
      emit({ ev: 'call', args, timer: { acc: H(t, 1)[0], limit: H(t.add(16), 1)[0], running: I(t, 0x28) }, k: this.context.rbx.toInt32() });
    },
    onLeave(r) { emit({ ev: 'ret', ret: r.toInt32(), work: work() }); depth--; }
  });
  Interceptor.attach(base.add(FN.merge), {
    onEnter(a) {
      this.on = depth > 0; if (!this.on) return;
      this.first = this.returnAddress.sub(base).toInt32() === 0x458ae9; this.o = a[4];
      if (this.first) { pass++; emit({ ev: 'pass', s: state(), g: graph() }); }
      else emit({ ev: 'end_in', g: graph(), work: work() });
    },
    onLeave(r) {
      if (!this.on) return;
      emit({ ev: this.first ? 'merged' : 'end', ret: r.toInt32(), g: graph(), work: work(), out: this.o.isNull() ? null : this.o.readS64().toNumber() });
    }
  });
  Interceptor.attach(base.add(FN.engine), {
    onEnter(a) { this.c = a[1]; this.cand = a[2]; this.nc = a[3].toInt32(); emit({ ev: 'ctx_in', c: ctx(a[1], true), cand: I32(a[2], this.nc), work: work() }); },
    onLeave(r) { emit({ ev: 'ctx_out', ret: r.toInt32(), c: ctx(this.c, false), cand: I32(this.cand, this.nc), work: work() }); }
  });
  Interceptor.attach(base.add(FN.limit), {
    onEnter(a) {
      if (depth === 0) return;
      const n = I(st, 0xc);
      emit({ ev: 'imp', lb: H(P(st, 0xa0), n), ub: H(P(st, 0xa8), n), col_cnt: I32(P(st, 0x68), n), nnz: st.add(0xd8).readS64().toNumber(),
             cols_removed: I(st, 0x408), rows_removed: I(st, 0x404), m: I(st, 0x10), g: graph(), work: work() });
    }
  });
}
Process.attachModuleObserver({ onAdded(m) { if (m.name === 'libgurobi130.so') { base = m.base; install(); } } });
recv('flush', () => { send(out); out = []; send('flushed'); });
`;
(async () => {
  const device = await frida.getLocalDevice();
  let res = [], text = '', done = false;
  const pid = await device.spawn([path.join(root, '.venv/bin/python'), '-c', pyCode], { stdio: 'pipe' });
  const session = await device.attach(pid);
  const script = await session.createScript(agent);
  script.message.connect(msg => {
    if (msg.type === 'error') { console.error(msg.stack); return; }
    if (msg.payload === 'flushed') {
      fs.writeFileSync(outPath, JSON.stringify(res));
      console.log(model, text.trim().split('\n').pop(), res.length, 'events');
      device.kill(pid).catch(() => {}).then(() => process.exit(0));
      return;
    }
    res = res.concat(msg.payload);
  });
  device.output.connect((p, fd, data) => {
    if (p !== pid) return;
    text += data.toString();
    if (fd === 2) process.stderr.write(data);
    if (text.includes('DONE') && !done) { done = true; script.post({ type: 'flush' }); }
  });
  session.detached.connect(reason => { if (!done) { console.error('detached', reason, text); process.exit(1); } });
  await script.load();
  await device.resume(pid);
})();
