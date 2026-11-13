// M2-06h-6: the M2-06h-3 loop capture, null safe for the VarBranch=3 call (62a05d passes Node = NULL
// and &iters = NULL to 691b56), plus the branch inputs of the direct (no propagation) path.
// mip_sb_loop.js and mip_sb_snapshot.js stay unchanged: their hashes are pinned by the 06h-3/06h-4 data.
const fs = require('fs'), path = require('path'), Module = require('module');
let source = fs.readFileSync(path.join(__dirname, 'mip_sb_loop.js'), 'utf8');
function replace(old, value) {
  if (!source.includes(old)) throw Error('loop source changed: ' + old);
  source = source.replace(old, value);
}
// Extra replacements applied to the snapshot agent after the loop tool's own ones.
replace('const capture = new Module', `
replace("kind: 'mip_sb_loop'", "kind: 'mip_sb_direct'");
// 691b56(double, LP, Node, ...): Node and &iters may be NULL.
replace("const I=a[1].add(0x18).readPointer(), M=I.readPointer(); this.T=M.add(0x3250).readPointer();",
  "const N=a[1], I=N.isNull()?ptr(0):N.add(0x18).readPointer(), M=I.isNull()?ptr(0):I.readPointer(); this.T=M.isNull()?ptr(0):M.add(0x3250).readPointer();");
replace("cost0:hx(this.T.add(0x68))", "node:!N.isNull(),cost0:this.T.isNull()?null:hx(this.T.add(0x68))");
replace("iters:hx(this.cost)", "iters:this.cost.isNull()?null:hx(this.cost)");
// 695bd7/266b6a(double, LP, Node, ...): C:343 propagates only when Node and Node+0x18 are non-null.
replace("this.ev={e:tag,id:serial++,batch:batch?batch.id:null,",
  "this.ev={e:tag,id:serial++,batch:batch?batch.id:null,node:!a[1].isNull(),propagation:!a[1].isNull() && !a[1].add(0x18).readPointer().isNull(),");
replace("cost0:hx(this.cost),steps:[]", "cost0:this.cost.isNull()?null:hx(this.cost),steps:[]");
replace("cost1:hx(this.cost),", "cost1:this.cost.isNull()?null:hx(this.cost),");
// Infinity threshold read at C:413-431 for the direct-path restore bounds.
replace("tol:hx(env(this.lp).add(0x2468)),", "tol:hx(env(this.lp).add(0x2468)),inf:hx(env(this.lp).add(0x2458)),");
const capture = new Module`);
const capture = new Module(__filename, module);
capture.filename = __filename;
capture.paths = module.paths;
capture._compile(source, __filename);
