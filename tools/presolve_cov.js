// Presolve coverage oracle (docs/m2_presolve.md §1): run Gurobi Model.presolve() on each model (Threads=1, WorkLimit 60,
// as tools/parity.py) under Frida and record, per model, how often each hooked libgurobi function ran, a backtrace of
// the first call of every non-presolve function, and the sequence of direct calls made by the entry function.
// usage: node tools/presolve_cov.js <entry short id, e.g. 3cd406> <out.json> [name ...]   (default: the 38 MIPLIB models)
// Hooks: functions reachable from the entry in re/decomp/callgraph.tsv, except modules common/01_model_api/excluded/extern.
// Gurobi: one process at a time (wrap in tools/gurun.sh).
const FRIDA = process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida';
const frida = require(FRIDA);
const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');

const [entry, outPath, ...names] = process.argv.slice(2);
const root = path.resolve(__dirname, '..', '..');
const tsv = p => fs.readFileSync(p, 'utf8').trim().split('\n').slice(1).map(l => l.split('\t'));
const shortOf = n => { const m = /^PRIVATE0*([0-9a-f]+)$/.exec(n); return m ? m[1].slice(-6).replace(/^0+/, '') : n; };
const fns = tsv(path.join(root, 're/decomp/index.tsv')).map(r => [parseInt(r[0], 16), shortOf(r[1]), +r[2]]).sort((a, b) => a[0] - b[0]);
const off = new Map(fns.map(f => [f[1], f])), mod = new Map(tsv(path.join(__dirname, '../docs/port_map.tsv')).map(r => [r[1], r[4]]));
const cal = new Map();
for (const [a, , b] of tsv(path.join(root, 're/decomp/callgraph.tsv'))) {
  const x = parseInt(a, 16), y = parseInt(b, 16);
  if (x !== y) { if (!cal.has(x)) cal.set(x, new Set()); cal.get(x).add(y); }
}
const e = off.get(entry), seen = new Set([e[0]]), q = [e[0]];
while (q.length) for (const v of cal.get(q.pop()) || []) if (!seen.has(v)) { seen.add(v); q.push(v); }
const byOff = new Map(fns.map(f => [f[0], f]));
const skip = new Set(['common', '01_model_api', 'excluded', 'extern', undefined]);
const hooks = [...seen].map(o => byOff.get(o)).filter(f => f && f[2] >= 6 && !skip.has(mod.get(f[1]))).map(f => [f[0], f[1], mod.get(f[1])]);
const list = JSON.parse(execFileSync(path.join(root, '.venv/bin/python'), ['-I', '-c',
  `import sys, json; sys.path.insert(0, ${JSON.stringify(__dirname)}); from parity import instances; i = instances(); ` +
  `print(json.dumps([[n, str(p)] for n, p in i.items() if (n in ${JSON.stringify(names)}) or (not ${names.length} and p.parent.name == 'miplib')]))`]));
const pyCode = `
import ctypes, json, gurobipy as gp
libc = ctypes.CDLL(None)
for i, (name, path) in enumerate(json.loads(${JSON.stringify(JSON.stringify(list))})):
    env = gp.Env(params={"LogToConsole": 0, "Threads": 1, "WorkLimit": 60})
    m = gp.read(path, env)
    libc.qsort(ctypes.c_void_p(0x5eed000000 + i), 0, 0, None)
    q = m.presolve()
    libc.qsort(ctypes.c_void_p(0x5eee000000 + i), 0, 0, None)
    q.dispose(); m.dispose(); env.dispose()
print("DONE", flush=True)
`;
const agent = `
const H = ${JSON.stringify(hooks)}, F = ${JSON.stringify(fns.map(f => [f[0], f[1]]))}, E = [${e[0]}, ${e[0] + e[2]}];
let base = null, cur = -1, cnt = {}, bt = {}, seq = [];
function owner(a) { let lo = 0, hi = F.length - 1; while (lo < hi) { const m = (lo + hi + 1) >> 1; if (F[m][0] <= a) lo = m; else hi = m - 1; } return F[lo][1]; }
function install() {
  for (const [o, s, m] of H) {
    try {
      Interceptor.attach(base.add(o), { onEnter() {
        if (cur < 0) return;
        const c = cnt[s] = (cnt[s] || 0) + 1;
        const r = this.returnAddress.sub(base).toInt32();
        if (r >= E[0] && r < E[1] && seq.length < 20000) seq.push([s, r.toString(16)]);
        if (c === 1 && m !== '03_presolve')
          bt[s] = Thread.backtrace(this.context, Backtracer.FUZZY).map(p => p.sub(base)).filter(x => x.compare(ptr(0x2000000)) < 0).slice(0, 12).map(x => owner(x.toInt32()));
      } });
    } catch (err) { }
  }
}
Process.attachModuleObserver({ onAdded(m) { if (m.name === 'libgurobi130.so') { base = m.base; install(); } } });
Interceptor.attach(Process.getModuleByName('libc.so.6').getExportByName('qsort'), { onEnter(args) {
  const v = args[0].toString(16);
  if (v.startsWith('5eed')) { cur = parseInt(v.slice(4), 16); cnt = {}; bt = {}; seq = []; }
  else if (v.startsWith('5eee')) { send({ i: cur, cnt, bt, seq }); cur = -1; }
} });
`;
(async () => {
  const device = await frida.getLocalDevice();
  const res = {}; let text = '';
  const pid = await device.spawn([path.join(root, '.venv/bin/python'), '-c', pyCode], { stdio: 'pipe' });
  const session = await device.attach(pid);
  const script = await session.createScript(agent);
  const finish = async code => { fs.writeFileSync(outPath, JSON.stringify(res)); await device.kill(pid).catch(() => {}); process.exit(code); };
  script.message.connect(msg => {
    if (msg.type === 'error') { console.error(msg.stack); return; }
    const p = msg.payload;
    res[list[p.i][0]] = { cnt: p.cnt, bt: p.bt, seq: p.seq };
    if (Object.keys(res).length === list.length && text.includes('DONE')) finish(0);
  });
  device.output.connect((p, fd, data) => {
    if (p !== pid) return;
    text += data.toString();
    if (fd === 2) process.stderr.write(data);
    if (text.includes('DONE') && Object.keys(res).length === list.length) finish(0);
  });
  session.detached.connect(reason => { console.error('detached', reason, text); finish(1); });
  console.error(`${hooks.length} hooks, ${list.length} models`);
  await script.load();
  await device.resume(pid);
})();
