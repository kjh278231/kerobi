# M2 `3cd406` 해독 B: 패스 루프 (디컴파일 L1696–4400)

MIP presolve 드라이버 `3cd406`의 라운드·패스·내부 루프 부분이다. 앞(L731–1696 prelude), 뒤(L4400–6450 라운드 끝, L6582–9600 마지막 단계)는 다른 노트가 맡는다. 근거는 디컴파일(`re/decomp/c/003b0530_PRIVATE00000000003cd406.c`)과, 디컴파일이 인자·분기를 놓친 곳은 `objdump`(주소는 파일 오프셋 = text vaddr)다. 런타임 근거는 기본 파라미터 38개 모델의 호출 순서(`seq/*.txt`), 호출 지점 coverage(`seq/callsites.tsv`), PRELIMIT 스윕(`sweep/<model>/sweep.tsv`)이다. 이 노트의 숫자 상수는 모두 `.rodata`에서 직접 읽었다.

표기: `st` = presolve 상태(Ghidra `local_150`, rsp+0xb48), `env` = Ghidra `lVar119`(레지스터 rbp), `w` = 작업량 카운터 포인터(`param_5`, rsp+0xa80. `3a484d`의 4번째 인자로 들어가 `st+0x458`에도 같은 포인터가 들어 있다), `tr` = 추적기 `st+0x3d8`, `μ` = `st+0x448`(작업 배수). 파라미터 이름은 `docs/m2_param_offsets.tsv`를 따른다(그 표에 없는 것: env+0x2458 INFBOUND, env+0x2460 FeasibilityTol, env+0x2640 PreSOS1BigM, env+0x2648 PreSOS2BigM, env+0x26c8 GURO_PAR_AGEQ(기본 −1), env+0x2728 sub-solve 깊이(최상위 0)).

## 1. 개요

### 1.1 범위와 주소

| 부분 | 디컴파일 | 주소(파일 오프셋) |
|---|---|---|
| 루프 전 블록(PREBOUND·QP·QC·NL) | L1696–1729 | 0x3b196d–0x3b1c4c |
| 루프 머리(라운드/패스 진입) | L1730–1740 | 0x3b1c4c–0x3b1cee |
| 내부 루프 | L1741–2028 | 0x3b1cee–0x3b23cb, I→B 본체 0x3be1ce–0x3be238 |
| 내부 루프 출구, 실패·마무리 발판 | L2029–2035 | 0x3b42d5–0x3b4384 |
| PWL 분기(`9ee0cf`) | L4225–4380 | 0x3b4384–0x3b4795 |
| 패스 머리 + 바깥 reduction | L2038–3943 | 0x3b4795–0x3b7b7c |
| 바깥 쪽 떨어진 블록 | L2176–2530, 3091–3610 | PREPARTOBJ 0x3bbaf0–0x3bbf43·0x3bdf17–0x3be1ce, 발판 0x3bbf43–0x3bc290, 0x3bf083 |
| 예외 분기 묶음(PRECOEFFICIENT·QP·QC·NL·SOS) | L3944–4224 | (위 주소에 섞여 있음) |

### 1.2 진입과 출구 (hand-off)

- **진입**: `LAB_004b196d`(0x3b196d). prelude가 0x3b185d, 0x3b1872, 0x3b1881, 0x3b1896, 0x3b18d6, 0x3be2bf에서 뛰어 들어온다(SOLREDUCTION 블록 뒤). 이때 카운터는 `local_68`(rsp+0xc30)이고, 기본값에서는 **2**다(0x3b0e6c의 정수 bound 반올림 뒤 1, `36d89a` 뒤 2. prelude 노트에서 확인할 것). 패스 번호 `local_40`(rsp+0xc58)은 함수 시작에서 0, 패스 상한 `local_928`(rsp+0x370)은 PrePasses(−1이면 Presolve==2 → 2e9, Presolve==1 → 20, 그 밖 100). 타이머 28개의 한도도 prelude가 정한다(§4.2).
- **마무리로**: `LAB_004b2415`(0x3b2415). 루프 안에서는 0x3b42f2(내부 루프)·0x3bbf98(바깥)을 거쳐 0x3b23f5로 가며, 카운터를 `local_68`에 쓰고 r13d에 싣는다.
- **오류로**: `LAB_004b2dee`(0x3b2dee), r15d = 상태 코드. 내부 루프는 0x3b42d5, 바깥은 0x3bbf43(callee 반환값)·0x3bbf70(`3ba5ac`/`3ba9de` 반환값).
- **라운드 끝으로**: `LAB_004b7b7c`(0x3b7b7c). 세 갈래: 0x3b7b4d(대부분), 0x3bbfde(빈 모델, 특수 블록·`3ff744` mode 4/5 뒤 재시작 신호), 0x3be238(라운드 시작에서 이미 p ≥ 패스 상한). 넘기는 값: rdx = st, **ebx = 카운터(`local_68`에는 쓰지 않는다. 라운드 끝 코드도 ebx로 센다, 0x3b7d69)**, r12d = 패스 번호 p(패스 상한으로 나갈 때만 +1된 값), r13b = 라운드 번호(`local_9a8`), r15b = 단계 번호(`local_9a0`), ebp = probeMode(`local_58`), r14 = w, `local_104`(rsp+0xb94) = 재시작 플래그, `local_1e8`(rsp+0xab0) = 마지막 내부 반복 시작 스냅숏, `local_198`(rsp+0xb00) = 마지막 패스의 바깥 시작 스냅숏.
- **라운드 끝에서 돌아오기**: 0x3b9021 → `LAB_004b1cb4`(라운드 +1, r13b < 5), 0x3b9088 → `LAB_004b1ca9`(단계 +1, r15b < 2, r13b = 0, r12d = `local_40`). 둘 다 r12d(p)를 0으로 되돌리지 않는다.

### 1.3 루프 겹과 변수

```
단계 phase (r15b, local_9a0)        0..1   ← 라운드 끝 코드가 올림(0x3b9081)
 └ 라운드 round (r13b, local_9a8)   0..4   ← 라운드 끝 코드가 올림(0x3b901a)
    └ 패스 p (r12d, local_40)       전역. 라운드·단계가 바뀌어도 0으로 돌아가지 않는다. p < local_928
       k (local_9b0)                 이 라운드에서 끝낸 패스 수. 라운드 시작에서 0
       └ 내부 반복 i (r13d)          −5, −4, −3, …  (Ghidra iVar129)
```

| 이름 | Ghidra | 위치 | 뜻 |
|---|---|---|---|
| cnt | `iVar116`(루프 안), `local_68`(밖) | ebx, rsp+0xc30 | PRELIMIT 카운터. 루프 안에서는 ebx만 맞다. `local_68`은 낡은 값 |
| p | `iVar115`/`local_40` | r12d, rsp+0xc58 | 패스 번호(전역) |
| k | `local_9b0` | rsp+0x2e8 | 라운드 안 패스 번호 |
| i | `iVar129` | r13d | 내부 반복 번호, −5부터 |
| round | `cVar128`/`local_9a8` | r13b, rsp+0x2f0 | 라운드 번호(단계 안) |
| phase | `cVar125`/`local_9a0` | r15b, rsp+0x2f8 | 단계 번호 |
| snapIt | `local_1e8` | rsp+0xab0 | 내부 반복마다 뜨는 스냅숏 |
| snapPass | `local_198` | rsp+0xb00 | 바깥 reduction 시작 스냅숏. 이 영역 밖에서는 쓰지 않는다 |
| restart | `local_104` | rsp+0xb94 | `3ba9de`가 쓰는 PRERESTART 플래그 |
| passLimit | `local_928` | rsp+0x370 | 패스 상한 |
| mipctx | `local_78` | rsp+0xc20 | 3번째 인자. `Model.presolve()` 경로에서 0. 그래서 `local_a0 = local_a8 = local_70 = 0`(L6148–6153에서만 바뀜) |
| gc | `local_950` | rsp+0x348 | `36c5cf`의 5번째(출력) 인자 |
| T0…T27 | `local_918`…`local_258` | rsp+0x380+0x40·n | 타이머 n. `local_X` ↔ rsp+(0xc98−X) |

## 2. 의사코드 (기본 파라미터 경로)

매크로(주소는 대표 예):

```rust
// POLL: 3ba5ac(env, st, …) = 로그·콜백(where=PRESOLVE). 0이 아니면 그 값으로 오류 종료(LAB_004b2dee).
// BUDGET0: 3ba9de(env, st, null). 시간·WorkLimit(a83b2f)이면 오류 종료.
//          (*w − st+0x450) > st+0x1f0 이면 st+0x1dc(PRELIMIT 사본) = −1 → 다음 COUNT에서 마무리.
// BUDGETR: 3ba9de(env, st, &restart) 뒤 restart != 0 이면 goto ROUND_END.
//          restart = PRERESTART≠0 && st+0x310==0 && 6a0643(env)
//                    && (st+0x408 > 0.8·st+0xc + 1e4 || (st+0x404+0x410+0x418+0x424+0x428) > 0.8·(st+0x10+0x1f8+0x27c+0x240+0x2f8) + 1e4)
// COUNT:   cnt += 1; if cnt >= st+0x1dc { goto FINISH }            // FINISH = LAB_004b2415, 부호 있는 비교
// RUN(Tn, f): ade8ca(Tn, w); rc = f; ade770(Tn, w); if rc != 0 { return rc }
// OK(Tn):  adf236(Tn, w) < 0.8
```

```rust
// ── 루프 전 (LAB_004b196d). cnt = 2 (기본값)
if PREBOUND != 0 && DualReductions != 0 {                 // 기본값 참, 38/38
    rc = 5a9282(st, env); if rc != 0 { return rc }        // 타이머 없음
    COUNT; POLL;
}
BUDGET0;
if st+0x250 != 0 && DualReductions != 0 { 42872b(env, st); COUNT; POLL }            // (기본값 거짓) QP 목적
if st+0x250 != 0 && DualReductions != 0 && Presolve != 0 { 429e1b(env, st); COUNT; POLL } // (기본값 거짓)
if st+0x27c != 0 { 451195(env, st, 0, 0, 0, local_70); COUNT; POLL }               // (기본값 거짓) QC
if st+0x2f8 != 0 || st+0x310 != 0 { 5b1161(env, st, local_a0, local_a8, 0); COUNT; POLL } // (기본값 거짓) NL
BUDGET0;
local_9b8 = max(2, probeMode);           // 라운드 끝에서 씀
phase = 0;
'phase: loop {                                           // LAB_004b1ca9
  round = 0;  p = local_40;
  'round: loop {                                         // LAB_004b1cb4
    if p >= passLimit { goto ROUND_END }                 // 0x3be238
    k = 0;
    'pass: loop {                                        // LAB_004b1cee
      local_40 = p;
      // ── 내부 루프
      i = -5;
      loop {
        3c546c(st, snapIt);
        536971(tr, 2); 5369b8(tr);
        POLL; BUDGET0; POLL;
        RUN(T0, 3e39a6(st, env));                 COUNT; POLL;
        RUN(T0, 36ddf2(st, env));                 COUNT; POLL;
        if PREBOUND != 0 { RUN(T2, 37168e(st, env)); COUNT; POLL; BUDGET0; }
        RUN(T0, 36c5cf(st, env, -1, 0, &gc));     COUNT; POLL;   // gc는 덮어씀
        if gc > 1_000_000 {                       // (38/38에서 안 일어남) i64 비교
            ade8ca(T27); 36e6b6(st, 1); ade770(T27);
            RUN(T0, 36ddf2(st, env)); COUNT; POLL; BUDGET0;
        }
        i_to_b();                                 COUNT; POLL;   // §5.1. 가드 없이 늘 센다
        if PRECOEFFICIENT != 0 { ade8ca(T3); 56f793(st, env) /*반환값 무시*/; ade770(T3); COUNT; POLL; }
        BUDGET0;
        if Aggregate != 0 {
            RUN(T5, 56c80e(st, env, 0, &s)); if s != 0 { return s }   // 4번째 인자는 출력 상태
            COUNT; POLL;
        }
        ade8ca(T27); 36e6b6(st, 0); ade770(T27);
        rho = 3c5595(st, snapIt); POLL;
        if rho <= 0.001 * (max(0, i) as f64) { break }            // i ≤ 0이면 rho == 0일 때만
        i += 1;
      }
      536971(tr, 1);
      if st+0x240 > 0 {                           // (기본값 거짓) PWL 일반 제약
          9ee0cf(env, model); 예산 B를 다시 구해 T0..T27 한도를 prelude와 같은 규칙으로 다시 씀;
          536971(tr, 0); RUN(T0, 3e39a6); COUNT; POLL; RUN(T0, 36ddf2); COUNT; POLL; 536971(tr, 1);
      }
      if st+0x300 != 0 {                          // (기본값 거짓) NL
          536971(tr, 0); RUN(T0, b6fd6f(env, st, 0, w)); COUNT;
          if st+0x2f8 > 0 { rc = b84382(env, st, w) /*카운트 없음*/ }
          536971(tr, 1);
      }
      // ── 패스 머리
      if st+0xc == 0 || st+0x10 == 0 { goto ROUND_END }     // 열 수 또는 행 수가 0
      BUDGETR;
      if p > 0 {
          rho = 3c5595(st, snapPass); POLL;
          if rho <= 0.001 * (max(0, p - 2) as f64) { goto ROUND_END }
      }
      3c546c(st, snapPass);
      5369b8(tr);                                  // 레벨 1에서 reset
      // 특수 블록 (모두 기본값 거짓, 38/38에서 호출 0)
      if st+0x1f8 - st+0x410 > 0 { 586049(env, st, INFBOUND); COUNT; POLL; BUDGETR }   // SOS
      if st+0x228 > 0            { 585a6e(env, st, INFBOUND); COUNT; POLL; BUDGETR }
      if st+0x330 > 0            { 58b41b(st, env);           COUNT; POLL; BUDGETR }   // PWL 열
      if st+0x250 != 0 && DualReductions { 42872b(env, st);   COUNT; POLL; BUDGETR }
      if st+0x27c != 0 { 451195(env, st, 0, 0, 1, local_70); COUNT; POLL;
                         if PREBOUND { RUN(T2, 37168e); COUNT; POLL }; BUDGETR }
      if st+0x2f8 != 0 || st+0x310 != 0 { 5b1161(env, st, local_a0, local_a8, 1); COUNT; POLL;
                         if PREBOUND { RUN(T2, 37168e); COUNT; POLL }; BUDGETR }
      if Aggregate && PREQSUBST && (st+0x250 != 0 || st+0x27c != 0) {
          RUN(T5, 56b360(st, env, 1, st+0x10, 0, &q)); COUNT;
          rc = 36d89a(st, env); if rc != 0 || q != 0 { return rc or q }; COUNT; POLL; BUDGETR }
      // ── 바깥 reduction (기본값 경로)
      if PREXORRED != 0 && PreCrush(st+0x1e0) == 0 { RUN(T26, 3e5363(env, st)); COUNT; POLL; }
      if PREPARALLEL != 0 && PreCrush == 0 {
          if DualReductions != 0 && OK(T6) { RUN(T6, 3ff744(st, env, T6, 3, 0, local_70)); COUNT; POLL; }
          BUDGETR;
      }
      if PRECOEFFICIENT != 0 {
          RUN(T3, 57627e(st, env)); COUNT; POLL;
          RUN(T3, 578fd2(st, env)); COUNT; POLL; BUDGETR;
      }
      if PREPARTOBJ != 0 { prepartobj(); COUNT; POLL; BUDGETR; }      // §5.2, 기본 −1이라 참
      if DualReductions != 0 && p < 3 && OK(T4) { RUN(T4, 374521(st, env)); COUNT; POLL; BUDGETR; }
      if PREDUALSLACK != 0 && DualReductions != 0 { RUN(T4, 4178ae(st, env)); COUNT; POLL; BUDGETR; } // (기본값 거짓)
      if PREPARALLEL != 0 {
          if DualReductions != 0 && OK(T6) { RUN(T6, 3ff744(st, env, T6, 1, 1, local_70)); COUNT; POLL; BUDGETR; }
          if OK(T6)                        { RUN(T6, 3ff744(st, env, T6, 0, 1, local_70)); COUNT; POLL; BUDGETR; }
          if st+0x27c > 0 && OK(T6) { RUN(T6, 3ff744(st, env, T6, 4, 0, local_70)); COUNT; POLL; BUDGETR; } // (거짓)
          if st+0x27c > 0 && OK(T6) { RUN(T6, 3ff744(st, env, T6, 5, 0, local_70)); COUNT; POLL; BUDGETR; } // (거짓)
      }
      BUDGETR;
      if round == 0 && k == 0 && env+0x2728 == 0 && PRETRANSRED != 0 {
          RUN(T25, 5431ee(st, env)); COUNT; POLL; BUDGETR;
      }
      if DualReductions != 0 && PREDOMINATE != 0 && OK(T17) { RUN(T17, 4030c9(st, env, T17)); COUNT; POLL; BUDGETR; }
      if PRECANCEL != 0 && OK(T15) {
          m = if PRECANCEL == -1 { 2 + (local_a8 > 0) as i32 } else { PRECANCEL - 1 };   // 기본 2
          RUN(T15, 414c03(st, env, m, T15)); COUNT; POLL; BUDGETR;
      }
      if k == 0 && PRESUBSUME != 0 && OK(T18) { RUN(T18, 416036(st, env, T18)); COUNT; POLL; BUDGETR; } // (거짓, 기본 0)
      if PRECLIQSUB != 0 && OK(T18)           { RUN(T18, 414ca3(st, env, T18)); COUNT; POLL; BUDGETR; } // (거짓, 기본 0)
      ade8ca(T27); 36e6b6(st, 1); ade770(T27);
      RUN(T27, 36ee92(env, st));                   // 압축. 카운트 없음
      BUDGETR;
      if DualReductions != 0 { ade8ca(T4); 3e40d1(st, env) /*반환값 무시*/; ade770(T4); COUNT; POLL; BUDGETR; }
      if Aggregate != 0 && OK(T1) {
          if PREBOUND != 0 { RUN(T2, 37168e(st, env)); COUNT; BUDGET0; }   // POLL 없음
          RUN(T1, 37f484(st, env)); COUNT; POLL;
      }
      ade8ca(T27); 370047(st, env, 0, 0); ade770(T27);   // activity 재계산, 늘 돈다, 카운트 없음
      BUDGETR;
      if PRECANCELCOL != 0 && OK(T16) {
          m = if PRECANCELCOL == -1 { 2 + (local_a8 > 0) as i32 } else { PRECANCELCOL - 1 };
          RUN(T16, 414c53(st, env, m, T16)); COUNT; POLL; BUDGETR;
      }
      if AGEQ >= 1 && env+0x2728 == 0 && Aggregate != 0 && OK(T1) {      // (기본값 거짓, AGEQ −1)
          RUN(T1, 37c34c(st, env, &ag)); POLL; local_98 = (local_98 != 0 || ag != 0) as u32;
          ade8ca(T27); 370047(st, env, 0, 0); ade770(T27); BUDGETR;        // 카운트 없음
      }
      if PREBOUND != 0 && DualReductions != 0 { implied_bounds(); COUNT; POLL; BUDGETR; }   // §5.3, T4
      if Aggregate != 0 && DualReductions != 0 {
          RUN(T4, 418542(st, env)); COUNT; POLL; BUDGETR;
          if st+0x1e4 != 0 { dual_conv_bin(); COUNT; POLL; BUDGETR; }     // §5.4, T4. st+0x1e4 = (OptimalityTarget != 1)
      }
      if PRECLIQUE != 0 {
          if OK(T14) { RUN(T14, 45885d(st, env, T14)); COUNT; POLL; BUDGETR; }
          if OK(T11) { RUN(T11, 52be10(st, env, T11)); COUNT; POLL; BUDGETR; }
          if OK(T12) {
              m = if mipctx != 0 || Presolve > 1 || PRECLIQUE > 1 { 2 } else { 1 };   // 기본 1
              RUN(T12, 46a62d(st, env, T12, m)); COUNT; POLL; BUDGETR;
          }
      }
      BUDGETR;
      if PREMERGEVUBS != 0 && OK(T13) { RUN(T13, 5466a7(env, st, T13)); COUNT; POLL; BUDGETR; }
      if PRETWOROWS != 0 && Presolve != 1 && OK(T7) { RUN(T7, 45feaa(st, env, T7, PRETWOROWS.abs())); COUNT; POLL; BUDGETR; }
      if k == 0 && PreDepRow > 0 && OK(T19) {                              // (기본값 거짓, PreDepRow −1)
          RUN(T19, 47c445(μ, st, 0, env, T19, 1, w)); COUNT; POLL; BUDGETR; }
      if k == 0 && PreDepRow != 0 && DualReductions != 0 && OK(T19) {
          RUN(T19, 47c445(μ, st, 0, env, T19, 2, w)); COUNT; POLL; BUDGETR; }
      // ── 패스 끝
      POLL; COUNT; BUDGETR;                                                // 늘 센다
      rho = 3c5595(st, snapIt); POLL;                                      // snapIt = 마지막 내부 반복 시작
      if rho <= 1e-4 { goto ROUND_END }
      p += 1; k += 1;
      if p >= passLimit { goto ROUND_END }
    }
  }
}
// ROUND_END = LAB_004b7b7c (다른 노트). 거기서 'round 또는 'phase 머리로 돌아온다.
```

메모:
- `47c445`의 PreDepRow는 env 값을 그대로 읽는다(LP `390989`처럼 자동값으로 바꾸지 않는다). 기본 −1이라 mode 1은 꺼지고 mode 2만 돈다.
- 374521의 `p < 3`은 전역 패스 번호다. 5431ee의 `round == 0`은 단계마다 0으로 돌아가는 라운드 번호라 단계 1의 첫 패스에서 다시 돈다(glass4 §7).
- `3ff744`의 6번째 인자(`local_70`), `414c03`/`414c53`의 4번째(타이머), `4030c9`·`416036`·`414ca3`·`45885d`·`52be10`·`46a62d`·`5466a7`·`45feaa`의 타이머 인자, `46a62d`·`45feaa`의 mode, `47c445`의 r9(=w), `36c5cf`의 5번째, `56c80e`의 4번째, `3ba9de`의 3번째 인자는 디컴파일이 빠뜨린 것을 asm으로 채웠다.

## 3. PRELIMIT 카운트 지점 (프로그램 순서)

모두 `inc %ebx; cmp 0x1dc(st),%ebx; jge → LAB_004b2415`(루프 전 5개는 `local_68`을 직접 올림). 카운트는 callee가 0을 돌려준 뒤에 올리고, 그다음 한도를 비교한다. 그래서 `GURO_PAR_PRELIMIT=k`의 오라클은 k번째 카운트 직후 상태에서 마무리한 모델이다.

| # | 주소(inc) | 무엇 뒤 | 가드 | 비고 |
|---|---|---|---|---|
| a | 0x3b19ae | `5a9282` | PREBOUND && DualReductions | 기본값 cnt = 3 |
| b | 0x3b1a59 / 0x3b1adf / 0x3b1b61 / 0x3b1bef | `42872b` / `429e1b` / `451195` / `5b1161` | QP·QC·NL | 기본값 거짓 |
| 1 | 0x3b1df9 | `3e39a6` | – | 내부 반복마다 |
| 2 | 0x3b1e69 | `36ddf2` | – | |
| 3 | 0x3b1ee6 | `37168e` | PREBOUND | |
| 4 | 0x3b1f82 | `36c5cf` | – | |
| 5 | 0x3b2036 | `36ddf2`(GC 뒤) | gc > 1e6 | 38/38에서 안 일어남 |
| 6 | 0x3b21dc | I→B 인라인 | **가드 없음** | 바꾼 열이 없어도 센다 |
| 7 | 0x3b2249 | `56f793` | PRECOEFFICIENT | 반환값을 보지 않고 센다 |
| 8 | 0x3b22ff | `56c80e` | Aggregate | 출력 상태 s == 0일 때 |
| c | 0x3b46b3, 0x3b473f | PWL 분기 `3e39a6`, `36ddf2` | st+0x240 > 0 | 거짓 |
| d | 0x3b4811 | `b6fd6f` | st+0x300 | 거짓. `b84382`는 안 센다 |
| e | 0x3b49a9, 0x3b4a5a, 0x3b4b03, 0x3b4bba, 0x3b4c75(+0x3b4d0e), 0x3b4dd5(+0x3b4e6e), 0x3b4f74+0x3b4fc0 | SOS·PWL열·QP·QC(+37168e)·NL(+37168e)·`56b360`+`36d89a` | 특수 구조 | 거짓. 마지막은 2번 센다 |
| 9 | 0x3b50a5 | `3e5363` | PREXORRED && PreCrush==0 | 타이머 판정 없음 |
| 10 | 0x3b51a0 | `3ff744(3,0)` | PREPARALLEL && PreCrush==0 && DualRed && T6 | |
| 11 | 0x3b5274 | `57627e` | PRECOEFFICIENT | |
| 12 | 0x3b5300 | `578fd2` | PRECOEFFICIENT | |
| 13 | 0x3bbeda | PREPARTOBJ 인라인 | PREPARTOBJ ≠ 0 | **아무것도 안 바꿔도 센다** |
| 14 | 0x3b5411 | `374521` | DualRed && p < 3 && T4 | |
| 15 | 0x3b54ee | `4178ae` | PREDUALSLACK && DualRed | 거짓 |
| 16 | 0x3b560b | `3ff744(1,1)` | PREPARALLEL && DualRed && T6 | |
| 17 | 0x3b570e | `3ff744(0,1)` | PREPARALLEL && T6 | |
| 18 | 0x3b582f, 0x3b5947 | `3ff744(4,0)`, `(5,0)` | QC | 거짓 |
| 19 | 0x3b5a89 | `5431ee` | round==0 && k==0 && env+0x2728==0 && PRETRANSRED | |
| 20 | 0x3b5b97 | `4030c9` | DualRed && PREDOMINATE && T17 | |
| 21 | 0x3b5cb0 | `414c03` | PRECANCEL && T15 | |
| 22 | 0x3b5dbf, 0x3b5ec0 | `416036`, `414ca3` | PRESUBSUME/PRECLIQSUB | 거짓 |
| 23 | 0x3b603d | `3e40d1` | DualRed | 반환값을 보지 않고 센다 |
| 24 | 0x3b6144 | `37168e` | Aggregate && T1 && PREBOUND | |
| 25 | 0x3b61c8 | `37f484` | Aggregate && T1 | |
| 26 | 0x3b631f | `414c53` | PRECANCELCOL && T16 | |
| – | – | `37c34c`(AGEQ) | AGEQ ≥ 1 … | **카운트 없음**(거짓) |
| 27 | 0x3b6807 | 함의 bound 인라인 | PREBOUND && DualRed | **안 바꿔도 센다** |
| 28 | 0x3b68e4 | `418542` | Aggregate && DualRed | |
| 29 | 0x3b7246 | 쌍대 이진화 인라인 | Aggregate && DualRed && st+0x1e4 | **이진 열만 있어 본문을 건너뛰어도 센다**(0x3b6a9b → 0x3b723e) |
| 30 | 0x3b7347 | `45885d` | PRECLIQUE && T14 | |
| 31 | 0x3b743b | `52be10` | PRECLIQUE && T11 | |
| 32 | 0x3b7547 | `46a62d` | PRECLIQUE && T12 | |
| 33 | 0x3b767f | `5466a7` | PREMERGEVUBS && T13 | |
| 34 | 0x3b779a | `45feaa` | PRETWOROWS && Presolve≠1 && T7 | |
| 35 | 0x3b78c1 | `47c445(1)` | k==0 && PreDepRow>0 && T19 | 거짓 |
| 36 | 0x3b79f5 | `47c445(2)` | k==0 && PreDepRow≠0 && DualRed && T19 | |
| 37 | 0x3b7a98 | 패스 끝 | **가드 없음** | reduction 없이 센다 |

카운트가 없는 호출: `36e6b6`, `36ee92`, `370047`, `b84382`, `37c34c`, `3c546c`/`3c5595`, 추적기 함수.

기본값 카운트 수(타이머가 모두 허락할 때):
- 내부 반복 하나 = **7**(1, 2, 3, 4, 6, 7, 8). gc > 1e6이면 8.
- 바깥 패스 = **22 + [p < 3] + [round == 0 && k == 0] + [k == 0]**(9–14, 16, 17, 20, 21, 23–34, 36, 37 중 해당분). 라운드 0 첫 패스는 25, 같은 라운드 둘째 패스(p=1)는 23.

주의:
- `BUDGET0`/`BUDGETR`가 예산 초과로 st+0x1dc를 −1로 바꾸면, 효과는 **다음 COUNT**에서 나타난다. 그 사이의 reduction 하나는 돈다(예: 반복 시작의 `3ba9de` 뒤 `3e39a6`은 돌고 나서 마무리로 간다).
- `adf236 ≥ 0.8`로 건너뛴 reduction은 세지 않는다.

## 4. 작업량과 타이머

### 4.1 타이머 함수 (asm·디컴파일 확인)

- `ade8ca(T, w)`: (T+0x2c면 벽시계) T.running = 1, **T.acc = (T.acc − *w) + 1.0**. 시작마다 1을 더한다.
- `ade770(T, w)`: T.running = 0, T.acc = T.acc + *w.
- `adf236(T, w)`: limit = T+0x10. limit < 0 → 1.0, limit == 0 → 0.0, 아니면 (T.acc + (running ? *w : 0)) / limit. 호출부는 `0.8 > 값`일 때만 돈다(`comisd`, NaN이면 건너뜀).
- `adf1a6(v, T)`: T.limit = v.

### 4.2 한도 (prelude 0x3b1445–0x3b15c7, PWL 분기 0x3b44ae–0x3b4646이 같은 규칙으로 다시 씀)

B = X + 1e7. base = st+0xd8(시작 nnz) + Q·SOS·QC·일반 제약 크기(prelude L1524–1553). X = 1e100(Presolve==2 또는 PrePasses>1 또는 PREPROBE>1 또는 PreSparsify>0), 아니면 mipctx==0일 때 base·500(Presolve==1) 또는 base·2500, mipctx≠0이면 `5e8e36` 기반 식(prelude 노트). 기본값 선형 MIP: **B = 2500·nnz + 1e7**.

| 한도 | 타이머 |
|---|---|
| B | 전부(28개)에 먼저 B |
| 0.5·B (`local_930` = 0.5) | T17, T15, T16, T18, T14, T12, T7, T20, T23 |
| 2·B | T24 |
| 4·B | T21 |
| 3·B | T8, T9, T10 (st+0xf8 ≠ 0 또는 probeMode ≥ 3일 때만) |

### 4.3 reduction별 타이머 (이 영역)

| 타이머 | Ghidra | 한도 | 쓰는 곳 | 0.8 판정 |
|---|---|---|---|---|
| T0 | local_918 | B | `3e39a6`, `36ddf2`, `36c5cf`, `b6fd6f`, PWL 분기 | 없음 |
| T1 | local_8d8 | B | 집계 블록(가드), `37f484`, `37c34c` | 있음(블록 가드) |
| T2 | local_898 | B | `37168e`(내부·집계 블록·QC·NL) | 없음 |
| T3 | local_858 | B | `56f793`, `57627e`, `578fd2`, PREPARTOBJ | 없음 |
| T4 | local_818 | B | `374521`(판정), `4178ae`, `3e40d1`, 함의 bound, `418542`, 쌍대 이진화 | `374521`만 |
| T5 | local_7d8 | B | `56c80e`, `56b360` | 없음 |
| T6 | local_798 | B | `3ff744` 모든 mode(같은 누적) | 있음 |
| T7 | local_758 | 0.5B | `45feaa` | 있음 |
| T11 | local_658 | B | `52be10` | 있음 |
| T12 | local_618 | 0.5B | `46a62d` | 있음 |
| T13 | local_5d8 | B | `5466a7` | 있음 |
| T14 | local_598 | 0.5B | `45885d` | 있음 |
| T15 | local_558 | 0.5B | `414c03` | 있음 |
| T16 | local_518 | 0.5B | `414c53` | 있음 |
| T17 | local_4d8 | 0.5B | `4030c9` | 있음 |
| T18 | local_498 | 0.5B | `416036`, `414ca3` | 있음 |
| T19 | local_458 | B | `47c445` | 있음 |
| T25 | local_2d8 | B | `5431ee` | 없음 |
| T26 | local_298 | B | `3e5363` | 없음 |
| T27 | local_258 | B | `36e6b6`, `36ee92`, `370047` | 없음 |

`5a9282`(루프 전)는 타이머 없이 부른다. 타이머 포인터를 인자로 받는 callee(`3ff744`, `4030c9`, `414c03`, `414c53`, `416036`, `414ca3`, `45885d`, `52be10`, `46a62d`, `5466a7`, `45feaa`, `47c445`)는 안에서 adf236을 볼 수 있다(callee 노트에서 확인).

### 4.4 이 영역이 직접 더하는 작업량 (`*w += …`, w가 null이면 생략)

각 식은 asm의 연산 순서 그대로다(마지막 `+ *w`는 덧셈 한 번).

| 곳 | 식 | 시점 |
|---|---|---|
| I→B (0x3b21ac) | `((n_list as f64) * 4.0) * μ + *w` | 내부 반복마다 한 번, 목록을 다 돈 뒤 |
| PREPARTOBJ 시작 (0x3bbdf2) | `(st.ncols as f64) * μ + *w` | |
| PREPARTOBJ 행 검사 (0x3bdffa) | `((m as f64) * 4.0) * μ + *w` | 개수 조건과 `36dce1`을 통과한 `=` 행마다. m = 본 노드 수(§5.2) |
| PREPARTOBJ 적용 (0x3be106) | `((t as f64) + (t as f64)) * μ + *w` | 소거한 행마다. t = 그 행의 노드 수 |
| PREPARTOBJ 끝 (0x3bbe98) | `((cnt as f64) * 3.0) * μ + *w` | cnt = 행 목록 길이 |
| 쌍대 이진화 행 훑기 (0x3b6e81) | `(μ * 4.0) * (q as f64) + *w` | 훑은 행마다 |
| 쌍대 이진화 열 (0x3b6f63) | `(μ * 6.0) * (v as f64) + *w` | 본 열마다(거른 열 제외) |
| 쌍대 이진화 끝 (0x3b7001) | `(8.0 * μ) * (n_list as f64) + *w`, 그다음 `μ * ((m as f64) + (m as f64)) + *w` | m = 표시한 행 수 |

함의 bound 인라인(§5.3)은 직접 더하지 않는다(`3dcc78`·`3dd6f1` 안에서 더할 수 있다).

## 5. 이 영역이 직접 하는 상태 수정 (그대로 옮길 것)

### 5.1 I→B (0x3b2080–0x3b21d4, 0x3be1ce–0x3be233)

```rust
let (n_list, list) = 5383d0(tr);        // 현재 레벨(2)의 원시 열 목록 = 이번 반복에서 표시된 열, 삽입 순서
for t in 0..n_list {
    let j = list[t];
    if st.colcnt[j] /*+0x68*/ >= 0 && st.vtype[j] /*+0x150*/ == b'I'
       && st.lb[j] /*+0xa0*/ == 0.0 && st.ub[j] /*+0xa8*/ == 1.0 {     // 정확히 같음(ucomisd, NaN 아님)
        st.vtype[j] = b'B';
        5379f1(tr, j);
        st.cnt_type /*+0x40c*/ += 1;     // 진척도 1e-4 항에 들어감
    }
}
if w != null { *w = ((n_list as f64) * 4.0) * μ + *w; }
```
coverage: 변환(0x3be21e)은 2개 모델에서 일어난다. seq로는 enlight_hard 반복 1에서 열 2개이고, 그 체크포인트(k=8)에서 MPS는 바뀌지 않았다. 출력에는 안 보이고 vtype과 진척도(그래서 반복 횟수)만 바꾼다.

### 5.2 PREPARTOBJ: 목적 함수 부분 소거 (0x3bbaf0–0x3bbf3e, 0x3bdf17–0x3be1ad)

```rust
ade8ca(T3, w);
let n = st.ncols /*+0xc*/;
let (cnt, rows) = 538f30(tr, st);          // 현재 레벨(1) 행 목록
let nnzc = (0..n).filter(|&j| c[j].abs() > 1e-10).count();     // c = st+0xb0
if w != null { *w = (n as f64) * μ + *w; }
for idx in 0..cnt {
    let i = rows[idx];
    if sense[i] /*+0xc0*/ != b'=' { continue }
    let rc = st.rowcnt[i] /*+0x60*/;
    if !(0 < rc && rc < nnzc) { continue }
    if 36dce1(env, st, i) != 0 { continue }   // maxinf==0 && maxfin<ft && (mininf==0 && minfin>−ft) 인 이미 고정된 = 행
    let (mut ratio, mut m, mut ok) = (0.0, 0, true);
    let mut p = st.rowhead[i] /*+0x178*/;
    while let Some(nd) = p {                   // 노드: +0 계수, +8 행, +0xc 열(−1이면 삭제), +0x10 행 다음, +0x18 열 다음
        if nd.col >= 0 {
            let cj = c[nd.col];
            if cj.abs() < 1e-10 { ok = false; break }
            if ratio == 0.0 { ratio = nd.val / cj }
            else if (nd.val - cj * ratio).abs() > 1e-10 { ok = false; break }
        }
        m += 1;                                // 삭제된 노드도 센다. 실패한 노드는 안 센다
        p = nd.next_in_row;
    }
    if w != null { *w = ((m as f64) * 4.0) * μ + *w; }
    if !ok { continue }
    st.objcon /*+0xc8*/ = rhs[i] /*+0xb8*/ / ratio + st.objcon;
    let mut t = 0;
    for nd in row(i) { if nd.col >= 0 { c[nd.col] = 0.0; 5379f1(tr, nd.col); } t += 1; }
    if w != null { *w = ((t as f64) + (t as f64)) * μ + *w; }
}
if w != null { *w = ((cnt as f64) * 3.0) * μ + *w; }      // cnt = 목록 길이(모든 행)
ade770(T3, w);
```
nnzc는 루프 전에 한 번만 센다. 행은 지우지 않고 진척도 카운터도 올리지 않는다. coverage: `36dce1`까지 간 모델 22개, 적용(0x3be0cd의 `5379f1`) 0회. 기본값 38개에서는 아무것도 바꾸지 않지만 카운트(§3 #13)와 작업량은 남는다.

### 5.3 함의 bound (PREBOUND && DualReductions, 0x3b64ec–0x3b6873)

```rust
ade8ca(T4, w);
let ft = FeasibilityTol; let inf = INFBOUND;
let (cnt, cols) = 53af0f(tr, st);            // 현재 레벨(1) 열 목록
let mut nchg = 0;
if cnt > 0 {
    let tol = ft * 1000.0;
    for idx in 0..cnt {
        let j = cols[idx];
        if st.colcnt[j] <= 0 { continue }
        let (mut lo, mut hi) = (0.0, 0.0);
        3dcc78(st, j, 2, 0, -1, &mut lo, &mut hi, /*xmm0*/ inf, /*xmm1*/ 0.0);
        let lbt = lo.abs() < 1e8 && lo > tol + lb[j];
        if lbt {
            nchg += 1;
            3dd6f1(st, j, lb[j], ub[j], lo, ub[j], inf);   // (옛 lb, 옛 ub, 새 lb, 새 ub, INF): activity 갱신
            lb[j] = lo;
        }
        let ubt = hi.abs() < 1e8 && ub[j] - tol > hi;
        if ubt {
            nchg += 1;
            3dd6f1(st, j, lb[j], ub[j], lb[j], hi, inf);   // lb[j]는 위에서 바뀐 값
            ub[j] = hi;
        }
        if lbt || ubt { 5379f1(tr, j); }
    }
}
st.cnt_bound /*+0x3f8*/ += nchg;
ade770(T4, w);
```
비교는 모두 엄격(`>`), |·| < 1e8도 엄격이다. `3dcc78`의 인자 순서는 디컴파일 시그니처 `(double inf, double 0.0, st, j, 2, 0, -1, &lo, &hi)`. coverage: ub 갱신(0x3b6782) 9개 모델, lb 갱신(0x3b66b3) 0개. mas74 k=33, csched008 k=40에서 출력이 바뀐다(§7).

### 5.4 쌍대 이진화 후보 (PREDUALCONVTOBIN, 0x3b697d–0x3b723e)

가드: Aggregate && DualReductions && st+0x1e4(OptimalityTarget ≠ 1). 기본값 38/38 참.

```rust
ade8ca(T4, w);
if st.nbin /*+0xf0*/ == st.ncols { ade770(T4, w); /* 카운트로 */ }
else {
  let (n_list, cols) = 5399fc(tr, st);                  // 현재 레벨(1) 열 목록
  let m1 = if PreSOS1BigM < 0.0 { 1000.0 } else { PreSOS1BigM };   // 57b1ce(env, &m1, null, &m2)
  let m2 = if PreSOS2BigM >= 0.0 { PreSOS2BigM } else { 1000.0 };
  let (mut kc, mut nm) = (0, 0);                        // 후보 수(st+0x3b8 목록), 표시 행 수(st+0x390 목록)
  for idx in 0..n_list {
    let j = cols[idx];
    if colcnt[j] < 0 { continue }
    let f = colflag[j];                                  // st+0x170 (int, bit0/1 = SOS 소속으로 보임)
    if colcnt[j] == 0 && f & 3 == 0 { continue }
    if f & !3 != 0 || vtype[j] == b'B' { continue }
    let (l, u) = (lb[j], ub[j]);
    if !(l >= -1e6) || !(u <= 1e6) { continue }          // NaN이면 건너뜀
    let d = u - l; if !(d > 1e-6) { continue }
    let (mut dn, mut up) = (true, true);
    if f & 3 != 0 {
      dn = !(1e-10 < l.abs()) && !(d <= 1e-5);
      up = !(1e-10 < u.abs()) && !(d <= 1e-5);
      if (f & 1 != 0 && d > m1) || (f & 2 != 0 && d > m2) { dn = false; up = false; }
    }
    let cj = c[j];
    if cj > 0.0 { dn = false } else { up = up && cj >= 0.0 }
    let mut s = dn || up; let mut v = 0;                 // v = 본 열 노드 수
    let mut p = colhead[j];                               // st+0x180, 다음 = +0x18
    'col: while let Some(nd) = p {
      if !s { break }
      if nd.col >= 0 {
        let a = nd.val; let da = (d * a).abs(); let r = nd.row;
        if da < 1e-10 || (d > 1.0 && da > 1e6) { s = false; break 'col }   // 이 노드는 v에 안 셈
        let mut eq;
        if sense[r] == b'=' { eq = true }
        else { if !((a > 0.0 && dn) || (a < 0.0 && up)) { goto NEXT } eq = false }
        let mut bnd = true;
        if rowmark[r] /*st+0x3a8*/ != 0 { eq = false; /* bnd는 참인 채로 CLR로 */ goto CLR_ALWAYS }
        // 행 r: maxinf=+0x70, maxfin=+0x90, mininf=+0x78, minfin=+0x98, cnt=+0x60
        if !(maxinf[r] <= 0 && (maxfin[r] - da).abs() <= 1e-10 && mininf[r] <= 0
             && (((rowcnt[r] - 1) as f64) * da + minfin[r]).abs() <= 1e-10) { bnd = false }
        if eq && !(mininf[r] <= 0 && (minfin[r] + da).abs() <= 1e-10 && maxinf[r] <= 0
                   && (maxfin[r] - ((rowcnt[r] - 1) as f64) * da).abs() <= 1e-10) { eq = false }
        if bnd || eq {
          let mut q = 0;
          for nq in row(r) {                              // 다음 = +0x10
            if nq.col >= 0 && nq.col != j && ((nq.val.abs() - da).abs() > 1e-10 || vtype[nq.col] != b'B') {
              bnd = false; eq = false; break }            // 이 노드는 q에 안 셈
            q += 1;
          }
          if w != null { *w = (μ * 4.0) * (q as f64) + *w; }
          rowmark[r] = 1; marklist[nm] = r; nm += 1;
        }
        if bnd { goto EQ_ADJ }
        CLR_ALWAYS: dn = dn && a <= 0.0; up = up && a > 0.0;
        EQ_ADJ:
        if !eq && sense[r] == b'=' {
          s = if a < 0.0 { up } else { dn };              // CLR 뒤 값
          dn = dn && a >= 0.0; up = up && a < 0.0;
          goto NEXT;
        }
        s = dn || up;
      }
      NEXT: v += 1; p = nd.next_in_col;
    }
    if w != null { *w = (μ * 6.0) * (v as f64) + *w; }
    if s { cand[kc] = j; kc += 1; }                       // st+0x3b8
  }
  if w != null { *w = (8.0 * μ) * (n_list as f64) + *w; }
  for t in 0..nm { rowmark[marklist[t]] = 0; }
  if w != null { *w = μ * ((nm as f64) + (nm as f64)) + *w; }
  let conv = match PREDUALCONVTOBIN { 1 => true, -1 => (kc as f64) <= (st.nint /*+0xf4*/ as f64) * 5.0, _ => false };
  for t in 0..kc {
    let j = cand[t];
    if conv || ((ub[j] - 1.0) - lb[j]).abs() > 1e-10 || colflag[j] != 0 {
      let fl = (colflag[j] & 3 != 0 && lb[j] < -1e-10) as i32;
      let rc = 3c2f36(env, st, j, fl, w); if rc != 0 { ade770(T4, w); return rc }
    }
  }
  ade770(T4, w);
}
```
`CLR_ALWAYS`로 가는 표시된 행은 bnd를 거치지 않고 곧장 `dn &= a≤0; up &= a>0`을 한다(0x3b6da8 → 0x3b6eca). 표시된 `=` 행이면 그 뒤 EQ_ADJ에서 s가 늘 0이 된다. coverage: `5399fc`/`57b1ce` 37개 모델(1개는 이진 열뿐), `3c2f36` 0회 → 기본값 38개에서 kc = 0이다. 그래도 위 작업량과 카운트는 남는다. st+0xf0/0xf4는 prelude가 센 이진/정수 열 수다(L810–850).

### 5.5 그 밖

- `local_98`: AGEQ 블록(거짓)에서만 `(local_98 != 0 || ag != 0)`로 바뀐다. 라운드 끝에서 읽는다.
- 인라인 수정은 이 다섯 곳이 전부다. 나머지 상태 변화는 callee 안이다.

## 6. 호출별 정리, 추적기, 스냅숏, 진척도

### 6.1 추적기 레벨과 reset

| 때 | 호출 |
|---|---|
| 내부 반복 시작 | `3c546c(st, snapIt)` → `536971(tr, 2)` → `5369b8(tr)` |
| 내부 루프 출구 | `536971(tr, 1)` |
| PWL·NL 분기 | `536971(tr, 0)` … `536971(tr, 1)` |
| 패스 머리 | (ρ 판정 뒤) `3c546c(st, snapPass)` → `5369b8(tr)` (레벨 1) |
| 라운드 끝 첫 줄 | `536971(tr, 0)` (다른 노트) |

목록을 읽는 곳: I→B는 `5383d0`(레벨 2 원시 열 목록), PREPARTOBJ `538f30`(레벨 1 행), 함의 bound `53af0f`(레벨 1 열), 쌍대 이진화 `5399fc`(레벨 1 열).

### 6.2 진척도 판정 (`3c5595`, 식은 노트 12 §3.4, LP 포트에 있음)

| 곳 | 기준 스냅숏 | 끝내는 조건 | 상수 주소 |
|---|---|---|---|
| 내부 반복 끝 (0x3b239e) | snapIt(이번 반복 시작) | `rho <= 0.001 * max(0, i)` → 내부 루프 끝 | 0x2a684b0 = 0.001 |
| 패스 머리 (0x3b48b8), p > 0일 때만 | snapPass(이전 패스의 바깥 시작) | `rho <= 0.001 * max(0, p − 2)` → 라운드 끝 | 0x2a684b0 |
| 패스 끝 (0x3b7b22) | snapIt(마지막 내부 반복 시작) | `rho <= 1e-4` → 라운드 끝 | 0x2a684d0 = 1e-4 |

- 내부 루프는 i ≤ 0(처음 6번)에서는 ρ가 정확히 0일 때만 끝난다. ρ ≥ 0이므로 "진척이 하나도 없는 반복"에서 끝난다. i = 1부터 0.001, 0.002, …
- 패스 머리 판정은 p = 1, 2에서 문턱이 0이다. 라운드가 바뀌어도 snapPass는 다시 뜨지 않으므로 새 라운드 첫 패스(p > 0)의 ρ는 이전 패스 바깥 시작부터 라운드 끝 단계까지 포함한다.
- 패스 끝 ρ는 마지막 내부 반복 + 바깥 reduction 전체를 잰다.

### 6.3 반환값 처리

- callee 반환값 ≠ 0 → 그 값으로 종료(0x3bbf43/0x3b42d5 → `LAB_004b2dee`). 예외: `56f793`, `3e40d1`, `36e6b6`, `370047`은 반환값을 보지 않는다.
- `56c80e`은 4번째 인자(rsp+0x320)로 상태를 돌려준다. 0이 아니면 그 값으로 종료.
- `56b360` → `36d89a` 뒤 출력 상태(rsp+0x29c)를 본다(거짓 분기).
- `3c2f36` 반환값 ≠ 0이면 후보 루프를 멈추고 ade770 뒤 종료.
- `3ba5ac`·`3ba9de` 반환값 ≠ 0 → 종료(0x3bbf70).

## 7. 교차 확인

### 7.1 호출 지점 coverage

`callsites.tsv`에서 이 영역 주소 범위(§1.1)에 드는 복귀 주소는 166개다(`ade770`, `536971`은 훅을 걸지 않아 없다).
- 38/38: 루프 전 `5a9282`(0x3b1993), 내부 루프 전부(0x3b1d1e–0x3b2392, GC 제외), 패스 머리(0x3b4895, 0x3b4949, 0x3b495d), 바깥의 기본값 경로 전부(0x3b5063–0x3b7b0d), PREPARTOBJ(0x3bbb05, 0x3bbb6f, 0x3bbefa, 0x3bbf1f).
- 24개: 패스 머리 ρ(0x3b48d2, 0x3b48f5) = 패스를 2번 이상 한 모델.
- 37개: `5399fc`/`57b1ce`(0x3b6ade, 0x3b6af8) = st+0xf0 ≠ ncols.
- 22개: `36dce1`(0x3bdf48, PREPARTOBJ 행 검사). 9개: 함의 bound ub 갱신(0x3b6787, 0x3b67ae). 2개: I→B(0x3be21e).
- 한 번도 안 불린 지점: 루프 전 `42872b`/`429e1b`/`451195`/`5b1161`, GC(`36e6b6(1)` 0x3b1fe2, `36ddf2` 0x3b2012), PWL 분기 전체(`9ee0cf` 0x3b436f), NL(`b6fd6f`, `b84382`), 특수 블록 전체(`586049`, `585a6e`, `58b41b`, `42872b`, `451195`+`37168e`, `5b1161`+`37168e`, `56b360`+`36d89a`), `4178ae`, `3ff744` mode 4·5, `416036`, `414ca3`, AGEQ(`37c34c`, 0x3b64a0의 `370047`), 함의 bound lb 갱신(0x3b66b8), PREPARTOBJ 적용(0x3be0d2), `3c2f36`(0x3b71f2), `47c445` mode 1(0x3b788f). §2의 "(기본값 거짓)"과 일치한다.

### 7.2 호출 순서와 PRELIMIT 번호

임시 스크립트(커밋 안 함)로 `seq/*.txt`를 §3 규칙에 따라 세어 스윕과 겹쳤다(* = 그 k에서 출력이 바뀜). 루프 전 카운트는 2(반올림, `36d89a`)로 두었다. 라운드 끝 앞까지만 센다.

- **neos5**: 3 `5a9282` | 내부 4–10(1회, ρ=0) | 바깥 11 `3e5363`, 12 `3ff744(3)`, 13 `57627e`, 14 `578fd2`, 15 PREPARTOBJ, 16 `374521`, 17 `3ff744(1)`, 18 `3ff744(0)`, 19 `5431ee`, 20 `4030c9`, 21 `414c03`, 22 `3e40d1`, 23 `37168e`, 24 `37f484`, 25 `414c53`, 26 함의 bound, 27 `418542`, 28 쌍대 이진화, 29 `45885d`, 30 `52be10`, 31 `46a62d`, 32 `5466a7`, 33 `45feaa`, 34 `47c445(2)`, 35 패스 끝 → ρ ≤ 1e-4 → 라운드 끝. 스윕은 k = 0..39 모두 같다(presolve가 아무것도 안 바꿈). seq의 호출 순서(지점 0x3b1993…0x3b7b0d)와 그대로 맞는다.
- **glass4**: 3 | 반복1 4*`3e39a6` 5 6*`37168e` 7 8 9*`56f793` 10 | 반복2 11*`3e39a6` … 17 | 반복3 18 19*`36ddf2` … 24 | 반복4 25–31(변화 없음 → ρ=0 → i=−2에서 끝) | 바깥 32–56(변화 없음) → 패스 끝 ρ ≤ 1e-4 → 라운드 끝. 스윕의 4, 6, 9, 11, 19와 정확히 맞고, 넷째 반복과 첫 바깥 패스에 변화가 없다는 것도 맞는다.
- **timtab1**: 반복 3회(4*, 6*, 7* / 11*, 13* / 18–24 변화 없음) | 바깥 25*`3e5363`, 31*`3ff744(1,1)`, 38*`37f484`, …, 48 `47c445`, 49 패스 끝 | p=1: 반복 1회 50*–56 | 패스 머리 ρ | 바깥 57–79(374521 있음, 5431ee·47c445 없음 = k=1) → 라운드 끝(80*부터 라운드 끝 단계). 라운드 1: p=1 그대로라 첫 패스에서도 패스 머리 ρ가 불리고(seq 1081 `3c5595@0x3b48d2`), `5431ee`는 없고 `47c445`는 있다(k=0, round=1). 그다음 p=2 패스(374521 있음) 뒤 라운드 끝.
- 덧붙여 mas74 k=33, csched008 k=40이 정확히 함의 bound 인라인 번호이고, 두 모델에서만 ub 갱신(0x3b6787)이 불린다. gmu-35-50 k=3은 `5a9282`다(m2_presolve.md의 "k=3에서 2개 모델"과 맞음).

### 7.3 glass4의 내부 반복 수

glass4는 내부 반복을 **6번** 한다.
1. 단계 0, 라운드 0, p=0: **4번**. 반복 1–3은 모델을 바꿨고(스윕 4, 6, 9 / 11 / 19), 반복 4는 아무것도 안 바꿔 ρ = 0 ≤ 0.001·max(0, −2) → 끝. 바깥 패스(32–56)도 아무것도 안 바꿔 패스 끝 ρ ≤ 1e-4 → 라운드 끝. p는 올리지 않는다(0).
2. 라운드 끝 단계와 0x3bcb21 블록(`3c546c@0x3bcb36`, `4d3a1d@0x3bcd45`, 다른 노트) 뒤 단계 1로 돌아온다(round = 0, p = 0). **1번**(ρ=0). 패스 머리 ρ는 p=0이라 안 본다. round==0 && k==0이라 `5431ee`, k==0이라 `47c445`가 다시 돈다. 패스 끝 ρ > 1e-4 → p=1.
3. p=1: **1번**(ρ=0). 패스 머리 ρ > 0.001·max(0, −1) = 0 → 계속. 바깥(`5431ee`·`47c445` 없음) 뒤 패스 끝 ρ ≤ 1e-4 → 라운드 끝 → 마지막 단계.

단계 1의 스윕 변화점 76, 93, 106은 라운드 끝 카운트 수에 달려 있어 아직 정렬하지 않았다. 패스 끝(56)과 단계 1 첫 반복 사이 reduction 호출은 20개다(`58d4ce`…`5a04fc` 19개, 그중 대칭 `5577ba` 하나, 0x3bcb21 블록의 `4d3a1d` 하나). 20개가 모두 센다면 76 = 0x3bcb21의 `4d3a1d`, 93 = `4030c9`, 106 = `45feaa`이다. `5577ba`가 LP처럼 안 센다면 76 = 단계 1 첫 `3e39a6`(1회로 끝난 반복, §8-1과 같은 꼴), 93 = `414c03`, 106 = `47c445(2)`이다. 라운드 끝 노트가 확정할 것.

## 8. 미확인 사항

1. **timtab1 k=50**: p=1 내부 루프는 1회(ρ = 0)인데 그 반복의 첫 카운트(`3e39a6`, k=50)에서 출력이 바뀐다. 라운드 1의 첫 내부 반복(1회)에도 변화점 99가 있다(라운드 끝 카운트 수에 따라 `3e39a6` 또는 `36ddf2`). 다른 모델의 "1회로 끝난 반복"에는 변화가 없다(glass4는 §7.3의 정렬에 따라 다름). `3e39a6`이 진척도 카운터에 안 잡히는 변경을 하거나(코드에 `st+0x404 −= 1`이 있다, L638), 마무리(`LAB_004b2415`)가 5369b8 reset 뒤 추적기 원시 목록(`538353`/`5383d0`)을 써서 결과가 달라지는 것일 수 있다. 포트가 ρ를 맞게 계산하면 반복 횟수는 seq와 같아야 하니, 이 항목은 반복 횟수가 아니라 "그 체크포인트의 출력"만의 문제다. 확정: timtab1의 PRELIMIT 49/50/51 MPS를 diff하거나, Frida로 `3e39a6` 앞뒤의 st+0x3f8..0x428·+0xc·+0x10을 덤프한다.
2. **루프 전 카운트 2**: 0x3b0e6c의 `local_68 = 1`(반올림 루프 끝)이 기본값 경로에서 늘 실행되는지는 prelude 노트가 확인할 것. 스윕(glass4 4 = `3e39a6`, gmu k=3 = `5a9282`)은 2와 맞는다.
3. **PRERESTART 플래그**: `6a0643`은 MIPLIM(env+0x27f0) < 1e100 && Presolve ≠ 0 && PREMAKEEQ(env+0x2638) == 0일 때 참이다. 기본값에서 참인지 모른다. 참이어도 열 수 ≤ 5e4이면 조건(제거 > 0.8·n + 1e4)이 성립할 수 없다. 확정: 기본값 env를 덤프하거나 큰 모델에서 `3ba9de`의 플래그를 Frida로 본다.
4. **st+0xc/+0x10의 뜻**: "0이면 라운드 끝"과 `3c5595`의 열 항에서 쓴다. 압축(`36ee92`)이 줄이는 현재 열·행 수로 보이나 확인하지 않았다. 확정: `36ee92` 해독.
5. **st+0x170(열 플래그)**: bit0/1을 SOS1/SOS2 소속으로 읽었다(`57b1ce`가 PreSOS1BigM/PreSOS2BigM을 돌려줌). 기본값 38개에서 후보가 0이라 영향은 없다. 확정: 이 배열을 채우는 곳(`3a484d`) 확인.
6. **타이머 한도에 걸리는 경우**: 기본값 B = 2500·nnz + 1e7이라 38개에서 0.8에 닿는지 확인하지 않았다. 닿으면 그 reduction과 카운트가 빠진다. 확정: Frida로 각 adf236 반환값을 기록하거나 포트에서 같은 값을 계산해 비교.
7. **callee 안의 adf236**: 타이머를 인자로 받는 callee(§4.3 끝)가 안에서 한도를 보는지는 callee 노트 몫이다.
8. **라운드 끝 쪽 확인 거리**(다른 노트): r12d를 라운드 끝 코드가 건드리지 않는지, 단계 1로 돌아가는 조건(0x3bcb21 블록), 라운드 끝 카운트 수.
