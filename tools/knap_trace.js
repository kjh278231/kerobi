// `6b5e11` (exact binary knapsack, docs/m2_red_outer_a.md §5.3) inputs and outputs per call, for the port's unit test.
// Records k, ind, val, rhs, W, ac, b0 at entry and return (doubles as hex bits). Gurobi: one process (tools/gurun.sh).
// usage: node tools/knap_trace.js <model> <out.json>
const FRIDA = process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida';
const frida = require(FRIDA);
const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');

const [model, outPath] = process.argv.slice(2);
const root = path.resolve(__dirname, '..', '..');
const mps = execFileSync(path.join(root, '.venv/bin/python'), ['-I', '-c',
  `import sys; sys.path.insert(0, ${JSON.stringify(__dirname)}); from parity import instances; print(instances()[${JSON.stringify(model)}])`]).toString().trim();
const pyCode = `
import gurobipy as gp
env = gp.Env(params={"LogToConsole": 0, "Threads": 1, "WorkLimit": 60})
m = gp.read(${JSON.stringify(mps)}, env)
q = m.presolve()
print("DONE", flush=True)
`;
const agent = `
let out = [];
function H(p, n) { if (p.isNull() || n <= 0) return []; return Array.from(new BigUint64Array(p.readByteArray(8 * n)), x => x.toString(16)); }
function I32(p, n) { if (p.isNull() || n <= 0) return []; return Array.from(new Int32Array(p.readByteArray(4 * n))); }
function snap(a) {
  const k = a.k.readS32();
  return { k, ind: I32(a.ind, k), val: H(a.val, k), rhs: H(a.rhs, 1)[0], w: a.w.isNull() ? null : H(a.w, 1)[0],
           ac: a.ac.readS32(), b0: a.b0.readS32() };
}
Process.attachModuleObserver({ onAdded(m) {
  if (m.name !== 'libgurobi130.so') return;
  Interceptor.attach(m.base.add(0x5882d0), {
    onEnter(a) { this.a = { k: a[1], ind: a[2], val: a[3], rhs: a[4], ac: a[7], b0: a[8], w: a[9] }; this.i = snap(this.a); },
    onLeave(r) { out.push({ ret: r.toInt32(), i: this.i, o: snap(this.a) }); }
  });
} });
recv('flush', () => { send(out); send('flushed'); });
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
      console.log(model, res.length, 'calls');
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
