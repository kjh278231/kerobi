# M3 barrier Rust 이식 기록

`tools/proto/bar_ipm.py`(스펙 [m3_ipm.md](m3_ipm.md))를 Rust로 옮긴 기록이다. 범위는 Presolve=0, Method=2, Crossover=0, Threads=1의 LP다. 함수 이름은 `PRIVATE…`의 뒤 6자리다.

## 0. 결과 요약 (2026-11-05, Netlib 64)

| 항목 | 결과 | 비교 대상 |
|---|---|---|
| 상태 | **64/64** | `bench/gurobi_bar_nopre` |
| 목적 상대오차 ≤ 1e-6 | **64/64** (최대 8.5e-9) | 같은 곳의 ObjVal |
| BarIterCount 같음 | **56/64** (프로토타입도 56/64) | 같은 곳 |
| 프로토타입과 반복 수 같음 | 60/64 | m3_ipm.md §7 |
| F+0x160, F+0x168 | **64/64 비트 일치** | `bench/oracle/bar/it0/*`의 F 블록 |
| 실행 시간 | 64개 전체 약 2초 | |

parity 결과는 `bench/parity_bar.csv`에 있다. `bar_stats`는 rsolver가 로그를 쓰지 않아서, `p0_presolved`는 presolve가 없어서 불일치로 나온다. `first_divergence`는 trace를 repr 문자열로 비교하므로 비트 일치 전에는 항상 행 0이다.

## 1. 구조

| 파일 | 내용 |
|---|---|
| `crates/lp/src/barrier/mod.rs` | 공개 API(`solve`, `Params`, `IterInfo`, `BarResult`), `2e7f38` work 추정(`work_estimates`), `a80670` CPU 상수(`cpu_divisor`) |
| `setup.rs` | `lp_from_model`: 스케일(`scale_flag3`), '>' 행 부호, 행 순열, `30f5b9`(자유 열 bound 전파, BARLOOSE, 유형, boxed u), `312a1e`(lb 이동), ncomp·노름·ptol·dtol. `Lp::ax`/`aty`(`2d3b3e`/`2d3d91`), `x_orig`(원 공간 복원) |
| `ipm.rs` | 초기점 `2d0663`·`2d252f`·`2d8838`, 잔차 `2d4977`, θ, 방향 `2f59f1` + refinement `2f9699`, step `2f15ba`/`2f4fe9`, μ_aff `2f28c2`, corrector `2d64b4`, K 자동 조정과 work 타이머, step 제한, 갱신 `2f2e95`, 종료 `b2aecf` |
| `normal.rs` | `Normal`(AΘAᵀ + diag 조립)과 **`Factor`**(분해·해법 이음매) |
| `ldl.rs` | up-looking LDLᵀ + Gurobi 작은 피벗 규칙. `numeric`이 F+0x100(min \|d/대각\|)을 돌려준다 |
| `ordering.rs` | `Symbolic`에 `ops_small`(F+0x68) 필드 추가 |

흐름: `ordering::analyze` → `setup::lp_from_model(perm)` → `work_estimates` → `ipm::run`.

### 1.1 공개 API

```rust
pub fn solve(m: &Model, p: &Params, cb: &mut dyn FnMut(&IterInfo)) -> BarResult
pub struct Params { bar_conv_tol: f64 /*1e-8*/, bar_iter_limit: i64 /*1000*/, bar_correctors: i32 /*-1*/ }
pub struct IterInfo { iter, pobj, dobj, priminf /*B+0x198*/, dualinf /*B+0x1a8*/, compl /*B+0x1f0*/ }
pub struct BarResult { status /*2, 7, 13*/, obj, bar_iters, x /*원 공간*/, work }
```

- 콜백은 반복마다 잔차 계산(과 재중심화) 뒤, 종료 판정 전에 부른다. 마지막 반복도 부른다.
- obj와 콜백 목적값은 원 공간·원 sense다. 내부는 sense·obj를 최소화하고 objcon도 sense를 곱해 둔다.
- `work`는 barrier 루프 타이머(분해·predictor·corrector·갱신)의 합 × 1e-9다. ordering·setup work는 넣지 않았다. Gurobi Work의 0.69–1.15배다(`bench/parity_bar.csv` work_ratio).

### 1.2 분해 인터페이스 (`chol.rs`를 끼울 자리)

```rust
impl<'a> Factor<'a> {
    pub fn new(lp: &'a Lp) -> Factor<'a>;
    /// θ(n), θ_slack(m, '=' 행은 0) → (F+0x100 = min |pivot/대각|, 작은 피벗 대체 수)
    pub fn factor(&mut self, theta: &[f64], theta_slack: &[f64]) -> (f64, usize);
    /// 2ee6c5: −Θ⁻¹dx + Aᵀdy = rhs_x(m+n), A dx = rhs_y(m) → (dx, dy)
    pub fn solve(&self, rhs_x: &[f64], rhs_y: &[f64]) -> (Vec<f64>, Vec<f64>);
}
```

- 지금 구현은 `normal.rs`의 `Factor`(Normal + Ldl)다. `ipm.rs`는 이 셋만 부른다.
- supernodal 판으로 바꿀 때는 `chol.rs`에 같은 세 함수를 가진 `Factor`를 만들고 `ipm.rs`의 `use super::normal::Factor;` 한 줄을 `use super::chol::Factor;`로 바꾼다.
- `Lp`의 행은 이미 ordering 순서라서 피벗 순서는 자연 순서다. supernode 구조가 필요하면 `ordering::analyze` 결과를 `new`에 더 넘긴다(호출은 `ipm::run` 한 곳).
- `solve` 안에서 A·(Θ rhs_x)와 Aᵀdy는 `Lp::ax`/`aty`로 한다. 이 둘의 합 순서(Gurobi는 SIMD 8레인)도 비트 일치 대상이다.

### 1.3 Python

- `CoreModel.barrier(callback, bar_conv_tol=1e-8, bar_iter_limit=1000, bar_correctors=-1)` → (status, obj, BarIterCount, x, work).
- `Model.optimize`: `self._core`가 있고 `Params["Method"] == 2`이면 `_barrier`로 간다. 콜백은 where = `GRB.Callback.BARRIER`(7)로 반복마다 부르고, `cbGet`이 BARRIER_ITRCNT/PRIMOBJ/DUALOBJ/PRIMINF/DUALINF/COMPL을 돌려준다.
- 속성은 Status, ObjVal, BarIterCount, IterCount = 0, NodeCount = 0, Work, Runtime이다. ObjBound·SolCount는 없어서 parity에서 None이다.

## 2. 프로토타입과의 차이

| 항목 | 프로토타입 | Rust | 영향 |
|---|---|---|---|
| 선형계 | 조밀 블록 LDLᵀ(numpy, 64열 블록) | 희소 up-looking LDLᵀ(`ldl.rs`), AAᵀ 패턴 위 | 반올림만 다르다. 작은 피벗 규칙·F+0x100은 같다 |
| 합 | numpy pairwise sum, BLAS dot | 인덱스 순서 순차 합 | 반올림 |
| `2d252f` 합 순서 | lo 전체(m+n) 뒤 up | 열마다 (lo, up), 그다음 slack(m3_ipm.md §2.5의 Gurobi 순서) | 반올림 |
| F+0x160/0x168 | 반복 0 오라클 F에서 읽음 | `2e7f38` 공식으로 계산(§4) | 값은 64/64 같다 |
| NaN 피벗 | 대체, F+0x100 = 0 | 같음(`ldl.rs` 수정) | 없음 |
| sense | 최소화만 | sense·obj 최소화 후 결과에 sense를 곱함 | 64개는 모두 최소화 |
| 해 x | 반환 안 함 | lb 이동·열 스케일을 되돌린 원 공간 x | |

규칙(초기점, θ, 방향, 비율 검사, corrector, K 조정, step 제한, 갱신, 종료)은 프로토타입 식과 연산 순서를 그대로 옮겼다. 재시작·homogeneous·상태 13 정체 규칙·최선 해 선택(`2c4689`)은 프로토타입처럼 없다(`ipm.rs` 머리의 `shortcut:`).

## 3. 64개 결과

표기: `이름 Rust(Gurobi, 프로토타입)`. Rust가 Gurobi나 프로토타입과 다른 것은 굵게 표시했다.

25fv47 23(23,23), adlittle 12(12,12), afiro 7(7,7), **agg 17(18,17)**, agg2 18(18,18), **agg3 17(18,17)**, bandm 16(16,16), beaconfd 9(9,9), blend 8(8,8), **bnl1 49(49,48)**, boeing1 18(18,18), boeing2 14(14,14), bore3d 16(16,16), brandy 13(13,13), capri 22(22,22), **degen2 11(12,11)**, **degen3 15(14,14)**, e226 17(17,17), etamacro 23(23,23), **fffff800 83(76,83)**, finnis 22(22,22), fit1d 15(15,15), fit1p 11(11,11), forplan 18(18,18), ganges 16(16,16), gfrd-pnc 15(15,15), grow15 12(12,12), grow22 12(12,12), grow7 10(10,10), israel 16(16,16), kb2 16(16,16), lotfi 17(17,17), **maros 20(19,20)**, modszk1 20(20,20), **perold 165(130,160)**, **pilot4 112(85,121)**, recipe 8(8,8), sc105 7(7,7), sc205 9(9,9), sc50a 7(7,7), sc50b 7(7,7), scagr25 15(15,15), scagr7 14(14,14), scfxm1 15(15,15), scfxm2 15(15,15), scfxm3 17(17,17), scorpion 10(10,10), scrs8 17(17,17), scsd1 8(8,8), scsd6 9(9,9), sctap1 16(16,16), sctap2 10(10,10), seba 12(12,12), share1b 19(19,19), share2b 9(9,9), shell 21(21,21), ship04s 13(13,13), stair 14(14,14), standata 10(10,10), standgub 10(10,10), standmps 15(15,15), stocfor1 9(9,9), tuff 10(10,10), vtpbase 50(50,50).

±5% 안은 56/64다(프로토타입 57/64). degen3이 15 대 14(7%)로 빠지고 bnl1이 정확히 맞게 되었다.

### 3.1 Gurobi 궤적에서 처음 갈라지는 반복

기준은 `bar_ipm_check.py`와 같다(pobj·dobj 상대 1e-6, compl 상대 1e-4). 반복 0은 64개 모두 1e-13 안이다.

| 인스턴스 | Rust | 프로토타입(m3_ipm.md §8) |
|---|---|---|
| agg | 17 | 17 |
| agg3 | 16 | 16 |
| degen2 | 9 | 9 |
| degen3 | 12 | (갈라짐 기록 없음, 반복 수는 맞음) |
| maros | 19 | 19 |
| bnl1 | 30 | 30 |
| fffff800 | 36 | 36 |
| perold | 25 | 25 |
| pilot4 | 22 | 17 |

갈라지는 반복이 프로토타입과 같다. 그래서 불일치 8개의 원인은 m3_ipm.md §8 그대로(해법·분해 반올림 누적)로 본다.

### 3.2 프로토타입과 반복 수가 다른 4개

- **degen3 15(14, 14)**: 종속 행이 있어 매 반복 작은 피벗을 대체하고 mode 1(F+0x100 ≈ 1e-15)이다. 반복 11의 해법 잔차 ‖r_y‖(B+0x770)가 Rust 6.25e-7, 프로토타입 6.19e-7로 크다(refinement 기준 B+0x280보다 작아서 보정하지 않음). 그래서 반복 12의 compl이 Rust 2.787109e-5, 프로토타입 2.787046e-5, Gurobi 2.785226e-5로 셋 다 갈린다(반복 11까지는 1e-11 수준으로 같음). 반복 14의 compl은 Rust 9.3e-9, 프로토타입 1.25e-9, Gurobi 6.1e-10이다. 프로토타입만 반복 14에서 종료 조건을 넘었다. 프로토타입이 14로 맞은 것도 우연이며, degen2와 같은 종류(종속 행의 영공간 잡음)다.
- **bnl1 49(49, 48)**: 반복 30부터 반올림 누적으로 갈라진다(프로토타입과 같은 반복). 끝 반복 수가 Gurobi와 같은 것은 우연이다.
- **perold 165(130, 160)**, **pilot4 112(85, 121)**: 둘 다 Gurobi와 이미 다르고(자유 열, 느린 궤적, 매 반복 refinement), 갈라진 뒤의 반올림이 달라 끝 반복 수가 달라졌다.

넷 다 규칙 차이의 흔적(K 수열이 갈리는 반복 등)은 찾지 못했다. 반올림 수준 일치에서 더 줄이려면 분해·해법·합 순서의 비트 일치가 필요하다(`chol.rs`).

## 4. F+0x168 (`2e7f38`, `a80670`)

asm(0x304a40)에서 읽은 연산 순서 그대로다. `2e4f14` 끝에도 같은 식이 인라인으로 있다.

```
F+0x160 = 2·nnzL                                   (nnzL = F+0x38 = Factor NZ)
F+0x170 = (8·nnzL + F68/5) + (2·F48)·(1 − F50)
F+0x170 = ((F60 − F68) + F48·F50)/cpu + F+0x170
F+0x168 = (pow(threads, −0.9)·F+0x170 + F58) + 75·m      (m = F+0x0 행 수)
```

- F48 = Σ A 열 길이², F60 = Factor Ops, F68 = 열 5개 미만 supernode의 ops 합이다(`2e4f14` L2058–2084, 블록 분할 전 supernode F+0x70 기준).
- F50·F58은 밀집 열 항이라 0이다(64개 모두 Dense cols 0). threads = 1이라 pow 항은 1이다.
- `a80670`은 CPUID 비트만 본다(OS의 XGETBV 검사 없음). 1.ECX bit 28(AVX)이 없으면 0, 7.EBX bit 5(AVX2)가 없으면 1, bit 17(AVX-512DQ)이 있으면 0x200, 아니면 2다. 각각 cpu = 10, 18, 50, 30이다. 이 머신(Core Ultra 7 155H, [SSE2|AVX|AVX2])은 30이다.
- 64개 F+0x160·F+0x168이 오라클과 **비트까지 같다**. 단위 테스트 `barrier::tests::work_estimate_matches_oracle`은 cpu = 30을 넣어 4개(afiro, 25fv47, perold, fit1p)를 고정한다.
- K₀ = int(clamp(0.1/r, 1, 20)), N₀ = (r > 0.2 ? 3 : 5), r = (2·B5a8 + F160)/F168, B5a8 = 2·nnz(A)이다(m = 0이면 r = 1).

## 5. 검증

```sh
PYO3_PYTHON=$PWD/../.venv/bin/python cargo build --release -p rsolver && cp target/release/lib_core.so python/rsolver/_core.cpython-313-x86_64-linux-gnu.so
PYTHONPATH=$PWD/python ../.venv/bin/python tools/parity.py capture rsolver <scratch>/rsolver_bar_nopre --lp-only --param Method=2 --param Presolve=0 --param Crossover=0
python3 -I tools/parity.py compare bench/gurobi_bar_nopre <scratch>/rsolver_bar_nopre --csv bench/parity_bar.csv
cargo test -p rsolver-lp barrier            # afiro 7반복·목적값, F+0x168
PYTHONPATH=$PWD/python ../.venv/bin/python -m pytest tests/test_barrier.py   # 64개 상태·목적, BarIterCount ≥ 56
```

## 6. 남은 일

- 분해·해법 비트 일치(`chol.rs`: supernodal 커널, `2d3b3e`/`2d3d91`의 SIMD 합 순서)로 불일치 8개 줄이기.
- 재시작·homogeneous·상태 13 규칙, `2c4689` 최선 해 선택, 밀집 열(F50/F58, `2fd14c`), Threads > 1의 pow 항.
- 로그(Barrier statistics, 반복 표)와 ObjBound·SolCount. 지금 parity의 `bar_stats` 열은 로그가 없어서 불일치다.
- Presolve≠0, Crossover≠0 경로(지금은 Method=2이면 무조건 Presolve=0, Crossover=0으로 돈다).
