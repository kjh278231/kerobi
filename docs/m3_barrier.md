# M3 barrier 인수인계 메모

목표(G3, [11 §4](11_progress.md)): Method=2, Presolve=0, Threads=1, Crossover=0에서 barrier 본체를 Gurobi와 같게 만든다. 방식은 [10 §9](10_porting.md)의 재구성 + 오라클 비트 검증이다. 함수 이름은 `PRIVATE…`의 뒤 6자리, 주소는 file offset이다.

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

- **결정성**: 같은 설정으로 다시 캡처해 비교하면 64/64가 상태·BarIterCount·Barrier statistics·BARRIER 콜백 궤적(repr) 모두 같다(`bench/gurobi_bar_determinism.csv`, 2026-11-05, 다른 에이전트가 Gurobi 오라클을 돌리는 부하 중). 따라서 corrector 수 자동 조정 등은 벽시계에 기대지 않는다고 본다(코드 확인은 docs/m3_ipm.md).
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
| 2 | ordering·symbolic(AA' NZ, Factor NZ/Ops, 순열, supernode) | **완료 64/64**(AA' NZ, 최종 순열 F+0x30, supernode 블록 F+0x78, 열 길이·행 번호, Factor NZ/Ops, F+0x48). `crates/lp/src/barrier/ordering.rs` `analyze()`, 테스트 `bench_lps_match_gurobi`(기대값 `bench/oracle/bar_order/expected.tsv`). 64개 모두 최종 순열은 기준선(mode 3, AMD + 이분할)이고 ND 후보는 채택되지 않는다. 세부는 `docs/m3_ordering.md` |
| 3 | 표준형·초기점·잔차·반복 규칙(프로토타입) | **프로토타입 완료**(`tools/proto/bar_ipm.py`, `docs/m3_ipm.md`): 내부 LP 64/64 비트, 반복 0 점 ≤4e-12, 한 반복 1e-16~1e-9, corrector 수 K·work 타이머 정확히 같음(벽시계 아님). 전체: 상태 64/64, 목적 ≤1e-6 64/64, **BarIterCount 56/64**(±5% 안 57/64). 불일치 8개(agg, agg3, maros, degen2, bnl1, fffff800, perold, pilot4)는 해법 반올림이 쌓여 갈라짐 |
| 4 | 수치 분해(supernodal LDLᵀ, `31c54b` → `2d8ee5` supernode 커널, `32af6a` Bunch–Kaufman, MKL DGEMM/DTRSM) | 지금은 `ldl.rs`(스칼라 up-looking LDLᵀ + 작은 피벗 규칙, 반올림 수준 일치)와 `normal.rs`(AΘAᵀ 조립). supernode 커널 비트 이식은 반복 수가 분해 반올림 때문에 갈릴 때 한다. `2d8e22`: F+0x178 정규화 모드 = 안전 모드·SOC면 2, 자유 변수·밀집 열이면 1, 아니면 0. 분해 뒤 모드 0이고 F+0x100 < 1e-10이면 1로 올림 |
| 5 | Rust 본체, Python 경로(`Method=2`), `tools/parity.py capture rsolver` | 진행 중 |
| 6 | 비트 일치(분해·해법·합 순서)로 불일치 8개 줄이기 | 진행 중, `docs/m3_chol.md` |
