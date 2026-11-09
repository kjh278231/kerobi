# M3 PDHG 인수인계 메모

목표(G4, [11 §4](../11_progress.md)): Method=6, Presolve=0, Threads=1, Crossover=0에서 PDHG 본체를 Gurobi와 같게 만든다. 방식은 [10 §9](../10_porting.md)의 재구성 + 오라클 비트 검증이다(barrier와 같은 방식, [m3_barrier.md](m3_barrier.md)). 함수 이름은 `PRIVATE…`의 뒤 6자리, 주소는 file offset이다.

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
| 0 | 호출 범위·외부 의존, 기준값, 결정성 | **완료**(§1–2, `984af95`) |
| 1 | 오라클 `tools/pdhg_oracle.py`: PDHG 콜백 cbdata+0x68 = P(`34c004`의 `&local_878`), P 블록 0x730바이트, P가 가리키는 벡터(`--vec`), 스케일된 모델 D | **완료**(`1e820a2`) |
| 2 | 전처리 `07d3ce` ScaleFlag 4 | **완료** 64/64 비트(§5) |
| 3 | 본체: 초기화, 반복 커널, 평가, restart, ω, power iteration, 종료, work | **완료**(§4, §6) |
| 4 | Rust `crates/lp/src/pdhg/`(`scale.rs`, `kern.rs`, `eval.rs`, `mod.rs`), Python `Method=6` 경로(`CoreModel.pdhg`, `Model._pdhg`, PDHG 콜백·`cbGet`, `PDHGIterCount`) | **완료**(`60b0b84`) |
| 5 | 비트 일치, 반복별 비교 도구 `tools/pdhg_cmp.py`, 테스트 | **완료**(§7) |

해독은 하위 에이전트 4개가 asm까지 읽어 나눠 했다(스케일, 본체 흐름, 반복 커널, 평가·restart·step). 각 결과는 오라클로 확인한 뒤 Rust로 옮겼다.

**오라클 사용법**
- Gurobi 쪽(라이선스 필요): `python tools/pdhg_oracle.py <model.mps.gz> <out.jsonl[.gz]> --vec [--iters 0,93,193|all]`. 콜백마다 P 블록과 벡터를 덤프한다. 원하는 반복을 다 기록하면 `terminate`로 끝낸다.
- Rust 쪽(Gurobi 불필요): `PDHGDUMP=<file>` 환경 변수를 주고 rsolver Method=6으로 풀면 같은 시점의 P 필드 21개(오프셋 키)와 x, y, x̂, ŷ, x⁰, y⁰를 비트로 쓴다.
- 비교: `python tools/pdhg_cmp.py <oracle> <dump> [--all]`. 첫 차이가 나는 콜백·필드·벡터 인덱스를 보여 준다. 콜백 값만 비교하려면 `tools/parity.py compare`의 `first_divergence`(비트)·`first_divergence_1e-6`을 쓴다.

## 4. 커널 (`kern.rs`, 해독 2026-11-05, asm 기준)

- **FMA 없음.** 모두 SSE2(`mulsd/addsd`, 2레인 `mulpd/addpd`)라 곱·합마다 반올림한다. max/min은 x86 `maxsd`/`minsd` 의미(앞 피연산자가 엄격히 클/작을 때만 그것, NaN·±0이면 뒤 피연산자)다.
- **합 순서 `red8`**(SpMV `35645c`/`3565e5`, 내적 `2f9515`): 길이 < peel+8이면 0.0부터 차례로 더한다. 아니면 peel(0/1, 정렬 기준 피연산자가 8 mod 16에서 시작하면 1) 항을 A 레인 0의 시작값으로 두고, 8개씩 2레인 누산기 4개(A,B,C,D)에 더한 뒤 레인마다 (A+B)+(C+D), lo+hi, 남은 항을 차례로 더한다(Ghidra의 ((A+B)+C)+D는 틀림). A·x는 행 방향 사본(D+0x118/0x120/0x130/0x138, CSC를 열 순서로 전치한 것과 같음, afiro 확인), Aᵀy는 CSC. D+0x110·D+0x138은 16바이트 정렬(오라클)이라 peel = 구간 시작 & 1. 내적은 둘째 벡터가 peel을 정하고 1스레드에서 `0.0 + 내적`이다.
- **primal `35207d`**: τ = η/ω, t = x − (c − Aᵀy)·τ, x̂ = minsd(maxsd(t, l), u), x_ref = (x̂ + x̂) − x. 평가 반복이면 x̂(P+0xb8)와 (x̂ − t)/τ(P+0xc8)를 저장한다.
- **dual `3524ec`**: σ = η·ω, t = y − (A·x_ref)·σ, lo = (−σ)·b, ŷ = t − (‘=’면 lo, 아니면 maxsd(t, lo)), y_ref = (ŷ − y) + ŷ. '='만 검사하므로 부등식 행은 모두 y ≤ 0이다. D는 '>' 행을 σ = −1로 뒤집어 둔 상태다(A와 b의 부호, sense 문자는 '>' 그대로).
- **고정점 노름 `352bcb`**(평가 반복과 restart 직후에만, Halpern 앞): dx = x − x_ref, dy = y − y_ref, Aᵀdy, g = √((⟨dx,Aᵀdy⟩·2)·η + (‖dx‖²·ω + ‖dy‖²/ω))(음수·NaN이면 0) → P+0x110, restart 직후면 P+0x120에도.
- **Halpern `3528a6`**: w = (k+1)/(k+2)(k = P+0x10), z = z_ref·w + z⁰·(1 − w).
- **반복 순서**: primal → dual → [고정점 노름] → Halpern → P[0], P[1], P[2] += 1.
- **work**(1스레드, 덧셈 순서 그대로): SpMV마다 +nnz·3, primal +n·6, dual +m·6, Halpern +(n+m)·3.

## 5. 전처리 (`scale.rs`, `07d3ce(m, 4)`)

PDLP식이다. s = r = 1에서 Ruiz ∞-노름 10회(같은 (s, r)로 행·열 최댓값을 함께 구하는 Jacobi 갱신, 수렴 판정 없음), Pock–Chambolle(α = 1) 1회, [2⁻⁴⁷, 2⁴⁷]로 자르기(열은 abs 먼저)를 한다. 각 |r_i a_ij s_j|는 fl(fl(s_j·r_i)·|a|), 인수는 1/√x(x = 0이면 1)를 곱한다. **2의 거듭제곱으로 반올림하지 않는다**(flag 5만 한다). 적용: a' = fl(fl(r_i·s_j)·a), l/s, u/s(유한할 때), b·r(유한할 때), c·s 다음 목적 스케일 ω(2의 거듭제곱)를 곱한다. ω는 공통 꼬리의 LP 규칙 x = min(100/cmax, 1/(10t)), t = √(max(1e-6, cmin)·cmax), x < 2면 1, x > 1000이면 512, 아니면 2^⌊log₂x⌋다.

- **오라클 검증**: 64/64에서 열·행 인수, A, c, l, u, b, ω가 비트 같다(`D` 스냅숏, '>' 행 σ 반영).
- **M1 `scaling.rs` 참고**(오케스트레이터용, 이 브랜치에서는 고치지 않음): 같은 공통 꼬리의 목적 스케일을 `0.1/t`, `x ≤ 2`, x > 10000 상한으로 옮겼는데 asm은 `1/(10t)`, `x < 2`, x > 1000 → 512다. 64개에서는 결과가 같다.
- 옮기지 않은 분기: lb = −inf·유한 ub 열 뒤집기(`*(m+0xf8) == 0`일 때, 64개에서 bound 비트 일치로 안 탐 확인), 저장된 스케일 재사용(D+0x3f0), 모델 등급 D[0] = 1의 ObjScale −2 규칙, RHSSCALE ≠ 1.

## 6. 본체 흐름·평가·restart·step (`mod.rs`, `eval.rs`)

**초기화**(`34c004` L355–1583): `07d3ce(m,4)` 전에 원 데이터 사본을 뜬다(끝에 복원). wc = 1/s(D+0x3e0), wr = 1/r(D+0x3d8), ‖c‖ = √Σ(c·wc)², ‖b‖ = √Σ(b·wr)²(`red8` 순서, b는 ±1e30도 그대로), cnt(P+0x168) = 유한 bound 열 수 + 부등식 행 수, ω₀ = ‖c‖/‖b‖(하나라도 0이면 1), tol = 1e-4, ε = 1e100, η = 0. work += (nnz + 5m) + 9n. 시작점 x = clamp(0, l, u)(같으면 bound), y = 0, x̂·ŷ·anchor도 같은 값. A·x, Aᵀy(+3nnz씩). power iteration 시작 벡터(P+0x180)는 모델 RNG(`a7b54f`, state 0)에서 m개(afiro 27/27 비트).

**주 루프**(asm 0x36195b–0x361db1):
```
ev = 1
loop:
  rs = 0
  if ev or P[0] ≥ IterLimit:
     eval(first = P[0]==0); if P[1] % 100 == 0: 콜백(where 10); st = 355602; if st ≠ 1: 끝
     rs = 350614(); g₋ = g
     if rs ≠ 0 or P[0] == 0: (rs ≠ 0이면 350723) ; 351b1f(청크마다 콜백·355602, st ≠ 1이면 끝)
     work += 3(m+n)
  ev = (P[1]+1) % 100 == 0
  primal(ev); dual; if ev or rs: g = 352bcb (rs면 g₀ = g); Halpern(k = P[2]); P[0]++, P[1]++, P[2]++
끝: eval(first = 1); x = x̂·s(`7a069`), 목적 = P+0x38·sense
```
P[0]은 PDHG + power 반복(= 콜백 ITRCNT = PDHGIterCount), P[1]은 PDHG 반복만, P[2]는 restart 뒤 반복, P[3]은 power 반복 누계다. 콜백이 0, 93, 193…으로 찍히는 이유: 반복 0 평가 뒤 power iteration이 93회 돌며 청크 끝에 콜백을 한 번 더 부르고, 다음 평가는 P[1] = 100(P[0] = 193)이다.

**평가 `354824`**: A·x̂, Aᵀŷ(+3nnz씩). compl = Σ_j[l 유한·d > 1e-10: (x̂−l)·d, u 유한·d < −1e-10: (x̂−u)·d] + Σ_i[부등식·s > 1e-10: (−s)·ŷ](d = c − Aᵀŷ, s = b − Ax̂) / cnt, 0 이상. 원 잔차 r = (b − Ax̂)·wr, 위반량 v = maxsd(0, −r)('='면 maxsd(v, r)), ∞-노름(P+0x50)과 2-노름(P+0x60). pobj = (0 + `2f9515`(x̂, c))/ω_obj + objcon. 쌍대: first면 열마다 d·wc를 bound 종류별로(자유 |d|, 양쪽 유한 0, 위만 max(0,d), 아래만 max(0,−d)), dobj = Σŷb + Σ(d > 0·l 유한: d·l, d < 0·u 유한: u·d). first가 아니면 r = ((c − Aᵀŷ) − λ)·wc(λ = P+0xc8), dobj = Σŷb + Σ(λ > 0: λ·l, λ < 0: u·λ). 상대값 P+0x70 = ‖r_p‖₂/(‖b‖+1), P+0x78 = ‖r_d‖₂/(‖c‖+1), gap = |p − d|/((1 + |p|) + |d|). work += 8m + 14n. 콜백은 PRIMOBJ P+0x38, DUALOBJ P+0x40, PRIMINF P+0x60(2-노름), DUALINF P+0x68, COMPL P+0x48이고, 로그와 종료 판정은 ∞-노름(P+0x50/0x58)을 쓴다.

**종료 `355602`**: (‖r_p‖∞ < AbsTol && ‖r_d‖∞ < AbsTol && gap < ConvTol) 또는 (RelTol > 0 && rel_p < RelTol && rel_d < RelTol && gap < ConvTol) → 2. IterLimit ≤ P[0] → 7. WorkLimit·1e9 < W → 16. |pobj|, |dobj|, |compl| 중 1e100 초과·NaN → 12. 그래서 한도는 100회마다(와 power 청크마다)만 검사한다.

**restart `350614`**: P[1] < 100 → 0, == 100 → 1, g == 0 → 5, 0.2·g₀ ≥ g → 2, 0.5·g₀ ≥ g && g > g₋ → 3, P[1]·0.36 ≤ P[2] → 4. **`350723`**: Δx = ‖x̂ − x⁰‖, Δy = ‖ŷ − y⁰‖(`354f42`). Δ가 (1e-16, 1e12) 안이고 rel_d/rel_p가 (1e-8, 1e8) 안이면 e = (ln Δy − ln Δx) − ln ω, I = 0.3I + e, ω ← ω·exp(0.99e + 0.01I). 아니면 ω = ω₀, I = 0. 그다음 k = 0, g₋ = 1e100, anchor와 현재점을 (x̂, ŷ)로 둔다. "최선 균형 ω"(P+0xf0/0xf8)는 P+0xf8이 0에서 시작해 새 solve에서는 갱신되지 않는다.

**step `351b1f`**: AAᵀ(행 공간) power iteration. 예산은 처음 250, g == 0이면 1000(ε ≤ tol이면 tol ← max(0.1·tol, 1e-16)), 아니면 trunc(0.1·P[0] − P[3])이고, ε > tol이며 예산 > 100일 때만 돈다. 100회 청크마다: u = v/‖v‖(참 나눗셈), w = Aᵀu, v = Aw, λ = vᵀu, u = v − u·λ, ε = ‖u‖, tol > ε이면 멈춤. 청크 끝에 work += m + 8m·it, P[0]·P[3] += it, 콜백·종료 판정. 끝에 ‖A‖ = √λ, η = ((1 − ε) − ε)/‖A‖(ε ≥ 0.5면 0.01/‖A‖), work += 2(m+n). 첫 실행이 수렴하면(ε ≤ tol) 다시 돌지 않는다(afiro 93회).

## 7. 결과 (2026-11-05)

| G4 기준 | 상태 | 근거 |
|---|---|---|
| (1) 상태·목적 ≤1e-6(Gurobi PDHG 대비) | **64/64, 64/64**(목적값은 비트 같음) | `bench/parity_pdhg.csv`, `tests/test_pdhg.py` |
| (2) PDHGIterCount 같음 ≥90%, 나머지 ±5% | **64/64**(나머지 없음). WORK_LIMIT 4개(bnl1, ganges, perold, pilot4)도 같은 반복에서 멈춘다 | 같은 곳 |
| (3) 반복별 오라클 비교 도구 | `tools/pdhg_oracle.py`, `PDHGDUMP` + `tools/pdhg_cmp.py`, `tools/parity.py compare`(PDHG 콜백 행) | afiro 7/7, capri 30/30 콜백에서 P 필드·벡터 전부 비트 같음 |
| (4) clippy·테스트·문서 | `cargo clippy --workspace --all-targets` 경고 0, `cargo test --workspace`, pytest 통과 | 아래 |

- **PDHG 콜백 궤적**(100회마다 + power 청크마다)이 64/64 비트 동일하고 Work도 64/64 같다. 첫 분기점은 없다.
- 비트 일치에 필요했던 것: `red8` 합 순서(4누산기 2레인, (A+B)+(C+D)), 16바이트 정렬 기준 peel, x86 `maxsd`/`minsd` 의미, FMA 없음, '>' 행 σ, 1/s·1/r 가중치, glibc `log`/`exp`/`sqrt`(Rust `f64::ln`/`exp`가 같은 glibc를 부른다), 모델 RNG(state 0)로 만든 power iteration 시작 벡터.

**남은 위험·안 한 것**
- 64개에서 안 타서 옮기지 않은 분기: 스케일의 lb = −inf·유한 ub 열 뒤집기, 저장된 스케일 재사용, 모델 등급 D[0] = 1의 목적 스케일, RHSSCALE. 시작 벡터(warm start, `35709b`, P[4] = 1), Seed ≠ 0의 무작위 x 시작, PDHGBNDSTRENGTH, 조기 crossover, GPU, 여러 스레드(nnz ≥ 150,001이면 Threads를 따름), TimeLimit(벽시계)도 옮기지 않았다.
- "최선 균형 ω"(P+0xf0/0xf8) 갱신은 새 solve에서 일어나지 않아서 옮기지 않았다.
- 정렬: Gurobi 배열이 glibc malloc의 16바이트 정렬이라는 관찰에 기댄다. peel을 정렬이 아니라 인덱스 홀짝으로 정한다.
- 해 벡터: x = x̂·s만 돌려준다. y·slack·reduced cost(`7a069`의 rs/os 변환, `7a542`의 y 부호), Pi·RC 속성은 옮기지 않았다(crossover와 속성 마무리 쪽).
- Method=6이면 Presolve·Crossover 값을 보지 않는다(Presolve=0, Crossover=0을 가정).

**main 병합 메모**(오케스트레이터용): 새 파일은 `crates/lp/src/pdhg/*`, `tools/pdhg_oracle.py`, `tools/pdhg_cmp.py`, `tests/test_pdhg.py`, `bench/gurobi_pdhg_nopre/`, `bench/oracle/pdhg/`, `bench/parity_pdhg.csv`, `bench/gurobi_pdhg_determinism.csv`다. 기존 파일 수정은 `crates/lp/src/lib.rs`(모듈 등록 한 줄), `crates/py/src/lib.rs`(`pdhg` 메서드 추가), `python/rsolver/__init__.py`(PDHG 콜백 상수, `optimize`의 Method=6 분기, `_pdhg`, `ObjVal`은 SolCount가 있으면 OPTIMAL이 아니어도 값을 줌), `tools/parity.py`(PDHG 콜백·`PDHGIterCount`·`pdhg_iters` 열)뿐이다. M1 `scaling.rs`의 목적 스케일 차이는 §5에 적었다.
