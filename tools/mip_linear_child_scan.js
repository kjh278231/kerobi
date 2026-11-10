// Read-only natural-path scan: counts node-LP statuses at 5ce67b normalization, retry entries and node status setter values.
// One invocation spawns exactly one original process. No state is written into the target.
// Usage: tools/gurun.sh node tools/mip_linear_child_scan.js MODEL OUTPUT 'JSON_PARAMS'
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [model, output, extra] = process.argv.slice(2);
if (!model || !output) throw Error('MODEL and OUTPUT required');
const root = path.resolve(__dirname, '../..');
const py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const expected = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c';
if (hash(binary) !== expected) throw Error('wrong Gurobi binary');
const params = Object.assign({Threads: 1, Seed: 0, LogToConsole: 0}, JSON.parse(extra || '{}'));
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\nm.optimize()\nprint('DONE',m.Status,int(m.NodeCount),flush=True)`;
const agent = `
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return;
 const b = m.base;
 // normalize_before / normalize_after: node N at rsp+0x3b0, LP result status at r13+0x38 (same offsets as mip_linear_child_experiment.js)
 for (const [name, off] of [['before', 0x4f1a98], ['after', 0x4f1aca]]) {
  Interceptor.attach(b.add(off), {onEnter() {
   const c = this.context, n = c.rsp.add(0x3b0).readPointer(), mm = n.add(0x18).readPointer().readPointer();
   send(['n', name, n.equals(mm.add(0x790).readPointer()) ? 1 : 0, mm.add(0xa8).readS32(), c.r13.add(0x38).readS32()]);
  }});
 }
 Interceptor.attach(b.add(0x4f53a8), {onEnter() { send(['retry']); }});
 Interceptor.attach(b.add(0x5180f0), {onEnter(a) { send(['set', a[1].toInt32()]); }});
}});
`;
(async () => {
 const device = await frida.getLocalDevice(); const counts = {}; let outputText = ''; const errors = [];
 const pid = await device.spawn([py, '-c', code], {stdio: 'pipe'}); const session = await device.attach(pid);
 const script = await session.createScript(agent);
 script.message.connect(msg => { if (msg.type !== 'send') { errors.push(msg.stack || JSON.stringify(msg)); return; }
   const k = msg.payload.join(':'); counts[k] = (counts[k] || 0) + 1; });
 device.output.connect((p, fd, data) => { if (p === pid) outputText += data.toString(); });
 session.detached.connect((reason, crash) => { const ok = outputText.includes('DONE') && !errors.length;
   fs.mkdirSync(path.dirname(output), {recursive: true});
   fs.writeFileSync(output, JSON.stringify({schema: 1, kind: 'natural_scan', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, params, output: outputText.trim(), errors, ok, detach_reason: reason, counts}, null, 2) + '\n');
   console.log(path.basename(model), outputText.trim(), JSON.stringify(counts), 'ok', ok, errors.join(' ')); process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
