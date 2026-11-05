# M2 MIP presolve: `3cd406` 마무리 단계 해독 노트 (spine D, `LAB_004b9027`)

[m2_presolve.md](m2_presolve.md) §2의 "라운드 후 → 마무리" 사이 구간이다. 2026-11-06에 디컴파일(`re/decomp/c/003b0530_PRIVATE00000000003cd406.c`)과 asm(`objdump`, libgurobi130.so 13.0.3)으로 읽었다. Ghidra가 변수를 합치거나(아래 §1.3) 인자를 빠뜨린 곳(`4d3a1d`, `451195`, `490ab6`, `adf1a6`)은 모두 asm으로 정했다. 주소는 file offset(= Ghidra − 0x100000)이다.

- 기호: `st` = presolve 상태(local_150, rsp+0xb48), `env`(lVar119, rsp+0xa88), `model`(param_1, rsp+0xa90), `mipctx`(param_3 = local_78, rsp+0xc20. `Model.presolve()`에서는 0), `W` = 전역 work 카운터(`*param_5` = `*(st+0x458)`, 레지스터 r14), `cnt` = PRELIMIT 카운터(local_68), `lim` = st+0x1dc(GURO_PAR_PRELIMIT, 예산 초과 시 −1), `μ` = st+0x448.
- 파라미터 이름은 `docs/m2_param_offsets.tsv`(정수)와 노트 12·m1 노트(실수: INFBOUND 0x2458 = 1e30, FeasibilityTol 0x2460, IntFeasTol 0x2468, OptimalityTol 0x2470)를 따른다.
- 노드(32바이트): +0 값, +8 행, +0xc 열(음수 = 죽은 노드), +0x10 행 안 다음, +0x18 열 안 다음. 행 머리 st+0x178[i], 열 머리 st+0x180[j].

## 0. 요약

- 이 구간은 두 부분이다. (a) **두 번째 probing**(`4d3a1d`)과 진척도 판정: ρ > 1e-4면 **라운드 루프를 한 번 더**(LAB_004b1ca9) 돌린다. (b) **마무리 reduction 사슬**: 연결 성분 `52cba4`, `521497`, (Q/SOS/NL 블록), sparsify `470936`, (Q/SOS/PWL 블록), 작은 계수 `57999f`, {`36ddf2` + 인라인 빈 열 쌍대 고정} 반복, 인라인 implied-integer 척도 바꾸기(type 9), SYMMTRANS `54807f`. 그 뒤 `LAB_004b2415`(마무리, 다른 담당)로 간다.
- 기본값 38개 모델에서 이 구간의 PRELIMIT 카운트는 7(두 번째 probing 없음) 또는 8이다. 순서: [probing] → `52cba4` → `521497` → `470936` → `57999f` → `36ddf2` → 쌍대 고정 → implied int. 38/38이 이 순서와 맞고(§7), 쌍대 고정 루프는 38개 모두 한 바퀴다.
- 3cd406 자신은 이 구간에서 `W`를 한 번도 바꾸지 않는다(§4). work는 callee 안에서만 늘어난다.
- sweep으로 본 이 구간의 모델 변화: 두 번째 probing(glass4·graphdraw-domain·ic97_potential·neos-4954672-berkel·tr12-30), sparsify(neos-860300), implied int(열 척도: neos-911970·neos-860300, 행 척도만: gen-ip054·neos-2657525-crna·neos-3754480-nidda). §7.3.

## 1. 범위, 진입, 출구

### 1.1 범위
| 부분 | 디컴파일 줄 | 기계어 |
|---|---|---|
| 진입, 두 번째 probing, 재시작 판정 | L6582–7098 | 0x3b9027–0x3b908e, 0x3bcb21–0x3bce98, 0x3bce89–0x3bd388('C' 개수 SIMD) |
| SOLREDUCTION·PWL·성분·sparsify·Q/SOS 사슬 | L7099–7527, **L10255–10361**(Ghidra가 마무리 뒤에 둔 조각: `451195`(2번째), `42d072`, `5b1161`(2번째), `5852f1`, `583cfb`) | 0x3b908e–0x3b9ea6, 0x3bc2ea–0x3bc354, 0x3bca69–0x3bcb1c |
| `36ddf2` + 쌍대 고정 루프 | L7528–7694 | 0x3b9ea6–0x3ba218, 0x3bc9ad–0x3bca64, 0x3bc354 |
| implied int 척도 | L7695–9566 | 0x3ba220–0x3bb9c4, 0x3bc388–0x3bc9a8 |
| SYMMTRANS | L9567–9600 | 0x3bb9c4–0x3bbaeb |

0x3bbaf0–0x3bbf70(`538f30`, `36dce1` 호출, L2190 근처)은 패스 루프 코드다. 물리적으로 이 구간 사이에 있지만 담당이 아니다.

### 1.2 진입 (라운드 끝 → 0x3b9027)
라운드 끝 단계(L4400–6450, 다른 담당)에서 들어오는 길은 셋이다.
1. 라운드 끝 마지막(0x3b8fab–0x3b9021): `3ba9de(&stop)` 뒤 `stop`이면 바로. 아니면 ρ = `3c5595(st, snap@rsp+0xab0)`, `3ba5ac`, ρ ≤ 1e-4면 진입. 아니면 round(r13b)+1 < 5면 다음 라운드(0x3b1cb4), 5면 진입(fall-through).
2. 라운드 끝 시작(0x3b7c33, L4419): ρ(rsp+0xb50 스냅숏 대비) ≤ rsp+0x2d0(재시작 뒤 r15b ≥ 1이면 0.05. r15b = 0일 때 값은 라운드 끝 담당)이면 진입.
3. 라운드 끝 reduction마다 `3ba9de(&stop)` 뒤 `stop ≠ 0`이면 진입(0x3b7bb3, 0x3b7dbf–0x3b8ea3의 22곳, 0x3b8fc0. L4405, L5303–6003).

진입 때 레지스터 약속(asm으로 확인):
| 레지스터 | 뜻 |
|---|---|
| ebx | `cnt`(레지스터에 있다. 0x3b908e에서 rsp+0xc30으로 내림) |
| ebp | `probe_mode` = local_58(rsp+0xc40). 처음 = PREPROBE(`999702`가 `3db19d`의 param_2로 넘김, 기본 1), 프렐류드의 대형 모델 규칙(L880–933)에서 3이 될 수 있다 |
| r12d | 지금까지의 전체 패스 수. 진입하자마자 local_40(rsp+0xc58)에 둔다 |
| r13b | 라운드 번호 |
| r15b | `restart`(cVar125, rsp+0x2f8): 처음 0, 재시작 뒤 1 |
| r14 | `param_5`(work 포인터) |
| rsp+0xb94 | `stop` = local_104(PRERESTART 플래그, `3ba9de`의 셋째 인자) |

### 1.3 Ghidra 변수 주의
- 디컴파일의 `iVar129`/`iVar115`는 이 구간에서 ebp/r12d와 일대일이 아니다(L6594 `iVar115 = iVar129`, L6971 `iVar129 = 2`, L7087 `iVar129 = iVar115`). asm 기준으로 ebp = probe_mode(바뀌지 않음), r12d = pr(probing에 넘길 모드, 'C' 분기에서 2가 됨)이다.
- `local_60`(rsp+0xc38) = 성분 블록 실행 플래그, `local_154`(rsp+0xb44) = 성분 수(`52cba4`의 넷째 인자), `local_48`(rsp+0xc50) = 'C' 비율 플래그. 셋 다 함수 시작(0x3b066f 등)에서 0이고 이 구간에서만 쓴다.
- `local_a0`(rsp+0xbf8) = mipctx→+0x620→+0x7fc, `local_a8`(rsp+0xbf0) = →+0x808, `local_70`(rsp+0xc28) = →+0x800. mipctx = 0이면 모두 0.

### 1.4 출구
- **마무리** `LAB_004b2415`(0x3b2415): PRELIMIT 카운트 검사가 걸린 곳(0x3b23f5, 0x3bc2b6, 0x3bc324, 0x3bc9c5)과 정상 끝(0x3b23e5)에서. 모두 rsp+0xc30 = `cnt`, rsp+0xaa0 = `st`를 맞추고 r13d = `cnt`로 들어간다. 마무리 담당은 이 r13d를 이어 쓴다.
- **오류·상태 출구** `LAB_004b2dee`(0x3b2dee, 모델 생성 없음): callee가 0이 아닌 값(상태 3/5 포함), `3ba5ac`/`3ba9de`가 0이 아닌 값, 인라인 블록의 3(불가능)/5(비유계)/0x2711(메모리). 경유 0x3bc28e, 0x3bce43, 0x3bc9ad, 0x3bc314, 0x3bc658.
- **재시작** `LAB_004b1ca9`(0x3b1ca9, 라운드 루프 담당): r12d = local_40(패스 수는 이어서 센다), r13b = 0(라운드 0부터), r15b = 1. 다시 0x3b9027에 오면 r15b = 1이라 probing 없이 r15b = 2로 (b)에 들어간다. 재시작은 최대 한 번이다.

## 2. 의사 코드 (기본 파라미터 경로)

`count!()` = `cnt += 1; if cnt >= lim { goto FINISH }`(검사가 `3ba5ac`보다 먼저다). `cb()` = `3ba5ac(env, st, 0, 0, -1)`, 0이 아니면 그 값으로 EXIT. `budget(p)` = `3ba9de(env, st, p)`, 0이 아니면 EXIT. `p = Some(&mut stop)`일 때만 `stop`이 새로 정해진다. callee가 0이 아닌 값을 돌려주면 언제나 EXIT(그 값). 타이머 T_k = rsp+0x380+0x40·k(§4.2). `start(T)`/`stop_t(T)` = `ade8ca`/`ade770`.

```rust
// ===== 0x3b9027 (L6582) =====
local_40 = r12d;
tracker(st+0x3d8).set_level(0);                     // 536971(trk, 0). 이 구간에 5369b8(reset)은 없다
budget(Some(&mut stop))?;                           // 0x3b9059
if stop { goto POST_SPARSIFY; }                     // 0x3bce17 → 0x3b9729
if restart == 0 {
    // ---- 두 번째 probing (§5.1) ----
    let snap = 3c546c(st);                          // rsp+0x1a8
    let mut pr = probe_mode;                        // r12d
    let mut c_flag = false;                         // local_48
    if probe_mode == 1 && env[0x2728] == 0 && st.n(+0xc) > st.n_int(+0xf4)
        && count_C_alive() as f64 > (st.n - st.ncol_del(+0x408)) as f64 * 0.05 {
        c_flag = true; pr = 2;                      // 0x3bd372: pr만 바뀌고 probe_mode는 1 그대로
    } else if probe_mode <= 1 { goto AFTER_PROBE; } // 0x3bcb42
    if !(adf236(T8, W) < 0.8) { goto AFTER_PROBE; }
    let flag = if PRELIFTGROW(0x28a4) == 0 { false }
        else if PREPROBE(0x2828) != 1 || Presolve(0x2424) != -1
             || (mipctx != 0 && (mipctx[0x32a8] == null || mipctx[0x32a8].i32[1] <= 0)) { true }
        else { obj_nonzeros_ge2(st) && (pr >= 4 || c_flag) };
    if !flag && probe_mode <= 1 { goto AFTER_PROBE; }   // 0x3bccde
    start(T8);
    r = 4d3a1d(model, mipctx, st, env, T8, max(pr, 2), 1, 1, flag as i32, 1);  // 0x3bcd40, 인자 7–10은 스택
    stop_t(T8);  r?;
    count!();                                       // C1 (ebx)
    cb()?; budget(Some(&mut stop))?;
    if stop { goto POST_SPARSIFY; }
  AFTER_PROBE:
    if 3c5595(st, snap) > 1e-4 { restart = 1; goto ROUND_RESTART; }   // LAB_004b1ca9
} else { restart = 2; }

// ===== 0x3b908e (L7099) =====
if stop { goto POST_SPARSIFY; }
if SOLREDUCTION(0x2590) >= 1 && DualReductions(0x2430) >= 1 && st.m(+0x10) - st.nrow_del(+0x404) == 1 {
    490ab6(1e6, min(env.f64[0x2938], 1e6), env, st, …)?;   // 행 1개 남은 모델 직접 풀기. 38개에서 0회
    count!(); cb()?;                                // C2
    if stop { goto FLAG_CHECK; }
}
if st[+0x330] > 0 {                                 // (기본값 거짓: PWL 목적)
    58b41b(st, env)?; count!(); cb()?; budget(Some(&mut stop))?;       // C3
    if stop { goto POST_SPARSIFY; }
}
let mut ncomp = 0;                                  // local_154
let mut comp_ran = false;                           // local_60
if st[+0x1e4] != 0 && PRECONNECTED(0x289c) != 0 && DualReductions != 0
   && (mipctx == 0 || mipctx[0x620].i32[0x35f4] == 0) {
    start(T21); 52cba4(st, env, T21, &mut ncomp)?; stop_t(T21);        // 38/38
    count!(); cb()?;                                // C4
    if stop { goto POST_SPARSIFY; }
}
if PREBICONNECTED(0x28a0) != 0 && DualReductions != 0 {
    start(T22); 521497(st, env, T22)?; stop_t(T22);                    // 38/38
    count!(); cb()?;                                // C5
    if stop { goto FLAG_CHECK; }
}
// ---- 성분 블록 (Q/SOS/NL 전용, ncomp > 0일 때만) ----
if ncomp > 0 {
    if st.nsos(+0x1f8) - st.sos_del(+0x410) > 0 {
        586049(INFBOUND, env, st)?; count!(); cb()?; comp_ran = true;  // C6
        if stop { goto FLAG_CHECK; }
    }
    if st.nqc(+0x27c) != 0 {
        451195(env, st, 0, 0, 1, local_70)?; count!(); cb()?; comp_ran = true;   // C7
        if stop { goto E39; }
    } else if stop { goto FLAG_CHECK; }
    if st[+0x2f8] != 0 || st[+0x310] != 0 {
        5b1161(env, st, local_a0, local_a8, 1)?; count!(); cb()?;     // C8
        goto E39;
    }
} else if stop { goto FLAG_CHECK; }
if comp_ran { goto E39; } else { goto SPARSIFY; }
FLAG_CHECK: if comp_ran { goto E39; } else { goto POST_SPARSIFY; }   // 0x3b9454
E39:                                                // 0x3b95a1, 성분 블록이 돌았을 때만
    start(T0); 3e39a6(st, env)?; stop_t(T0); count!(); cb()?;         // C9
    if stop { goto POST_SPARSIFY; }
SPARSIFY:                                           // 0x3b9639
    if env[0x2728] == 0 && Presolve != 0 && PreSparsify(0x2680) != 0
       && (PreSparsify > 0 || mipctx == 0 || mipctx[0x32a8] == null)
       && adf236(T20, W) < 0.8 {
        start(T20); 470936(st, env, T20, local_a0, local_a8)?; stop_t(T20);   // 38/38
        count!(); cb()?;                            // C10
    }
POST_SPARSIFY:                                      // 0x3b9729 (L7152)
budget(None)?;                                      // stop은 그대로
if !stop && sos_left() > 0 { 586049(INFBOUND, env, st)?; count!(); cb()?; }          // C11 (SOS 전용)
if sos_left() > 0 && (env.f64[0x2640] != 0.0 || env.f64[0x2648] != 0.0) {           // PreSOS1BigM/PreSOS2BigM (SOS 전용)
    57b2bf(env, st, (local_a8 > 0) as i32, W)?; cb()?;                               // 카운트 없음
}
if st[+0x228] > 0 { 585a6e(INFBOUND, env, st)?; count!(); cb()?; }                   // C12 (기본값 거짓)
if stop { goto DUALFIX; }
if st[+0x330] > 0 { 58b41b(st, env)?; count!(); cb()?; if stop { goto DUALFIX; } }  // C13 (PWL)
if st.qdiag(+0x250) != 0 && DualReductions != 0 && Presolve != 0 {                   // (Q 전용)
    42872b(env, st)?; count!(); cb()?; if stop { goto DUALFIX; }                     // C14
}
if st.qdiag(+0x250) != 0 && DualReductions != 0 && Presolve != 0 {                   // (Q 전용, PREQOBJPROBE)
    loop {
        429e1b(env, st)?; count!(); cb()?;          // C15
        let c0 = st.ncol_del;
        if PREQOBJPROBE(0x25bc) == 0 || st[+0x258] == 0 { break; }
        42ba68(env, st)?; count!(); cb()?;          // C16
        if st.ncol_del == c0 { break; }             // 42ba68이 열을 지우는 동안 반복
    }
    if stop { goto DUALFIX; }
}
if st.nqc != 0 { 451195(env, st, 0, 0, 1, local_70)?; count!(); cb()?; if stop { goto DUALFIX; } }   // C17 (QC)
if st.nqc != 0 && PreQLinearize(0x2580) > 0 { 42d072(env, st)?; count!(); cb()?; if stop { goto DUALFIX; } }  // C18
if st[+0x2f8] > 0 || st[+0x310] != 0 { 5b1161(env, st, local_a0, local_a8, 1)?; count!(); cb()?; if stop { goto DUALFIX; } }  // C19 (일반 제약/NL)
if 1e100 > env.f64[0x2650] { 5852f1(env, st, W)?; cb()?; if stop { goto DUALFIX; } }   // 카운트 없음 (38개에서 0회)
if sos_left() > 0 { 583cfb(env, st, W)?; cb()?; if stop { goto DUALFIX; } }           // 카운트 없음 (SOS 전용)
if Presolve != 0 { 57999f(st, env); count!(); cb()?; }      // C20, 반환값 무시(void)
DUALFIX:                                            // 0x3b9ea6 (L7528)
loop {
    36ddf2(st, env)?; count!();                     // C21 (3ba5ac 없음)
    let c0 = st.ncol_del;                           // 36ddf2 다음에 뜬다
    if DualReductions != 0 { dual_fix_empty_cols()?; }      // §5.2. 3/5/0x2711 → EXIT
    count!(); cb()?;                                // C22: DualReductions = 0이어도 센다
    if !(c0 < st.ncol_del) { break; }               // 쌍대 고정(39ab33/3df916)이 열을 지웠을 때만 반복
}
if !stop && env[0x2728] == 0 && IMPLIEDINT(0x2820) != 0 && Presolve != 0 {
    implied_int_scale()?;                           // §5.3 (0x2711, 3a061e 오류 → EXIT)
    count!(); cb()?;                                // C23
}
budget(None)?;                                      // 0x3bb9d1
if !stop && Symmetry(0x2864) != 0 && DualReductions != 0 && env[0x2728] == 0 && SYMMTRANS(0x27b8) != 0
   && (SYMMTRANS >= 2 || (local_a8 > 0 && mipctx != 0 && mipctx[0x620].i32[0x3480] >= 0)) {
    54807f(env, st)?;                               // 카운트 없음. 기본값 거짓(SYMMTRANS −1, mipctx 0)
    if mipctx != 0 {
        if st[+0xec] == 0 { mipctx[0x620].i32[0x3480] = -1; }
        else { mipctx.i32[0x34c8] = 0; mipctx[0x620].i32[0x3480] = 1; }
    }
}
goto FINISH;                                        // LAB_004b2415
```

보충:
- `sos_left()` = st+0x1f8 − st+0x410 > 0. `count_C_alive()` = #{j < n : st+0x68[j] ≥ 0 ∧ st+0x150[j] = 'C'}(SIMD로 16개씩 세지만 결과는 같다).
- `obj_nonzeros_ge2`: c = st+0xb0에서 처음으로 c ≠ 0(NaN 포함)인 j0을 찾고, j0+1..n에서 c ≠ 0인 것이 하나 더 있으면 참.
- 기본값(PRELIFTGROW 1, PREPROBE 1, Presolve −1, mipctx 0)에서 두 번째 probing은: probe_mode = 1이면 'C' 비율 > 5%이고 목적 비영 ≥ 2일 때만 `4d3a1d(…, 2, 1, 1, 1, 1)`. probe_mode = 3(대형 모델)이면 늘 `4d3a1d(…, 3, 1, 1, 0, 1)`(타이머만 맞으면).
- (기본값 거짓) 블록 한 줄 요약: `490ab6` 행 1개 남은 모델 직접 풀기(SOLREDUCTION), `58b41b` PWL 목적, 성분 블록 `586049`/`451195`/`5b1161`/`3e39a6`(SOS/QC/일반 제약이 있고 성분 > 0), `57b2bf`/`583cfb`/`586049` SOS, `585a6e` st+0x228 > 0, `42872b`/`429e1b`/`42ba68` Q 목적, `451195`/`42d072` QC·PreQLinearize, `5b1161` 일반 제약/NL, `5852f1` env+0x2650 < 1e100(이름 미확인, 38개에서 0회), `54807f` SYMMTRANS.

## 3. PRELIMIT 카운트 지점 (프로그램 순서)

모든 검사는 `cnt >= st+0x1dc`(signed, `jge`)이고 걸리면 `LAB_004b2415`로 간다. `3ba9de`가 예산 초과로 st+0x1dc = −1을 쓰면 다음 카운트 지점에서 반드시 나간다. 카운트가 없는 callee(`57b2bf`, `5852f1`, `583cfb`, `54807f`) 뒤에는 검사도 없다.

| # | 주소(inc) | 무엇 뒤 | 조건(호출이 0을 돌려준 뒤) | 기본값 38개 |
|---|---|---|---|---|
| C1 | 0x3bcd6d | `4d3a1d`(두 번째 probing) | restart = 0이고 probing을 불렀을 때 | 19개 |
| C2 | 0x3bcac4 | `490ab6` | SOLREDUCTION·DualReductions ≥ 1, 남은 행 1 | 0 |
| C3 | 0x3b912d | `58b41b` | st+0x330 > 0 | 0 |
| C4 | 0x3b924e | `52cba4` | §2 조건 | 38 |
| C5 | 0x3b930a | `521497` | PREBICONNECTED·DualReductions | 38 |
| C6 | 0x3b93af | `586049` | ncomp > 0, SOS 남음 | 0 |
| C7 | 0x3b949d | `451195` | ncomp > 0, QC | 0 |
| C8 | 0x3b9562 | `5b1161` | ncomp > 0, 일반 제약/NL | 0 |
| C9 | 0x3b95ec | `3e39a6` | 성분 블록이 돌았을 때 | 0 |
| C10 | 0x3b96ea | `470936` | sparsify 조건과 T20 < 0.8 | 38 |
| C11 | 0x3b97a6 | `586049` | !stop, SOS 남음 | 0 |
| C12 | 0x3b98b4 | `585a6e` | st+0x228 > 0 | 0 |
| C13 | 0x3b9944 | `58b41b` | st+0x330 > 0 | 0 |
| C14 | 0x3b99ef | `42872b` | Q 대각, DualReductions, Presolve | 0 |
| C15 | 0x3b9a9f | `429e1b` | 같은 조건, 루프마다 | 0 |
| C16 | 0x3b9b05 | `42ba68` | PREQOBJPROBE, st+0x258, 루프마다 | 0 |
| C17 | 0x3b9bb9 | `451195` | QC | 0 |
| C18 | 0x3b9c56 | `42d072` | QC, PreQLinearize > 0 | 0 |
| C19 | 0x3b9cfa | `5b1161` | st+0x2f8 > 0 또는 st+0x310 ≠ 0 | 0 |
| C20 | 0x3b9e57 | `57999f` | Presolve ≠ 0 | 38 |
| C21 | 0x3b9ee4 | `36ddf2` | 루프마다(뒤에 `3ba5ac` 없음) | 38(각 1회) |
| C22 | 0x3ba1bb | 인라인 쌍대 고정 | 루프마다, **DualReductions = 0이어도** | 38(각 1회) |
| C23 | 0x3bb975 | 인라인 implied int | 블록을 돌았을 때만(가드 거짓이면 세지 않음) | 38 |

- C1의 카운터는 레지스터 ebx, C2–C20·C23은 메모리(rsp+0xc30), C15–C16·C21–C22는 r12d다. 값은 하나다.
- 기본값 38개의 이 구간 카운트 = [C1] + C4 + C5 + C10 + C20 + 2·(루프 바퀴 = 1) + C23 = 7 또는 8.
- 출력 비교 주의: PRELIMIT = k의 출력은 "카운트 k가 붙은 호출까지"다. 카운트 k와 k+1 사이의 카운트 없는 호출(`57b2bf`, `5852f1`, `583cfb`)은 k+1에 속한다. C21과 C22는 `36ddf2`와 쌍대 고정을 따로 센다.

## 4. work와 타이머

### 4.1 work(`W`)
- **3cd406 자신은 이 구간에서 `W`를 바꾸지 않는다.** asm 0x3b9027–0x3bb9f0, 0x3bc2b6–0x3bd388에 st+0x448/+0x458 읽기와 (%r14) 쓰기가 없다(0x3bbb35·0x3bbdfe·0x3bbe8b는 패스 루프 코드). 인라인 쌍대 고정과 implied int 블록도 work를 더하지 않는다.
- work는 callee 안에서만 늘어난다: 각 reduction, `39ab33`(`st+0xfc = 0 ∧ st+0x278 = 0`일 때만 `(2·전체 노드 + 4·살아 있는 노드)·μ`. 빈 열은 0), `3df916`(여러 항, 별도 해독), `57999f`(시작에 `st+0xd8·μ`). `3a061e`, `a7bea8`은 work 없음.

### 4.2 타이머
타이머는 rsp+0x380부터 0x40바이트씩 28개(T0–T27)다. `ade8ca(T, W)`: `T[0] = T[0] − W + 1.0`(시작마다 **+1.0**), `ade770(T, W)`: `T[0] += W`. 그래서 누적 = Σ(호출 안 work) + 시작 횟수. `adf236(T, W)` = 한도 T[2] < 0이면 1.0, = 0이면 0, 아니면 (T[0] + (실행 중이면 W))/T[2].

| 타이머 | rsp | 쓰는 곳(이 구간) | 함께 쓰는 곳 | 한도(`adf1a6`, 프렐류드 L1576–1601 / 패스 재설정 L4296–4349) | 0.8 검사 |
|---|---|---|---|---|---|
| T0 | 0x380 (local_918) | `3e39a6`(C9) | 내부 루프 `3e39a6`, 마무리 `b6fd6f` | B | 없음 |
| T8 | 0x580 (local_718) | `4d3a1d`(두 번째 probing) | 없음(첫 probing은 T9 = 0x5c0) | B. st+0xf8 ≠ 0 또는 probe_mode ≥ 3이면 3B, 라운드 끝(L5268)에서 st+0xf8을 세우면 2배 | `adf236(T8) < 0.8` |
| T20 | 0x880 (local_418) | `470936` | 라운드 끝 `55a779` | 0.5B | `adf236(T20) < 0.8` |
| T21 | 0x8c0 (local_3d8) | `52cba4` | 없음 | 4B | 없음 |
| T22 | 0x900 (local_398) | `521497` | 없음 | B | 없음 |

B = 크기 × {Presolve 2, PrePasses > 1, PREPROBE > 1, PreSparsify > 0이면 1e100(크기 무관), Presolve 1이면 500, 기본 2500}(mipctx가 있으면 다른 식) + 1e7. 크기 계산과 재설정 시점은 프렐류드·패스 담당이다.

## 5. 3cd406의 인라인 상태 변경

### 5.1 두 번째 probing 앞 판정 (상태 변경 없음)
§2 그대로다. 정리하면:
1. 스냅숏 `3c546c(st, rsp+0x1a8)`를 probing 판정보다 **먼저** 뜬다. probing을 안 하면 ρ = 0이라 바로 (b)로 간다.
2. 'C' 비율: `(f64)nC > (f64)(n − st+0x408) · 0.05`(n = st+0xc, 엄격한 >). 참이면 local_48 = 1, pr = 2.
3. `4d3a1d`의 여섯째 인자는 `max(pr, 2)`, 일곱·여덟·열째는 1, 아홉째는 flag(0/1). 첫 probing(라운드 끝, 0x3b8c00)은 `(…, max(probe_mode,2), 0, 0, 0, 0)`이다.
4. 재시작 판정: `3c5595(st, rsp+0x1a8) ≤ 1e-4`(상수 0x2a684d0)면 (b), 아니면 재시작.

### 5.2 최종 쌍대 고정 루프 (`dual_fix_empty_cols`, 0x3b9f0e–0x3ba1a8, L7549–7678)
DualReductions ≠ 0일 때. INF = env+0x2458(INFBOUND), ft = FeasibilityTol, ift = IntFeasTol, ot = OptimalityTol(루프 전에 한 번 읽음). l = st+0xa0, u = st+0xa8, c = st+0xb0, cnt = st+0x68, flg = st+0x170(i32), t = st+0x150.

```rust
for j in 0..st.n {                                   // 1단계: 빈 열 고정
    if cnt[j] != 0 || flg[j] != 0 { continue; }      // 살아 있는 빈 열(개수 0)이고 플래그 0인 열만
    let (lj, uj) = (l[j], u[j]);  let d = lj - uj;
    if (t[j] != 'C' && d > ift) || d > ft { st[+0x470] = j; return 3; }   // 'C'는 ft만, 나머지는 ift 다음 ft
    let cj = c[j];
    let v = if cj > ot {
        if -INF >= lj { st[+0x470] = j; return 5; }   lj
    } else if cj < -ot {
        if uj >= INF { st[+0x470] = j; return 5; }    uj
    } else if cj >= 0.0 {                             // |c| ≤ ot, c ≥ 0
        if lj > -INF { lj } else if INF > uj { uj } else { 0.0 }
    } else {                                          // |c| ≤ ot, c < 0
        if INF > uj { uj } else if lj > -INF { lj } else { 0.0 }
    };
    if 39ab33(v, env, st, j) != 0 { return 0x2711; }  // type 1 레코드(j, v), st+0x408 += 1
    st.f64[+0xc8] = v * c[j] + st.f64[+0xc8];         // 순서: v·c 먼저 계산, 그다음 더함
    cnt[j] = -1;
    c[j] = 0.0;                                       // l, u는 그대로 둔다
}
for j in 0..st.n {                                   // 2단계: 거의 고정된 열
    if cnt[j] > 0 && l[j] + 1e-10 > u[j] {
        let r = 3df916(ft, env, st, j);  if r != 0 { return r; }
    }
}
return 0;
```
- `39ab33`: st+0xfc = 0 ∧ st+0x278 = 0이면 긴 레코드(열 항목 포함, 위치 표시 −4/−3/−2/−1), 아니면 짧은 레코드 {type 1, j, v}. MIP 상태(st+0xfc ≠ 0)는 짧은 쪽이다. 둘 다 st+0x408++.
- 비교는 모두 asm의 comisd 방향 그대로다(NaN이면 거짓 쪽). 38개 모델에서 이 루프는 `39ab33`/`3df916`을 한 번도 부르지 않았다(callsites에 0x3ba09d, 0x3ba17a 없음).

### 5.3 인라인 implied-integer 척도 (`implied_int_scale`, 0x3ba255–0x3bb966, L7695–9555)
가드: `stop == 0 ∧ env+0x2728 == 0 ∧ IMPLIEDINT ≠ 0 ∧ Presolve ≠ 0`. 열 j의 "정수 간격" s_j(x[j])를 구해, 연속 열을 x_j = s·x'_j로 바꾸고(type 9) 행을 공약수로 나눈다. **열 타입은 바꾸지 않는다**('C' 그대로). 추적기 표시·계수 카운터(+0x3fc)·activity 무한 개수도 건드리지 않는다.

버퍼: x = st+0x3a0(f64, n), v = st+0x398(f64, 행 길이 + 1), k_C = st+0x390(i32, m), 새로 할당: Q1 = i32[m], d = f64[m], Q2 = i32[n+m](DualReductions일 때). 할당 실패면 0x2711(버퍼를 풀고 EXIT). `gcd(v)` = `a7bea8(len, v)`(실수 벡터의 유리 최대공약수, 실패 0. 공통 모듈, 별도 포팅).

```rust
// A0. 초기화
for j in 0..n { x[j] = if cnt[j] > 0 && t[j] != 'C' { 1.0 } else { 0.0 }; }
let mut q1 = vec![];
for i in 0..m {                                       // 행 번호순
    k_C[i] = 0; d[i] = 0.0;
    if rowcnt(st+0x60)[i] > 0 {
        for node in row(i) { if node.col >= 0 && t[node.col] == 'C' { k_C[i] += 1; } }
        if sense(st+0xc0)[i] == '=' && k_C[i] == 1 { q1.push(i); }
    }
}
// A. '=' 행으로 전파 (FIFO, 처리 중에 q1이 자란다)
let mut h = 0;
while h < q1.len() {
    let i = q1[h]; h += 1;
    if k_C[i] != 1 { continue; }
    let nj = first node in row(i) with col >= 0 && x[col] == 0.0;   // 반드시 있다(없으면 원본은 NULL 역참조)
    let (j, a) = (nj.col, nj.val);
    if flg[j] & 0xffffffbc != 0 { continue; }        // 허용 비트 0x1, 0x2, 0x40뿐
    v.clear();
    for node in row(i) { if node.col >= 0 && node.col != j { v.push(((node.val * x[node.col]) / a).abs()); } }
    if rhs(st+0xb8)[i].abs() > 1e-10 { v.push((rhs[i] / a).abs()); }
    let mut g = gcd(&v);
    if !(g > 0.001) { continue; }
    while g > 1000.0 { g /= 10.0; }
    x[j] = g;
    for node in col(j) { if node.col >= 0 {
        let r = node.row; k_C[r] -= 1;
        if sense[r] == '=' && k_C[r] == 1 { q1.push(r); }
    }}
}
// B. 쌍대 근거 (DualReductions ≠ 0일 때만)
if DualReductions != 0 {
    for i in 0..m { if sense[i] != '=' && k_C[i] == 1 { row_scale(i); } }   // 아래 row_scale
    let mut q2: Vec<usize> = (0..n).filter(|&j| x[j] == 0.0 && cnt[j] > 0).collect();   // 열 번호순
    let mut h = 0;
    while h < q2.len() {
        let j = q2[h]; h += 1;
        if x[j] > 0.0 || cnt[j] <= 0 || flg[j] & 0xffffffbc != 0 { continue; }
        let cj = c[j];
        let mut s_lo = if cj > -1e-10 { if l[j] > -INF && l[j].abs() > 1e-10 { l[j].abs() } else { 1e100 } } else { 0.0 };
        let mut s_up = if cj <  1e-10 { if INF > u[j] && u[j].abs() > 1e-10 { u[j].abs() } else { 1e100 } } else { 0.0 };
        let mut eq = false;
        for node in col(j) { if node.col >= 0 {
            let r = node.row;
            if sense[r] == '=' { eq = true; break; }
            let a = node.val;
            if a < -1e-10      { s_lo = if s_lo > 1e99 { d[r] } else { gcd(&[d[r], s_lo]) }; }
            else if a > 1e-10  { s_up = if s_up > 1e99 { d[r] } else { gcd(&[d[r], s_up]) }; }
        }}
        if eq { s_lo = 0.0; s_up = 0.0; }
        if !(s_up > 0.001 || s_lo > 0.001) { continue; }
        let mut g = if s_up > s_lo { s_up } else { s_lo };   // maxsd
        if g > 1e99 { g = 1.0; }
        while g > 1000.0 { g /= 10.0; }
        x[j] = g;
        for node in col(j) { if node.col >= 0 {
            let r = node.row; k_C[r] -= 1;
            if k_C[r] == 1 { let c2 = row_scale(r); q2.push(c2); }   // sense 검사 없음
        }}
    }
}
fn row_scale(i) -> col {                              // 0x3bc688 / 0x3bc433
    let nj = first node in row(i) with col >= 0 && x[col] == 0.0;  let (c2, a) = (nj.col, nj.val);
    // (0x3bc433 경로는 여기서 c2를 q2에 먼저 넣는다)
    v = [|node.val·x[node.col] / a| for node in row(i) if node.col >= 0 && node.col != c2]
        ++ [|rhs[i] / a| if |rhs[i]| > 1e-10];
    d[i] = gcd(&v);  if d[i] == 0.0 { k_C[i] = -1; }
    c2
}
// C. 행 공약수 (DualReductions와 무관)
for i in 0..m {
    d[i] = 0.0;
    if k_C[i] != 0 || rowcnt[i] <= 0 { continue; }
    v = [|node.val · x[node.col]| for node in row(i) if node.col >= 0] ++ [|rhs[i]| if |rhs[i]| > 1e-10];
    let mut g = gcd(&v);
    if g > 0.001 { while g > 1000.0 { g /= 10.0; } d[i] = g; }
}
// D. 열 척도 (type 9)
for j in 0..n {
    let s = x[j];
    if !(s > 0.0) || !((s - 1.0).abs() > 1e-10) { continue; }
    3a061e(0.0, s, env, st, j)?;                       // 레코드 {type 9, j, shift 0.0, scale s}: x_j = 0 + s·x'_j
    for node in col(j) { node.val = node.val * s; }    // 죽은 노드 포함, 살아 있는지 보지 않는다
    c[j] = c[j] * s;
    if l[j] > -INF { l[j] = l[j] / s; }
    if INF > u[j] { u[j] = u[j] / s; }
}
// E. 행 척도 (레코드 없음)
for i in 0..m {
    let g = d[i];
    if !(g > 0.0) || !((g - 1.0).abs() > 1e-10) { continue; }
    for node in row(i) { node.val = node.val / d[i]; } // 죽은 노드 포함
    rhs[i] = rhs[i] / d[i];
    st.f64[+0x98][i] /= d[i];                          // max activity 유한 부분
    st.f64[+0x90][i] /= d[i];                          // min activity 유한 부분 (이 순서)
}
free(Q2, Q1, d);
```
- A의 v 원소는 `(a_ik · x_k) / a_ij`를 곱 → 나눗셈 → 절댓값 순서로 계산한다(asm 0x3bc853–0x3bc861). C는 나눗셈이 없다.
- `x[k] = 0`인 열(아직 모르는 연속 열)이 v에 들어가면 0이 들어가 `a7bea8`이 0(최솟값 < 1e-5)을 돌려준다. 그래서 그 행은 실패한다.
- B의 a7bea8 인자 순서는 `[d[r], s]`다(0x3bc3b0: v[0] = d[r], v[1] = s).
- 레코드는 D의 type 9뿐이다. E의 행 나누기는 원시해에 영향이 없어 레코드를 남기지 않는다.
- 오라클 증거: gen-ip054 k = 97 → 98에서 R0000026이 0.439658385로 나뉘어 계수 −1, −2, rhs −2가 된다(E). neos-911970 k = 61 → 62에서 C0025 등의 목적이 1 → 0.01(D, s = 0.01).

## 6. 호출 표 (이 구간의 모든 callee)

| callee | 인자(asm) | 가드 | 반환 처리 | 카운트 | 기본값 38개 |
|---|---|---|---|---|---|
| `536971` | (st+0x3d8, 0) | 진입 | 없음 | | 38 |
| `3ba9de` | (env, st, &stop) 0x3b9059·0x3bcdb2·0x3b917b, (env, st, NULL) 0x3b9736·0x3bb9d1 | | ≠0 → EXIT | | 38 |
| `3c546c` / `3c5595` | (st, rsp+0x1a8) | restart = 0 | ρ ≤ 1e-4 | | 38 / 38 |
| `adf236` | (T8, W) / (T20, W) | §2 | < 0.8 | | 21 / 38 |
| `4d3a1d` | (model, mipctx, st, env, T8, max(pr,2), 1, 1, flag, 1) | §2 | ≠0 → EXIT | C1 | 19 |
| `490ab6` | (xmm0 1e6, xmm1 min(env+0x2938, 1e6), env, st, rdx 미확인) | 남은 행 1 | ≠0 → EXIT | C2 | 0 |
| `58b41b` | (st, env) | st+0x330 > 0 | ≠0 → EXIT | C3/C13 | 0 |
| `52cba4` | (st, env, T21, &ncomp) | §2 | ≠0 → EXIT | C4 | 38 |
| `521497` | (st, env, T22) | PREBICONNECTED ∧ DualReductions | ≠0 → EXIT | C5 | 38 |
| `586049` | (xmm0 INFBOUND, env, st) | SOS 남음(+ ncomp > 0 / !stop) | ≠0 → EXIT | C6/C11 | 0 |
| `451195` | (env, st, 0, 0, 1, local_70) | QC | ≠0 → EXIT | C7/C17 | 0 |
| `5b1161` | (env, st, local_a0, local_a8, 1) | st+0x2f8 / st+0x310 | ≠0 → EXIT | C8/C19 | 0 |
| `3e39a6` | (st, env), T0 | 성분 블록이 돌았을 때 | ≠0 → EXIT | C9 | 0(이 지점) |
| `470936` | (st, env, T20, local_a0, local_a8) | §2 | ≠0 → EXIT | C10 | 38 |
| `57b2bf` | (env, st, local_a8 > 0, W) | SOS ∧ PreSOS1/2BigM ≠ 0 | ≠0 → EXIT | 없음 | 0 |
| `585a6e` | (xmm0 INFBOUND, env, st) | st+0x228 > 0 | ≠0 → EXIT | C12 | 0 |
| `42872b` | (env, st) | Q 대각 ∧ DualReductions ∧ Presolve | ≠0 → EXIT | C14 | 0 |
| `429e1b` / `42ba68` | (env, st) | 같은 조건 / + PREQOBJPROBE ∧ st+0x258 | ≠0 → EXIT (0x3bc314) | C15/C16 | 0 |
| `42d072` | (env, st) | QC ∧ PreQLinearize > 0 | ≠0 → EXIT | C18 | 0 |
| `5852f1` | (env, st, W) | env.f64+0x2650 < 1e100 | ≠0 → EXIT | 없음 | 0 |
| `583cfb` | (env, st, W) | SOS 남음 | ≠0 → EXIT | 없음 | 0 |
| `57999f` | (st, env) | Presolve ≠ 0 | void(무시) | C20 | 38 |
| `36ddf2` | (st, env) | 루프 | ≠0 → EXIT | C21 | 38 |
| `39ab33` | (xmm0 v, env, st, j) | §5.2 | ≠0 → 0x2711 | | 0 |
| `3df916` | (xmm0 ft, env, st, j) | §5.2 | ≠0 → 그 값 | | 0 |
| `ac645d` / `ac661f` | 할당 / 해제 | §5.3 | NULL → 0x2711 | | |
| `a7bea8` | (len, v) | §5.3 | 값 | | 훅 없음 |
| `3a061e` | (xmm0 0.0, xmm1 s, env, st, j) | §5.3 D | ≠0 → EXIT | | 2개(56회) |
| `54807f` | (env, st) | SYMMTRANS | ≠0 → EXIT | 없음 | 0 |
| `3ba5ac` | (env, st, 0, 0, −1) | 카운트 뒤 + 카운트 없는 callee 뒤 | ≠0 → EXIT | | |

- 추적기: 진입에서 레벨 0(`536971`)만 한다. reset(`5369b8`)은 이 구간에 없다(재시작하면 라운드 루프가 레벨 2 + reset을 한다).
- `57999f`(작은 계수 삭제): 행마다 Σ|a·(u−l)| < 0.1·min(1e-6, ft)이고 Σ|c·(u−l)| < 0.1·ot인 항목을 rhs −= a·l로 옮기고 지운다(개수 −1, 노드 열 = −1). 지운 행은 `370047(st, env, 1, &i)`로 activity를 다시 잰다. callee라 포팅은 따로.

## 7. 교차 확인

### 7.1 callsites.tsv 중 이 구간
| ret | callee | 모델 수 | 의사 코드 위치 |
|---|---|---|---|
| 0x3b905e | 3ba9de | 38 | 진입 budget |
| 0x3bcb36 | 3c546c | 38 | 두 번째 probing 스냅숏(재시작 모델도 첫 진입에서 한 번) |
| 0x3bcb5b | adf236 | 21 | T8 검사 |
| 0x3bccfa / 0x3bcd45 / 0x3bcd92 / 0x3bcdb7 | ade8ca / 4d3a1d / 3ba5ac / 3ba9de | 19 | probing + C1 |
| 0x3bcde2 | 3c5595 | 38 | 재시작 판정 |
| 0x3b9204 / 0x3b9223 / 0x3b9281 | ade8ca / 52cba4 / 3ba5ac | 38 | C4 |
| 0x3b92c8 / 0x3b92df / 0x3b933d | ade8ca / 521497 / 3ba5ac | 38 | C5 |
| 0x3b9676 / 0x3b969b / 0x3b96bf / 0x3b971d | adf236 / ade8ca / 470936 / 3ba5ac | 38 | C10 |
| 0x3b973b | 3ba9de | 38 | POST_SPARSIFY budget(NULL) |
| 0x3b9e48 / 0x3b9e8a | 57999f / 3ba5ac | 38 | C20 |
| 0x3b9ed4 / 0x3ba1dd | 36ddf2 / 3ba5ac | 38 | C21, C22(루프 1바퀴) |
| 0x3bb799 | 3a061e | 2 | §5.3 D (neos-860300 32회, neos-911970 24회) |
| 0x3bb9a8 / 0x3bb9d6 | 3ba5ac / 3ba9de | 38 | C23, SYMMTRANS 앞 budget |

이 범위(0x3b9027–0x3bbaf0, 0x3bc28e–0x3bd390)에 위 목록 밖의 호출은 38개 seq(`cov2.json`) 어디에도 없다. `ade770`은 훅이 없어 seq에 안 보인다. `3e39a6`(0x3b95c1), 성분 블록, Q/SOS 호출, `39ab33`(0x3ba09d), `3df916`(0x3ba17a), `54807f`는 0회다.

### 7.2 seq 끝부분
- **neos5**(#301–331): 라운드 끝 `5a04fc` … `3c5595`@0x3b8fd7, `3ba5ac`@0x3b8fff → (ρ ≤ 1e-4) 진입 `3ba9de`@0x3b905e → `3c546c`, `adf236`@0x3bcb5b, `ade8ca`, `4d3a1d`(C1), `3ba5ac`, `3ba9de`, `3c5595`(ρ ≤ 1e-4) → `52cba4`(C4), `521497`(C5)(ncomp > 0이지만 SOS/QC/NL 없음, comp_ran = 0이라 `3e39a6` 없음) → `adf236`, `470936`(C10) → `3ba9de`@0x3b973b → `57999f`(C20) → `36ddf2`(C21), 쌍대 고정(C22), `3ba5ac`@0x3ba1dd(+0x408 그대로라 끝) → implied int(C23, `3a061e` 없음), `3ba5ac`@0x3bb9a8 → `3ba9de`@0x3bb9d6 → 마무리 `538353`@0x3b261d. 의사 코드와 같다.
- **timtab1**: neos5와 같은 모양(probing 있음).
- **gmu-35-50**: `3c546c` 다음 곧바로 `3c5595`(adf236 없음): probe_mode = 1이고 'C' 비율 판정이 거짓 → probing 없음, ρ = 0 → (b). 나머지 같음.
- **glass4**: 첫 진입(#689) `3ba9de` → probing(C1) → `3c5595` > 1e-4 → 재시작: #697 `3c546c`@0x3b1d1e(라운드 루프 처음). 두 번째 진입(#1355)은 probing 없이 `52cba4` … `3ba9de`@0x3bb9d6 → 마무리. 재시작한 모델은 glass4, graphdraw-domain, ic97_potential, neos-4954672-berkel 넷이다.

### 7.3 sweep (PRELIMIT 카운터 재현)
`cov2.json`의 seq에 카운트 규칙(3cd406의 `cmp 0x1dc(…)` 114곳 각각 바로 다음 호출 앞에서 +1. `cmpl $1` 한 곳은 뺌. C21은 `36ddf2` 바로 뒤)을 적용해 0x3b905e 시점의 `cnt`를 구했다(스크립트는 scratchpad의 `cnt.py`, `cnt2.py`). 이 값과 sweep의 "바뀐 k"가 아래처럼 정확히 맞물린다.

| 모델 | 진입 cnt | 끝 cnt | 이 구간에서 바뀐 k | 바뀐 단계 |
|---|---|---|---|---|
| glass4 | 75, 138 | 145 | 76 | 두 번째 probing(→ 재시작). 마지막 변화 k = 106은 재시작한 라운드 루프 안(다른 담당) |
| neos-911970 | 54 | 62 | 62 | implied int(열 척도 24개, type 9) |
| neos-860300 | 98 | 105 | 101, 105 | 101 = sparsify `470936`, 105 = implied int(열 척도 32개) |
| gen-ip054 | 91 | 98 | 98 | implied int, 행 척도만(`3a061e` 0회) |
| neos-2657525-crna | 98 | 105 | 105 | implied int, 행 척도만 |
| neos-3754480-nidda | 68 | 76 | 76 | implied int, 행 척도만 |
| tr12-30 | 208 | 216 | 209 | 두 번째 probing(ρ ≤ 1e-4라 재시작 없음) |
| graphdraw-domain / ic97_potential / neos-4954672-berkel | 91·124 / 98·138 / 135·168 | 131 / 145 / 175 | 92 / 99 / 136 | 두 번째 probing(→ 재시작) |
| timtab1, gmu-35-50, mas74 외 26개 | | | 없음 | 마지막 변화는 이 구간 앞(timtab1 160 < 175, gmu-35-50 176 < 203, mas74 33 < 91) |

`52cba4`, `521497`, `57999f`, `36ddf2`, 쌍대 고정은 38개 어디에서도 출력을 바꾸지 않았다.

## 8. 미확인 사항

1. **`4d3a1d`의 일곱–열째 인자**(1, 1, flag, 1)와 여섯째(mode)의 뜻. 첫 probing은 0, 0, 0, 0이다. → `4d3a1d` 본문에서 param_7–10 쓰임을 해독(probing 담당).
2. **pk1, fhnw-binpack4-4**는 `adf236`(T8)까지 갔는데 probing을 안 했다. 목적 비영 < 2인지 T8 ≥ 0.8인지 모른다. → Frida로 0x3bcb5b의 xmm0(사용률)과 그 시점 st+0xb0 비영 수를 덤프.
3. **implied int B단계(쌍대 근거)가 실제로 x를 정한 적이 있는지** 모른다(neos-911970·neos-860300의 척도가 A에서 왔는지 B에서 왔는지). → 0x3bb05d(A 끝)와 0x3bb54e(B 끝)에서 x(st+0x3a0) 덤프, 또는 포팅 뒤 PRELIMIT k = 62/105 비트 비교.
4. **`a7bea8`**: 공통 모듈이라 훅이 없다. floor·연분수·정수 GCD 경로를 그대로 옮겨야 비트가 맞는다. → 포팅 뒤 위 모델의 d[i]·x[j]를 Frida 덤프와 비교.
5. **인라인 쌍대 고정(§5.2)은 38개에서 한 번도 고정하지 않았다**(39ab33/3df916 직접 호출 0회). 오라클 증거가 없다. → 빈 열 + 비용이 있는 합성 MIP로 PRELIMIT k = (C22) 비교.
6. **env+0x2650**(`5852f1` 가드, 1e100 > 값)의 이름·기본값, env+0x2640/0x2648을 PreSOS1BigM/PreSOS2BigM로 본 것. → `ac81d7`의 실수 파라미터 등록에서 오프셋 추출.
7. **st+0x228**(`585a6e` 가드)과 st+0x1e4·st+0xec의 정확한 뜻. → `3a484d`에서 쓰는 곳 확인.
8. **`490ab6`의 rdx 인자**(asm에서 직전 값이 남은 레지스터일 수 있음)와 의미. 행 1개로 줄어드는 모델이 없어 관찰 못 했다. → `490ab6` 프롤로그 해독, 또는 행 1개짜리 합성 MIP.
9. **scratch 버퍼 크기**: st+0x390 ≥ m(i32), st+0x398 ≥ 최대 행 길이 + 1(f64), st+0x3a0 ≥ n(f64)이어야 한다. → `3a484d`의 할당 크기 확인(포트에서 크기를 맞출 때).
10. **PRERESTART(`stop`)**: `6a0643`(env+0x27f0 < 1e100 ∧ Presolve ≠ 0 ∧ env+0x2638 == 0)의 두 실수 파라미터 이름. 38개에서는 끝까지 0이었다(C23이 38/38). → 실수 파라미터 표.
11. 라운드 끝에서 0x3b7c33의 문턱 rsp+0x2d0(r15b = 0일 때 값)은 라운드 끝 담당이 정한다. 재시작 뒤(r15b = 1)에는 0.05다.
