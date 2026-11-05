// MIP presolve state oracle (docs/m2_presolve.md §2a): run Gurobi Model.presolve() on one model under Frida and dump
// the 3cd406 presolve state (linked-list matrix, bounds, types, rows, activities, counters, work) at entry and return
// and the postsolve record list (type + raw data bytes, push order) of the n-th call of a chosen function, so one reduction can be checked in isolation (load "before", run, compare "after").
// usage: node tools/mip_state_dump.js <model name | .mps/.lp path> <fn short id> <call no. (1-based) | all> <out.json> [Param=Value ...]
// The state pointer comes from 5b84b1(env, state+0x3f0), which 3cd406 calls right after building the state.
// Gurobi: one process at a time (wrap in tools/gurun.sh).
const FRIDA = process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida';
const frida = require(FRIDA);
const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');

const [model, fn, nth, outPath, ...kv] = process.argv.slice(2);
const root = path.resolve(__dirname, '..', '..');
const shortOf = n => { const m = /^PRIVATE0*([0-9a-f]+)$/.exec(n); return m ? m[1].slice(-6).replace(/^0+/, '') : n; };
const idx = fs.readFileSync(path.join(root, 're/decomp/index.tsv'), 'utf8').trim().split('\n').slice(1).map(l => l.split('\t'));
const off = s => { const r = idx.find(r => shortOf(r[1]) === s); if (!r) throw new Error('no fn ' + s); return parseInt(r[0], 16); };
const mps = model.endsWith('.mps') || model.endsWith('.lp') ? model : execFileSync(path.join(root, '.venv/bin/python'), ['-I', '-c',
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
const agent = `
const F = ${off(fn)}, P = ${off('5b84b1')}, NTH = ${nth === 'all' ? 0 : Number(nth)};
let base = null, st = null, calls = 0;
const I = (p, o) => p.add(o).readS32(), D = (p, o) => p.add(o).readDouble(), Pt = (p, o) => p.add(o).readPointer();
function arrD(p, n) { const a = []; if (p.isNull()) return null; for (let i = 0; i < n; i++) a.push(p.add(8 * i).readDouble()); return a; }
function arrI(p, n) { const a = []; if (p.isNull()) return null; for (let i = 0; i < n; i++) a.push(p.add(4 * i).readS32()); return a; }
function arrC(p, n) { if (p.isNull()) return null; let s = ''; for (let i = 0; i < n; i++) s += String.fromCharCode(p.add(i).readU8()); return s; }
function walk(heads, k, nextOff) {  // list k: [[row, col, value], ...] in list order
  const out = []; let q = heads.add(8 * k).readPointer(), guard = 0;
  while (!q.isNull() && guard++ < 50000000) { out.push([q.add(8).readS32(), q.add(12).readS32(), q.readDouble()]); q = q.add(nextOff).readPointer(); }
  return out;
}
function dump(tag) {
  const n = I(st, 0xc), m = I(st, 0x10), s = {tag, n, m};
  s.ptype = I(st, 0x08); s.nbin = I(st, 0xf0); s.nint = I(st, 0xf4); s.fc = I(st, 0xe0); s.mip = I(st, 0xfc);
  s.lb = arrD(Pt(st, 0xa0), n); s.ub = arrD(Pt(st, 0xa8), n); s.obj = arrD(Pt(st, 0xb0), n);
  s.vtype = arrC(Pt(st, 0x150), n); s.rhs = arrD(Pt(st, 0xb8), m); s.sense = arrC(Pt(st, 0xc0), m);
  s.row_cnt = arrI(Pt(st, 0x60), m); s.col_cnt = arrI(Pt(st, 0x68), n);
  s.inf_max = arrI(Pt(st, 0x70), m); s.inf_min = arrI(Pt(st, 0x78), m); s.act_max = arrD(Pt(st, 0x90), m); s.act_min = arrD(Pt(st, 0x98), m);
  s.mult_lo = arrD(Pt(st, 0x80), n); s.mult_hi = arrD(Pt(st, 0x88), n);
  s.col_flag = (() => { const p = Pt(st, 0x170); if (p.isNull()) return null; const o = []; for (let j = 0; j < n; j++) o.push(p.add(4 * j).readU32()); return o; })();
  s.objcon = D(st, 0xc8); s.nnz = st.add(0xd8).readS64().toString();
  s.counters = arrI(st.add(0x3f8), 14);  // +0x3f8 tightened, +0x3fc coefs, +0x400 ineq->eq, +0x404 rows, +0x408 cols, ...
  const w = Pt(st, 0x458); s.work = w.isNull() ? null : w.readDouble(); s.work_mult = D(st, 0x448);
  s.prelimit = I(st, 0x1dc); s.budget = D(st, 0x1f0);
  const rh = Pt(st, 0x178), ch = Pt(st, 0x180);
  if (!rh.isNull()) { s.rows = []; for (let i = 0; i < m; i++) s.rows.push(walk(rh, i, 0x10)); }
  if (!ch.isNull()) { s.cols = []; for (let j = 0; j < n; j++) s.cols.push(walk(ch, j, 0x18)); }
  s.free = [st.add(0x198).readS64().toString(), st.add(0x188).readS64().toString()];
  // postsolve records (list at +0x370, newest first; rec = {type, data, next} + data in a bump arena), in push order
  // with their raw data bytes (size = gap to the next record, capped at 64 KiB when the next one is in another chunk)
  const recs = []; let r = Pt(st, 0x370), guard = 0;
  while (!r.isNull() && guard++ < 10000000) { recs.push(r); r = r.add(0x10).readPointer(); }
  recs.reverse();
  s.recs = recs.map((q, i) => {
    const t = q.readS32(), d = q.add(8).readPointer();
    let len = 512;
    if (i + 1 < recs.length) { const g = recs[i + 1].sub(d).toInt32(); if (g > 0 && g < 65536) len = g; }
    const b = new Uint8Array(d.readByteArray(len));
    return [t, Array.from(b, x => x.toString(16).padStart(2, "0")).join(""), q.toString(), d.toString()];
  });
  // change tracker (state+0x3d8): level, flag bytes, per level a/b lengths and lists, the list getters' caches
  // (538f30 rows +0x94/+0xa0, 5399fc cols +0xb8/+0xc8, 53a487 rows +0xe0/+0xf0, 53af0f cols +0x108/+0x118; -1 none)
  const tr = Pt(st, 0x3d8);
  if (!tr.isNull()) {
    const T = { level: I(tr, 0x10), active: [0, 1, 2].map(L => I(tr, 0x88 + 4 * L)) };
    const lst = (c, p) => (c < 0 || p.isNull()) ? null : arrI(p, c);
    for (const [k, fl, a, b, l, c1, l1, c2, l2, dim] of [['r', 0x18, 0x28, 0x34, 0x40, 0x94, 0xa0, 0xe0, 0xf0, m],
                                                         ['c', 0x20, 0x58, 0x64, 0x70, 0xb8, 0xc8, 0x108, 0x118, n]]) {
      const fp = Pt(tr, fl); const f = []; if (!fp.isNull()) for (let i = 0; i < dim; i++) f.push(fp.add(i).readU8());
      T[k] = { flag: f, a: [0, 1, 2].map(L => I(tr, a + 4 * L)), b: [0, 1, 2].map(L => I(tr, b + 4 * L)),
               list: [0, 1, 2].map(L => lst(I(tr, b + 4 * L), Pt(tr, l + 8 * L))),
               cache: [0, 1, 2].map(L => lst(I(tr, c1 + 4 * L), Pt(tr, l1 + 8 * L))),
               cache2: [0, 1, 2].map(L => lst(I(tr, c2 + 4 * L), Pt(tr, l2 + 8 * L))) };
    }
    s.tracker = T;
  }
  return s;
}
function install() {
  Interceptor.attach(base.add(P), { onEnter(a) { st = a[1].sub(0x3f0); } });
  Interceptor.attach(base.add(F), {
    onEnter(a) { calls++; this.on = st && (NTH === 0 || calls === NTH); if (this.on) send({ call: calls, s: dump('enter') }); },
    onLeave(r) { if (this.on) send({ call: calls, ret: r.toInt32(), s: dump('leave') }); }
  });
}
Process.attachModuleObserver({ onAdded(m) { if (m.name === 'libgurobi130.so') { base = m.base; install(); } } });
`;
(async () => {
  const device = await frida.getLocalDevice();
  const res = []; let text = '';
  const pid = await device.spawn([path.join(root, '.venv/bin/python'), '-c', pyCode], { stdio: 'pipe' });
  const session = await device.attach(pid);
  const script = await session.createScript(agent);
  script.message.connect(msg => { if (msg.type === 'error') console.error(msg.stack); else res.push(msg.payload); });
  device.output.connect(async (p, fd, data) => {
    if (p !== pid) return;
    text += data.toString();
    if (fd === 2) process.stderr.write(data);
    if (text.includes('DONE')) {
      setTimeout(async () => { fs.writeFileSync(outPath, JSON.stringify(res)); console.log(text.trim().split('\n').pop(), res.length, 'dumps'); await device.kill(pid).catch(() => {}); process.exit(0); }, 300);
    }
  });
  session.detached.connect(reason => { if (!text.includes('DONE')) { console.error('detached', reason, text); process.exit(1); } });
  await script.load();
  await device.resume(pid);
})();
