// Call-sequence oracle: run Gurobi on one Netlib model (Presolve=0, Threads=1) under Frida and log entries/returns
// of chosen libgurobi functions, with spx fields read at each event. SIMPLEX callbacks appear as "CB <itcnt>" (marker: qsort with zero elements).
// usage: node tools/fn_trace.js <spec.json> <out.jsonl>
// spec: {"model": "afiro", "params": {"Method": 1}, "max": 200000,
//        "fns": [{"name": "chuzr", "off": "0xccbc0", "enter": ["i:0x2f8"], "leave": ["i:0x2f8", "Pi:0x40"]}]}
// field codes, relative to args[0] (spx) saved at entry: i:/d:/l: int/double/long at spx+off;
// Pi:/Pd: at P+off with P = *(spx+0x478); Li:/Ld:/Ll: at L+off with L = *(spx+0x470) (LU); dv:/Pdv:off:k = k-th double of the array *(base+off), iw:/Piw: int; ai:N / al:N raw integer argument N; SV:off sparse vector {count, idx, val} at *(spx+off); AX/AXI:off array gathered at the alpha indices; XR leaving row; OUT:i:j output list; aD:i:N / aI:i:N arrays at args[i] (N = n0: args[0]); ret = return value (rax).
// Gurobi may run in one process only (gurobi_guard.pth): retry later on "already running".
const FRIDA = process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida';
const frida = require(FRIDA);
const fs = require('fs');
const path = require('path');

const [specPath, outPath] = process.argv.slice(2);
const spec = JSON.parse(fs.readFileSync(specPath, 'utf8'));
const root = path.resolve(__dirname, '..', '..');
const py = path.join(root, '.venv/bin/python');
const mps = path.join(root, 're/bench/data/netlib', spec.model + '.mps.gz');
const params = Object.assign({ OutputFlag: 0, Threads: 1, Presolve: 0, Method: 1 }, spec.params || {});
const pyCode = `
import ctypes, sys, gurobipy as gp
libc = ctypes.CDLL(None)
env = gp.Env(params=${JSON.stringify(params)})
m = gp.read(${JSON.stringify(mps)}, env)
def cb(mdl, where):
    if where == gp.GRB.Callback.SIMPLEX:
        libc.qsort(ctypes.c_void_p(0x5eed000000 + int(mdl.cbGet(gp.GRB.Callback.SPX_ITRCNT))), 0, 0, None)
m.optimize(cb)
print("DONE", m.Status, m.IterCount, flush=True)
`;

const agent = `
let base = null;
const spec = ${JSON.stringify(spec.fns)};
const max = ${spec.max || 200000};
let n = 0, buf = [];
function flush() { if (buf.length) { send(buf); buf = []; } }
function rd(spx, code, args, ret) {
  const [k, v] = code.split(':');
  if (k === 'ret') return ret.toString();
  if (k === 'ai') return args[+v].toInt32();
  if (k === 'aD' || k === 'aI') {  // aD:i:N / aI:i:N: N doubles/ints at args[i]; N = 'n0' takes args[0]
    const c = code.split(':')[2], cnt = c === 'n0' ? args[0].toInt32() : +c, out = [];
    for (let t = 0; t < cnt; t++) out.push(k === 'aD' ? args[+v].add(8 * t).readDouble() : args[+v].add(4 * t).readS32());
    return out;
  }
  if (k === 'al') return args[+v].toString();
  let p = spx;
  if (k[0] === 'P') p = spx.add(0x478).readPointer();
  if (k[0] === 'L') p = spx.add(0x470).readPointer();
  let a = p.add(parseInt(v));
  if (k === 'AX' || k === 'AXI') {  // AX:off values of array *(spx+off) gathered at the alpha indices (spx+0x208)
    const sv = spx.add(0x208).readPointer(), cnt = sv.readS32(), ii = sv.add(8).readPointer();
    const b = spx.add(parseInt(v)).readPointer(), out = [];
    for (let t = 0; t < Math.max(cnt, 0); t++) { const j = ii.add(4 * t).readS32(); out.push(k === 'AX' ? b.add(8 * j).readDouble() : b.add(4 * j).readS32()); }
    return out;
  }
  if (k === 'XR') {  // leaving row: [r, var, x_B[r], l, u]
    const r = spx.add(0x2f8).readS32(), j = spx.add(0x470).readPointer().add(0x18).readPointer().add(4 * r).readS32();
    return [r, j, spx.add(0x1e8).readPointer().add(8 * r).readDouble(), spx.add(0xa8).readPointer().add(8 * j).readDouble(), spx.add(0xb0).readPointer().add(8 * j).readDouble()];
  }
  if (k === 'OUT') {  // OUT:i:j count *(args[i]) and that many ints of args[j]
    const [a, b] = code.split(':').slice(1).map(Number), cnt = args[a].readS32(), out = [];
    for (let t = 0; t < Math.max(cnt, 0); t++) out.push(args[b].add(4 * t).readS32());
    return [cnt, out];
  }
  if (k === 'SV') {  // SV:off sparse vector struct at *(spx+off): [count, indices, values]
    const sv = spx.add(parseInt(v)).readPointer(), cnt = sv.readS32(), ii = sv.add(8).readPointer(), vv = sv.add(16).readPointer();
    const I = [], V = [];
    for (let t = 0; t < Math.max(cnt, 0); t++) { I.push(ii.add(4 * t).readS32()); V.push(vv.add(8 * t).readDouble()); }
    return [cnt, I, V];
  }
  if (k === 'A' || k === 'PA' || k === 'AI' || k === 'PAI') {  // A:/PA:off:N first N doubles (AI/PAI: ints) of *(base+off)
    const b = a.readPointer(), cnt = +code.split(':')[2], out = [];
    for (let t = 0; t < cnt; t++) out.push(k.endsWith('I') ? b.add(4 * t).readS32() : b.add(8 * t).readDouble());
    return out;
  }
  if (k.endsWith('v')) { a = a.readPointer().add(8 * parseInt(code.split(':')[2])); return a.readDouble(); }
  if (k.endsWith('w')) { a = a.readPointer().add(4 * parseInt(code.split(':')[2])); return a.readS32(); }
  const t = k[k.length - 1];
  return t === 'i' ? a.readS32() : t === 'd' ? a.readDouble() : a.readS64().toString();
}
function install() {
for (const f of spec) {
  Interceptor.attach(base.add(parseInt(f.off)), {
    onEnter(args) {
      this.spx = args[0]; this.args = [args[0], args[1], args[2], args[3], args[4], args[5]];
      if (n++ < max) buf.push([f.name, '>'].concat((f.enter || []).map(c => rd(this.spx, c, this.args, null))));
      if (buf.length > 2000) flush();
    },
    onLeave(ret) {
      if (f.leave && n++ < max) buf.push([f.name, '<'].concat(f.leave.map(c => rd(this.spx, c, this.args, ret))));
    }
  });
}
Interceptor.attach(Process.getModuleByName('libc.so.6').getExportByName('qsort'), {
  onEnter(args) { const v = args[0].toString(16); if (v.startsWith('5eed')) { buf.push(['CB', parseInt(v.slice(4), 16)]); } }
});
}
Process.attachModuleObserver({ onAdded(m) { if (m.name === 'libgurobi130.so') { base = m.base; install(); } } });
recv('flush', () => { flush(); send('flushed'); });
`;

(async () => {
  const device = await frida.getLocalDevice();
  const out = fs.createWriteStream(outPath);
  let text = '', flushed = null;
  const pid = await device.spawn([py, '-c', pyCode], { stdio: 'pipe' });
  const session = await device.attach(pid);
  const script = await session.createScript(agent);
  script.message.connect(msg => {
    if (msg.type === 'error') { console.error(msg.stack); return; }
    if (msg.payload === 'flushed') { flushed(); return; }
    for (const r of msg.payload) out.write(JSON.stringify(r) + '\n');
  });
  device.output.connect(async (p, fd, data) => {
    if (p !== pid) return;
    text += data.toString();
    if (fd === 2) process.stderr.write(data);
    if (text.includes('DONE') && !flushed) {
      await new Promise(res => { flushed = res; script.post({ type: 'flush' }); });
      console.log(text.trim().split('\n').pop());
      out.end();
      await device.kill(pid).catch(() => {});
      process.exit(0);
    }
  });
  session.detached.connect(reason => { if (!flushed) { console.error('detached', reason, text); process.exit(1); } });
  await script.load();
  await device.resume(pid);
})();
