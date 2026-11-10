const fs = require('fs'), path = require('path'), crypto = require('crypto');
const frida = require('/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [kind, model, output] = process.argv.slice(2);
if (!['pdhg', 'barrier', 'warm', 'control'].includes(kind) || !model || !output) throw Error('사용법: KIND MODEL OUTPUT');
const root = path.resolve(__dirname, '../..'), py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const bh = hash(binary), expected = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c';
if (bh !== expected) throw Error('원본 바이너리 해시 불일치');
const params = {Threads: 1, Seed: 0, OutputFlag: 0, Presolve: 0, Method: kind === 'pdhg' ? 6 : kind === 'warm' ? 1 : 2, Crossover: kind === 'control' ? 0 : 1, LPWarmStart: 2};
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))},e)\n${kind === 'warm' ? "m.setAttr('PStart',m.getVars(),[2.,1.,1.]); m.setAttr('DStart',m.getConstrs(),[1.,0.,0.]); m.update()" : ''}\nm.optimize()\nimport json\nprint('CROSSOVER_RESULT '+json.dumps({'status':m.Status}),flush=True)`;
const agent = `
Process.attachModuleObserver({onAdded(mod) { if (mod.name !== 'libgurobi130.so') return;
 const b=mod.base; let seq=0; const ids=new Map();
 function id(p) {if(p.isNull()) return null; const s=p.toString(); if(!ids.has(s)) ids.set(s,ids.size+1); return ids.get(s);}
 function arr(q,k,step) { if(q.isNull()) return null; let v=[]; for(let i=0;i<k;i++) v.push(q.add(i*step).readByteArray(step)); return v.map(x=>Array.from(new Uint8Array(x)).map(y=>y.toString(16).padStart(2,'0')).join('')); }
 function snap(p) {const d=p.add(0xe8).readPointer(), env=p.add(0x100).readPointer(), w=p.add(0xe0).readPointer();
  const n=d.add(12).readS32(), m=d.add(8).readS32(); if(n<0||m<0||n+m>100000) throw Error('차원 오류');
  const r=p.add(0x1b0).readPointer(), s=p.add(0x1a8).readPointer(), start=p.add(0x238).readPointer();
  function vec(q,offset,k){return q.isNull()?null:arr(q.add(offset).readPointer(),k,8)}
  return {model:id(p), n,m,status:p.add(100).readS32(), env:{field_2478:arr(env.add(0x2478),1,8),field_24a0:arr(env.add(0x24a0),1,8),field_26fc:env.add(0x26fc).readS32()},
   result:r.isNull()?null:{id:id(r),x:vec(r,8,n),slack:vec(r,16,m),y:vec(r,24,m),reduced:vec(r,32,n)},
   copy:s.isNull()?null:{id:id(s),primal:vec(s,0x60,n+m),dual:vec(s,0x68,n+m)},
   start:start.isNull()?null:{id:id(start),count:start.add(4).readS32(),status:start.add(0x14).readS32(),values:vec(start,0x20,n+m)},
   scale:{c:vec(d,0x3e0,n),r:vec(d,0x3d8,m),o:arr(d.add(1000),1,8),flip:arr(d.add(0x408).readPointer(),n,1),sense:arr(d.add(0x388).readPointer(),m,1),flip_enabled:d.add(0x3c).readS32()},
   workspace:w.isNull()?null:(()=>{const rows=w.add(0x64).readS32(),cols=w.add(0x68).readS32(); if(rows<0||cols<0||rows+cols>100000) throw Error('작업 차원 오류'); return {id:id(w),rows,cols,vstat:arr(w.add(0x168).readPointer(),rows+cols,4)};})()};
 }
 Interceptor.attach(b.add(0x1bd281), {onEnter(){const c=this.context, p=c.rbp, d=p.add(0xe8).readPointer(), n=d.add(12).readS32(), m=d.add(8).readS32(); if(n<0||m<0||n+m>100000) throw Error('준비 배열 차원 오류'); send({event:'prepared',model:id(p),x:arr(c.rbx,n,8),slack:arr(c.rsp.add(0x238).readPointer(),m,8),reduced:arr(c.r12,n,8),y:arr(c.rsp.add(0x240).readPointer(),m,8)});}});
 for (const [name,off] of [['pdhg',0x3639a0],['helper',0x2dc620],['barrier',0x2dd9b0],['warm',0x14acb0],['dispatch',0x1bc570],['body',0x1bc5d0]]) Interceptor.attach(b.add(off),{
  onEnter(a){this.p=a[0]; this.call=++seq; send({event:name+':enter',call:this.call,mode:name==='body'?a[1].toInt32():null,state:snap(this.p)});},
  onLeave(r){send({event:name+':leave',call:this.call,ret:r.toInt32(),state:snap(this.p)});}
 });
}});`;
(async()=>{const device=await frida.getLocalDevice(), events=[], errors=[]; let out='';
 const pid=await device.spawn([py,'-c',code],{stdio:'pipe'}), session=await device.attach(pid), script=await session.createScript(agent);
 script.message.connect(msg=>{if(msg.type==='send') events.push(msg.payload); else errors.push(msg.stack||JSON.stringify(msg));});
 device.output.connect((p,fd,data)=>{if(p===pid) out+=data.toString()});
 session.detached.connect(reason=>{const markers=out.split('\n').filter(x=>x.startsWith('CROSSOVER_RESULT ')); let result=null; try { if(markers.length===1) result=JSON.parse(markers[0].slice(17)); } catch(e) { errors.push(String(e)); } const ok=!!result && Number.isInteger(result.status) && !errors.length && reason==='process-terminated'; fs.writeFileSync(ok?output:output+'.failed',JSON.stringify({origin:'natural',kind,model_sha256:hash(model),binary_sha256:bh,tool_sha256:hash(__filename),params,output:out,result,reason,errors,events},null,2)+'\n'); console.log(kind,'사건',events.length,'성공',ok,errors.join(' '),out.trim()); process.exit(ok?0:1)});
 await script.load(); await device.resume(pid);
})().catch(e=>{console.error(e);process.exit(1)});
