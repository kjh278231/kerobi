// M3-04f-2: crossover 본체 13f716의 push 반환 소비 → cleanup 선택 → 0cda5b 진입 → 0325b4 상태 배열 설치 읽기 전용 관측.
// 함수 중간 지점은 함수형 probe(교훈 tooling: 객체형 중간 훅 금지). 주소는 ELF file offset = 모듈 base 기준.
//   0x1bf2ee  retry 검사(push 시도 루프·뒤처리 끝)      → sel:enter
//   0x1bf6c2  Crossover 읽기(retry 잔존 재설정 뒤 합류)  → sel:reset
//   0x1bfae2  auto 쌍대 검사 251b4f 반환 직후            → sel:dual
//   0x1bf6f2  선택 결과 r14d                             → sel:decide
//   0x153cc0  0cda5b 진입(반환 주소가 13f716 안일 때만)  → cda5b:enter / cda5b:leave
//   0xd3150   0325b4 진입·반환(위 cda5b 안)              → i325b4:enter / i325b4:leave
// 스택 slot: rsp+0x50 retry, +0x54 remaining, +0x230 S, +0x2d0 local_98, +0x2d8 OptimalityTol, +0x250 n+m.
const CONFIG = __CONFIG__;
Process.attachModuleObserver({onAdded(mod) {
  if (mod.name !== 'libgurobi130.so') return;
  const b = mod.base, BODY0 = b.add(0x1bc5d0), BODY1 = b.add(0x1bc5d0 + 15006);
  const hex = (p, size) => Array.from(new Uint8Array(p.readByteArray(size))).map(x => x.toString(16).padStart(2, '0')).join('');
  function arr(p, n, size) {
    if (n < 0 || n > 2000000) throw Error('배열 길이 오류: ' + n);
    if (n === 0) return [];
    if (p.isNull()) throw Error('필수 배열 null');
    if (size === 4) return Array.from({length:n}, (_, i) => p.add(i*4).readS32());
    const raw = new Uint8Array(p.readByteArray(n*size)), out = new Array(n);
    for (let i = 0; i < n; i++) { let s = ''; for (let k = 0; k < size; k++) s += raw[i*size+k].toString(16).padStart(2, '0'); out[i] = s; }
    return out;
  }
  const vec = (s, off, n, size=8) => arr(s.add(off).readPointer(), n, size);
  let xo = null, seq = 0, cl = null, i325 = null;
  function emit(event, fields) { send({event, seq: seq++, ...fields}); }
  function dims(S) { return {m: S.add(0x64).readS32(), n: S.add(0x68).readS32()}; }
  Interceptor.attach(BODY0, {
    onEnter(a) { if (xo) throw Error('13f716 중첩'); xo = {model: a[0], mode: a[1].toInt32()};
      const env = a[0].add(0x100).readPointer();
      emit('xo:enter', {mode: xo.mode, crossover: env.add(0x2ba0).readS32(), opttol: hex(env.add(0x2470), 8), feastol: hex(env.add(0x2460), 8)}); },
    onLeave(r) { emit('xo:leave', {ret: r.toInt32(), model_status: xo.model.add(100).readS32()}); xo = null; }
  });
  // push 반환: primal 134a87 (rdi S, esi status, edx attempt, rcx x, r8 slack, r9 first, [6]&cond, [7]&retry, [8]&remaining)
  //            dual   13b3d1 (rdi S, esi status, edx attempt, rcx, r8, r9, [6], [7] first, [8]&cond, [9]&retry)
  for (const [kind, off] of [['primal', 0x1c0070], ['dual', 0x1c6440]]) Interceptor.attach(b.add(off), {
    onEnter(a) { if (!xo) return; this.k = kind; this.S = a[0]; this.att = a[2].toInt32();
      if (kind === 'primal') { this.first = a[5].toInt32(); this.retry = a[7]; this.rem = a[8]; }
      else { this.first = a[7].toInt32(); this.retry = a[9]; this.rem = null; } },
    onLeave(r) { if (!this.k) return;
      emit('push:leave', {kind: this.k, attempt: this.att, first: this.first, ret: r.toInt32(), status: this.S.add(0xf0).readS32(),
        retry: this.retry.readS32(), remaining: this.rem ? this.rem.readS32() : null}); }
  });
  function frame(c) { const sp = c.rsp, S = sp.add(0x230).readPointer(); return {sp, S}; }
  Interceptor.attach(b.add(0x1bf2ee), function () {
    const {sp, S} = frame(this.context), {m, n} = dims(S), nm = sp.add(0x250).readS32();
    if (nm !== n + m) throw Error('n+m slot 불일치');
    const env = xo.model.add(0x100).readPointer();
    emit('sel:enter', {m, n, retry: sp.add(0x50).readS32(), remaining: sp.add(0x54).readS32(), local_98: sp.add(0x2d0).readS32(),
      opttol_slot: hex(sp.add(0x2d8), 8), crossover: env.add(0x2ba0).readS32(), env_2340: env.add(0x2340).readS32(),
      s2d8: hex(S.add(0x2d8), 8), s120: hex(S.add(0x120), 8), precision: S.add(0x28).readS32(), status: S.add(0xf0).readS32(),
      s184: S.add(0x184).readS32(), s178: S.add(0x178).readS32(), s180: S.add(0x180).readS32(),
      tol: Object.fromEntries([0x370, 0x380].map(o => [o.toString(16), hex(S.add(o), 8)])),
      lb: vec(S, 0xa8, n), ub: vec(S, 0xb0, n), x: arr(this.context.rbx, n, 8), vstat: vec(S, 0x168, n + m, 4)});
  });
  Interceptor.attach(b.add(0x1bf6c2), function () {
    const {sp, S} = frame(this.context), {m, n} = dims(S), env = xo.model.add(0x100).readPointer();
    emit('sel:reset', {retry: sp.add(0x50).readS32(), remaining: sp.add(0x54).readS32(), s184: S.add(0x184).readS32(),
      env_24a0: hex(env.add(0x24a0), 8), vstat: vec(S, 0x168, n + m, 4)});
  });
  Interceptor.attach(b.add(0x1bfae2), function () {
    const {sp, S} = frame(this.context), {m, n} = dims(S);
    emit('sel:dual', {d: vec(S, 0x1e0, n + m), vstat: vec(S, 0x168, n + m, 4)});
  });
  Interceptor.attach(b.add(0x1bf6f2), function () {
    const {sp, S} = frame(this.context);
    emit('sel:decide', {r14: this.context.r14.toInt32(), local_98: sp.add(0x2d0).readS32(), s178: S.add(0x178).readS32()});
  });
  Interceptor.attach(b.add(0x153cc0), {
    onEnter(a) {
      const ra = this.returnAddress;
      if (!xo || ra.compare(BODY0) < 0 || ra.compare(BODY1) >= 0) return;
      if (cl) throw Error('cda5b 중첩');
      const model = a[0], S = model.add(0xe0).readPointer();
      const r = {ret_site: ra.sub(b).toInt32(), alg: a[1].toInt32(), vb_null: a[2].isNull(), cb_null: a[3].isNull(), x_null: a[4].isNull(), sl_null: a[5].isNull(),
        p7_null: a[6].isNull(), p8_null: a[7].isNull(), S_null: S.isNull()};
      if (!S.isNull()) { const {m, n} = dims(S), vs = S.add(0x168).readPointer();
        Object.assign(r, {m, n, status: S.add(0xf0).readS32(), s180: S.add(0x180).readS32(), s178: S.add(0x178).readS32(), s4d8: S.add(0x4d8).readS32(),
          vb_is_vstat: a[2].equals(vs), cb_is_vstat_n: a[3].equals(vs.add(4*n)),
          sl_is_x_n: a[4].isNull() ? null : a[5].equals(a[4].add(8*n))}); }
      cl = {model, S}; this.on = true; emit('cda5b:enter', r);
    },
    onLeave(rv) { if (!this.on) return; const S = cl.model.add(0xe0).readPointer();
      emit('cda5b:leave', {ret: rv.toInt32(), model_status: cl.model.add(100).readS32(), ws_status: S.isNull() ? null : S.add(0xf0).readS32(),
        i325: !!cl.saw325}); cl = null; }
  });
  Interceptor.attach(b.add(0xd3150), {
    onEnter(a) {
      if (!cl) return;
      if (i325) throw Error('0325b4 중첩');
      const S = a[0], {m, n} = dims(S), model = S.add(0x4b8).readPointer(), D = model.add(0xe8).readPointer(), flag = D.add(0x3c).readS32();
      cl.saw325 = true; this.on = true;
      i325 = {S, x: a[3], sl: a[4], m, n};
      const g = (o) => S.add(o).readPointer().isNull();
      emit('i325b4:enter', {m, n, vb_is_vstat: a[1].equals(S.add(0x168).readPointer()), cb_is_vstat_n: a[2].equals(S.add(0x168).readPointer().add(4*n)),
        x_null: a[3].isNull(), sl_null: a[4].isNull(), s4d8: S.add(0x4d8).readS32(), s28: S.add(0x28).readS32(), colscale_null: g(0xe0), rowscale_null: g(0xe8),
        d_flag3c: flag, d_flags: flag === 1 ? arr(D.add(0x408).readPointer(), n, 1) : null,
        tol: Object.fromEntries([0x370, 0x380, 0x388, 0x3b0].map(o => [o.toString(16), hex(S.add(o), 8)])),
        vb: arr(a[1], n, 4), cb: arr(a[2], m, 4), vstat: vec(S, 0x168, n + m, 4),
        x: a[3].isNull() ? null : arr(a[3], n, 8), slack: a[4].isNull() ? null : arr(a[4], m, 8),
        lb: vec(S, 0xa8, n + m), ub: vec(S, 0xb0, n + m), cost: vec(S, 0x98, n), collen: vec(S, 0x78, n, 4),
        s450: S.add(0x450).readS32()});
    },
    onLeave(rv) {
      if (!this.on) return; const {S, x, sl, m, n} = i325, cnt = S.add(0x450).readS32();
      const p440 = S.add(0x440).readPointer(), p448 = S.add(0x448).readPointer();
      emit('i325b4:leave', {ret: rv.toInt32(), vstat: vec(S, 0x168, n + m, 4), x: x.isNull() ? null : arr(x, n, 8), slack: sl.isNull() ? null : arr(sl, m, 8),
        s450: cnt, list: cnt > 0 && !p440.isNull() ? arr(p440, cnt, 4) : null, vals: cnt > 0 && !p448.isNull() ? arr(p448, cnt, 8) : null});
      i325 = null;
    }
  });
}});
