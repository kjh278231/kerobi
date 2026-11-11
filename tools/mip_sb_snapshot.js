// M2-06g-2: read-only SB boundary capture. See docs/m2/mip_sb_snapshot.md.
const fs = require('fs'), path = require('path'), crypto = require('crypto'), zlib = require('zlib');
const frida = require(process.env.FRIDA_NODE || '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida');
const [model, output, extra, mode] = process.argv.slice(2);
if (!model || !output) throw Error('usage: MODEL OUTPUT [JSON_PARAMS]');
if (fs.existsSync(output)) throw Error('output exists: ' + output);
const root = path.resolve(__dirname, '../..');
const py = path.join(root, '.venv/bin/python');
const binary = path.join(root, '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const expected = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c';
if (hash(binary) !== expected) throw Error('wrong Gurobi binary');
const funcs = fs.readFileSync(path.join(root, 're/decomp/index.tsv'), 'utf8').trim().split('\n').slice(1).map(l => l.split('\t'))
  .map(r => [parseInt(r[0], 16), parseInt(r[2], 10), r[1].replace(/^PRIVATE0+/, '')]).sort((a, b) => a[0] - b[0]);
const params = Object.assign({Threads: 1, Seed: 0, LogToConsole: 0}, JSON.parse(extra || '{}'));
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\n` +
  `m.optimize()\nprint('DONE',m.Status,int(m.NodeCount),float(m.IterCount).hex(),float(m.ObjBound).hex(),m.SolCount,float(m.ObjVal).hex() if m.SolCount else '-',flush=True)`;
const agent = mode === 'control' ? '' : `
const hx = p => p.readU64().toString(16).padStart(16, '0');
const ints = (p, n) => n ? Array.from(new Int32Array(p.readByteArray(n * 4))) : [];
const doubles = (p, n) => Array.from({length:n}, (_, i) => hx(p.add(8*i)));
const raw = (p, n) => n ? (p.isNull() ? null : Array.from(new Uint8Array(p.readByteArray(n)), v=>v.toString(16).padStart(2,'0')).join('')) : '';
const wk = p => p.isNull() ? null : hx(p);
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return;
 const b=m.base, off=p=>p.sub(b).toUInt32().toString(16); let part=null, evl=null, batch=null, serial=0;
 const env = lp => lp.add(0x100).readPointer();
 const pars = lp => { const e=env(lp); return [e.add(0x2454).readS32(), e.add(0x244c).readS32(), e.add(0x241c).readS32(), e.add(0x2510).readS32()]; };
 const bounds = lp => { const d=lp.add(0xe8).readPointer(), n=d.add(0xc).readS32();
   return [n,raw(d.add(0x390).readPointer(), n*8),raw(d.add(0x398).readPointer(), n*8)]; };
 const state = (s, v) => { const nr=s.add(0x64).readS32(), nc=s.add(0x68).readS32(), n=nr+nc, lu=s.add(v?0x470:0x498).readPointer();
   return {dim:[nr,nc], status:[s.add(0x294).readS32(),s.add(0x3ec).readS32(),s.add(0x2f0).readS32()],
     stat:raw(s.add(0x168).readPointer(),n*4), flags:raw(s.add(0x418).readPointer(),n),
     vec:raw(s.add(v?0x1e0:0x230).readPointer(),n*(v?8:16)), basic:raw(lu.add(0x18).readPointer(),nr*4)}; };
 const factor = (p,m,saved) => {
   const at = o=>p.add(o).readPointer();
   const u=at(saved?0x90:0xb0).add(m*8).readS64().toNumber();
   const l=at(saved?0x68:0x88).add(m*8).readS64().toNumber();
   const eta=p.add(saved?0xcc:0xec).readS32();
   if(m<0 || u<0 || l<0 || eta<0 || Math.max(m,u,l,eta)>10000000)throw Error('factor length');
   const specs=[[0x30,0x50,m,8],[0x90,0xb0,m+1,8],[0xb0,0xd0,u,4],[0xa8,0xc8,u,8],
     [0x68,0x88,m+1,8],[0x78,0x98,m,8],[0x88,0xa8,l,4],[0x80,0xa0,l,8],
     [0xd0,0xf0,eta,4],[0xd8,0xf8,m,4]];
   return {m,u,l,eta,arrays:specs.map(([a,b,n,w])=>raw(at(saved?a:b),n*w))};
 };
 Interceptor.attach(b.add(0x576220),{onEnter(a){this.ev=evl;if(!this.ev)return;this.out=a[1];this.m=a[0].add(0x64).readS32();this.src=factor(a[0].add(0x470).readPointer(),this.m,false);},onLeave(r){if(this.ev && !r.isNull())this.ev.steps.push({kind:'factorSave',src:this.src,dst:factor(this.out.readPointer(),this.m,true)});}});
 Interceptor.attach(b.add(0x184e60),{onEnter(a){this.ev=evl;if(this.ev)this.lp=a[0];},onLeave(r){if(this.ev)this.ev.steps.push({kind:'flush',rc:r.toInt32(),bounds:bounds(this.lp)});}});
 Interceptor.attach(b.add(0x1690d0),{onEnter(a){this.ev=evl;if(!this.ev)return;
   this.lp=a[0];this.before=bounds(this.lp);this.n=a[1].toInt32();this.j=ints(a[2],this.n);
   this.val=doubles(a[3],this.n);this.dir=raw(a[4],this.n);
 },onLeave(r){if(this.ev)this.ev.steps.push({kind:'boundwrite',rc:r.toInt32(),j:this.j,val:this.val,dir:this.dir,before:this.before,after:bounds(this.lp)});}});
 // 62ec1d: stack arguments are &start, &count, work.
 Interceptor.attach(b.add(0x543740),{onEnter(a){ const sp=this.context.rsp;
   this.j=a[2]; this.n=a[3].toInt32(); this.extra=a[4].toInt32(); this.pp=a[5];
   this.start=sp.add(8).readPointer(); this.count=sp.add(16).readPointer(); this.work=sp.add(24).readPointer();
   const len=this.n+this.extra; this.len=len;
   this.arr=()=>[ints(this.j,len),doubles(this.pp.readPointer(),len),doubles(this.pp.add(8).readPointer(),len)];
   part={e:'partition',id:serial++,n:this.n,extra:this.extra,before:this.arr(),work0:wk(this.work),q:[]}; this.ev=part;
  },onLeave(){ Object.assign(this.ev,{after:this.arr(),start:this.start.readS32(),count:this.count.readS32(),work1:wk(this.work)}); send(this.ev); part=null; }});
 Interceptor.attach(b.add(0x5b77a0),{onEnter(a){this.p=part;this.j=a[3].toInt32();},onLeave(r){if(this.p)this.p.q.push([this.j,r.toInt32()]);}});
 // 691453(Node, LP, &classification, flag, work): no candidate list argument.
 Interceptor.attach(b.add(0x56f470),{onEnter(a){this.lp=a[1];this.out=a[2];this.ev={e:'parent',id:serial++,flag:a[3].toInt32(),status0:this.lp.add(0x64).readS32(),p0:pars(this.lp)};},
  onLeave(r){Object.assign(this.ev,{rc:r.toInt32(),classification:this.out.isNull()?null:this.out.readS32(),status1:this.lp.add(0x64).readS32(),p1:pars(this.lp)});send(this.ev);}});
 // 691b56(double time, LP, Node, symmetry, n, candidates, threads, types, stall, A, B, &iters, work)
 Interceptor.attach(b.add(0x56faf0),{onEnter(a){const sp=this.context.rsp;this.lp=a[0];this.n=a[3].toInt32();this.j=a[4];
   this.a=sp.add(24).readPointer();this.z=sp.add(32).readPointer();this.cost=sp.add(40).readPointer();
   const I=a[1].add(0x18).readPointer(), M=I.readPointer(); this.T=M.add(0x3250).readPointer();this.e=env(this.lp);
   const sym=a[2], types=sp.add(8).readPointer(), js=ints(this.j,this.n);
   const orbit=!sym.isNull() && sym.add(4).readS32()>0?sym.add(0x40).readPointer():ptr(0);
   this.ev={e:'batch',id:serial++,ra:off(this.returnAddress),n:this.n,j:js,groups:js.map(j=>!orbit.isNull() && types.add(j).readU8()===66?orbit.add(j*4).readS32():null),threads:a[5].toInt32(),stall:sp.add(16).readS32(),
     cost0:hx(this.T.add(0x68)),limit0:hx(this.e.add(0x2960)),limitSaved:off(this.returnAddress)==='542623'?hx(sp.add(56)):null,p0:pars(this.lp)};batch=this.ev;},
  onLeave(r){Object.assign(this.ev,{rc:r.toInt32(),a:doubles(this.a,this.n),z:doubles(this.z,this.n),iters:hx(this.cost),p1:pars(this.lp)});send(this.ev);batch=null;}});
 // Probe after 62f909 has added the returned iteration sum and restored IterationLimit.
 Interceptor.attach(b.add(0x542660),function(){const x=this.context;send({e:'cost',cost1:hx(x.rcx.add(0x68)),limit1:hx(x.r10.add(0x2960))});});
 // Selection budget/base/cutoff/weight after pow has spilled its result.
 Interceptor.attach(b.add(0x541bb9),function(){const x=this.context,sp=x.rsp,N=sp.add(0x128).readPointer(),I=N.add(0x18).readPointer();
   send({e:'budget',cap:x.r15.toInt32(),total:sp.add(0xf8).readS32(),threads:sp.add(0x110).readS32(),
     base:hx(sp.add(0xc8)),nodeobj:hx(I.add(0x10)),cut:hx(sp.add(0xd0)),weight:hx(sp.add(0xd8)),depth:I.add(0x2c).readS32()});});
 // 266b6a shares the ABI with 695bd7, which it tail-dispatches to for the double kernel.
 for(const [addr,tag] of [[0x29d980,'kernel'],[0x571770,'double']])
 Interceptor.attach(b.add(addr),{onEnter(a){const sp=this.context.rsp;this.prev=evl;this.lp=a[0];this.n=a[2].toInt32();this.stride=a[3].toInt32();
   this.a=sp.add(16).readPointer();this.z=sp.add(24).readPointer();this.cost=sp.add(32).readPointer();
   this.ev={e:tag,id:serial++,batch:batch?batch.id:null,n:this.n,stride:this.stride,j:ints(a[4],this.n),
     p0:pars(this.lp),bounds0:bounds(this.lp),status0:this.lp.add(0x64).readS32(),cost0:hx(this.cost),steps:[]};this.ev._a=this.a;this.ev._z=this.z;evl=this.ev;},
  onLeave(r){Object.assign(this.ev,{rc:r.toInt32(),p1:pars(this.lp),bounds1:bounds(this.lp),status1:this.lp.add(0x64).readS32(),
    cost1:hx(this.cost),a:doubles(this.a,this.n),z:doubles(this.z,this.n)});delete this.ev._a;delete this.ev._z;delete this.ev._br;send(this.ev);evl=this.prev;}});
 // Copy and restore: capture only fields that are explicitly copied by both C variants.
 for(const [addr,tag,v] of [[0x29ff60,'save',0],[0x2a49a0,'restore',0],[0x576a30,'save',1],[0x5754e0,'restore',1]])
 Interceptor.attach(b.add(addr),{onEnter(a){this.ev=evl;if(!this.ev)return;this.dst=a[1];this.src=state(a[0],v);if(v && tag==='restore')this.factor=factor(a[2],this.src.dim[0],true);},
  onLeave(){if(this.ev){const br=this.ev._br,k=br?this.ev.j.indexOf(br[0]):-1;
    this.ev.steps.push({kind:tag,addr:addr.toString(16),src:this.src,dst:state(this.dst,v),branch:br||null,
      result:k<0?null:[hx(this.ev._a.add(8*k)),hx(this.ev._z.add(8*k))],
      factor:this.factor||null,factorDst:this.factor?factor(this.dst.add(0x470).readPointer(),this.src.dim[0],false):null});}}});
 for(const addr of [0x2a31f0,0x573d30])
 Interceptor.attach(b.add(addr),{onEnter(a){if(evl)evl._br=[a[4].toInt32(),a[5].toInt32()];}});
 Interceptor.attach(b.add(0x7a3820),{onEnter(a){this.ev=evl;if(!this.ev)return;this.attr=a[1].readUtf8String();this.out=a[2];},
  onLeave(r){if(this.ev && ['IterCount','ObjVal'].includes(this.attr))this.ev.steps.push({kind:'attr',attr:this.attr,rc:r.toInt32(),value:hx(this.out)});}});
 // Iteration/LP status consumption is recorded at the actual solve boundary inside the candidate kernel.
 Interceptor.attach(b.add(0x153cc0),{onEnter(a){this.ev=evl;if(this.ev){this.lp=a[0];this.b0=bounds(this.lp);}},onLeave(r){if(this.ev)this.ev.steps.push({kind:'solve',rc:r.toInt32(),status:this.lp.add(0x64).readS32(),dual:hx(this.lp.add(0xe0).readPointer().add(0x138)),before:this.b0,after:bounds(this.lp)});}});
}});
`;
(async () => {
 const device = await frida.getLocalDevice(); const counts = {}; const events = []; let outputText = ''; const errors = [];
 const pid = await device.spawn([py, '-c', code], {stdio: 'pipe'}); const session = await device.attach(pid);
 const script = await session.createScript(agent);
 script.message.connect(msg => { if (msg.type !== 'send') { errors.push(msg.stack || JSON.stringify(msg)); return; }
   const p = msg.payload;
   if (Array.isArray(p) && p[0] === 'rd') { const k = p.join(':'); counts[k] = (counts[k] || 0) + 1; } else events.push(p); });
 device.output.connect((p, fd, data) => { if (p === pid) outputText += data.toString(); });
 session.detached.connect((reason) => {
   if (outputText.includes('Gurobi is already running')) { console.log(outputText.trim()); process.exit(1); }
   const ok = outputText.includes('DONE') && !errors.length;
   fs.mkdirSync(path.dirname(output), {recursive: true});
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'mip_sb_snapshot', control: mode === 'control', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, runtime: {arch:process.arch,os:require('os').release(),node:process.version}, output: outputText.trim(), errors, ok,
     detach_reason: reason, counts, events})));
   const c = {}; for (const e of events) { const k = Array.isArray(e) ? e[0] + (e[0] === 'sb' ? ':' + e[1] : '') : e.e; c[k] = (c[k] || 0) + 1; }
   console.log(path.basename(model), outputText.trim(), JSON.stringify(c), 'ok', ok, errors.join(' ').slice(0, 300));
   process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
