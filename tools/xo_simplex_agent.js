// M3-04f-4c: crossover 본체 13f716이 부른 cleanup 0cda5b 안 경로·반복·basis 변화 읽기 전용 관측(쓰기 없음).
//   0x153cc0  0cda5b 진입·반환(반환 주소가 13f716 안일 때만)        → cs:enter / cs:leave
//   내부 호출(cleanup 진행 중에만): 진입·반환 사건(call)과 횟수(count)
// 주소는 ELF file offset = 모듈 base 기준.
const CONFIG = __CONFIG__;
Process.attachModuleObserver({onAdded(mod) {
  if (mod.name !== 'libgurobi130.so') return;
  const b = mod.base, BODY0 = b.add(0x1bc5d0), BODY1 = b.add(0x1bc5d0 + 15006);
  const hex = (p, size) => Array.from(new Uint8Array(p.readByteArray(size))).map(x => x.toString(16).padStart(2, '0')).join('');
  const ints = (p, n) => { if (n <= 0) return []; if (p.isNull()) throw Error('필수 배열 null'); return Array.from({length: n}, (_, i) => p.add(i * 4).readS32()); };
  let seq = 0, xo = null, cl = null;
  function emit(event, fields) { send({event, seq: seq++, ...fields}); }
  function dims(S) { return {m: S.add(0x64).readS32(), n: S.add(0x68).readS32()}; }
  function spx(model) { return model.add(0xe0).readPointer(); }
  function state(model) {
    const S = spx(model), env = model.add(0x100).readPointer(), D = model.add(0xe8).readPointer();
    const r = {model_status: model.add(100).readS32(), env_2454: env.add(0x2454).readS32(), env_2438: env.add(0x2438).readS32(),
      d324: D.add(0x324).readS32(), S_null: S.isNull()};
    if (!S.isNull()) Object.assign(r, {status: S.add(0xf0).readS32(), alg_358: S.add(0x358).readS32(), s35c: S.add(0x35c).readS32(),
      it: hex(S.add(0x2a0), 8), it2a8: hex(S.add(0x2a8), 8), s20: hex(S.add(0x20), 8), s180: S.add(0x180).readS32(), s4d8: S.add(0x4d8).readS32(),
      s450: S.add(0x450).readS32(), s294: S.add(0x294).readS32(), s2f8: S.add(0x2f8).readS32(), s2d0: S.add(0x2d0).readS32(), s28: S.add(0x28).readS32(), s3ec: S.add(0x3ec).readS32(), s340: hex(S.add(0x340), 8),
      tol380: hex(S.add(0x380), 8), tol388: hex(S.add(0x388), 8), lu470_null: S.add(0x470).readPointer().isNull(), lu498_null: S.add(0x498).readPointer().isNull()});
    return r;
  }
  function vstat(model) { const S = spx(model); if (S.isNull()) return null; const {m, n} = dims(S); return ints(S.add(0x168).readPointer(), n + m); }
  Interceptor.attach(BODY0, {
    onEnter(a) { if (xo) throw Error('13f716 중첩'); xo = {model: a[0]};
      const env = a[0].add(0x100).readPointer();
      emit('xo:enter', {mode: a[1].toInt32(), crossover: env.add(0x2ba0).readS32()}); },
    onLeave(r) { emit('xo:leave', {ret: r.toInt32(), model_status: xo.model.add(100).readS32()}); xo = null; }
  });
  Interceptor.attach(b.add(0x153cc0), {
    onEnter(a) {
      const ra = this.returnAddress;
      if (cl) { cl.counts['cda5b_nested'] = (cl.counts['cda5b_nested'] || 0) + 1; return; }
      if (!xo || ra.compare(BODY0) < 0 || ra.compare(BODY1) >= 0) return;
      const model = a[0], S = spx(model), {m, n} = dims(S);
      cl = {model, counts: {}, depth: 0};
      this.on = true;
      emit('cs:enter', {ret_site: ra.sub(b).toInt32(), alg: a[1].toInt32(), vb_null: a[2].isNull(), cb_null: a[3].isNull(),
        x_null: a[4].isNull(), sl_null: a[5].isNull(), m, n, ...state(model), vstat: vstat(model)});
    },
    onLeave(rv) {
      if (!this.on) return;
      const model = cl.model;
      emit('cs:leave', {ret: rv.toInt32(), counts: cl.counts, ...state(model), vstat: vstat(model)});
      cl = null;
    }
  });
  // 진입·반환 사건을 남기는 내부 호출
  const CALLS = {'02f7e1': 0xd06f0, '0325b4': 0xd3150, '194efc': 0x1fcd50, '12e6ec': 0x1bbb20, '12ed7c': 0x1bbd80, '12e738': 0x1bbb40,
    '12e790': 0x1bbb60, '0478d3': 0xe3db0, '06268b': 0xf6a00, '02e9ae': 0xcff40, '02e315': 0xcfbc0, '12f282': 0x1bbf80, '02d1de': 0xcec20,
    '02d3b2': 0xcedb0, '02de43': 0xcf8d0, '12ddac': 0x1bb730, '0af361': 0x137ce0, '229999': 0x1387d0, '0b16d3': 0x1415d0, '07373e': 0x105f20,
    '06899a': 0xf94d0, '0b44ce': 0x142d70, '0b3a19': 0x145520, '0b2cdf': 0x142350, '0b2fc2': 0x142500, '048d5f': 0xe45c0, '028c93': 0xc4fe0};
  const LIMIT = 40;  // 함수별 cleanup당 사건 상한(넘으면 횟수만 셈)
  for (const [name, off] of Object.entries(CALLS)) Interceptor.attach(b.add(off), {
    onEnter(a) { if (!cl) return; const k = 'ev_' + name; cl.counts[k] = (cl.counts[k] || 0) + 1; if (cl.counts[k] > LIMIT) return; this.on = true; this.d = cl.depth++;
      emit('call', {fn: name, ph: 'enter', depth: this.d, ra: this.returnAddress.sub(b).toInt32(), ...state(cl.model)}); },
    onLeave(rv) { if (!this.on) return; cl.depth--;
      emit('call', {fn: name, ph: 'leave', depth: this.d, ret: rv.toInt32(), ...state(cl.model)}); }
  });
  // 횟수만 세는 내부 호출(반복마다 많이 불림)
  const COUNTS = {'036ade': 0xd4190, '05b257': 0xf15c0, '12dd49': 0x1bb6f0, '08b129': 0x11f430};
  for (const [name, off] of Object.entries(COUNTS)) Interceptor.attach(b.add(off), {
    onEnter() { if (!cl) return; cl.counts[name] = (cl.counts[name] || 0) + 1; }
  });
}});
