# M3 barrier 인수인계 메모

목표(G3, [11 §4](../11_progress.md)): Method=2, Presolve=0, Threads=1, Crossover=0에서 barrier 본체를 Gurobi와 같게 만든다. 방식은 [10 §9](../10_porting.md)의 재구성 + 오라클 비트 검증이다. 함수 이름은 `PRIVATE…`의 뒤 6자리, 주소는 file offset이다.

## 1. 호출 범위와 외부 의존 (2026-11-05, `re/decomp/callgraph.tsv`)

**진입**: `c23877`은 Method=2 && Crossover=0이면 `2bb770`을 바로 부른다. Presolve=0이면 `2bb770`은 presolve·PreDual·crossover를 모두 건너뛰고 `2bd9ab`(래퍼) → `2ca086`(작업 구조체 B) → `2bf1ea`(주 루프)로 간다(노트 14 §3.1.1).

**도달 함수**: `2bf1ea`·`2ca086`에서 정적으로 닿는 함수를 콜백 디스패처 `9a91f3`(사용자 코드로 감)과 SOC 준비 `33c1b1`을 빼고 셌다. 270개: barrier 모듈 110, common 100, libc·MKL 47, simplex 5, driver 3, presolve 2, genconstr 3.

| 질문 | 답 | 근거 |
|---|---|---|
| simplex를 부르나 | 반복·pricing·ratio test는 **안 부른다**. `2ca086`이 simplex 스케일링 **`07d3ce`**(와 보조 `07b892`, `10f720/789/888`)를 부른다. M1이 `crates/lp/src/scaling.rs`로 옮긴 함수다. barrier가 어느 스케일 경로(ScaleFlag)를 타는지는 아직 확인하지 않았다 | `07d3ce <- 2ca086` |
| LU를 부르나 | **안 부른다**. `036ade`·`1da0b2`·FTRAN·BTRAN 모두 도달 집합 밖이다 | 도달 집합 |
| presolve를 부르나 | 본체는 **안 부른다**. 도달하는 presolve 함수 둘은 `a71757`(`07d3ce`의 로그 보조 `a9a78e`가 부름)과 73바이트 보조 `ade8ca`다. `390989`는 `2bd9ab`가 Presolve≠0일 때만 부른다 | `a71757 <- a9a78e <- 07d3ce` |
| crossover·simplex 재진입 | `2bd9ab` 전체에서 보면 대칭 축소 복원 `32578f` → `98b671` → `c23877`로 드라이버 전체가 닿는다. PRESYMMPASSES 경로라 LP 기본에서는 타지 않는다(확인 필요) | `32578f <- 2bd9ab` |
| MKL | `DGEMM`(블록 갱신 `32d72f` → `31f71d`, 시간 추정 `2e44ba`), `DTRSM`(supernode 커널 `2d8ee5`), 단정밀도 `sgemm`·`strsm_`(`3279a1`), `mkl_thread_free_buffers`(`2cf99d`). dpotrf·PARDISO 없음 | libc·MKL 47개 목록 |
| libm | `sqrt`, `log`, `log10`, `floor`, `pow`, `scalbn`, `ilogb`, `fabs`, `__isnan`, **`sqrtl`**(갱신 `2f2e95` → `343d1b`, x87 long double) | 같은 목록 |
| 스레드 | 스레드 풀 `ac761d`/`ac7bef`(pthread), `pthread_barrier`(해법 `2ee6c5` → `337be3` → `34d5fa`@0x34c880). Threads=1에서 실제로 스레드를 만드는지는 확인 필요 | 같은 목록 |

결론: barrier 본체는 M1의 simplex·LU와 독립이다. 공유하는 것은 스케일링(`07d3ce`)뿐이고, 이건 `crates/lp/src/scaling.rs`를 수정 없이 불러 쓸 수 있는지 먼저 본다(worktree 규칙상 기존 파일은 못 고친다). 밀집 블록에서 MKL DGEMM/DTRSM을 쓰므로 비트 일치에는 같은 MKL 커널이 필요할 수 있다. 작은 Netlib에서 블록 경로가 실제로 쓰이는지부터 오라클로 본다.

## 2. 기준값

| 디렉터리 | 설정 | 내용 |
|---|---|---|
| `bench/gurobi_bar_nopre/` | Method=2, Presolve=0, Crossover=0, Threads=1, WorkLimit 60 (LP 64개) | `tools/parity.py capture`. 64/64 OPTIMAL. BarIterCount 7(afiro 등)–130(perold). `summary.json`의 `log.barstats`에 "Barrier statistics"(Free vars, AA' NZ, Factor NZ, Factor Ops) |

- **결정성**: 같은 설정으로 다시 캡처해 비교하면 64/64가 상태·BarIterCount·Barrier statistics·BARRIER 콜백 궤적(repr) 모두 같다(`bench/gurobi_bar_determinism.csv`, 2026-11-05, 다른 에이전트가 Gurobi 오라클을 돌리는 부하 중). 따라서 corrector 수 자동 조정 등은 벽시계에 기대지 않는다고 본다(코드 확인은 docs/m3/m3_ipm.md).
- 64개 모두 Dense cols가 0이다(밀집 열 검출은 m ≥ 1000에서만 돈다). Free vars가 있는 것은 4개(49, 72, 1, 2).
- BARRIER 콜백 값(`trace.csv.gz`의 `barrier` 행)의 PRIMINF는 로그의 Primal 잔차와 다른 노름이다(afiro 반복 0: 4471 대 2.27e3).

## 3. 오라클

- BARRIER 콜백의 cbdata(`model._callback._cbdata` 캡슐) **+0x68이 B**다(B+0x18 = B+0x1c = m, B+0x20 = n, `2ca086` L80–83).
- F(인수 구조체, `2cd7cb`)는 B에 저장되지 않는다. `2bf1ea` 프레임의 local_98 = B+0x5c8, local_90 = B+0x5b8, local_88 = F가 붙어 있어서 주 스레드 스택에서 그 세 값을 찾는다(`tools/bar_oracle.py` `find_F`). afiro: F+0x38 = 218 = Factor NZ, F+0x60 = 2146 = Factor Ops.
- **barrier 중의 모델 데이터 D**(사용자 model +0xe8)는 제자리에서 바뀌어 있다. (1) **스케일**: D+0x3e0 = 스케일 배열(열 n개, 행 m개, 원래 행 순서), D+0x3e8 = 목적 스케일. 값은 M1 `scale_flag3`과 같다(25fv47, perold 열·행·목적 모두 비트 일치). 즉 `2ca086` → `07d3ce`(ScaleFlag=-1)는 simplex와 같은 ScaleFlag 3 경로다. (2) **행 순열**: 행이 ordering 순열로 재배치되고 각 열의 원소도 새 행 번호 순으로 다시 정렬된다(afiro 옛 행 1→11, 2→13, 23→14, 0→17). B의 행 인덱스 배열(y 등)은 이 순서다.

## 4. 진행

| 단계 | 대상 | 상태 |
|---|---|---|
| 0 | 호출 범위·MKL 확인, 기준값, 결정성 | 완료 |
| 1 | 오라클(`tools/bar_oracle.py`): B 스칼라·x/y/z, F(nnz(L), ops) | 동작. 필드는 해독하면서 늘린다 |
| 2 | ordering·symbolic(AA' NZ, Factor NZ/Ops, 순열, supernode) | **완료 64/64**(AA' NZ, 최종 순열 F+0x30, supernode 블록 F+0x78, 열 길이·행 번호, Factor NZ/Ops, F+0x48). `crates/lp/src/barrier/ordering.rs` `analyze()`, 테스트 `bench_lps_match_gurobi`(기대값 `bench/oracle/bar_order/expected.tsv`). 64개 모두 최종 순열은 기준선(mode 3, AMD + 이분할)이고 ND 후보는 채택되지 않는다. 세부는 `docs/m3/m3_ordering.md` |
| 3 | 표준형·초기점·잔차·반복 규칙(프로토타입) | **프로토타입 완료**(`tools/proto/bar_ipm.py`, `docs/m3/m3_ipm.md`): 내부 LP 64/64 비트, 반복 0 점 ≤4e-12, 한 반복 1e-16~1e-9, corrector 수 K·work 타이머 정확히 같음(벽시계 아님). 전체: 상태 64/64, 목적 ≤1e-6 64/64, **BarIterCount 56/64**(±5% 안 57/64). 불일치 8개(agg, agg3, maros, degen2, bnl1, fffff800, perold, pilot4)는 해법 반올림이 쌓여 갈라짐 |
| 4 | 수치 분해(supernodal LDLᵀ, MKL DGEMM/DTRSM) | **완료**(`chol.rs`, `mkl.rs`, `docs/m3/m3_chol.md`): L, d, 1/d, F+0xf0, F+0x100이 Gurobi와 비트 같음(7개 인스턴스 × 반복 0–2, 원소 전부). MKL AVX2 커널을 재현해 64개에서 나온 DGEMM 2660, DTRSM 746 모양 전부 비트 일치(`tools/proto/mkl/test_all.py`로 재검증). LP 피벗 규칙은 §5 |
| 5 | Rust 본체, Python 경로(`Method=2`), `tools/parity.py capture rsolver` | **완료**(`docs/m3/m3_rust.md`): `barrier::solve`, `CoreModel.barrier`, `Model.optimize` Method=2 분기, BARRIER 콜백·`cbGet`, LogFile에 "Barrier statistics". F+0x160/0x168(work)는 오라클과 64/64 비트 같음(CPU 상수는 CPUID, 이 머신 AVX2 → 30). 64개 약 2초 |
| 6 | 비트 일치(분해·해법·IPM 합 순서), 재시작 | **완료**(`docs/m3/m3_chol.md`): 해법 `2ee6c5` dx·dy 비트, compl x87 Kahan(`f80.rs`), 잔차 8레인, μ_aff 4레인, rd 결합 순서, 재시작(`2bf1ea` L956–994, perold 반복 99·pilot4 반복 57). **BarIterCount 64/64**, 콜백 궤적 비트 동일 55/64 |

## 5. 결과 (2026-11-06)

| G3 기준 | 상태 | 근거 |
|---|---|---|
| (1) 상태·목적 ≤1e-6 | 64/64, 64/64 | `tests/test_barrier.py`, `bench/parity_bar.csv` |
| (2) BarIterCount 같음 ≥90%, 나머지 ±5% | **64/64** (나머지 없음) | 같은 곳(`same == 64` 단언) |
| (3) ordering·Factor NZ | 64/64(AA' NZ, Factor NZ/Ops, Free vars, 순열·supernode) | `test_barrier_statistics_equal_gurobi_log`, Rust `bench_lps_match_gurobi`, parity `bar_stats` |
| (4) 반복별 오라클 비교 도구 | `tools/bar_oracle.py`(`--iters`, `--vec`, `--chol`, `--solve`), `tools/parity.py compare`의 `first_divergence`(비트)·`first_divergence_1e-6` | — |
| (5) clippy·테스트·문서 | **이 브랜치에서 `cargo clippy --workspace --all-targets` 경고 0**, `cargo test --workspace`, pytest 11 통과. 남아 있던 M1 코드 경고 4개는 main과 같은 방식으로 없앴다(아래 병합 메모). main과 시험 병합한 트리에서도 clippy 0·테스트 통과 | 아래 병합 메모 |

**궤적 첫 분기**(반복 수는 같고 콜백 값의 비트가 갈리는 9개, `first_divergence` 반복): 25fv47 22, bnl1 48, degen3 1(분해 3321개 중 15개가 다름), finnis 20, forplan 17, ganges 14, maros 18, modszk1 12(상대 1e-6로는 16. 반복 11까지 상태·K·N·θ가 같고 반복 11의 step이 다름. `2f4fe9`의 ω = 0 분기부터 볼 것), pilot4 79. 나머지 55개는 마지막 반복까지 비트 동일.

**피벗 규칙**: `bar_oracle.py --chol`로 degen2 반복 0–12의 Gurobi 분해를 떴다. 작은 피벗을 대체하지 않는다: |d| ≤ 1e-16·|원 대각|(또는 NaN)만 d = 0·역수 0, 나머지(예: d = −3.6e-15, 원 대각 12)는 그대로 두고 F+0xe8 = 1/d를 곱한다(e = 1/d 예외 0). F+0x100은 버리기 전 min|d/원 대각|. 이것만으로 56 → 59/64(`ddc3040`), 재시작과 비트 일치로 64/64.

**main 병합 메모**(오케스트레이터용, 2026-11-06 main `abeeaf7`과 시험 병합): 충돌은 두 파일뿐이다. (1) `crates/py/src/lib.rs`: main의 `solve`와 이 브랜치의 `barrier_symbolic`·`barrier`를 둘 다 둔다. (2) `python/rsolver/__init__.py`: main의 `optimize` 맨 앞에 `if self._core is not None and self._param("Method", -1) == 2: return self._barrier(callback)`, main의 `cbGet` 맨 앞에 `if what in (getattr(self, "_cbdata", None) or {}): return self._cbdata[what]`를 넣고 이 브랜치의 `_barrier`를 그대로 둔다. 이렇게 푼 트리에서 clippy 0, cargo test, pytest가 통과했다. 그 뒤 이 브랜치에서 M1 코드 경고 4개를 없앴다: `crates/lp/src/lib.rs` 2행에 main과 **같은 줄** `#![allow(clippy::needless_range_loop)]`(양쪽 동일 변경이라 충돌 없음, `lu.rs`는 손대지 않음), `crates/core`(main에서 삭제됨)의 `lp.rs` 루프와 `Row` 타입 별칭, `crates/py/src/lib.rs` `solve_problem`의 `Vec<Row>`. 병합 때 `crates/core`는 modify/delete 충돌이 나는데 main처럼 삭제하면 되고, `solve_problem`도 main 쪽(삭제)을 따르면 된다.

**남은 위험·안 한 것**: homogeneous 전환·상태 13·`2c4689` 최선 해·밀집 열(m ≥ 1000)·Bunch–Kaufman·단정밀도·SOC/QP 경로는 64개에서 안 타서 옮기지 않았거나 확인하지 않았다. MKL 재현은 AVX2 전용(AVX-512 머신은 커널이 다름), dtrsv는 m ≤ 32까지만 비트 보장, 블록·벡터 16바이트 정렬은 관찰에 기댄다. K₀가 CPU 상수(`a80670`)를 타므로 다른 CPU에서는 반복 수가 달라질 수 있다. Method=2이면 Presolve·Crossover 값을 보지 않는다.

## 6. 재개 기록 (2026-11-06, 중간)

- **피벗 규칙 확정**: `bar_oracle.py --chol`로 degen2 반복 0–12의 Gurobi 분해를 떴다. 작은 피벗을 대체하지 않는다. |d| ≤ 1e-16·|원 대각|(또는 NaN)만 d = 0·역수 0으로 버리고, 나머지는 d = −3.6e-15(원 대각 12) 같은 값도 그대로 두고 F+0xe8 = 1/d를 곱한다(13개 반복 전부, e = 1/d 예외 0). F+0x100은 버리기 전 값의 min |d/원 대각|이다. ±clamp(1e-10|diag|, 1e-12, 1e-6) 규칙(노트 14 §3.1.7)은 F+0x110≠0·Bunch–Kaufman 경로라 LP에서는 안 탄다. `ldl.rs`를 이 규칙으로 바꿔 **BarIterCount 56 → 59/64**(`ddc3040`).
- **콜백 PRIMINF** = √(B+0x198² + B+0x1b8²) = √(‖r_p‖² + ‖r_u‖²)/τ(반복 0 오라클 64/64 비트, hypot 아님). 고친 뒤 상대 1e-6 안에서 궤적 전체가 같은 인스턴스 51/64(`df38e2a`). `tools/parity.py compare`에 `first_divergence_1e-6` 열을 더했다.
- **남은 불일치**(Rust(Gurobi), 첫 분기 = `bench/parity_bar.csv`의 `first_divergence_1e-6`): degen2 11(12) 끝 반복(Gurobi 값으로는 반복 11의 gap_rel 6.8e-8 > tol이라 계속, 끝에서 gap ~1e-4라 1e-6 상대 일치로는 판정이 뒤집힘), fffff800 79(76) 반복 28, modszk1 21(20) 반복 11, perold 161(130) 반복 19, pilot4 127(85) 반복 13. ±5% 밖: degen2(8%), perold(24%), pilot4(49%). 셋 다 규칙 차이 흔적은 없고(종속 행·자유 열로 조건이 나쁜 계), 분해·해법 비트 일치가 필요하다.

## 7. 일시 정지 메모 (2026-11-05, 사용량 한도, 해결됨)

**완료 기준 현황**(Presolve=0, Threads=1, Crossover=0, Netlib 64)

| 기준 | 상태 |
|---|---|
| (1) 상태·목적 ≤1e-6 | 64/64, 64/64 (Rust, `bench/parity_bar.csv`) |
| (2) BarIterCount 같음 ≥90% | **56/64 (87.5%)**, 목표 58/64. 불일치: agg 17(18), agg3 17(18), degen2 11(12), degen3 15(14), fffff800 83(76), maros 20(19), perold 165(130), pilot4 112(85) (괄호 Gurobi). 첫 분기점은 `docs/m3/m3_rust.md`, `docs/m3/m3_ipm.md` §8 |
| (3) ordering·Factor NZ | 64/64 (`tests/test_barrier.py`, parity `bar_stats`) |
| (4) 반복별 오라클 도구 | `tools/bar_oracle.py`(`--iters all`, `--vec`, WIP `--chol`) |
| (5) clippy·테스트·문서 | 내 파일 경고 0(남은 4개는 M1 파일), `cargo test --workspace`·pytest 통과, docs/05 갱신 |

**중단 시점 WIP**(커밋 `M3 chol WIP`, 비트 일치 에이전트가 남긴 것, 검증 안 됨):
- `tools/proto/bar_ipm.py`: LP 분해 피벗 규칙을 `2d8ee5` L767–815(F+0x110 = 0) 기준으로 바꿈: |d| ≤ 1e-16·|원 대각|(또는 NaN)이면 d = 0·역수 0, 아니면 그대로, 승수는 col·(1/d). ±clamp(1e-10|diag|, 1e-12, 1e-6) 대체는 F+0x110≠0 경로와 Bunch–Kaufman(`32af6a`, F+0xa0≠0) 경로라 이 64개에서는 안 탄다는 주장이다. **Rust `ldl.rs`는 아직 clamp 규칙**이다. 64개 중 36개까지 돈 부분 결과: fffff800 81(76)로 조금 나아졌지만 modszk1 21(20), degen3 15(14)가 새로 어긋남(agg, bnl1, degen2, perold는 그대로). 규칙 자체를 오라클(분해 결과)로 먼저 확인할 것.
- `tools/bar_oracle.py --chol`: F 안의 수치 인수(블록 시작 c마다 F+0xb8[c]에 w×w 하삼각 행 순 + (len−w)×w 직사각, F+0xe0 피벗 d, F+0xf0 조립 대각, F+0x100/0x114/0x118/0x178 스칼라)를 덤프.
- `tools/proto/bar_mkl_hook.py`: libgurobi 안 MKL DGEMM/DTRSM 입구를 메모리 패치(movabs+jmp)해 호출 인자·입출력을 기록하는 도구. 64개에서 실제 호출 여부와 모양(`--shapes`)을 보려던 것.

**다음에 할 일**(순서대로)
1. `bar_oracle.py --chol`로 분해 결과(d, L)를 뜨고, 피벗 규칙(1e-16 버림 대 clamp)을 오라클로 확정한다. 확정되면 `ldl.rs`(또는 `chol.rs`)와 프로토타입을 맞추고 64개 재측정.
2. `bar_mkl_hook.py --shapes`로 64개에서 DGEMM/DTRSM 호출 여부와 크기를 본다. 불리면 작은 입력으로 MKL 누적 순서·FMA 여부를 실험해 재현 규칙을 찾는다.
3. supernodal 수치 순서(`31c54b` 스케줄, `2d8ee5`/`2d946c`, AΘAᵀ 조립 순서), 해법(`2ee6c5` → `337be3`/`34d5fa`@0x34c880), SIMD 8레인·long double 합(`2d4977`, `2d252f`, `2f28c2`, `2f2e95`)을 프로토타입에서 비트로 맞춰 반복 0부터 비트 일치를 넓힌다. 그다음 `chol.rs`에 같은 `Factor` 시그니처(`new`, `factor(theta, theta_slack) -> (min_ratio, n_small)`, `solve(rhs_x, rhs_y) -> (dx, dy)`)로 쓰고 `ipm.rs`의 `use super::normal::Factor`를 바꾼다.
4. 목표: BarIterCount ≥58/64, 나머지 ±5%. 그 뒤 재시작·homogeneous·상태 13·`2c4689` 최선 해를 옮기고 오라클로 확인(64개에서는 안 걸림).

실행 메모: Rust 확장 빌드 `PYO3_PYTHON=$PWD/../.venv/bin/python cargo build --release -p rsolver && cp target/release/lib_core.so python/rsolver/_core.cpython-313-x86_64-linux-gnu.so`, rsolver 캡처 `PYTHONPATH=$PWD/python ../.venv/bin/python tools/parity.py capture rsolver <dir> --lp-only --param Method=2 --param Presolve=0 --param Crossover=0`(64개 약 2초), 프로토타입 검사 `PYTHONPATH=$PWD/python python tools/proto/bar_ipm_check.py`(조밀 LDLᵀ라 느림, BLAS 스레드 4개 제한 권장). Herdr pane w4:pH(내 것), w4:pN(비트 일치 에이전트 것)은 비어 있다.
