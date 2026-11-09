// MIP root LP oracle (docs/m2_root.md): run Gurobi's MIP solve (Threads=1, defaults) on one model under Frida until the
// first MIPNODE callback and log every LP solve 5ccea5 -> {cda5b simplex | c3d0d LP driver | 98b671 | 14abfb}: the
// method argument, warm-start basis pointers, and at each cda5b/c3d0d entry the LP model written with GRBwrite
// (<outdir>/<k>.mps) and its environment's non-default parameters (<k>.prm), plus IterCount/ObjVal/Status at return.
// usage: node tools/mip_root_trace.js <model name | .mps path> <outdir> [Param=Value ...]
// SPX_K=<k>: also dump the simplex workspace (lp+0xe0) when setup 2f7e1 returns inside the k-th cda5b/c3d0d call
// (<outdir>/spx_<k>.json: first 0x600 bytes as u32 words, scales, bounds, costs, vstat). LP=<x.prm>: instead of the
// MIP, solve the .mps as an LP with those parameters (then Presolve=0, Method=1, Threads=1): the standalone reference.
// Gurobi: one process at a time (wrap in tools/gurun.sh).
// SPX_TRACE=1 with LP=<x.prm>: save callback workspace snapshots to spx_trace.jsonl (last callback per iteration).
const FRIDA = process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida';
const frida = require(FRIDA);
const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');

const [model, outDir, ...kv] = process.argv.slice(2);
const root = path.resolve(__dirname, '..', '..');
const shortOf = n => { const m = /^PRIVATE0*([0-9a-f]+)$/.exec(n); return m ? m[1].slice(-6).replace(/^0+/, '') : n; };
const idx = fs.readFileSync(path.join(root, 're/decomp/index.tsv'), 'utf8').trim().split('\n').slice(1).map(l => l.split('\t'));
const off = s => { const r = idx.find(r => shortOf(r[1]) === s); if (!r) throw new Error('no fn ' + s); return parseInt(r[0], 16); };
const mps = model.endsWith('.mps') || model.endsWith('.gz') ? model : execFileSync(path.join(root, '.venv/bin/python'), ['-I', '-c',
  `import sys; sys.path.insert(0, ${JSON.stringify(__dirname)}); from parity import instances; print(instances()[${JSON.stringify(model)}])`]).toString().trim();
fs.mkdirSync(outDir, { recursive: true });
const out = path.resolve(outDir);
const params = { LogToConsole: 0, Threads: 1, WorkLimit: 60, LogFile: path.join(out, 'log.txt') };
for (const p of kv) { const [k, v] = p.split('='); params[k] = Number(v); }
const pyCode = `
import gurobipy as gp
import sys, json
sys.path.insert(0, ${JSON.stringify(__dirname)})
from spx_oracle import api, L, dump
env = gp.Env(params=${JSON.stringify(params)})
m = gp.read(${JSON.stringify(mps)}, env)
lp = ${JSON.stringify(process.env.LP || '')}
if lp:
    m.read(lp); m.Params.Presolve = 0; m.Params.Method = 1; m.Params.Threads = 1; m.Params.LogFile = ${JSON.stringify(path.join(out, 'log.txt'))}
snaps = {}
trace = ${process.env.SPX_TRACE === '1' ? 'True' : 'False'} and bool(lp)
ptr = api.PyCapsule_GetPointer(m._cmodel, api.PyCapsule_GetName(m._cmodel))
def cb(mdl, where):
    if where == gp.GRB.Callback.MIPNODE: mdl.terminate()
    if trace and where == gp.GRB.Callback.SIMPLEX and L(ptr + 0xe0):
        it = int(mdl.cbGet(gp.GRB.Callback.SPX_ITRCNT))
        snaps[it] = dump(L(ptr + 0xe0), it, full=it == 0)
m.optimize(cb if trace or not lp else None)
if trace:
    with open(${JSON.stringify(path.join(out, 'spx_trace.jsonl'))}, 'w') as f:
        for it in sorted(snaps): f.write(json.dumps(snaps[it]) + '\\n')

print("DONE", m.Status, m.IterCount, flush=True)
`;
const fns = { '5ce67b': off('5ce67b'), '5ccea5': off('5ccea5'), 'cda5b': off('cda5b'), 'c3d0d': off('c3d0d'),
  '98b671': off('98b671'), '14abfb': off('14abfb'), '69f0bd': off('69f0bd'), '2f7e1': off('2f7e1') };
const spxK = process.env.SPX_K === undefined ? -1 : Number(process.env.SPX_K);

const agent = `
const fns = ${JSON.stringify(fns)}, out = ${JSON.stringify(out)}, spxK = ${spxK}, patchVt = ${process.env.PATCH_VT ? JSON.stringify(Array.from(fs.readFileSync(process.env.PATCH_VT))) : 'null'};
let k = 0, depth = 0, G = null, cur = -1, nload = 0;
function dbls(p, c) { const r = []; if (p.isNull()) return null; for (let i = 0; i < c; i++) r.push(p.add(8 * i).readDouble()); return r; }
function dumpSpx(spx, kk) {
  const m = spx.add(0x64).readS32(), n = spx.add(0x68).readS32(), w = [];
  for (let i = 0; i < 0x600; i += 4) w.push(spx.add(i).readU32());
  const P = o => spx.add(o).readPointer();
  const d = { m, n, words: w, colscale: dbls(P(0xe0), n), rowscale: dbls(P(0xe8), m), c: dbls(P(0x98), n + m),
    l: dbls(P(0xa8), n + m), u: dbls(P(0xb0), n + m), b: dbls(P(0xa0), m), vstat: P(0x168).isNull() ? null : ints(P(0x168), n + m),
    vtype: P(0xb8).isNull() ? null : Array.from(new Uint8Array(P(0xb8).readByteArray(n + m))) };
  const f = new File(out + '/spx_' + kk + '.json', 'w'); f.write(JSON.stringify(d)); f.close();
}
function api(m) {
  const f = (n, r, a) => new NativeFunction(m.getExportByName(n), r, a);
  return { write: f('GRBwrite', 'int', ['pointer', 'pointer']), getenv: f('GRBgetenv', 'pointer', ['pointer']),
    wparams: f('GRBwriteparams', 'int', ['pointer', 'pointer']), gi: f('GRBgetintattr', 'int', ['pointer', 'pointer', 'pointer']),
    gd: f('GRBgetdblattr', 'int', ['pointer', 'pointer', 'pointer']) };
}
function gi(lp, a) { const p = Memory.alloc(8); return G.gi(lp, Memory.allocUtf8String(a), p) === 0 ? p.readS32() : null; }
function gd(lp, a) { const p = Memory.alloc(8); return G.gd(lp, Memory.allocUtf8String(a), p) === 0 ? p.readDouble() : null; }
function ints(p, c) { const r = []; for (let i = 0; i < c; i++) r.push(p.add(4 * i).readS32()); return r; }
function install(base) {
  for (const [name, o] of Object.entries(fns)) {
    Interceptor.attach(base.add(o), {
      onEnter(args) {
        this.d = depth++; this.lp = args[0];
        const ev = { fn: name, ev: '>', depth: this.d, ra: '0x' + this.returnAddress.sub(base).toString(16) };
        if (name === '2f7e1') { this.cur = cur; return; }
        if (name === '69f0bd') this.out = args[2];
        if (name === 'cda5b' || name === 'c3d0d') {
          this.k = k++; cur = this.k;
          const lp = args[0], n = gi(lp, 'NumVars'), m = gi(lp, 'NumConstrs');
          Object.assign(ev, { k: this.k, n, m, env: { presolve: lp.add(0x100).readPointer().add(0x2424).readS32(),
            method: lp.add(0x100).readPointer().add(0x2438).readS32() } });
          if (name === 'cda5b') {
            ev.method = args[1].toInt32();
            ev.vb = !args[2].isNull(); ev.cb = !args[3].isNull();
            if (ev.vb) ev.vbasis = ints(args[2], n);
            if (ev.cb) ev.cbasis = ints(args[3], m);
            // existing simplex workspace (lp+0xe0): 5ccea5 then warm-starts cda5b from its basis (spx+0x168 vstat)
            const ws = lp.add(0xe0).readPointer();
            ev.ws = !ws.isNull();
            if (ev.ws) { ev.ws3f0 = ws.add(0x3f0).readS32(); ev.vstat = ints(ws.add(0x168).readPointer(), ws.add(0x68).readS32() + ws.add(0x64).readS32()); }
          }
          ev.write = G.write(lp, Memory.allocUtf8String(out + '/' + this.k + '.mps'));
          G.wparams(G.getenv(lp), Memory.allocUtf8String(out + '/' + this.k + '.prm'));
        }
        send(ev);
      },
      onLeave(ret) {
        depth--;
        if (name === '2f7e1') {
          const spx = this.lp.add(0xe0).readPointer();
          if (this.cur === spxK && !spx.isNull()) {
            // PATCH_VT (LP mode): write these column types into spx+0xb8 first, to test the integer-weight effect
            if (patchVt) spx.add(0xb8).readPointer().writeByteArray(patchVt);
            dumpSpx(spx, this.cur);
          }
          return;
        }
        const ev = { fn: name, ev: '<', depth: this.d, ret: ret.toInt32() };
        // 69f0bd(M, ., &holder, ., .): holder[1] is the LP model it built (5ca999); write it as load_<i>.mps
        if (name === '69f0bd' && nload < 4) { const h = this.out.readPointer(); if (!h.isNull()) { ev.load = nload; G.write(h.add(8).readPointer(), Memory.allocUtf8String(out + '/load_' + nload++ + '.mps')); } }
        if (name === 'cda5b' || name === 'c3d0d') Object.assign(ev, { k: this.k, iter: gd(this.lp, 'IterCount'),
          obj: gd(this.lp, 'ObjVal'), status: gi(this.lp, 'Status') });
        send(ev);
      }
    });
  }
}
Process.attachModuleObserver({ onAdded(m) { if (m.name === 'libgurobi130.so') { G = api(m); install(m.base); } } });
`;

(async () => {
  const device = await frida.getLocalDevice();
  const ev = fs.createWriteStream(path.join(out, 'events.jsonl'));
  let text = '';
  const pid = await device.spawn([path.join(root, '.venv/bin/python'), '-c', pyCode], { stdio: 'pipe' });
  const session = await device.attach(pid);
  const script = await session.createScript(agent);
  script.message.connect(msg => {
    if (msg.type === 'error') { console.error(msg.stack); return; }
    ev.write(JSON.stringify(msg.payload) + '\n');
  });
  device.output.connect(async (p, fd, data) => {
    if (p !== pid) return;
    text += data.toString();
    if (fd === 2) process.stderr.write(data);
    if (text.includes('DONE')) {
      console.log(text.trim().split('\n').pop());
      ev.end();
      await device.kill(pid).catch(() => {});
      process.exit(0);
    }
  });
  session.detached.connect(reason => { console.error('detached', reason, text); process.exit(1); });
  await script.load();
  await device.resume(pid);
})();
