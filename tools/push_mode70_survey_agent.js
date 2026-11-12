// M3-04c-21j 조사(21i survey agent 사본 + local_70 풀 검사·풀 부족·반환≠0): 036ade 조밀 전환 경로의 자연 발생을 센다(읽기 전용, 판정 근거가 아닌 탐색).
// push(134a87@0x1c0070) 활성 여부, 036ade(0xd4190) 호출마다 반환 주소·m, 그리고 함수 중간 함수형 probe:
//   0xd85b4 local_70=1(조밀 배열 Markowitz 모드), 0xd88bf 조밀 LU 진입(ecx 직전 0x100(%rsp)=남은 n),
//   0xd8a7d 조밀 배열 채우기 루프 머리(재시도 포함), 0xdd1b2 자체 조밀 LU(n<1001 또는 n²<=m), 0xde68c LU 뒤 변환.
// DGETRF(0x9fef00)는 함수 진입 훅으로 036ade 안 호출만 센다. probe 지점은 asm에서 명령 경계·창 안 착지점 0을 확인했다.
Process.attachModuleObserver({onAdded(mod) {
  if (mod.name !== 'libgurobi130.so') return;
  const b = mod.base;
  let push = 0, inPush = false, cur = null, n036 = 0;
  Interceptor.attach(b.add(0x1c0070), {onEnter() {inPush = true; push++;}, onLeave() {inPush = false;}});
  Interceptor.attach(b.add(0xd4190), {
    onEnter(a) {cur = {id: ++n036, push: inPush ? push : null, ret: this.returnAddress.sub(b).toUInt32(), m: a[2].toInt32(), marks: []};},
    onLeave(r) {if (cur.marks.length || r.toInt32() !== 0) send({event: 'lu:dense', ...cur, rv: r.toInt32()}); cur = null;}
  });
  const mark = (off, name, fn) => Interceptor.attach(b.add(off), function () {
    if (cur) cur.marks.push(fn ? [name, fn(this.context)] : [name]);
  });
  mark(0xd85b4, 'mode70');
  // M3-04c-21j: local_70 풀 검사 입력(0xd85ee: n=0x100, local_60=0x2d8, local_f0=0x248, local_1b8=0x180, local_f8=0x240,
  // local_1c0=0x178, param_15=r14: [3]=+0x18, [2]=+0x10, m=rbx)와 풀 부족 경로 머리(0xd881b, 창 안 착지점 0).
  mark(0xd85ee, 'pool70', c => {const s = c.rsp, P = c.r14; return [s.add(0x100).readS32(), s.add(0x2d8).readS32(), s.add(0x248).readS32(),
    s.add(0x180).readS64().toNumber(), P.add(0x18).readS64().toNumber(), s.add(0x240).readS64().toNumber(), s.add(0x178).readS64().toNumber(),
    P.add(0x10).readS64().toNumber(), c.rbx.toInt32()];});
  mark(0xd881b, 'short70');
  mark(0xd88bf, 'dense', c => c.rsp.add(0x100).readS32());
  mark(0xd8a7d, 'fill', c => c.rdx.toInt32());
  mark(0xdd1b2, 'own', c => c.rcx.toInt32());
  mark(0xde68c, 'done');
  Interceptor.attach(b.add(0x9fef00), {onEnter(a) {if (cur) cur.marks.push(['dgetrf', a[0].readS32()]);}});
}});
