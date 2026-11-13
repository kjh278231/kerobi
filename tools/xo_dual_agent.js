// M3-04f-4a: crossover 13f716 문맥의 쌍대값 재계산 251b4f와 그 안의 BTRAN(09e288 → 221173, cache 없음) 읽기 전용 관측,
// 그리고 push 뒤 S+0x2d8 재계산(C 1225–1316) 결과. 호출과 메모리를 대체하지 않는다(CONFIG.edits 합성 편집만 예외).
// 주소는 ELF file offset = 모듈 base 기준.
//   0x1bc5d0  13f716 진입·반환                         → xo:enter / xo:leave
//   0x1c0070 / 0x1c6440  primal·dual push 진입·반환    → push:enter / push:leave (문맥 표지)
//   0x1c9730  251b4f 진입·반환                          → dv:enter / dv:leave
//   0x12de60  221173 (반환 주소 0x12de33 = 09e288, 251b4f 안)  → dv:btran:enter / dv:btran:leave
//   0x1bf2ee  retry 검사(push 시도 루프·뒤처리 끝)      → sel:enter (편집 전 S+0x2d8)
//   0x1bf6f2  선택 결과 r14d                             → sel:decide
// CONFIG.factor = false 이면 factor 직렬화를 생략한다(호출 수 조사용, 기준 자료 아님).
// CONFIG.edits: 04f-2 synthetic.json과 같은 형식(0x1bf2ee에서 기록 뒤 적용).
const CONFIG = __CONFIG__;
Process.attachModuleObserver({onAdded(mod) {
  if (mod.name !== 'libgurobi130.so') return;
  const b = mod.base, withFactor = CONFIG.factor !== false;
  const hex = (p, size) => Array.from(new Uint8Array(p.readByteArray(size))).map(x => x.toString(16).padStart(2, '0')).join('');
  function arr(p, n, size) {
    if (n < 0 || n > 4000000) throw Error('배열 길이 오류: ' + n);
    if (n === 0) return [];
    if (p.isNull()) throw Error('필수 배열 null');
    if (size === 4) return Array.from({length:n}, (_, i) => p.add(i*4).readS32());
    const raw = new Uint8Array(p.readByteArray(n*size)), out = new Array(n);
    for (let i = 0; i < n; i++) { let s = ''; for (let k = 0; k < size; k++) s += raw[i*size+k].toString(16).padStart(2, '0'); out[i] = s; }
    return out;
  }
  const vec = (s, off, n, size=8) => arr(s.add(off).readPointer(), n, size);
  const ints = (p, n) => arr(p, n, 4), dbls = (p, n) => arr(p, n, 8);
  const longs = (p, n) => { if (n < 0 || n > 4000000) throw Error('long 길이 오류'); return Array.from({length:n}, (_, i) => p.add(i*8).readS64().toNumber()); };
  function nonzero(p, n, size) { const r = []; for (let i = 0; i < n; i++) { const v = size === 4 ? p.add(i*4).readS32() : hex(p.add(i*8), 8); if (size === 4 ? v !== 0 : v !== '0000000000000000') r.push([i, v]); } return r; }
  function spikeState(p, m) {
    if (p.isNull()) return null;
    const count = p.readS32(), indices = count < 0 ? null : vec(p, 8, count, 4), values = p.add(16).readPointer();
    return {count, indices, values: count < 0 ? arr(values, m, 8) : indices.map(r => hex(values.add(r*8), 8))};
  }
  // push_btran_agent.js(M3-04c-21c)의 factorState·btFactor와 같은 직렬화(Rust 재생 하네스가 같은 스키마를 읽는다).
  function factorState(F) {
    const m = F.add(8).readS32(), P = o => F.add(o).readPointer(), I = o => F.add(o).readS32(), L64 = o => F.add(o).readS64().toNumber();
    const lb = longs(P(0x58), m+1), l0 = lb[0], l1 = lb[m];
    const ucol = [], urow = [];
    const cs = longs(P(0x88), m), ce = longs(P(0x90), m), cb = longs(P(0x98), m), cv = P(0xa0), cr = P(0xa8);
    for (let j = 0; j < m; j++) ucol.push({rows: ints(cr.add(4*cs[j]), ce[j]-cs[j]), vals: dbls(cv.add(8*cs[j]), ce[j]-cs[j])});
    const rs = longs(P(0xb0), m), re = longs(P(0xb8), m), rv = P(0xc8), rc = P(0xd0);
    for (let i = 0; i < m; i++) urow.push({cols: ints(rc.add(4*rs[i]), re[i]-rs[i]), vals: dbls(rv.add(8*rs[i]), re[i]-rs[i])});
    const ne = I(0x180), rec = P(0x198), p148 = P(0x148), p150 = P(0x150), etas = [];
    for (let q = 0; q < ne; q++) { const e = rec.add(0x18*q), r = e.readS32(), len = e.add(4).readS32(), ip = e.add(8).readPointer(), vp = e.add(0x10).readPointer();
      etas.push({r, len, idx_off: ip.sub(p148).toInt32()/4, val_off: vp.sub(p150).toInt32()/8, idx_mod16: ip.and(0xf).toInt32(), val_mod16: vp.and(0xf).toInt32(), idx: ints(ip, len), val: dbls(vp, len)}); }
    const x128 = I(0x128);
    return {m, zero: hex(F.add(0x10), 8), header: ints(P(0x18), m), perm: {x20: ints(P(0x20), m), x28: ints(P(0x28), m), x30: ints(P(0x30), m), x38: ints(P(0x38), m), x40: ints(P(0x40), m), x48: ints(P(0x48), m)},
      diag: dbls(P(0x50), m), L: {beg: lb, rows: ints(P(0x68).add(4*l0), l1-l0), vals: dbls(P(0x60).add(8*l0), l1-l0), levels: x128,
        lev: x128 >= 0 ? ints(P(0x118), m) : null, lst: x128 >= 0 ? ints(P(0x120), x128+2) : null},
      U: {col_start: cs, col_end: ce, col_bound: cb, col: ucol, row_start: rs, row_end: re, row: urow},
      scalars: {xd8: I(0xd8), xdc: I(0xdc), xe4: I(0xe4), xe8: I(0xe8), xec: I(0xec), x100: I(0x100), x104: I(0x104), x108: hex(F.add(0x108), 8), x110: hex(F.add(0x110), 8),
        x128: x128, x180: ne, x184: I(0x184), x188: I(0x188), x18c: I(0x18c), x158: L64(0x158), x160: L64(0x160), x168: L64(0x168), x1f8: hex(F.add(0x1f8), 8),
        pool_mod16: [p148.and(0xf).toInt32(), p150.and(0xf).toInt32()]},
      prefix: {list: ints(P(0xf0), I(0xec)), flag: ints(P(0xf8), m)}, eta_oldest: ints(P(0x1a0), m), etas,
      spike: spikeState(P(0x1c8), m), scratch: {x1b0: nonzero(P(0x1b0), m, 4), x1b8: nonzero(P(0x1b8), m, 8)}};
  }
  function cacheState(F, m) {
    const p = F.add(0x1d0).readPointer();
    if (p.isNull()) return null;
    const count = p.readS32(), ok = count >= 0 && count <= m;
    return {count, idx: ok ? ints(p.add(8).readPointer(), count) : null, val: ok ? dbls(p.add(16).readPointer(), count) : null};
  }
  function btFactor(F) {
    const m = F.add(8).readS32(), P = o => F.add(o).readPointer(), I = o => F.add(o).readS32();
    const base = factorState(F), rb = longs(P(0x70), m+1), r0 = rb[0], r1 = rb[m], x140 = I(0x140);
    base.Lrow = {beg: rb, cols: ints(P(0x80).add(4*r0), r1-r0), vals: dbls(P(0x78).add(8*r0), r1-r0), levels: x140,
      lev: x140 >= 0 ? ints(P(0x130), m) : null, lst: x140 >= 0 ? ints(P(0x138), x140+1) : null,
      bucket: x140 >= 0 ? ints(P(0x1a8).add(4*m), x140+1) : null};
    base.scalars.x140 = x140; base.scalars.x1e0 = F.add(0x1e0).readS64().toNumber();
    base.cache = cacheState(F, m); base.x1d8 = I(0x1d8);
    return base;
  }
  function ftVec(p, m) { const c = p.readS32(), n = c < 0 ? m : c; return {count: c, idx: c < 0 ? null : ints(p.add(8).readPointer(), n), val: dbls(p.add(16).readPointer(), n)}; }

  let xo = null, seq = 0, push = null, dv = null, bt = null, dvCalls = 0, rowA = null;
  function emit(event, fields) { send({event, seq: seq++, ...fields}); }
  function dims(S) { return {m: S.add(0x64).readS32(), n: S.add(0x68).readS32()}; }
  Interceptor.attach(b.add(0x1bc5d0), {
    onEnter(a) { if (xo) throw Error('13f716 중첩'); xo = {model: a[0], mode: a[1].toInt32()}; rowA = null;
      const env = a[0].add(0x100).readPointer();
      emit('xo:enter', {mode: xo.mode, crossover: env.add(0x2ba0).readS32()}); },
    onLeave(r) { emit('xo:leave', {ret: r.toInt32(), model_status: xo.model.add(100).readS32()}); xo = null; }
  });
  for (const [kind, off] of [['primal', 0x1c0070], ['dual', 0x1c6440]]) Interceptor.attach(b.add(off), {
    onEnter(a) { if (!xo) return; if (push) throw Error('push 중첩'); this.k = kind; push = kind; emit('push:enter', {kind, attempt: a[2].toInt32()}); },
    onLeave(r) { if (!this.k) return; emit('push:leave', {kind: this.k, ret: r.toInt32()}); push = null; }
  });
  Interceptor.attach(b.add(0x1bf2ee), function () {
    const sp = this.context.rsp, S = sp.add(0x230).readPointer(), {m, n} = dims(S);
    emit('sel:enter', {m, n, retry: sp.add(0x50).readS32(), remaining: sp.add(0x54).readS32(), s2d8: hex(S.add(0x2d8), 8), s120: hex(S.add(0x120), 8),
      precision: S.add(0x28).readS32(), tol380: hex(S.add(0x380), 8), opttol_slot: hex(sp.add(0x2d8), 8),
      crossover: xo.model.add(0x100).readPointer().add(0x2ba0).readS32(), vstat: vec(S, 0x168, n + m, 4)});
    for (const e of (CONFIG.edits || [])) {
      const p = e.field === 'retry' ? sp.add(0x50) : e.field === 'remaining' ? sp.add(0x54) : e.field === 's2d8' ? S.add(0x2d8) : e.field === 's120' ? S.add(0x120) : null;
      if (!p) throw Error('알 수 없는 편집 ' + e.field);
      const before = e.int !== undefined ? p.readS32() : hex(p, 8);
      if (e.int !== undefined) p.writeS32(e.int); else p.writeByteArray(e.hex.match(/../g).map(t => parseInt(t, 16)));
      emit('edit', {field: e.field, before, after: e.int !== undefined ? p.readS32() : hex(p, 8)});
    }
  });
  Interceptor.attach(b.add(0x1bf6f2), function () {
    emit('sel:decide', {r14: this.context.r14.toInt32()});
  });
  // 행 방향 A(S+0x1b0 시작 i64·0x1b8 길이 i32·0x1c0 열·0x1c8 값): 13f716 문맥마다 처음 한 번 싣고 이후 호출은 같은지 바이트 비교한다.
  function rowMatrix(S, m) {
    const st = longs(S.add(0x1b0).readPointer(), m), ln = ints(S.add(0x1b8).readPointer(), m);
    let lo = Infinity, hi = 0; for (let i = 0; i < m; i++) if (ln[i] > 0) { lo = Math.min(lo, st[i]); hi = Math.max(hi, st[i] + ln[i]); }
    if (lo === Infinity) lo = 0;
    const cols = S.add(0x1c0).readPointer().add(4*lo), vals = S.add(0x1c8).readPointer().add(8*lo);
    const raw = [st.join(','), ln.join(','), hi > lo ? hex(cols, 4*(hi-lo)) : '', hi > lo ? hex(vals, 8*(hi-lo)) : ''].join('|');
    return {raw, data: {base: lo, start: st, len: ln, cols: hi > lo ? ints(cols, hi-lo) : [], vals: hi > lo ? dbls(vals, hi-lo) : []}};
  }
  Interceptor.attach(b.add(0x1c9730), {
    onEnter(a) {
      if (!xo) return;
      if (dv) throw Error('251b4f 중첩');
      const S = a[0], {m, n} = dims(S), F = S.add(0x470).readPointer(), sp = this.context.rsp;
      const p3 = a[2].toInt32(), p4 = a[3].toInt32(), p5 = a[4].toInt32();
      // 스택 인자: [rsp+8] p7, [rsp+0x10] p8, [rsp+0x18] p9, [rsp+0x20] p10(work*)
      const p7 = sp.add(8).readPointer(), p8 = sp.add(0x10).readPointer(), work = sp.add(0x20).readPointer();
      const sub = [];
      const pick = k => { const j = a[5].add(4*k).readS32(); sub.push([k, j, j < n ? hex(p7.add(8*j), 8) : hex(p8.add(8*(j-n)), 8)]); };
      for (let k = 0; k < p3; k++) pick(k);
      for (let k = p4; k < p5; k++) pick(k);
      const A = rowMatrix(S, m);
      let a_same = null;
      if (rowA === null) rowA = A.raw; else a_same = rowA === A.raw;
      dv = {id: ++dvCalls, S, F, m, n, out: a[1], work};
      this.on = true;
      emit('dv:enter', {id: dv.id, ret_site: this.returnAddress.sub(b).toInt32(), push, m, n, p3, p4, p5, sub,
        out_is_s1e0: a[1].equals(S.add(0x1e0).readPointer()), in_is_s220: true, s220: S.add(0x220).readPointer().isNull() ? null : 'ok',
        header: ints(F.add(0x18).readPointer(), m), cost: vec(S, 0x98, n + m), vstat: vec(S, 0x168, n + m, 4),
        tol3a8: hex(S.add(0x3a8), 8), s148: hex(S.add(0x148), 8), f1f8: hex(F.add(0x1f8), 8), work: work.isNull() ? null : hex(work, 8),
        rowA: a_same === null ? A.data : null, a_same});
    },
    onLeave() {
      if (!this.on) return;
      const {S, F, m, n, out, work} = dv;
      emit('dv:leave', {id: dv.id, d: arr(out, n + m, 8), f1f8: hex(F.add(0x1f8), 8), work: work.isNull() ? null : hex(work, 8),
        y: vec(S.add(0x228).readPointer(), 0x10, m), y_count: S.add(0x228).readPointer().readS32()});
      dv = null;
    }
  });
  Interceptor.attach(b.add(0x12de60), {
    onEnter(a) {
      this.on = dv !== null && this.returnAddress.sub(b).toUInt32() === 0x12de33; if (!this.on) return;
      if (bt) throw Error('BTRAN 중첩');
      const F = a[0], m = F.add(8).readS32(), S = dv.S;
      bt = {F, m, in: a[1], out: a[2], inIdx: a[1].add(8).readPointer(), inVal: a[1].add(16).readPointer(), outIdx: a[2].add(8).readPointer(), outVal: a[2].add(16).readPointer()};
      emit('dv:btran:enter', {id: dv.id, args: {factor_s470: F.equals(S.add(0x470).readPointer()), input_s220: a[1].equals(S.add(0x220).readPointer()),
        output_s228: a[2].equals(S.add(0x228).readPointer()), cache_null: a[3].isNull()},
        input: ftVec(a[1], m), output_count: a[2].readS32(), factor: withFactor ? btFactor(F) : null, x200: hex(F.add(0x200), 8)});
    },
    onLeave() {
      if (!this.on) return;
      const {F, m} = bt;
      const al = p => { const i = p.add(8).readPointer(), v = p.add(16).readPointer(), t = x => x.equals(bt.inIdx) ? 'in' : x.equals(bt.outIdx) ? 'out' : 'other', u = x => x.equals(bt.inVal) ? 'in' : x.equals(bt.outVal) ? 'out' : 'other'; return {idx: t(i), val: u(v)}; };
      emit('dv:btran:leave', {id: dv.id, output: ftVec(bt.out, m), input_after: ftVec(bt.in, m), alias: {in: al(bt.in), out: al(bt.out)},
        x200: hex(F.add(0x200), 8), scratch: withFactor ? {x1b0: nonzero(F.add(0x1b0).readPointer(), m, 4), x1b8: nonzero(F.add(0x1b8).readPointer(), m, 8)} : null});
      bt = null;
    }
  });
}});
