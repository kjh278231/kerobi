# M3 PDHG 인수인계 메모

목표(G4, [11 §4](11_progress.md)): Method=6, Presolve=0, Threads=1, Crossover=0에서 PDHG 본체를 Gurobi와 같게 만든다. 방식은 [10 §9](10_porting.md)의 재구성 + 오라클 비트 검증이다(barrier와 같은 방식, [m3_barrier.md](m3_barrier.md)). 함수 이름은 `PRIVATE…`의 뒤 6자리, 주소는 file offset이다.

## 1. 호출 범위와 외부 의존 (2026-11-05, `re/decomp/callgraph.tsv`)

**진입**: `c23877`은 Method=6 && Crossover=0 && SolutionTarget≠1이면 `34f033`을 바로 부른다(노트 14 §1.2). `34f033`에서 Presolve=0(env+0x2424 = 0)이고 warm start가 없으면 presolve(`390989`)를 건너뛴다. m+0x1d8 = 0, PRESYMMPASSES(env+0x2c28) < 1, m+0x218 = 0이므로 대칭 축소(`32220a`)와 postsolve(`32578f`/`36578f`)도 타지 않고 `34c004(m, timers, work)`(본체)를 바로 부른다. 그 뒤 `2baa47`(LP이면서 저장된 Crossover env+0x2ba0 ≠ 0)이 거짓이라 crossover(`2baaa4`)를 부르지 않는다. 대신 `7a542`로 해를 설치하고, PDHGINSTALLSTART(env+0x2b34, 기본 1)이면 `10b63c`로 x·y를 시작 벡터로 넣는다(추정, 범위 밖).

**도달 함수**: `34c004`에서 정적으로 닿는 함수를 셌다. 다음 분기는 기본값에서 타지 않으므로 뺐다. 조기 crossover(env+0x2b40 = 1e100: `9951ed`, `356a46`, `2baaa4`, `3575a5`, 복제 모델을 해제하는 `88c65`(null 인자)), GPU(env+0x2b0c = 0: `bb0353` 등), PDHGBNDSTRENGTH(env+0x2b30 = 0: `3551b0`), 콜백 디스패처 `9a91f3`, 속성 마무리 `a6ad23`(드라이버, barrier도 거침). 이렇게 세면 **156개**다. PDHG 모듈 48개(그중 GPU 스텁 20개는 1–8바이트), common 61, libc·libm·pthread 32, simplex 5, driver 5, presolve 3, genconstr 2. 빼지 않고 세면 `88c65`(모델 해제)를 거쳐 드라이버 전체(2966개)가 잡힌다.

| 질문 | 답 | 근거 |
|---|---|---|
| simplex를 부르나 | 반복·pricing·LU는 **안 부른다**. 스케일링 `07d3ce`(와 보조 `07b892`, `10f720/789/888`)만 부르는데, **`34c004`가 직접 ScaleFlag 4로 부른다**(L139–160). ScaleFlag(env+0x243c)가 −1(기본)이면 4로 바꾸고, PDHGSAVEMATRIX(env+0x2b38)가 −1이면서 m이 원 모델(m+0x50)이면 obj·lb·ub·rhs(D+0x110/0x390/0x398/0x50/0x380) 사본을 떠 둔다. 0이면 4 + (원 모델이면 1)이다. flag 4·5 분기는 `07d3ce` L3836부터 약 1000줄이다. M1 `scaling.rs`는 flag 3만 옮겼으므로 **flag 4는 새로 옮긴다**(PDHG 전처리, 노트 14 §6.2-5의 미확인 항목) | `07d3ce <- 34c004` |
| 끝난 뒤 스케일 | 사본이 있으면 `7a069`(해 벡터 변환) → `f7dc1(m,2)` → 사본 복원 → `07b797` → `07d3ce(m,−1)`(기본 경로로 다시 스케일, crossover용) → `079dff` 순서로 처리한다(L2145–2195). 해 x·y의 원 공간 값은 `7a069`가 정한다 | 같은 함수 |
| LU를 부르나 | **안 부른다** | 도달 집합 |
| presolve를 부르나 | 본체는 **안 부른다**. 도달하는 presolve 함수 셋은 `7a069`(해 스케일 변환, barrier·crossover·postsolve 공용), `a71757`(`07d3ce`의 로그 보조 `a9a78e`가 부름), 타이머 `ade8ca`다 | `7a069 <- 34c004` |
| barrier와 공유 | **`2f9515`(8레인 내적)**를 평가 `354824`와 노름 보조 `354e23`이 부른다. Rust barrier에서는 `barrier/ipm.rs`의 비공개 `dot8`이 이 함수다. 그 밖에는 스레드 풀(`ac761d`/`ac7bef`), pthread barrier(`ae19b9`), 메모리·로그·타이머 common만 공유한다 | `2f9515 <- 354824, 354e23` |
| MKL | **없다**. BLAS 호출이 도달 집합에 없다. 행렬-벡터 곱은 자체 `35645c`(A·x), `3565e5`(Aᵀ·y)다 | libc 목록 |
| libm | `sqrt`, `log`, `log10`, `exp`, `pow`, `floor`, `round`, `scalbn`, `ilogb` | 같은 목록 |
| 스레드 | nnz(A) < 150,001이거나 GPU이면 1개(L381). 64개 중 가장 큰 nnz는 degen3 24,646이라 모두 1스레드이고 작업 스레드(`34d5fa`@0x35e120)를 띄우지 않는다. `ae19b9(1)`로 pthread barrier는 만든다 | L381–386, L971–990 |
| work | 시작 때 `*work += nnz + 5m + 9n`(L879). 반복 중 누적 규칙은 §4에서 확정한다. WorkLimit 판정이 이 값을 쓴다 | L879 |

결론: PDHG 본체는 M1 simplex·LU·presolve, barrier의 분해·MKL과 독립이다. 공유하는 것은 스케일링 함수 `07d3ce`(다른 분기인 flag 4)와 8레인 내적 `2f9515`이다. `crates/lp`의 기존 파일은 고치지 않으므로 둘 다 새 모듈 `crates/lp/src/pdhg/`에 둔다. `scaling.rs`의 비공개 보조(`round_pow2` 등)는 필요하면 복사한다. MKL이 없으니 비트 일치는 SpMV·내적 합 순서·libm만 맞추면 된다.

## 2. 기준값

| 디렉터리 | 설정 | 내용 |
|---|---|---|
| `bench/gurobi_pdhg_nopre/` | Method=6, Presolve=0, Crossover=0, Threads=1, WorkLimit 60 (LP 64개) | `tools/parity.py capture`(PDHG 콜백 where=10과 `PDHGIterCount` 추가). **OPTIMAL 60개, WORK_LIMIT(16) 4개**(bnl1 1,254,909회, ganges 864,369회, perold 1,091,983회, pilot4 1,357,966회). PDHGIterCount 593(afiro)–971,921(forplan). 콜백은 100회마다 한 행(`trace.csv.gz`의 `pdhg` 행: ITRCNT, PRIMOBJ, DUALOBJ, PRIMINF, DUALINF, COMPL) |

- **결정성**: 같은 설정으로 다시 캡처해 비교하면 64/64가 상태, PDHGIterCount, PDHG 콜백 궤적(repr), Work 모두 같다(`bench/gurobi_pdhg_determinism.csv`, 2026-11-05). WORK_LIMIT 4개도 같은 반복에서 멈춘다. 따라서 work 누적은 벽시계가 아니라 결정적 계수다(bnl1: 10.42초에 60.00 work units).
- WORK_LIMIT 4개의 상태와 반복 수를 맞추려면 work 누적 규칙이 Gurobi와 같아야 한다. 목적값은 수렴하지 않은 반복점의 값이므로 궤적이 1e-6 안에서 같아야 맞는다.
- 로그는 반복 0과 마지막 반복만 찍는다(5초 간격 추정). 반복별 비교는 콜백 궤적과 오라클(§3)로 한다.

## 3. 진행

| 단계 | 대상 | 상태 |
|---|---|---|
| 0 | 호출 범위·외부 의존, 기준값, 결정성 | **완료**(이 문서 §1–2) |
| 1 | 오라클: PDHG 콜백 cbdata에서 P 구조체(노트 14 §4.4)와 스케일된 작업 모델 읽기 | 다음 |
| 2 | 전처리: `07d3ce` ScaleFlag 4 분기(L3836–) | |
| 3 | 본체: 초기화(ω, η power iteration `351b1f`), 반복(`35207d`/`3524ec`/`3528a6`/`352bcb`), 평가 `354824`, restart `350614`/`350723`, 종료 `355602`, work 누적 | |
| 4 | Rust `crates/lp/src/pdhg/`, Python `Method=6` 경로, PDHG 콜백·`cbGet`, `tools/parity.py capture rsolver` | |
| 5 | 비트 일치(SpMV·내적 합 순서, libm), 64개 PDHGIterCount | |
