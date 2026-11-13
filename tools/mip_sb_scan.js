// M2-06h-7: light exploratory scan for 695bd7 branches that full captures never hit (propagation
// contradiction, continuous candidate). Only the kernel entry and 6953ff are hooked, so deep NodeLimit
// runs stay cheap. Exploration only: reference data still comes from mip_sb_loop.js / mip_sb_direct.js.
// Usage: tools/gurun.sh node tools/mip_sb_scan.js MODEL OUTPUT.json [JSON_PARAMS]
const fs = require('fs'), path = require('path'), crypto = require('crypto');
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
  `m.optimize()\nprint('DONE',m.Status,int(m.NodeCount),float(m.IterCount).hex(),float(m.ObjBound).hex(),m.SolCount,float(m.ObjVal).hex() if m.SolCount else '-',flush=True)`;
const agent = `
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return;
 const b=m.base, off=p=>p.sub(b).toUInt32().toString(16); let serial=0, cur=null;
 // 266b6a (0x29d980) and 695bd7 (0x571770): (LP, Node, n, stride, cands, types, ...).
 for(const [addr,tag] of [[0x29d980,'kernel'],[0x571770,'double']])
 Interceptor.attach(b.add(addr),{onEnter(a){const n=a[2].toInt32(),js=Array.from(new Int32Array(a[4].readByteArray(n*4)));
   const node=!a[1].isNull(), prop=node && !a[1].add(0x18).readPointer().isNull();
   const cont=js.filter(j=>a[5].add(j).readU8()===67);
   this.prev=cur; cur={tag,id:serial++,ra:off(this.returnAddress),n,stride:a[3].toInt32(),node,prop,cont};
   send(['k',cur]);},onLeave(){cur=this.prev;}});
 // 6953ff copies: stack +32 is &contradiction (06h-3 ABI for 0x573d30).
 for(const addr of [0x573d30,0x2a31f0])
 Interceptor.attach(b.add(addr),{onEnter(a){this.k=cur;this.out=this.context.rsp.add(32).readPointer();
   this.col=a[4].toInt32();this.dir=a[5].toInt32()&0xff;this.ra=off(this.returnAddress);},
  onLeave(r){const o=this.out.readS32();send(['p',addr.toString(16),this.ra,this.dir,o,o?{k:this.k?this.k.id:null,col:this.col,rc:r.toInt32()}:null]);}});
}});
`;
(async () => {
 const device = await frida.getLocalDevice(); const counts = {}; const kernels = []; const hits = []; let outputText = ''; const errors = [];
 const pid = await device.spawn([py, '-c', code], {stdio: 'pipe'}); const session = await device.attach(pid);
 const script = await session.createScript(agent);
 const inc = k => { counts[k] = (counts[k] || 0) + 1; };
 script.message.connect(msg => { if (msg.type !== 'send') { errors.push(msg.stack || JSON.stringify(msg)); return; }
   const p = msg.payload;
   if (p[0] === 'k') { const k = p[1]; inc(k.tag + ':' + k.ra); if (!k.prop) inc('no propagation'); if (k.stride > 1) inc('stride > 1');
     if (k.cont.length) { inc('continuous candidate'); hits.push(k); } else if (kernels.length < 5) kernels.push(k); }
   else { const [, addr, ra, dir, o, ctx] = p; inc(`prop:${addr}:${ra}:${dir}`); if (o) { inc(`contradiction:${ra}:${dir}`); hits.push(Object.assign({addr, ra, dir, out: o}, ctx)); } } });
 device.output.connect((p, fd, data) => { if (p === pid) outputText += data.toString(); });
 session.detached.connect((reason) => {
   if (outputText.includes('Gurobi is already running')) { console.log(outputText.trim()); process.exit(1); }
   const ok = outputText.includes('DONE') && !errors.length;
   fs.mkdirSync(path.dirname(output), {recursive: true});
   fs.writeFileSync(output, JSON.stringify({schema: 1, kind: 'mip_sb_scan', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok, detach_reason: reason,
     counts, hits: hits.slice(0, 200), sample: kernels}));
   console.log(path.basename(model), outputText.trim(), JSON.stringify(counts), 'ok', ok, errors.join(' ').slice(0, 300));
   process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
