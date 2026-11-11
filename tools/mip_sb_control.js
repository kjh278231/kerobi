// M2-06h-2: extend the fixed snapshot capture with independent control inputs.
const fs = require('fs'), path = require('path'), Module = require('module');
let source = fs.readFileSync(path.join(__dirname, 'mip_sb_snapshot.js'), 'utf8');
function replace(old, value) {
  if (!source.includes(old)) throw Error('snapshot source changed: ' + old);
  source = source.replace(old, value);
}
replace("kind: 'mip_sb_snapshot'", "kind: 'mip_sb_control'");
replace(' // Copy and restore:', `
 Interceptor.attach(b.add(0x1a45d0),{onEnter(a){this.ev=evl;if(!this.ev)return;this.out=a[1];this.lp=a[0];},
  onLeave(r){if(this.ev && !this.out.isNull()){
   const d=this.lp.add(0xe8).readPointer(), n=d.add(0xc).readS32(),sc=d.add(0x3e0).readPointer();
   this.ev.steps.push({kind:'input',rc:r.toInt32(),x:doubles(this.out,n),tol:hx(env(this.lp).add(0x2468)),
    scales:Array.from({length:n},(_,j)=>sc.isNull()?'3ff0000000000000':hx(sc.add(j*8))),
    flipped:d.add(0x3c).readS32(),flips:raw(d.add(0x408).readPointer(),n)});
  }}});
 Interceptor.attach(b.add(0x575ce0),{onEnter(){this.ev=evl;},onLeave(r){if(this.ev)this.ev.steps.push({kind:'reset',rc:r.toInt32()});}});
 // Copy and restore:`);
const capture = new Module(__filename, module);
capture.filename = __filename;
capture.paths = module.paths;
capture._compile(source, __filename);
