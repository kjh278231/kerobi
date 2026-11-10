// Read-only Gurobi 13.0.3 trace. One invocation spawns exactly one original process.
// Usage: tools/gurun.sh node tools/mip_linear_child_trace.js MODEL OUTPUT
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [model, output] = process.argv.slice(2);
if (!model || !output) throw Error('MODEL and OUTPUT required');
const root = path.resolve(__dirname, '../..');
const py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const expected = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c';
if (hash(binary) !== expected) throw Error('wrong Gurobi binary');
const params = {Threads:1, Presolve:0, Cuts:0, Heuristics:0, Seed:0, NodeLimit:100, LogToConsole:0};
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\nm.optimize()\nprint('DONE',m.Status,int(m.NodeCount),flush=True)`;
const agent = `
function words(p,n) {
  if(p.isNull()) return null;
  const out=[]; for(let j=0;j<n;j++) out.push(p.add(8*j).readU64().toString(16).padStart(16,'0')); return out;
}
function node(n) {
  if (n.isNull()) return null;
  const i=n.add(0x18).readPointer(), m=i.readPointer(), p=n.add(0x10).readPointer();
  const root=m.add(0x790).readPointer(), model=m.add(8).readPointer(), data=model.add(0xe8).readPointer();
  const count=data.add(0xc).readS32(), top=m.add(0x620).readPointer();
  if(count<0 || count>4096) throw Error('unsupported domain length '+count);
  return {id:n.readDouble(), parent:p.isNull()?null:p.readDouble(), depth:i.add(0x2c).readS32(),
    root:n.equals(root), parent_is_root:!p.isNull() && p.equals(root), status:i.add(0x28).readS32(), cache:m.add(0xa8).readS32(), mip:m.readS32(),
    counter_d4:top.add(0xd4).readS32(), domain_lower:words(i.add(0x80).readPointer(),count), domain_upper:words(i.add(0x88).readPointer(),count),
    basis_98_present:!i.add(0x98).readPointer().isNull(), basis_a8_present:!i.add(0xa8).readPointer().isNull(), basis_b0_present:!i.add(0xb0).readPointer().isNull()};
}
function lpstate(lp) {
  if(lp.isNull()) return null;
  const env=lp.add(0x100).readPointer(), ws=lp.add(0xe0).readPointer();
  let basis=null; if(!ws.isNull()) { const rows=ws.add(0x64).readS32(), cols=ws.add(0x68).readS32(), v=ws.add(0x168).readPointer(); if(rows<0 || cols<0 || rows+cols>8192) throw Error('unsupported basis length'); if(!v.isNull()) {basis=[]; for(let j=0;j<rows+cols;j++) basis.push(v.add(4*j).readS32());} }
  return {status:lp.add(0x64).readS32(), presolve:env.add(0x2424).readS32(), method:env.add(0x2438).readS32(),
    concurrent_method:env.add(0x233c).readS32(), threads:env.add(0x2708).readS32(), workspace_present:!ws.isNull(), workspace_vstat:basis};
}
function emit(e) { send(e); }
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return;
 const b=m.base; let active=0, nextCall=0, currentLp=ptr(0);
 for (const [name,offset] of Object.entries({solve:0x4f0e10, branch:0x5320c0, qualify:0x519b10, gate:0x517690, finish:0x518170})) {
  Interceptor.attach(b.add(offset), {onEnter(a) { this.call=++nextCall; if(name==='solve') {active++; currentLp=a[2].add(8).readPointer();} this.h=a[0]; this.n=(name==='solve'||name==='branch'||name==='gate')?a[0].readPointer():a[0]; emit({event:name+':enter',call:this.call,node:node(this.n),lp:name==='solve'?lpstate(currentLp):null}); },
   onLeave(r) { const holder=(name==='solve'||name==='branch'||name==='gate'); const after=holder?this.h.readPointer():this.n; emit({event:name+':leave',call:this.call,ret:r.toInt32(),holder_changed:holder?!after.equals(this.n):null,node:name==='gate'?null:node(after),lp:name==='solve'?lpstate(currentLp):null}); if(name==='solve') {active--; currentLp=ptr(0);} }});
 }
 for (const [name,offset] of Object.entries({lpdriver:0x14acb0,simplex:0x153cc0,reopt:0x1cb7e0})) {
  Interceptor.attach(b.add(offset), {onEnter(a) {this.enabled=active>0; this.lp=a[0]; this.call=++nextCall; if(this.enabled) emit({event:name+':enter',call:this.call,lp:lpstate(this.lp)});},
   onLeave(r) {if(this.enabled) emit({event:name+':leave',call:this.call,ret:r.toInt32(),lp:lpstate(this.lp)});}});
 }
 Interceptor.attach(b.add(0x5177f0), {onEnter(a) { this.call=++nextCall; this.parent=a[0]; this.out=a[2]; emit({event:'create:enter',call:this.call,node:node(this.parent)}); },
   onLeave(r) { emit({event:'create:leave',call:this.call,ret:r.toInt32(),node:r.toInt32()===0?node(this.out.readPointer()):null}); }});
 Interceptor.attach(b.add(0x5180f0), {onEnter(a) { this.call=++nextCall; this.n=a[0]; this.value=a[1].toInt32(); emit({event:'set:enter',call:this.call,node:node(this.n),value:this.value}); },
   onLeave() {emit({event:'set:leave',call:this.call,node:node(this.n),value:this.value}); }});
 for (const [name,offset] of Object.entries({normalize_before:0x4f1a98,normalize_after:0x4f1aca})) {
  Interceptor.attach(b.add(offset), {onEnter() { const c=this.context; emit({event:name,node:node(c.rsp.add(0x3b0).readPointer()),lp_status:c.r13.add(0x38).readS32(),lp:lpstate(currentLp)}); }});
 }
}});
`;
(async () => {
 const device=await frida.getLocalDevice(); const events=[]; let outputText='', errors=[];
 const pid=await device.spawn([py,'-c',code],{stdio:'pipe'}); const session=await device.attach(pid);
 const script=await session.createScript(agent);
 script.message.connect(msg=>{if(msg.type==='send') events.push(msg.payload); else errors.push(msg.stack||JSON.stringify(msg));});
 device.output.connect((p,fd,data)=>{if(p===pid) outputText+=data.toString();});
 session.detached.connect(()=>{if(!outputText.includes('DONE')||errors.length){console.error(outputText,errors);process.exit(1);}
   fs.writeFileSync(output,JSON.stringify({schema:1,model_sha256:hash(model),binary_sha256:expected,params,output:outputText.trim(),events},null,2)+'\n');
   console.log(outputText.trim(), 'events',events.length); process.exit(0);});
 await script.load(); await device.resume(pid);
})().catch(e=>{console.error(e);process.exit(1);});
