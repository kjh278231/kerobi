// M2-06h-3: whole 695bd7 candidate loop. Extends the M2-06h-2 control capture (same replacements)
// with propagation (6953ff), later-candidate solution reads (12ee1a), timer/poll calls,
// original bound getters, the stall limit and the work counter after every recorded step.
const fs = require('fs'), path = require('path'), Module = require('module');
let source = fs.readFileSync(path.join(__dirname, 'mip_sb_snapshot.js'), 'utf8');
function replace(old, value) {
  if (!source.includes(old)) throw Error('snapshot source changed: ' + old);
  source = source.replace(old, value);
}
replace("kind: 'mip_sb_snapshot'", "kind: 'mip_sb_loop'");
// Factor arrays were verified in M2-06h-1; tr12-30 would exceed the JSON string limit, so keep length + FNV-1a only.
replace('arrays:specs.map(([a,b,n,w])=>raw(at(saved?a:b),n*w))}', 'arrays:specs.map(([a,b,n,w])=>digest(raw(at(saved?a:b),n*w)))}');
replace('const wk = p =>', `const digest = s => { if (s === null) return null; let h = 0x811c9dc5;
  for (let i = 0; i < s.length; i++) h = Math.imul(h ^ s.charCodeAt(i), 0x01000193) >>> 0;
  return s.length + ':' + h.toString(16); };
const wk = p =>`);
// Full bound arrays only at kernel entry/exit; per-request/flush/solve copies are length + FNV-1a (tr12-30 size).
replace(' const bounds = lp => {', ' const bounds = lp => { const r = rawBounds(lp); return [r[0], digest(r[1]), digest(r[2])]; };\n const rawBounds = lp => {');
replace('stat:raw(s.add(0x168).readPointer(),n*4), flags:raw(s.add(0x418).readPointer(),n),', 'stat:digest(raw(s.add(0x168).readPointer(),n*4)), flags:digest(raw(s.add(0x418).readPointer(),n)),');
replace('vec:raw(s.add(v?0x1e0:0x230).readPointer(),n*(v?8:16)), basic:raw(lu.add(0x18).readPointer(),nr*4)}', 'vec:digest(raw(s.add(v?0x1e0:0x230).readPointer(),n*(v?8:16))), basic:digest(raw(lu.add(0x18).readPointer(),nr*4))}');
replace('bounds0:bounds(this.lp)', 'bounds0:rawBounds(this.lp)');
replace('bounds1:bounds(this.lp)', 'bounds1:rawBounds(this.lp)');
replace(' // Copy and restore:', `
 Interceptor.attach(b.add(0x1a45d0),{onEnter(a){this.ev=evl;if(!this.ev)return;this.out=a[1];this.lp=a[0];},
  onLeave(r){if(this.ev && !this.out.isNull()){
   const d=this.lp.add(0xe8).readPointer(), n=d.add(0xc).readS32(),sc=d.add(0x3e0).readPointer();
   this.ev.steps.push({kind:'input',rc:r.toInt32(),x:doubles(this.out,n),tol:hx(env(this.lp).add(0x2468)),
    scales:Array.from({length:n},(_,j)=>sc.isNull()?'3ff0000000000000':hx(sc.add(j*8))),
    flipped:d.add(0x3c).readS32(),flips:raw(d.add(0x408).readPointer(),n)});
  }}});
 Interceptor.attach(b.add(0x575ce0),{onEnter(){this.ev=evl;},onLeave(r){if(this.ev)this.ev.steps.push({kind:'reset',rc:r.toInt32()});}});
 // Original-bound getters (lp, 0, n, 0, out): the propagation restore loop compares against these.
 for(const [addr,tag] of [[0x1a7fc0,'origLower'],[0x1a8070,'origUpper']])
 Interceptor.attach(b.add(addr),{onEnter(a){this.ev=evl;if(!this.ev)return;this.n=a[2].toInt32();this.out=a[4];},
  // void functions (C:368-369 ignore the return), so rax is not recorded.
  onLeave(){if(this.ev)this.ev.steps.push({kind:tag,v:this.out.isNull()?null:doubles(this.out,this.n)});}});
 // 6953ff(xmm0 value, lp, ?, lower, upper, col, dir; +8 &count, +16 list, +24 flags, +32 &out, +40 work).
 Interceptor.attach(b.add(0x573d30),{onEnter(a){this.ev=evl;if(!this.ev)return;const sp=this.context.rsp;
   this.lo=a[2];this.up=a[3];this.cnt=sp.add(8).readPointer();this.list=sp.add(16).readPointer();
   this.fl=sp.add(24).readPointer();this.out=sp.add(32).readPointer();
   // xmm0 is not readable: read the caller slot it was loaded from (up 0x572352, down 0x572c86).
   const ra=off(this.returnAddress), slot={'572375':0x210,'572ca6':0xd8}[ra];
   this.ev.steps.push({kind:'propEnter',ra,value:slot===undefined?null:hx(sp.add(slot)),col:a[4].toInt32(),dir:a[5].toInt32()&0xff,count0:this.cnt.readS32(),out0:this.out.readS32()});},
  onLeave(r){if(!this.ev)return;const c=this.cnt.readS32(),l=ints(this.list,Math.max(c,0));
   this.ev.steps.push({kind:'prop',rc:r.toInt32(),count:c,out:this.out.readS32(),list:l,
    flags:l.map(j=>this.fl.add(j).readU8()),lower:l.map(j=>hx(this.lo.add(8*j))),upper:l.map(j=>hx(this.up.add(8*j)))});}});
 // 12ee1a(lp, 0, count, cols, out): solution values of the later candidates after an optimal solve.
 Interceptor.attach(b.add(0x1bbdb0),{onEnter(a){this.ev=evl;if(!this.ev)return;this.n=a[2].toInt32();this.j=a[3];this.out=a[4];},
  onLeave(r){if(this.ev)this.ev.steps.push({kind:'values',rc:r.toInt32(),j:ints(this.j,this.n),v:doubles(this.out,this.n)});}});
 Interceptor.attach(b.add(0x8e70a0),{onEnter(){this.ev=evl;},onLeave(){if(this.ev)this.ev.steps.push({kind:'poll'});}});
 Interceptor.attach(b.add(0x8eb340),{onEnter(){this.ev=evl;},onLeave(r){if(this.ev)this.ev.steps.push({kind:'timer',r:r.toInt32()});}});
 // Copy and restore:`);
replace('steps:[]};this.ev._a', `steps:[],types:raw(a[5],this.lp.add(0xe8).readPointer().add(0xc).readS32()),
     stall:sp.add(8).readS32(),work0:sp.add(40).readPointer().isNull()?null:hx(sp.add(40).readPointer())};
   { const W=sp.add(40).readPointer(); this.W=W; this.ev.steps.push=function(s){s.w=W.isNull()?null:hx(W);return Array.prototype.push.call(this,s);}; }
   this.ev._a`);
replace('cost1:hx(this.cost),', 'cost1:hx(this.cost),work1:this.W.isNull()?null:hx(this.W),');
const capture = new Module(__filename, module);
capture.filename = __filename;
capture.paths = module.paths;
capture._compile(source, __filename);
