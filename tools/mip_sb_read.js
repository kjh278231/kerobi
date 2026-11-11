// M2-06g (docs/m2/mip_sb_read.md): read-only trace of the pseudocost readers, the SB cache and the reliability
// threshold eta. Events are arrays (first element = kind) in execution order:
//   new   6ec3da / 6eca1a: table serial, slot, size, eta (+0x60), sbit (+0x64), sbcost (+0x68)
//   free  6edb26: table serial
//   f     6ef12f: ra, T, T2, j, x (probe 0x5b794f reads the spill [rsp]), j entries of T and T2, cache entries, dn, up
//   e     6eed2e: ra, W/N/L facts, j entries, eta, cache entries, return
//   a     6ee4ac: ra, T, j, x (read from the caller's registers per call site), cache entries, return
//   w     6ee5d1: ra, T, j, spilled xmm args (probe 0x5b75ea), allocation facts, written j entries, rc
//   ws    62f909 0x542863 (before the 6ee5d1 call at 0x54287f; rbx r12-r15 rbp rsp unchanged until the call): the
//         caller memory the xmm args are built from
//   sb    62f909 entry / leave: T, eta and sbit
//   eb    62f909 0x5436a0 (eta block, call of adf00e): eta0, sbcost, it spill, timer, *W, M+0x8c0
//   set   6edd89 call sites (probes): return address, caller, T, value; 'i<addr>' = inline write in 62f909
//   it    6edda1 entry / leave: register args, the fields it reads, T+0x64 after
//   rd    counts (not events), same format as tools/mip_pcost_bc.js, for the non-interference check
// Cache entries other than stamp are read only when stamp[j] == W+0x30 (the readers read them only then; the
// arrays are not initialised beyond stamp). xmm registers are not readable (lessons/tooling.md). Doubles are u64
// hex. Tables are named by creation order. Nothing is written into the target. One invocation spawns exactly one
// original process.
// Usage: tools/gurun.sh node tools/mip_sb_read.js MODEL OUTPUT.json.gz 'JSON_PARAMS'
const fs = require('fs'), path = require('path'), crypto = require('crypto'), zlib = require('zlib');
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
const funcs = fs.readFileSync(path.join(root, 're/decomp/index.tsv'), 'utf8').trim().split('\n').slice(1).map(l => l.split('\t'))
  .map(r => [parseInt(r[0], 16), parseInt(r[2], 10), r[1].replace(/^PRIVATE0+/, '')]).sort((a, b) => a[0] - b[0]);
const params = Object.assign({Threads: 1, Seed: 0, LogToConsole: 0}, JSON.parse(extra || '{}'));
const code = `import gurobipy as gp\ne=gp.Env(params=${JSON.stringify(params)})\nm=gp.read(${JSON.stringify(path.resolve(model))}, e)\n` +
  `m.optimize()\nprint('DONE',m.Status,int(m.NodeCount),flush=True)`;
const agent = `
const FUNCS = ${JSON.stringify(funcs)};
function fname(o) { let lo = 0, hi = FUNCS.length - 1;
  while (lo < hi) { const mid = (lo + hi + 1) >> 1; if (FUNCS[mid][0] <= o) lo = mid; else hi = mid - 1; }
  const f = FUNCS[lo]; return f[0] <= o && o < f[0] + f[1] ? f[2] : 'raw:' + o.toString(16); }
const hx = p => p.readU64().toString(16).padStart(16, '0');
Process.attachModuleObserver({onAdded(m) { if (m.name !== 'libgurobi130.so') return;
 const b = m.base, off = p => p.sub(b).toUInt32();
 // ---- tables by creation order (same slot names as tools/mip_pcost_bc.js)
 const tabs = new Map(); let tabSerial = 0; const slots = new Map();
 const tser = t => t.isNull() ? -2 : (tabs.has(t.toString()) ? tabs.get(t.toString()) : -1);
 const tslot = t => { if (t.isNull()) return 'null'; const k = tser(t); return k < 0 ? '?' : slots.get(k); };
 const tsize = t => { const d = t.add(0x70).readPointer().add(8).readPointer().add(0xe8).readPointer();
   return [d.add(0xc).readS32(), d.add(8).readS32()]; };
 const ent = (t, j) => [t.readPointer().add(4 * j).readS32(), t.add(8).readPointer().add(4 * j).readS32(),
   hx(t.add(0x10).readPointer().add(8 * j)), hx(t.add(0x18).readPointer().add(8 * j))];
 // cache entries of j: null if no cache; [stamp] if stamp != w30; [stamp, work, xsb, c-, c+] otherwise
 const cache = (t, j, w30) => { const s = t.add(0x20).readPointer(); if (s.isNull()) return null;
   const st = s.add(4 * j).readS32(); if (w30 === null || st !== w30) return [st];
   return [st, hx(t.add(0x28).readPointer().add(8 * j)), hx(t.add(0x30).readPointer().add(8 * j)),
     hx(t.add(0x38).readPointer().add(8 * j)), hx(t.add(0x40).readPointer().add(8 * j))]; };
 const tinfo = t => [t.add(0x60).readS32(), t.add(0x64).readS32(), hx(t.add(0x68))];
 for (const [fa, fn, oi] of [[0x5b6690, '6ec3da', 3], [0x5b6a30, '6eca1a', 2]])
 Interceptor.attach(b.add(fa), {onEnter(a) { this.out = a[oi]; this.slot = this.out.sub(a[0]).toInt32(); this.from = fname(off(this.returnAddress)); },
  onLeave(r) { const t = this.out.readPointer();
   if (t.isNull()) { send({e: 'new', f: fn, from: this.from, rc: r.toInt32(), t: -2}); return; }
   const k = tabSerial++; tabs.set(t.toString(), k); slots.set(k, fn === '6ec3da' ? this.slot.toString(16) : 'copy');
   send({e: 'new', f: fn, from: this.from, rc: r.toInt32(), t: k, slot: slots.get(k), size: tsize(t), info: tinfo(t),
     c20: t.add(0x20).readPointer().isNull() ? 0 : 1}); }});
 Interceptor.attach(b.add(0x5b70c0), {onEnter(a) { const t = a[1].readPointer(); if (t.isNull()) return;
   send({e: 'free', from: fname(off(this.returnAddress)), t: tser(t)}); tabs.delete(t.toString()); }});
 const rd = (fn, ra, t, t2) => send(['rd', fn, fname(ra), ra.toString(16), tslot(t), t2 === undefined ? '-' : tslot(t2)]);
 // ---- 6ef12f(x, W, T, T2, j, &dn, &up)
 let F = null;
 Interceptor.attach(b.add(0x5b7920), {onEnter() { const x = this.context, ra = off(this.returnAddress);
   rd('6ef12f', ra, x.rsi, x.rdx);
   const W = x.rdi, T = x.rsi, T2 = x.rdx, j = x.rcx.toInt32();
   const w30 = !T.add(0x20).readPointer().isNull() ? W.add(0x30).readS32() : null;
   F = {ev: ['f', ra.toString(16), tser(T), tser(T2), j, null, w30, w30 === null ? null : hx(W.add(0x38)),
     ent(T, j), T2.isNull() ? null : ent(T2, j), cache(T, j, w30)], dn: x.r8, up: x.r9}; },
  onLeave() { const ev = F.ev; ev.push(hx(F.dn), hx(F.up)); send(ev); F = null; }});
 Interceptor.attach(b.add(0x5b794f), function () { F.ev[5] = hx(this.context.rsp); });
 // ---- 6eed2e(W, N, T, j)
 let E = null;
 Interceptor.attach(b.add(0x5b77a0), {onEnter() { const x = this.context, ra = off(this.returnAddress);
   rd('6eed2e', ra, x.rdx);
   const W = x.rdi, N = x.rsi, T = x.rdx, j = x.rcx.toInt32(), I = N.add(0x18).readPointer();
   const M = I.readPointer(), L = I.add(8).readPointer(), xl = L.isNull() ? ptr(0) : L.add(0x18).readPointer();
   const w30 = W.isNull() || T.add(0x20).readPointer().isNull() ? null : W.add(0x30).readS32();
   E = ['e', ra.toString(16), tser(T), j, M.add(0x790).readPointer().equals(N) ? 1 : 0, L.isNull() ? 0 : 1, xl.isNull() ? 0 : 1,
     W.isNull() ? 0 : 1, w30, w30 === null ? null : hx(W.add(0x38)), xl.isNull() ? null : hx(xl.add(8 * j)),
     ent(T, j).slice(0, 2), T.add(0x60).readS32(), cache(T, j, w30)]; },
  onLeave(r) { E.push(r.toInt32()); send(E); E = null; }});
 // ---- 6ee4ac(x, W, T, j): x from the caller registers (asm 0x542d6c / 0x535f6b reload the same cell)
 const XSITE = {0x542d89: ['r14', 'rbp'], 0x535f88: ['r14', 'rbx']};
 let A = null;
 Interceptor.attach(b.add(0x5b7540), {onEnter() { const x = this.context, ra = off(this.returnAddress);
   const W = x.rdi, T = x.rsi, j = x.rdx.toInt32(), s = XSITE[ra];
   const w30 = !T.add(0x20).readPointer().isNull() ? W.add(0x30).readS32() : null;
   A = ['a', ra.toString(16), tser(T), j, s ? hx(x[s[0]].add(8 * x[s[1]].toInt32())) : null, w30,
     w30 === null ? null : hx(W.add(0x38)), cache(T, j, w30)]; },
  onLeave(r) { A.push(r.toInt32() & 0xffffffff); send(A); A = null; }});
 // ---- 6ee5d1(x, f-, d-, f+, d+, env, W, T, j)
 let Wr = null;
 Interceptor.attach(b.add(0x5b75a0), {onEnter() { const x = this.context, ra = off(this.returnAddress);
   const W = x.rsi, T = x.rdx, j = x.rcx.toInt32();
   Wr = {T, j, ev: ['w', ra.toString(16), tser(T), j, W.add(0x30).readS32(), hx(W.add(0x38)), null,
     T.add(0x20).readPointer().isNull() ? 0 : 1, tsize(T)[0]]}; },
  onLeave(r) { const T = Wr.T, j = Wr.j, ev = Wr.ev, s = T.add(0x20).readPointer();
   let neg = null;
   if (!ev[7] && !s.isNull()) { const n = ev[8], a = new Int32Array(s.readByteArray(4 * n)); neg = 0; for (const v of a) if (v === -1) neg++; }
   ev.push(neg, s.isNull() ? null : [s.add(4 * j).readS32(), hx(T.add(0x28).readPointer().add(8 * j)), hx(T.add(0x30).readPointer().add(8 * j)),
     hx(T.add(0x38).readPointer().add(8 * j)), hx(T.add(0x40).readPointer().add(8 * j))], r.toInt32());
   send(ev); Wr = null; }});
 Interceptor.attach(b.add(0x5b75ea), function () { const sp = this.context.rsp;
   Wr.ev[6] = [hx(sp.add(8)), hx(sp.add(0x18)), hx(sp.add(0x20)), hx(sp.add(0x10)), hx(sp.add(0x28))]; });
 // probe 0x542863, not the call at 0x54287f: a probe that relocates a call changes the callee's return address
 Interceptor.attach(b.add(0x542863), function () { const x = this.context, sp = x.rsp, k = x.r15.toInt32();
   send(['ws', hx(x.rbx.add(8 * x.r14.toInt32())), hx(sp.add(0x60)), hx(x.rbp.add(8 * k)), hx(sp.add(0xc8)), hx(sp.add(0xd8)),
     hx(sp.add(0x18)), hx(x.r12.add(8 * k)), x.r13.toInt32()]); });
 // ---- 62f909 entry/leave and the eta block
 Interceptor.attach(b.add(0x541280), {onEnter(a) { const M = a[0].readPointer().add(0x18).readPointer().readPointer();
   const T = M.add(0x3250).readPointer(); this.T = T;
   send(['sb', 'in', fname(off(this.returnAddress)), tser(T)].concat(tinfo(T))); },
  onLeave(r) { send(['sb', 'out', r.toInt32(), tser(this.T)].concat(tinfo(this.T))); }});
 Interceptor.attach(b.add(0x5436a0), function () { const x = this.context, sp = x.rsp, tm = x.rdi, M = x.rbx, T = x.r13;
   send(['eb', tser(T), sp.add(0x168).readS32(), hx(T.add(0x68)), hx(sp), hx(tm), tm.add(0x28).readS32(),
     x.rsi.isNull() ? null : hx(x.rsi), hx(M.add(0x8c0))]); });
 // 6edd89 is 4 bytes (mov %esi,0x60(%rdi); ret) and cannot be intercepted: probe every call site (callgraph)
 for (const cs of [0x4e45c5, 0x4ea656, 0x53e01b, 0x53e6d7, 0x53e6f3, 0x5819ce, 0x542fab, 0x5436f3, 0x543700, 0x543712, 0x543724])
   Interceptor.attach(b.add(cs), function () { const x = this.context;
     send(['set', (cs + 5).toString(16), fname(cs), tser(x.rdi), x.rsi.toInt32()]); });
 // 62f909 writes T+0x60 inline at five places (C:454/457/496/524/577). Probe start chosen so that the relocated window
 // holds no jump target (0x5421a0 is one): [probe, write insn, T register, value: 1 or [rsp+0x40]]
 for (const [pa, wa, tr, one] of [[0x541ce1, 0x541ce1, 'rax', 1], [0x541e4b, 0x541e53, 'rsi', 0], [0x541f90, 0x541f90, 'rsi', 1],
     [0x542199, 0x54219d, 'rsi', 0], [0x5435f8, 0x543604, 'rsi', 0]])
   Interceptor.attach(b.add(pa), function () { const x = this.context;
     send(['set', 'i' + wa.toString(16), '62f909', tser(x[tr]), one ? 1 : x.rsp.add(0x40).readS32()]); });
 // ---- 6edda1(M, T, a, b, p)
 Interceptor.attach(b.add(0x5b7220), {onEnter(a) { const M = a[0], T = a[1], g = M.add(0x620).readPointer();
   this.T = T; this.ev = ['it', fname(off(this.returnAddress)), tser(T), a[2].toInt32(), a[3].toInt32(), a[4].toInt32(),
     M.add(0x3458).readS32(), g.add(0x808).readS32(), M.add(0x618).readPointer().isNull() ? 0 : 1,
     M.add(8).readPointer().add(0x100).readPointer().add(10000).readS32(), hx(g.add(0x60)), T.add(0x64).readS32()]; },
  onLeave() { this.ev.push(this.T.add(0x64).readS32()); send(this.ev); }});
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
   fs.writeFileSync(output, zlib.gzipSync(JSON.stringify({schema: 1, kind: 'mip_sb_read', model: path.basename(model), model_sha256: hash(model),
     binary_sha256: expected, tool_sha256: hash(__filename), params, output: outputText.trim(), errors, ok,
     detach_reason: reason, counts, events})));
   const c = {}; for (const e of events) { const k = Array.isArray(e) ? e[0] + (e[0] === 'sb' ? ':' + e[1] : '') : e.e; c[k] = (c[k] || 0) + 1; }
   console.log(path.basename(model), outputText.trim(), JSON.stringify(c), 'ok', ok, errors.join(' ').slice(0, 300));
   process.exit(ok ? 0 : 1); });
 await script.load(); await device.resume(pid);
})().catch(e => { console.error(e); process.exit(1); });
