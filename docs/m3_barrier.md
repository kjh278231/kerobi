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

- 64개 모두 Dense cols가 0이다(밀집 열 검출은 m ≥ 1000에서만 돈다). Free vars가 있는 것은 4개(49, 72, 1, 2).
- BARRIER 콜백 값(`trace.csv.gz`의 `barrier` 행)의 PRIMINF는 로그의 Primal 잔차와 다른 노름이다(afiro 반복 0: 4471 대 2.27e3).

## 3. 오라클

- BARRIER 콜백의 cbdata(`model._callback._cbdata` 캡슐) **+0x68이 B**다(B+0x18 = B+0x1c = m, B+0x20 = n, `2ca086` L80–83).

## 4. 진행

| 단계 | 대상 | 상태 |
|---|---|---|
| 0 | 호출 범위·MKL 확인, 기준값 | 완료 |
