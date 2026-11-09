# M2 MIP presolve `3cd406`: 라운드 끝 단계 (spine C)

`3cd406`의 라운드 끝 단계를 해독한 노트다. 패스 루프가 끝난 뒤 실행되고, 다음 라운드로 돌아갈지 마무리 단계(`LAB_004b9027`)로 갈지 정한다. 2026-11-06에 asm(`objdump`)과 Ghidra로 정적 해독했고, Frida 호출 순서(`seq/*.txt`, `cov2.json`)로 맞춰 보았다. 줄 번호는 `re/decomp/c/003b0530_PRIVATE00000000003cd406.c` 기준이다. 주소는 파일 오프셋(= `.text` vaddr, Ghidra 주소 − 0x100000)이다. Gurobi는 돌리지 않았다.

## 1. 개요

### 1.1 범위
- Ghidra **L4400 `LAB_004b7b7c` – L6013**(`LAB_004bc28e` 블록 끝)이 이 단계다. asm으로는 본체 0x3b7b7c–0x3b9026과 줄 밖 조각 세 개다. 0x3bd38a–0x3bd3c7(SOS big-M 검사), 0x3bd3cc–0x3bdd9e(열 타입 통계, SIMD), 0x3bddf7–0x3bde36(임계값 선택). 나가는 길은 0x3bc28e(오류)와 0x3bc2b6(PRELIMIT)이다.
- **L6015–6450은 이 단계가 아니다.** Ghidra가 그 자리에 다른 구역의 조각을 붙여 놓았다.
  - L6015–6098: 프렐류드의 `5b1161`/`451195`/`429e1b`/`42872b`/`5a9282` 성공 분기다. 돌아가는 곳이 `LAB_004b19fd`–`LAB_004b1c36`(0x3b19fd–0x3b1c36)이다.
  - L6100–6137 `LAB_004b1863`: 프렐류드에서 재시작 문맥(mipctx)일 때만 타는 작은 모델 경로다(`490ab6`, `4962bd`).
  - L6172–6447: 프렐류드의 할당, `564ce0`, `56c89d`.
  - L6449 `LAB_004b0775` → L6451 `LAB_004b2dee`: 공통 출구다.
- **작은 모델 직접 해결은 라운드 끝 단계에 없다.** 노트 12 §3.3은 여기에 넣었지만 틀렸다. `3cd406` 안의 호출 지점은 세 곳이다.
  - `4962bd`(0x3b18fb): 프렐류드. mipctx가 있고, state+0x244==0, mipctx+0x3460>0, rs_a8>0일 때.
  - `490ab6`(0x3be28b): 프렐류드. 같은 mipctx 조건에서 m==1일 때.
  - `490ab6`(0x3bcaa4): 마무리 단계. SOLREDUCTION>0, DualReductions>0이고 남은 행이 1개일 때(0x3b90c6–0x3b90f4).
  - 기본값 38개에서는 모두 0회다.

### 1.2 변수 대응

| Ghidra | asm | 뜻 |
|---|---|---|
| `cVar128` | `r13b`, 슬롯 0x2f0 | **라운드 번호** round (0..4) |
| `cVar125` | `r15b`, 슬롯 0x2f8 | **바깥 반복** outer (0..1). 마무리 단계가 패스 루프로 다시 보낼 때 1이 된다(§1.4) |
| `iVar129`, `local_58` | `ebp`, 0xc40 | probe_mode = `3cd406`의 `param_2`. `GRBpresolvemodel`은 `3db19d(model, env.PREPROBE(0x2828)=1, …)`로 부른다. 프렐류드(0x3b3626)가 조건에 따라 3으로 올린다 |
| `iVar116`, `local_68` | `ebx`, 0xc30 | **PRELIMIT 카운터** cnt |
| `r12d`, 0xc58 | `r12d`, 0xc58 | 전체 패스 수. 라운드·바깥 반복을 넘어서도 0으로 돌리지 않는다. PrePasses(0x370)와 비교한다 |
| `local_150` | 0xb48 | state |
| `local_210`, `lVar119` | 0xa88 | env |
| `local_208` | 0xa90 | model(`param_1`) |
| `local_218`, `param_5` | `r14`, 0xa80 | work 포인터 W |
| `local_78` | 0xc20 | mipctx(`param_3`). P0에서는 NULL |
| `local_a8` / `local_a0` | 0xbf0 / 0xbf8 | rs_a8 / rs_a0. mipctx가 있으면 mipctx+0x620→+0x808 / +0x7fc에서 읽고, 없으면 0 |
| `local_104` | 0xb94 | stop. `3ba9de`의 세 번째 인자로, PRERESTART 판정 결과다 |
| `local_148` | 0xb50 | **라운드 스냅숏** snap_round |
| `local_1e8` | 0xab0 | 내부 반복 스냅숏 snap_inner. 패스 루프 0x3b1d11이 내부 반복마다 찍는다 |
| `local_9c8` / `local_9c0` | 0x2d0 / 0x2d8 | 라운드 시작 판정의 thr / ρ |
| `local_a08` | 0x290 | 라운드 끝 판정의 ρ |
| `local_aa0` / `local_aa8` | 0x1f8 / 0x1f0 | 대칭의 work 한도 / 노드 한도 |
| `local_9b8` | 0x2e0 | max(probe_mode, 2). 0x3b1c80에서 한 번 정한다 |
| `local_1a4` | 0xaf4 | `5577ba`의 출력 f4. 모델 생성 뒤 0x3b29bd가 새 모델의 +0x218에 쓴다 |
| `local_948` | 0x350 | 타이머로 감싼 callee의 반환값 |
| timer i | rsp+0x380+0x40·i | Ghidra `local_(0x918−0x40·i)` (i = 0..27) |

μ = state+0x448(작업 배수). n = state+0xc, m = state+0x10. 지운 열 수는 +0x408, 지운 행 수는 +0x404다.

### 1.3 들어오는 길 (패스 루프 → 0x3b7b7c)
패스 루프 담당 구역에 있다. 모두 `r13b`/`r15b`/`ebp`/`r14`를 슬롯에서 다시 읽은 뒤 0x3b7b7c로 온다.
- 0x3b7b33: 패스 끝 ρ(snap_inner) ≤ 1e-4. 0x3b7b47에서 떨어지는 경우는 전체 패스 수 ≥ PrePasses다.
- 0x3b492e: 패스 시작 판정. r12 > 0이고 ρ(0xb00) ≤ 0.001·max(0, r12−2)일 때다. r12는 라운드 안 패스 번호가 아니라 전체 패스 수다.
- 0x3b48b2/0x3bbfde: 내부 루프 뒤 stop이거나, n==0 또는 m==0.
- `jne 3b7b4d` 33곳: 패스 reduction 뒤 `3ba9de`가 stop을 세웠을 때.
- 0x3be238: **라운드 머리 `LAB_004b1cb4`에서 전체 패스 수 ≥ PrePasses이면 패스 없이 곧장 이 단계로 온다.**

### 1.4 나가는 길과 라운드 수
- **다음 라운드**: 0x3b9021 `jl 3b1cb4`. round+1 < 5일 때다. 라운드 머리는 패스 루프 담당이다. 거기서 0x2e8(라운드 안 패스 수) = 0으로 두고, round·outer를 슬롯에 저장한다.
- **마무리 단계** `LAB_004b9027`(0x3b9027, L6582): stop이 섰을 때, 라운드 시작 판정 ρ ≤ thr일 때, 라운드 끝 판정 ρ ≤ 1e-4일 때, round+1 == 5일 때. 0x3b9027은 `r12d`를 0xc58에 저장하고, 추적기 레벨을 0으로, `3ba9de`를 부른다.
- **PRELIMIT 마무리** `LAB_004b2415`: 0x3bc2b6에서 cnt를 0xc30에 저장하고 0x3b23f5로 간다. 마무리 단계는 건너뛴다.
- **오류 출구** `LAB_004b2dee`: 0x3bc28e에서 r15d = rc로 둔다. callee, `3ba5ac`, `3ba9de` 중 하나라도 rc ≠ 0이면 이리로 간다.
- **최대 라운드**: 바깥 반복 하나에 5라운드(round 0..4)다. 바깥 반복은 마무리 단계(0x3b9078)가 정한다. outer==0이면 0x3bcb21에서 스냅숏 0x1a8을 찍고 마무리 probing(`4d3a1d`, timer 8)을 한다. 그 ρ(0x1a8)가 1e-4보다 크면 outer = 1로 0x3b1ca9로 돌아간다. 0x3b1ca9는 r12d를 다시 읽고 **round = 0**으로 둔다. outer==1이면 다시 돌아가지 않는다. 그래서 최대 2×5라운드다. 전체 패스 수 상한 PrePasses는 그 모두에 걸친다.

## 2. 의사코드 (기본값 경로)

표기:
- `count!()`: `cnt += 1; if cnt >= state.i32(0x1dc) { goto FINISH /*LAB_004b2415*/ }` (`cmp 0x1dc(%rsi),%ebx; jge`).
- `cb!()`: `rc = 3ba5ac(env, state, 0, 0, -1); if rc != 0 { goto EXIT(rc) }`.
- `chk!()`: `cb!(); rc = 3ba9de(env, state, &stop); if rc != 0 { goto EXIT(rc) }; if stop { goto FINAL }`.
- `timed!(T, f)`: `ade8ca(T, W); rc = f; ade770(T, W); if rc != 0 { goto EXIT(rc) }`.
- `usage(T) = adf236(T, W)`. 아래 순서에서 rc 검사는 늘 `count!()`보다 먼저 온다.

P0(`Model.presolve()`)에서 늘 성립하는 값:
- mipctx = NULL, rs_a0 = rs_a8 = 0.
- subdepth(env+0x2728) = 0.
- stop = 0. `GRBpresolvemodel`이 PRERESTART(env+0x25c8)를 0으로 두므로 `3ba9de`는 *stop = 0만 쓴다.

```rust
// LAB_004b7b7c (0x3b7b7c)
tracker.set_level(0);                                         // 536971(state+0x3d8, 0)
rc = ba9de(env, state, &mut stop)?; if stop { goto FINAL }    // 0x3b7b9e
if outer > 0 || round > 0 {                                   // 0x3b7bb9, 0x3bde2d
    let thr = if outer == 0 && (round < 2 || probe_mode > 1 || env.Presolve > 1) { 0.01 } else { 0.05 };
    let rho = c5595(state, &snap_round);                      // 0x3b7bec
    cb!();                                                    // 0x3b7c14
    if !(rho < thr) { goto FINAL }   // comisd thr,ρ; jae: ρ ≤ thr이면 FINAL, NaN이면 계속
}
// 열 타입 통계 (인라인, §5.1)
if probe_mode < 3 && state.f8 == 0 && env.subdepth == 0 && env.Presolve != 0 {
    let cols = (n - state.x408) as f64; let rows = (m - state.x404) as f64;
    if rows > 1.2 * cols && mipctx.is_null() { col_type_stats(); }   // §5.1
}
c546c(state, &mut snap_round);                                // 0x3b7cc4  라운드 스냅숏
tracker.reset();                                              // 5369b8, 0x3b7cd8

// A. 58d4ce (Q 대각 열 → 3c2f36. Q(+0x250)가 없으면 아무것도 안 함)
if env.Aggregate != 0 && env.DualReductions != 0 && state.x1e4 != 0 {   // +0x1e4 = (OptimalityTarget != 1)
    timed!(T4, d58d4ce(env, state, W)); count!(); chk!();
}
// B. SOS (기본값 거짓: SOS 없는 모델. 38개 모두 0회)
let sos = || state.x1f8 - state.x410 > 0 && (env.PreSOS1BigM(0x2640) != 0.0 || env.PreSOS2BigM(0x2648) != 0.0);
if sos() { d586049(env, state /*xmm0 = INFBOUND(env+0x2458)*/); count!(); chk!(); }
if sos() { d57b2bf(env, state, rs_a8 > 0, W)?; /* 카운트 없음 */ chk!(); }
// C. 대칭 (round 0만. 바깥 반복 1의 round 0에서도 다시 돈다)
if round == 0 && env.Symmetry != 0 && env.DualReductions != 0 && env.subdepth == 0 {
    let mut limit = 1e100; let mut nodes = 20000;
    if adf205(T24) /*한도 != 0*/ { limit = adf1f2(T24) - adf00e(T24, W); }   // 2B − 사용량
    if env.Symmetry >= 2 { nodes = 50000; if limit < 1e100 { limit *= 10.0; } }
    timed!(T24, d5577ba(env, state, nodes, 1, &mut f4 /*0xaf4*/; xmm0 = 1.01, xmm1 = limit));
    count!(); chk!();
}
// D. PREBOUND 쌍
if env.PREBOUND != 0 && env.DualReductions != 0 { d5a9282(state, env); count!(); chk!();
    if env.PREBOUND != 0 && env.DualReductions != 0 { d5ab429(state, env); count!(); chk!(); } }
// E. VUB lifting
if env.PREVUBLIFT != 0 { timed!(T13, d59c193(state, env, W)); count!(); chk!(); }
// F. round 0 전용 둘 (같은 timer 20)
if env.subdepth == 0 && round == 0 && rs_a0 == 0 && rs_a8 == 0 && env.PreSparsify != 0 {
    timed!(T20, d55a779(env, state, &T20)); count!(); chk!(); }
if env.PRESS2MIP != 0 && env.subdepth == 0 && round == 0 && rs_a0 == 0 && rs_a8 == 0 {
    timed!(T20, d594c0c(env, state, &T20)); count!(); chk!(); }
// G. timer 사용률을 보는 둘
if env.PREPROBE0 != 0 && usage(T10) < 0.8 {
    timed!(T10, d423e35(state, env, &T10, probe_mode)); count!(); chk!(); }
if env.subdepth == 0 && env.PRECOVERS != 0 && usage(T23) < 0.8 {
    timed!(T23, d419ffc(state, env, &T23)); count!(); chk!(); }
// H. DualReductions 블록 (가드는 매번 env에서 다시 읽지만 실행 중에 바뀌지 않는다)
if env.DualReductions != 0 {
    if env.PREDUALREDUNDANT != 0 && env.PreCrush == 0 { timed!(T4, d5a1293(state, env)); count!(); chk!(); }
    if env.PREDERIVED != 0 && env.PreCrush == 0 {
        d5a4d8b(state, env)?; count!(); cb!();               // 3ba9de·stop 검사 없음
        d5a5eff(state, env)?; count!(); chk!();
    }
    if env.PREDUALREDUNDANT != 0 { d5aaaf7(state, env)?; count!(); chk!(); }
}
// I. mod-k 세 번 (파라미터 가드 없음)
for k in [2, 3, 5] { d838543(env, state, mipctx.is_null() as u32, k, W)?; count!(); chk!(); }
// J
d579e67(state, env)?; count!(); chk!();
// K. probing (라운드 끝 호출 지점)
if probe_mode != 0 && usage(T9) < 0.8 {
    timed!(T9, d4d3a1d(model, mipctx, state, env, &T9, max(probe_mode, 2) /*0x2e0*/, 0, 0, 0, 0));
    count!(); chk!();
}
// L. (기본값 거짓) PRECLIQSUB=0
if env.PRECLIQSUB != 0 && usage(T18) < 0.8 { timed!(T18, d414ca3(state, env, &T18)); count!(); chk!(); }
// M. smart disaggregation
if env.PRESMARTDISAGG >= 2 { d5a04fc(state, env)?; count!(); chk!(); }   // 기본 5
// N. (기본값 거짓) rs_a8 > 0은 재시작 문맥 전용
if env.PREDOMINATE != 0 && rs_a8 > 0 { d5a7c0f(state, env)?; count!(); chk!(); }
// O. (기본값 거짓) PRETWOROWS = -1, rs_a8 = 0
if env.PRETWOROWS == 2 || (rs_a8 > 0 && env.PRETWOROWS == -1 && env.Presolve != 1) {
    if usage(T7) < 0.8 { timed!(T7, d45feaa(state, env, &T7, 2)); count!(); cb!(); }
    rc = ba9de(env, state, &mut stop)?; if stop { goto FINAL }   // 0x3b8f97: 건너뛰어도 부른다
}
// P. 라운드 끝 판정
let rho = c5595(state, &snap_inner);                          // 0x3b8fd2, 스냅숏 0xab0
cb!();                                                        // 0x3b8ffa
if !(rho > 1e-4) { goto FINAL }      // comisd 1e-4,ρ; jae (NaN이면 계속)
round += 1; if round < 5 { goto LAB_004b1cb4 } else { goto FINAL }
```

기본값 거짓인 분기 요약:
- B: SOS가 남아 있고 PreSOS1BigM 또는 PreSOS2BigM이 0이 아니면 `586049`(SOS 축약)와 `57b2bf`(big-M 인코딩)를 부른다.
- L: PRECLIQSUB=1이면 `414ca3`.
- N: 재시작 문맥(rs_a8 > 0)에서만 `5a7c0f`.
- O: PRETWOROWS=2이거나 재시작 문맥이면 `45feaa(mode 2)`.
- stop: PRERESTART가 켜진 최적화 경로에서만 선다.

라운드 시작 판정의 thr 분기:
- outer > 0: 0.05.
- outer == 0이고 round ≥ 2: probe_mode ≤ 1이고 Presolve ≤ 1이면 0.05(기본값), 아니면 0.01.
- 38개에서 round 2에 닿는 모델은 없다(§7).

## 3. PRELIMIT 카운트 지점 (프로그램 순서)

모두 `inc %ebx; cmp 0x1dc(%rsi),%ebx; jge 3bc2b6`(→ 0x3b23f5 → `LAB_004b2415`)이다. 카운트는 해당 reduction이 rc == 0으로 돌아온 직후, `3ba5ac`보다 먼저 한다.

| # | 카운트 주소 | 앞 호출 | 가드 (기본값) |
|---|---|---|---|
| 1 | 0x3b7d69 | `58d4ce` | Aggregate·DualReductions·state+0x1e4 (참) |
| 2 | 0x3b7e21 | `586049` | SOS (거짓) |
| — | 없음 | `57b2bf` | SOS (거짓). **세지 않는다** |
| 3 | 0x3b8089 | `5577ba` | round 0 (참). **LP(`390989`)와 달리 센다** |
| 4 | 0x3b8127 | `5a9282` | PREBOUND·DualReductions (참) |
| 5 | 0x3b81c1 | `5ab429` | 같음 (참) |
| 6 | 0x3b8287 | `59c193` | PREVUBLIFT (참) |
| 7 | 0x3b8383 | `55a779` | round 0, PreSparsify≠0 (참) |
| 8 | 0x3b847f | `594c0c` | round 0, PRESS2MIP (참) |
| 9 | 0x3b856f | `423e35` | PREPROBE0, timer 10 < 0.8 |
| 10 | 0x3b866b | `419ffc` | PRECOVERS, timer 23 < 0.8 |
| 11 | 0x3b8748 | `5a1293` | DualReductions·PREDUALREDUNDANT·PreCrush==0 (참) |
| 12 | 0x3b8802 | `5a4d8b` | DualReductions·PREDERIVED·PreCrush==0 (참) |
| 13 | 0x3b8854 | `5a5eff` | 같음 (참) |
| 14 | 0x3b88ef | `5aaaf7` | DualReductions·PREDUALREDUNDANT (참) |
| 15–17 | 0x3b8986, 0x3b8a1d, 0x3b8ab4 | `838543` k=2, 3, 5 | 없음 |
| 18 | 0x3b8b35 | `579e67` | 없음 |
| 19 | 0x3b8c36 | `4d3a1d` | probe_mode≠0, timer 9 < 0.8 |
| 20 | 0x3b8d25 | `414ca3` | PRECLIQSUB (거짓) |
| 21 | 0x3b8db2 | `5a04fc` | PRESMARTDISAGG ≥ 2 (참) |
| 22 | 0x3b8e4d | `5a7c0f` | rs_a8 > 0 (거짓) |
| 23 | 0x3b8f6a | `45feaa` | PRETWOROWS (거짓) |

- **건너뛴 reduction을 세는 곳은 이 구역에 없다.** 카운트 창(호출 복귀 ~ `cmp`) 안으로 들어오는 점프가 이 구역에는 하나도 없다(0x3b0530–0x3c0000 전체에서 점프 대상을 검사했다). 열 타입 통계, 스냅숏, 진척도 판정도 세지 않는다.
- 기본값 P0에서 이 단계 하나가 세는 수:
  - round 0: 19번이다. 58d4ce, 5577ba, 5a9282, 5ab429, 59c193, 55a779, 594c0c, 423e35, 419ffc, 5a1293, 5a4d8b, 5a5eff, 5aaaf7, 838543×3, 579e67, 4d3a1d, 5a04fc. 단 423e35, 419ffc, 4d3a1d는 timer 사용률이 0.8 미만일 때만 돈다.
  - round ≥ 1: 16번이다(5577ba, 55a779, 594c0c가 빠진다).
- 예산 초과(`3ba9de`가 state+0x1dc = −1로 둔다)는 **다음** `count!()`에서야 걸린다. 그래서 초과를 알아챈 뒤에도 가드가 참인 다음 reduction 하나가 더 돈다. 이 구역에 그 뒤로 센 호출이 없으면, 다음 라운드 패스 루프의 첫 카운트나 마무리 단계의 첫 카운트에서 멈춘다.
- `3ba9de`의 rc ≠ 0(TimeLimit/WorkLimit, `a83b2f`)은 PRELIMIT 마무리가 아니라 오류 출구로 간다.

## 4. 작업량과 timer

### 4.1 `*W += …` (이 구역의 인라인)
열 타입 통계 안에만 있다(§5.1). 순서는 다음과 같다.
1. `W += (2·max(n,0)) · μ`. 열 루프의 끝 인덱스는 n이다.
2. 이진 비율 조건이 참이면 `W += (2·max(m,0)) · μ`.

W가 NULL이면 둘 다 건너뛴다. 나머지 work는 모두 callee 안에서 쌓인다.

### 4.2 timer 동작 (`ade8ca`/`ade770`/`adf236`)
- `ade8ca(T, W)`: `T.w = T.w − *W + 1.0`이고 running = 1이다. **시작할 때마다 1.0을 더한다.**
- `ade770(T, W)`: `T.w += *W`이고 running = 0이다.
- `adf236(T, W)` = (T.w + (running이면 *W)) / T.limit. limit < 0이면 1.0, limit == 0이면 0이다.
- `adf205(T)` = (limit ≠ 0). `adf1f2(T)` = limit. `adf00e(T, W)` = T.w (+*W, running일 때).

### 4.3 이 구역이 쓰는 timer
B는 프렐류드가 정하는 예산이다(0x358, +1e7 포함). 28개 timer는 모두 B로 시작하고, 프렐류드 0x3b4495–0x3b4646(무한 예산이면 0x3b1435–0x3b15c7)이 일부를 바꾼다.

| timer (슬롯) | 한도 | 이 구역 사용 | 같은 timer를 쓰는 다른 곳 |
|---|---|---|---|
| 4 (0x480) | B | `58d4ce`, `5a1293` (시작·정지만) | 패스: `374521`(**0.8 검사**), `4178ae`, `3e40d1`, `53af0f`, `418542` |
| 7 (0x540) | 0.5B | `45feaa` (검사, 기본값 거짓) | 패스 `45feaa` |
| 8 (0x580) | B. 조건부 3B, 통계가 2배로 | 없음 | 마무리 단계 `4d3a1d` |
| 9 (0x5c0) | B. 조건부 3B, 통계가 2배로 | `4d3a1d` (검사) | 없음 |
| 10 (0x600) | B. 조건부 3B, 통계가 2배로 | `423e35` (검사) | 없음 |
| 13 (0x6c0) | B | `59c193` (시작·정지만) | 패스 `5466a7`(**0.8 검사**) |
| 18 (0x800) | 0.5B | `414ca3` (검사, 기본값 거짓) | 패스 `416036`, `414ca3` |
| 20 (0x880) | 0.5B | `55a779`, `594c0c` (시작·정지만) | 마무리 `470936`(**0.8 검사**) |
| 23 (0x940) | 0.5B | `419ffc` (검사) | 없음 |
| 24 (0x980) | 2B | `5577ba` (한도 계산에 사용) | 없음 |

- "조건부 3B"는 프렐류드에서 state+0xf8 ≠ 0이거나 probe_mode ≥ 3일 때다.
- **timer를 함께 쓰므로 포트도 timer 객체를 공유해야 한다.** 예를 들어 라운드 끝의 `58d4ce`/`5a1293`이 쌓은 work(시작마다 +1 포함)가 다음 라운드 `374521`의 0.8 판정에 들어간다. `59c193`의 work는 `5466a7`의 판정에, `55a779`/`594c0c`의 work는 마무리 `470936`의 판정에 들어간다.
- timer 없이 부르는 것: `586049`, `57b2bf`, `5a9282`, `5ab429`, `5a4d8b`, `5a5eff`, `5aaaf7`, `838543`(안에서 따로 `adf1a6(100·m + 1e7)`), `579e67`, `5a04fc`, `5a7c0f`.

## 5. 3cd406이 이 구역에서 직접 바꾸는 상태

### 5.1 열 타입 통계 (0x3b7c39–0x3b7cb6, 0x3bd3cc–0x3bdd9e)
SIMD 루프지만 결과는 아래 스칼라 코드와 같다. 개수를 세는 순서는 결과에 영향이 없다.

```rust
// 가드: probe_mode < 3 && state.f8 == 0 && env.subdepth == 0 && env.Presolve != 0
//       && (m - x404) as f64 > 1.2 * (n - x408) as f64 && mipctx.is_null()
let cols = (n - state.x408) as f64; let rows = (m - state.x404) as f64;   // cvtsi2sd(i32 뺄셈)
let nbin = (0..n).filter(|&j| state.colcnt[j] /*+0x68*/ > 0 && state.vtype[j] /*+0x150*/ == b'B').count();
if let Some(w) = W { *w = (2.0 * max(n,0) as f64) * state.mu + *w; }     // (i+i)·μ + W 순서
if (nbin as f64) >= 0.99 * cols {                                          // comisd: 같아도 통과
    let neq = (0..m).filter(|&i| state.rowcnt[i] /*+0x60*/ > 0 && state.sense[i] /*+0xc0*/ == b'=').count();
    if let Some(w) = W { *w = (2.0 * max(m,0) as f64) * state.mu + *w; }
    if (neq as f64) * 1000.0 > rows {
        state.f8 = 1;                                                      // +0xf8
        for t in [T8, T9, T10] { adf1a6(t, 2.0 * adf1f2(t)); }            // 한도 2배
    }
}
```

- 덧셈 순서는 asm 그대로다. `xmm1 = (double)i; xmm1 += xmm1; xmm1 *= μ; xmm1 += *W`.
- state+0xf8은 한 번 1이 되면 다시 0이 되지 않는다. 그래서 통계는 다시 돌지 않는다.
- 프렐류드(L1005–1012)에도 같은 판정이 있다. 0xf8을 읽는 곳은 `3cd406` 안에서는 프렐류드의 timer 3B 판정과 이 가드뿐이다. callee 중에는 `4d3a1d`가 읽는다(L464, L611, L823). 라운드 끝 호출은 mode=2, param_7=0이라 L464 조건에 걸린다. 그래서 **f8은 이 단계의 probing 동작도 바꾼다.**

### 5.2 그 밖
- `536971(tracker, 0)`: 단계에 들어올 때 레벨 0으로 둔다. 이 단계 전체가 레벨 0에서 돈다. 다음 라운드의 패스 루프가 내부 반복마다 레벨 2와 reset을 한다.
- `3c546c(state, 0xb50)`: 라운드 스냅숏. 통계 바로 뒤, `5369b8`(추적기 reset) 바로 앞에서 찍는다. **찍는 곳은 이곳 하나뿐이다.** 그래서 라운드 시작 판정의 ρ는 "직전 라운드 끝 단계의 reduction + (마무리 단계 probing) + 이번 라운드 패스"의 진척이다. 0xb50은 프롤로그에서 0으로 채운다.
- 라운드 끝 판정은 snap_inner(0xab0)를 쓴다. 0xab0은 패스 루프가 내부 반복을 시작할 때마다 찍는다(0x3b1d11). 그래서 ρ는 "마지막 패스의 마지막 내부 반복 + 그 패스의 바깥 reduction + 이 라운드 끝 단계"의 진척이다.
- `round += 1`. 0x2d0, 0x2d8, 0x290, 0x1f8, 0x1f0, 0x350은 스택 임시값이다. 0xaf4(f4)는 `5577ba`가 쓴다.

## 6. Reduction 호출 상세

| 호출 (복귀 주소) | 인자 | 가드 | rc ≠ 0 | 뒤처리 |
|---|---|---|---|---|
| `58d4ce` (0x3b7d3c) | (env, state, W) | Aggregate≠0 && DualReductions≠0 && state+0x1e4≠0 | 오류 출구 | count, cb, 3ba9de, stop |
| `586049` (0x3b7e11) | (env, state, rdx=?; xmm0=INFBOUND) | SOS 남음 && (SOS1BigM≠0 \|\| SOS2BigM≠0). NaN은 ≠0으로 친다(`jp`) | 오류 | count, chk |
| `57b2bf` (0x3b7ed1) | (env, state, rs_a8>0, W) | 같은 조건을 다시 평가 | 오류 | **카운트 없음**, chk |
| `5577ba` (0x3b805c) | (env, state, nodes, 1, &f4; xmm0=1.01, xmm1=limit) | round==0 && Symmetry≠0 && DualReductions≠0 && subdepth==0 | 오류 | count, chk |
| `5a9282` (0x3b8117) | (state, env) | PREBOUND≠0 && DualReductions≠0 | 오류 | count, chk |
| `5ab429` (0x3b81b1) | (state, env) | 같음 | 오류 | count, chk |
| `59c193` (0x3b825a) | (state, env, W) | PREVUBLIFT≠0 | 오류 | count, chk |
| `55a779` (0x3b8356) | (env, state, &T20) | subdepth==0 && round==0 && rs_a0==0 && rs_a8==0 && PreSparsify≠0 | 오류 | count, chk |
| `594c0c` (0x3b8452) | (env, state, &T20) | PRESS2MIP≠0 && subdepth==0 && round==0 && rs_a0==0 && rs_a8==0 | 오류 | count, chk |
| `423e35` (0x3b8542) | (state, env, &T10, probe_mode) | PREPROBE0≠0 && usage(T10)<0.8 | 오류 | count, chk |
| `419ffc` (0x3b863e) | (state, env, &T23) | subdepth==0 && PRECOVERS≠0 && usage(T23)<0.8 | 오류 | count, chk |
| `5a1293` (0x3b871b) | (state, env) | DualReductions && PREDUALREDUNDANT && PreCrush==0 | 오류 | count, chk |
| `5a4d8b` (0x3b87f2) | (state, env) | DualReductions && PREDERIVED && PreCrush==0 | 오류 | count, **cb만** |
| `5a5eff` (0x3b8844) | (state, env) | 5a4d8b 바로 뒤, 무조건 | 오류 | count, chk |
| `5aaaf7` (0x3b88df) | (state, env) | DualReductions && PREDUALREDUNDANT | 오류 | count, chk |
| `838543` (0x3b8976 / 0x3b8a0d / 0x3b8aa4) | (env, state, mipctx==NULL, **k = 2, 3, 5**, W) | 없음 | 오류 | 각각 count, chk |
| `579e67` (0x3b8b25) | (state, env) | 없음 | 오류 | count, chk |
| `4d3a1d` (0x3b8c05) | (model, mipctx, state, env, &T9, max(probe_mode,2), 0, 0, 0, 0) | probe_mode≠0 && usage(T9)<0.8 | 오류 | count, chk |
| `414ca3` (0x3b8cf8) | (state, env, &T18) | PRECLIQSUB≠0 && usage(T18)<0.8 | 오류 | count, chk |
| `5a04fc` (0x3b8da2) | (state, env) | PRESMARTDISAGG ≥ 2 | 오류 | count, chk |
| `5a7c0f` (0x3b8e3d) | (state, env) | PREDOMINATE≠0 && rs_a8>0 | 오류 | count, chk |
| `45feaa` (0x3b8f3d) | (state, env, &T7, 2) | (PRETWOROWS==2 \|\| (rs_a8>0 && PRETWOROWS==−1 && Presolve≠1)) && usage(T7)<0.8 | 오류 | count, cb. 그 뒤 3ba9de와 stop은 건너뛴 경우에도 한다 |

- **`838543` 세 번**: mode는 법 k다. 안에서 `(v>>31 & k) + v`로 음수 나머지를 고치고, `83c8d1(env, k, m, n, lb, ub, …)`로 mod-k 행렬을 만든다(cut 모듈의 mod-k/zero-half 루틴을 재사용). 세 번째 인자는 mipctx가 없으면 1이다. 안에서 `local_108`로 쓰고, 0이면 다른 분기를 탄다(L417, L436). 노트 12가 "PRESS2MIP/PREPROBE0/PRECOVERS 블록"이라고 한 것은 틀렸다. 그 세 파라미터는 각각 `594c0c`, `423e35`, `419ffc`의 가드다.
- **`4d3a1d` 두 호출 지점**:
  - 라운드 끝 0x3b8c00: (model, mipctx, state, env, &T9, max(probe_mode,2), 0, 0, 0, 0)이고 timer 9를 쓴다.
  - 마무리 0x3bcd40: (model, mipctx, state, env, &T8, max(probe_mode',2), 1, 1, flag, 1)이고 timer 8을 쓴다. flag는 0x3bccd9/0x3bccde에서 정하는 r13b다. 마무리 단계 담당이다.
  - `3cd406` 안에 다른 호출 지점은 없다.
- `5577ba`: 여섯 번째 인자는 LP의 0과 달리 1이다. 그래서 대칭을 쓰는 블록(type 19 `3a0a17`, `564ce0`, `397538`)이 돌 수 있다. coverage에서 `564ce0`은 1개 모델에서 불렸다. 노드 한도는 20000(Symmetry ≥ 2면 50000)이다. work 한도는 timer 24의 (한도 − 사용량)이고, 처음에는 2B다. Symmetry ≥ 2이고 그 값이 1e100보다 작으면 10배로 한다.
- `55a779`의 가드는 asm(0x3b831d)에서 env+0x2680 = **PreSparsify ≠ 0**이다. 노트 12는 이 함수를 "집계 패스"로 분류했다.
- 이 구역은 추적기 레벨을 바꾸지 않는다. 바꾸는 것은 들어올 때의 레벨 0과 스냅숏 뒤의 `5369b8` reset 하나뿐이다.
- 스냅숏과 진척도 판정은 다음 세 가지다.
  - 라운드 스냅숏: `3c546c(0xb50)`.
  - 라운드 시작 판정: `3c5595(0xb50)` ≤ 0.01/0.05.
  - 라운드 끝 판정: `3c5595(0xab0)` ≤ 1e-4.

## 7. 교차 확인

### 7.1 callsites.tsv에서 이 구역 (복귀 주소 0x3b7b7c–0x3b9026)
- 38/38 모델:
  - 3ba9de@0x3b7ba3, 3c546c@0x3b7cc9, 5369b8@0x3b7cdd.
  - 58d4ce@0x3b7d3c, adf205@0x3b7f8e, adf1f2@0x3b7f9f, 5577ba@0x3b805c.
  - 5a9282@0x3b8117, 5ab429@0x3b81b1, 59c193@0x3b825a, 55a779@0x3b8356, 594c0c@0x3b8452.
  - adf236@0x3b8500 → 423e35@0x3b8542, adf236@0x3b85fd → 419ffc@0x3b863e.
  - 5a1293@0x3b871b, 5a4d8b@0x3b87f2, 5a5eff@0x3b8844, 5aaaf7@0x3b88df.
  - 838543@0x3b8976/0x3b8a0d/0x3b8aa4, 579e67@0x3b8b25.
  - adf236@0x3b8ba9 → 4d3a1d@0x3b8c05, 5a04fc@0x3b8da2.
  - 3c5595@0x3b8fd7, 3ba5ac@0x3b8fff.
  - 각 reduction 뒤의 3ba5ac/3ba9de와 ade8ca(0x3b7d24, 0x3b802a, 0x3b8242, 0x3b833a, 0x3b8436, 0x3b8525, 0x3b8622, 0x3b8706, 0x3b8bce).
- 9개 모델: 3c5595@0x3b7bf1, 3ba5ac@0x3b7c19. 라운드 시작 판정이다.
- 0회:
  - 586049, 57b2bf: 지점 0x3b7e11, 0x3b7ed1.
  - 414ca3, 5a7c0f, 45feaa: 지점 0x3b8cf8, 0x3b8e3d, 0x3b8f3d.
  - 0x3b8f97 3ba9de는 45feaa 가드 안에 있다.
  - 열 타입 통계는 호출이 없어 보이지 않는다.
- ade770과 536971은 기록에 없다(훅이 걸리지 않음). adf00e도 없다. adf205가 참이면 asm상 불리므로 이것도 훅 누락으로 본다.

38개 모델의 라운드 구성(`cov2.json`에서 0x3b7ba3/0x3b7bf1/0x3b7d3c/0x3b8fd7/0x3b905e 횟수로 셈):

| 유형 | 모델 | 흐름 |
|---|---|---|
| 1라운드 (29개) | neos5, cod105, mas74, csched008 등 | round 0 끝 판정 ρ ≤ 1e-4 → FINAL. 마무리 ρ(0x1a8) ≤ 1e-4 |
| 2라운드 (4개) | timtab1, gmu-35-50, gmu-35-40, enlight_hard | round 0 끝 ρ > 1e-4 → round 1. round 1 시작 판정 ρ > 0.01 → 축약 16개 → 끝 ρ ≤ 1e-4 → FINAL |
| round 1에서 바로 끝 (1개) | tr12-30 | round 0 끝 ρ > 1e-4 → round 1. 시작 판정 ρ ≤ 0.01 → FINAL |
| 바깥 재시작 (4개) | glass4, graphdraw-domain, ic97_potential, neos-4954672-berkel | round 0 끝 → FINAL. 마무리 probing 뒤 ρ(0x1a8) > 1e-4 → outer 1, round 0. 그 라운드 끝 시작 판정 ρ ≤ 0.05 → FINAL. outer==1이라 다시 돌아가지 않는다 |

round 2 이상에 닿는 모델은 없다. 그래서 thr의 "round ≥ 2" 분기는 오라클로 확인되지 않았다.

### 7.2 네 모델 (seq 번호)
- **neos5** (1패스, 1라운드): 231 3ba9de → 232 3c546c → 233 5369b8 → 58d4ce(235) … 5a04fc(301) → 304 3c5595(0x3b8fd7) → 305 3ba5ac → 306 3ba9de@0x3b905e(FINAL). 시작 판정 호출이 없는 것은 outer=round=0이기 때문이다. 축약 19개의 순서는 §2와 같다.
- **glass4**: 614–688은 round 0 끝(19개) → 689 FINAL → 693 마무리 4d3a1d → 696 3c5595@0x3bcde2(> 1e-4) → 697 3c546c@0x3b1d1e(바깥 재시작, round 0). 패스 끝 885, 1351 → 1352 3ba9de, 1353 3c5595@0x3b7bf1(thr 0.05), 1354 3ba5ac → 1355 3ba9de@0x3b905e(FINAL). 두 번째 방문에는 축약 호출이 없다.
- **timtab1**: round 0 끝 974–1048(19개). 1047 판정이 > 1e-4라 1049부터 round 1의 패스가 돈다. 1475 3ba9de, 1476 3c5595@0x3b7bf1(thr 0.01, 통과) → 1478 3c546c … 5577ba·55a779·594c0c 없이 16개 → 1536 판정 ≤ 1e-4 → 1538 FINAL.
- **gmu-35-50**: round 0(패스 끝 1257, 2569, 3788) 끝 3789–3863(19개). 3862 판정이 > 1e-4라 round 1(패스 끝 4991). 5026 3ba9de, 5027 시작 판정(0.01) 통과 → 16개 → 5087 판정 ≤ 1e-4 → 5089 FINAL.

### 7.3 PRELIMIT 번호 (추정)
이 구역 안의 상대 번호(§3)는 asm으로 확정했다. 전체 번호는 다른 구역의 카운트 규칙에 달려 있다.

정적으로 찾은 "호출 없이 세는 곳"은 다음과 같다. 다른 담당에게 넘긴다.
- 0x3b11ea: 프렐류드 정수 bound 반올림. 인라인, 1회.
- 0x3b21de: 내부 루프 I→B 블록(`5383d0` 호출 뒤). 반복마다.
- 0x3b6809: 패스 루프 인라인 열 implied bound 블록. `5379f1`를 부르지 않아도 센다.
- 0x3b7248: 패스 루프 `3c2f36` 블록. 부르지 않아도 센다.
- 0x3bb986, 0x3bbedc: `538f30` activity 블록.
- 0x3ba1be: 마무리 단계 `3df916`.
- 0x3b0e7f(카운터 = 1로 대입), 0x3b18c1(mipctx 전용).

이것들을 넣어 세면(앞의 다섯 곳 반영) sweep 변화점이 그럴듯하게 들어맞는다.
- enlight_hard: k=56이 `838543(k=2)`, 57이 `838543(k=3)`이다. lights-out의 GF(2) 소거로 x 100개가 고정되고, 다음 mod 3으로 남은 단일항 행이 풀린다(57.mps는 빈 모델).
- glass4: k=76이 마무리 단계 `4d3a1d`다. 바깥 재시작의 원인과 맞는다.
- gmu-35-50: k=148이 round 0 끝의 `4d3a1d`다.
- timtab1: k=80과 k=160이 각 라운드 끝의 첫 카운트 `58d4ce`이고, 둘 다 c32의 RHS −2.84e-14를 지운다. 그런데 `58d4ce`는 Q(+0x250)가 없으면 아무것도 하지 않는다. 그래서 이 변화는 그 사이의 단계 진입 처리(레벨 0, `5369b8` reset)에서 온 것으로 보인다(§8-3).

이 구역의 번호는 이 셈법으로 다음과 같다.
- neos5: k32–50.
- glass4: k57–75.
- timtab1: k80–98, k160–175.
- gmu-35-50: k131–149, k188–203.

## 8. 미확인 사항

1. **probe_mode를 3으로 올리는 프렐류드 조건**(0x3b35b1–0x3b3632). 이진 열 수 < 20, 무언가 < 50, 비율 비교가 들어 있다. 이 값이 thr(round ≥ 2), 통계 가드, `423e35`의 4번째 인자, timer 3B를 바꾼다. 프렐류드 담당이 해독하면 정해진다. 모델마다 확인하려면 Frida로 `4d3a1d`의 6번째 인자를 기록하면 된다(`max(pm,2)`이므로 3이면 보인다).
2. **열 타입 통계가 실제로 켜지는 모델.** 호출이 없어 coverage에 보이지 않는다. Frida로 0x3bdd47(`movl $1,0xf8(%rdi)`)에 훅을 걸거나, 포트에서 같은 식을 돌려 비교하면 된다. 켜지면 `4d3a1d` 동작(f8 분기)과 timer 8/9/10 한도가 바뀐다.
3. **PRELIMIT 오라클 출력이 라운드 끝 첫 카운트에서 바뀌는 이유**(timtab1 k=80, 160 추정). 기대: `LAB_004b2415` 마무리가 추적기 레벨이나 목록에 따라 다르게 정리한다. 마무리 담당의 `LAB_004b2415` 해독, 또는 PRELIMIT=79/80에서 Frida로 추적기 상태를 비교하면 정해진다. 포트가 같은 k에서 같은 출력을 내려면 꼭 필요하다.
4. **전체 PRELIMIT 번호.** §7.3은 다른 구역의 "호출 없이 세는 곳"을 정적으로 넣은 추정이다. 각 담당의 카운트 표를 합쳐 sweep과 다시 맞추면 확정된다. 또는 Frida로 0x3b8c36 등 `inc %ebx` 지점에서 ebx를 기록하면 된다.
5. **round ≥ 2와 outer 1의 round ≥ 1 경로.** 38개에서 0회다. thr 0.05(round ≥ 2)와 outer 1의 round 1 이후는 asm으로만 읽었다. PrePasses나 모델을 바꿔 3라운드 이상 도는 경우를 찾아 seq로 확인하면 된다.
6. **`586049`의 세 번째 정수 인자.** rdx는 직전 호출이 남긴 값으로 정해지지 않은 상태다. SOS 모델에서 `586049` 디컴파일을 읽거나 Frida로 확인하면 된다(기본값 38개에는 없음).
7. **callee 쪽 timer 사용.** `419ffc`(`adf2be(param_3)`), `423e35`, `55a779`, `594c0c`, `414ca3`, `45feaa`는 넘겨받은 timer로 내부 중단을 판정하는 것으로 보인다. 각 callee 해독 때 정한다.
8. Frida 기록에 `ade770`, `536971`, `adf00e`가 없다. 훅을 걸지 않은 것으로 보이는데 확인하지 않았다. 이 구역의 결론은 asm에 기댄다.
