# M3 barrier 반복(IPM) 해독 노트

Gurobi 13.0.3 barrier(Method=2, LP, Presolve=0, Crossover=0, Threads=1)의 수치 규칙을 정리한다. 규칙은 의사코드와 objdump로 읽었고, Python 프로토타입(`tools/proto/bar_ipm.py`)을 오라클(`tools/bar_oracle.py`)과 맞춰 확인했다. 함수 이름은 `PRIVATE…`의 뒤 6자리다. B는 barrier 작업 구조체, F는 인수 구조체, D는 모델 데이터다([m3_barrier.md](m3_barrier.md) §3).

## 0. 결과 요약 (2026-11-05)

| 단계 | 대상 | 일치 | 검증 |
|---|---|---|---|
| 표준형 | MPS에서 만든 내부 LP(`lp_from_model`): A, b, c, lb, ub, 유형, ncomp, 이동 상수, ‖b‖, ‖c‖ | **64/64 비트** | `lp_from_oracle`과 비교 |
| 반복 0 | 초기점 x, z, w, s (y는 영공간 성분 제외) | **64/64** (상대 ≤ 4e-12) | it0 오라클 |
| 반복 0 | pobj, dobj, 잔차 노름, compl, μ, θ | afiro·sc50a·adlittle·25fv47·perold에서 1e-15 수준 | `--vec` 오라클 |
| 한 반복 | 오라클 상태 k → k+1 (x, y, z, w, s, α_p, α_d) | 기록한 모든 반복에서 1e-16~1e-9 (afiro, sc50a, adlittle, perold 0–39, 25fv47 0–11) | `--vec` 오라클 |
| 보정 수 K | work 타이머(B+0x2a8…) 값과 K 수열 | 타이머 **정확히 같음**(afiro, perold) | `--vec` 오라클의 raw B |
| 전체 | 상태 | **64/64** | `bench/gurobi_bar_nopre` |
| 전체 | 목적 상대오차 ≤ 1e-6 | **64/64** (최대 5e-8) | `summary.json` ObjVal |
| 전체 | **BarIterCount 같음** | **56/64** (±5% 안 57/64) | 같은 곳 |

**2026-11-06 갱신(Rust, [m3_chol.md](m3_chol.md))**: 분해·해법·합 순서를 비트로 맞추고 재시작을 옮겨 **BarIterCount 64/64**, BARRIER 궤적 비트 동일 55/64, 상대 1e-6 안 63/64(modszk1 반복 16)이다. 반복 상태 x, y, z, w, s는 afiro·adlittle·sc50a 전 반복, perold 반복 0–3·50, fffff800 0–5에서 원소 전부 비트 같다. 아래 표와 §7·§8은 조밀 LDLᵀ 프로토타입 시절 기록이다.

전체 실행은 `PYTHONPATH=$PWD/python python tools/proto/bar_ipm_check.py`로 한다. 선형계는 조밀 LDLᵀ(Gurobi의 작은 피벗 규칙 포함)라서 값은 반올림 수준까지만 같다. 불일치 8개는 §8에 있다.

## 1. 내부 표준형 (`lp_from_model`, 64/64 비트)

데이터 흐름은 `2ca086` → `07d3ce`(스케일) → `2e9086`(행 순열) → `30f5b9` → `2eb47b`(열 안 행 정렬) → `312a1e`(lb 이동) 순서다.

1. **스케일**: M1 `scale_flag3`과 같다. a' = σ·a·r_i·s_j, b' = σ·b·r_i, c' = c·s_j·objscale, 유한 bound는 /s_j(±1e30은 그대로)다. 인자가 2의 거듭제곱이라 곱하는 순서와 무관하게 비트가 같다.
2. **'>' 행**: D에서 값과 rhs의 부호를 바꾸고 sense 문자는 '>'로 둔다(σ = −1). 그래서 등식이 아닌 행은 모두 `Ax + s = b`, s ≥ 0이다. slack 계수는 +1이다.
3. **행 순열**: ordering 결과 `perm[i]`(원래 행 i의 최종 위치)로 행을 재배치한다. 행 순서가 중요한 곳은 `30f5b9`의 행 순회와 분해 피벗 순서다. 바인딩은 `rsolver.read_model(p).barrier_symbolic()[0]`이다.
4. **열 뒤집기 없음**: 64개에 원래 상한만 있는 열이 없어서 `07d3ce`의 x' = −x를 확인하지 못했다. barrier가 만드는 상한만 있는 열(아래 5)은 뒤집지 않는다.
5. **`30f5b9`**:
   - (a) 자유 열이 있으면 원래 bound 사본으로 행 활동량 전파를 2패스 한다(`a7d29c`). 행은 순열 순서, 행 안 원소는 열 오름차순이다. '=' 행은 s = −1, +1 두 방향, 나머지 행은 s = +1이다.
     - slack = s·b − minact(s·a·x)이고, slack ≥ −1e-6일 때만 갱신한다.
     - 무한 항이 0개이면 s·a > 1e-6에서 ub = min(ub, slack/(s·a) + lb), s·a < −1e-6에서 lb = max(lb, slack/(s·a) + ub)이다.
     - 무한 항이 1개이면 그 열 하나에 slack/(s·a)를 대입한다.
     - 그다음 자유 열마다 min(|lb'|, |ub'|) < 1e6이면, |ub'| ≤ |lb'|일 때 ub = ub' + 0.01, 아니면 lb = lb' − 0.01로 bound를 하나만 단다. 남은 자유 열 수가 B+0x208이다.
   - (b) BARLOOSE(env+0x2bdc, 기본 1): 행마다 min/max 활동량(무한대 검사 없음)을 구한다. 그 행으로 함의되는 boxed 열의 ub를 1e100으로 지운다. 조건은 a > 1e-4이고 (b − minact) < 0.9999·a·(ub−lb)이거나, 음수 원소가 없고 (b − minact) − 1e-10 < a·(ub−lb)일 때다. '=' 행의 a < −1e-4 쪽도 같은 형태다. 한 행에서 한쪽 bound는 한 번만 지운다.
   - (c) 유형 T: 열은 bit0 = lb > −1e30, bit1 = ub < 1e30이다. slack은 '='이면 0, 아니면 1이다.
   - (d) boxed 열은 ub ← ub − lb(범위 u)다. 고정 열은 u = 0인 boxed가 된다.
   - B+0x630/0x634는 ub < 1e30인 첫 열과 마지막 열+1이고, B+0x180 = max|u|다.
6. **`312a1e`**: 유한하고 0이 아닌 lb를 가진 열마다 열 순서로 b −= A_j·lb_j, B+0x618 += c_j·lb_j를 한다. 내부 x는 x − lb다.
7. **`2ca086`에서 먼저 정해 두는 값**(5, 6보다 앞, 원래 bound와 rhs 기준):
   - ncomp(B+0x210) = #(lb > −1e30) + #(ub < 1e30) + #(등식이 아닌 행)
   - ‖b‖₂, ‖b‖∞(B+0x160/0x168), ‖c‖₂, ‖c‖∞(B+0x170/0x178)
   - ptol B+0x260 = (√(m+1) + ‖b‖₂)·tol, dtol B+0x268 = (√(n+1) + ‖c‖₂)·tol, B+0x270 = B+0x278 = 10·tol
   - 등식이 아닌 행의 rhs는 1e30에서 자른다.
8. 목적: pobj = (cᵀx + B618·τ)/objscale/τ + objcon(D+0x60), dobj = (bᵀy − uᵀs + B618·τ)/objscale/τ + objcon이다.

## 2. 초기점 `2d0663` (BARSTART=0)

1. θ⁰: 자유 열 1e-6, boxed 0.5, 나머지 열 1, slack은 '='이면 0, 아니면 1이다. A θ⁰ Aᵀ를 분해한다(타이머 B+0x2a8, work 0.75·F+0x168).
2. x는 `2ee6c5(rhs_x = −u (T&2 열), rhs_y = b)`로 구한다. 즉 x = θ⁰(Aᵀy − rhs_x)이고 A θ⁰ Aᵀ y = b + A θ⁰ rhs_x다. w = u − x다(T&2).
3. 쌍대 후보 1: y = 0, z = c(T=1), z = c/2·s = −c/2(T=3), s = −c(T=2), 자유 열과 slack은 0이다.
4. 쌍대 후보 2: `2ee6c5(rhs_x = [c; 0], rhs_y = 0)`로 v, y를 얻고, z = −v(T&1, slack 포함), s = v(T&2)로 둔다.
5. 이동 `2d252f`:
   - δx = max(−1.5·min(x_lo, w), 1e-4), δz = max(−1.5·min(z_lo, s), 1e-4)
   - 쌍 순서(열의 lo, 열의 up, 그다음 slack)로 Σ(x+δx), Σ(z+δz), Σ(x+δx)(z+δz)를 더한다.
   - δx' = ½Σxz/Σz + δx, δz' = δz + ½Σxz/Σx. 분모가 0이면 100이다.
   - LP이면 √(‖b‖∞·1.0001·δx') ≥ δx'일 때 δx' = √(δx'·‖b‖∞·1.0001)이다(δz'는 ‖c‖∞로 같은 방식).
   - 하한은 0.1(B+0x30 = BARSAFE = 0, 안전 모드 아님)이다. 재시작 때는 B+0x30이 하한이 된다.
6. 후보 2의 δz'가 후보 1의 δz'보다 작으면(또는 BARSTART=2) 후보 2를 쓴다.
7. x += δx(T&1, 1e10 상한), 그 밖의 x는 ±1e10으로 자르고, z += δz, w += δx, s += δz, y는 ±1e10으로 자른다. B+0x38 = min(δx, δz)다.
8. `2d4977`로 compl을 구한 뒤 `2d8838`로 재중심화한다.
   - T&1 원소마다 t = μ/x − z를 r_d로 자른다: t > 0이면 min(t, max(r_d, 1e-20)), 아니면 max(t, min(r_d, −1e-20)).
   - z += t, r_d −= t를 한다. 상한 쪽은 μ/w − s를 −r_d로 잘라 s += t, r_d += t를 한다.
   - 그래서 오라클 반복 0의 compl(1689.89, afiro)은 이 보정 뒤의 값이다.

## 3. 잔차·목적·θ `2d4977` (매 반복, 콜백 직전)

- r_p = bτ − Ax − s, r_d = cτ − Aᵀy − z + s(열), r_d[n+i] = −y_i − z_{n+i}, r_u = uτ − x − w(T&2)다.
- 2-노름: B+0x198 = ‖r_p‖/τ(콜백 PRIMINF), B+0x1a8 = ‖r_d‖/τ(DUALINF), B+0x1b8 = ‖r_u‖/τ다.
- 로그 Primal = B+0x1d8 = max(‖r_p‖∞, ‖r_u‖∞)/τ다. 로그 Dual = B+0x1e0 = max_j(T&1이고 r_d < 0이면 max(−(r_d+z), 0), 아니면 |r_d|)와 max(등식이 아닌 행의 y_i)를 합친 값/τ다.
- compl B+0x1f0 = (τκ + Σ_{T&1} xz + Σ_{T&2} ws)/ncomp(콜백 COMPL), μ = B+0x1f8 = compl/τ²다.
- **재중심화**: 반복 ≥ 11이고 μ 이력[it−1] > 0.99·μ 이력[it−2]이면 `2d8838`과 같은 보정을 compl로 한다. 잔차 노름은 보정 전에 구하고, r_d 벡터는 보정된다.
- θ(B+0x808):
  - d = z/x(T&1) + s/w(T&2)이고 이 값이 B+0x800이다. d > 0이면 θ = min(1/d, 1e12)다.
  - 자유 열(d ≤ 0, BARFREEPERT 자동 = 1)은 θ = 1/clamp((|pobj|+1)·1e-8/max(|x|,1e-10)², 1e-12, 1e-6)다.
  - '=' slack은 0이다.
- predictor RHS: rxz = −xz(T&1), rws = −ws(T&2), rhs_x = r_d − rxz/x + (rws − r_u·s)/w다.

## 4. 한 반복 (`2bf1ea` L455–1030)

순서: `2d4977` → 콜백 → `2c4689`(이력·최선 해 저장) → `b2aecf` → 분해 `2d8e22` → `b2aecf` → predictor → correctors → K 조정 → step 제한 → `2f2e95` 갱신 → 재시작/homogeneous 검사.

1. **분해**: A θ Aᵀ(+ slack θ) LDLᵀ, 피벗 순서는 행 순열 순서다(`32af6a` L264–280).
   - 피벗 |d| ≤ t = clamp(1e-10·|원 대각|, 1e-12, 1e-6)이면 d = ±t(부호 유지)로 바꾸고 카운터를 올린다.
   - F+0x100 = min |d/원 대각|이다.
   - F+0x178(정규화 모드) = 1이면 `2f59f1`이 refinement를 한다. 조건은 자유 열이 남았거나(B+0x208 > 0) F+0x100 < 1e-10일 때다. 안전 모드나 SOC는 2다.
2. **방향 `2f59f1`/`2ee6c5`**:
   - 연립식은 −Θ⁻¹dx + Aᵀdy = rhs_x, A dx = rhs_y다. (AΘAᵀ)dy = rhs_y + AΘ·rhs_x, dx = Θ(Aᵀdy − rhs_x)로 푼다.
   - 나머지는 dw = r_u − dx(T&2), ds = (rws − s·dw)/w, dz = (rxz − z·dx)/x(T&1, 나머지 0)다.
   - mode 1(F+0x178 = 1)이면 `2f9699` refinement를 한다.
     - 잔차 r_y = rhs_y − A dx, r_x = rhs_x − Aᵀdy + D∘dx(D = B+0x800, 섭동 없는 값)다.
     - ‖r_y‖² + ‖r_x‖² ≥ B+0x280²인 동안 보정을 같은 인수로 풀어 더한다. 새 노름 < 0.99·이전일 때만 받고, 최대 5번 받는다.
     - B+0x280 = max(√(rp2² + rd2²)/10, 1e-8)다.
     - 최종 ‖r_y‖, ‖r_x‖를 B+0x770/0x780(+8k)에 둔다. refinement가 없으면 0이다.
3. **predictor step `2f15ba`**: α를 1.001에서 시작해 x(T&1), w(ub 범위), z(T&1), s 순서로 원소마다 `a·d + v < 0`이면 a = −(v − 1e-30)/d − 1e-13으로 줄인다. 이 순차 규칙은 min 비율과 다르다. 줄이는 인자는 없고, max(a, 0)이다.
4. **σ**: μ_aff = max(Σ(x+α_p dx)(z+α_d dz) + Σ(w+α_p dw)(s+α_d ds), …)/ncomp(`2f28c2`, 하한 1e-30)다. r = μ_aff/max(compl, 1e-30), σ = r²·min(r, 0.1)(QP는 0.9)다. μ_t = compl·σ이고, B+0xf4 < it(직전 step이 작았음)이면 μ_t = 0.9·compl이다.
5. **corrector k = 1..K** (`2d64b4`, `2f59f1` mode 1, `2f4fe9`):
   - k = 1(Mehrotra): rxz = μ_t − dx_aff·dz_aff, rws = μ_t − dw_aff·ds_aff다.
   - k ≥ 2(Gondzio): 시행점 α' = min(1.5α + 0.3, 1.001)에서 v = (x+α'dx)(z+α'dz)를 구한다. v < μ_t/10이면 μ_t/10 − v, v > 10μ_t이면 max(10μ_t − v, −10μ_t), 아니면 0이다. 여기서 dx는 지금까지 합친 방향이다.
   - corrector RHS에는 r_p, r_d, r_u 항이 없다: rhs_x = −rxz/x + rws/w, rhs_y = 0이다.
   - `2f4fe9` 선 탐색:
     - N = B+0x29c개의 ω를 1에서 lo까지 등간격으로 둔다. lo는 k = 1이면 clamp(α_pα_d, 0.01, 1), k ≥ 2이면 0이다.
     - 각 ω에서 방향 d0 + ω·d1로 `2f15ba` 비율 검사를 한다. ω = 0이면 검사 없이 현재 α를 쓴다.
     - primal과 dual이 따로, 처음으로 1.0001배 넘게 나아지는 ω를 고른다. 그다음 d0 += ω·d1(primal은 dx, dw; dual은 dz, ds, dy)로 합친다.
     - B770 = max(B770, ω_p·B778)이다.
   - 멈춤 조건: ω_p = ω_d = 0, 두 α ≥ 1.001, k > 1이고 α_pα_d < 1e-100, k > max(1, K/10)이고 0.99·새 곱 < 이전 곱, 또는 B770·B780 > 1e30.
6. **K 자동 조정**(BarCorrectors = −1). 타이머는 **결정적 work**다. 벽시계가 아니다.
   - 타이머 값 = 구간 안의 work 증가 + 시작 횟수(`ade8ca`: t −= work − 1, `ade770`: t += work)이고, 11개 타이머는 `ade552(t, 0)`으로 벽시계 플래그가 꺼져 있다. 콜백 오라클의 B+0x2a8/0x2e8/0x328/0x368/0x3a8 값과 프로토타입 값이 정확히 같다(afiro, perold).
   - ratio = t(추가 corrector 0x368) / (t(분해 0x2a8) + t(predictor 0x2e8) + t(첫 corrector 0x328) + t(갱신 0x3a8))다.
   - work 증가(0.75를 곱한 단위, nub = B+0x634 − B+0x630, B5a8 = 2·nnz(A)):

     | 함수 | 증가 |
     |---|---|
     | 분해 `2d8e22` | 0.75·F+0x168 |
     | 해법 `2ee6c5` | (2·B5a8 + F+0x160 + 11m + 4n)·0.75 |
     | `2f59f1` 추가 | (5(m+n) + 9·nub)·0.75 |
     | 비율 검사 `2f15ba` | (7(m+n) + 6·nub)·0.75. `2f4fe9`는 ω ≠ 0인 격자점마다 같은 양 |
     | corrector RHS `2d64b4` | (3(m+nub) + 13n)·0.75 |
     | refinement `2f9699` | 1.5·B5a8 + 보정당 (해법 + 1.5·B5a8) |
     | 갱신 `2f2e95` | 0 |

   - base = max(1, ⌊log10 n⌋ − 2), avg = 최근 세 α_pα_d 평균(it < 3이면 1)이다.
     - K < 20이면: ratio < 0.25 → K+1. 아니면 K < base이고 ratio < 0.5이고 avg < 0.1 → K+1, K > base이고 ratio > 0.25 → K−1, 그 밖에는 K > 1이고 ratio > 0.25이고 avg > 0.1 → K−1.
     - K ≥ 20일 때는 `adjust_K`에 있다.
   - N = max(3, min(K+1, 10))이다. 직전 두 step이 모두 작으면(B+0xf4 < it) K = 10(K ≥ 11이면 유지), N = 10이다.
   - **시작값 `2e7f38`**(`31e114`에서 한 번): r = (2·B5a8 + F+0x160)/F+0x168, K₀ = int(clamp(0.1/r, 1, 20)), N₀ = r > 0.2이면 3, 아니면 5다.
     - F+0x160 = 2·nnz(L)이다.
     - F+0x168은 symbolic 통계(F+0x48, 0x50, 0x60, 0x68), 행 수, CPU 종류 상수(`a80670`: 50/30/18/10)로 정해진다. 그래서 머신의 명령어 집합에 따라 달라질 수 있다.
7. **step 제한**: μ < 1e-4이고 B770 > max(1.2·rp2, ptol)이면 α_p ≤ max(1.2·rp2, ptol)/B770이다. 쌍대 쪽도 같다(refinement가 있을 때만 의미가 있다).
8. **갱신 `2f2e95`**:
   - η₀ = 1e-6(안전 모드 0.1, SOC·QP 1e-3)이다. 먼저 a = min(α(1−η₀), 1)로 둔다. a_p·a_d > 5e-5이면 B+0xf4 = it + 1이다.
   - 시행점 곱 p = (x+a_p dx)(z+a_d dz)(T&1)와 (w+a_p dw)(s+a_d ds)(T&2)를 구하고, avg = Σp/ncomp로 둔다.
   - min p < 0.01·avg이면 η = max(η₀, 0.01·η₀·avg/min p)이고, 그다음 η ≤ 0.5다.
   - 최종 a = min(α(1−η), 1)이다. x, z(T&1)와 w, s(T&2)는 max(·, 1e-30)이고, 나머지 x, z는 그대로, y += a_d·dy다.
   - sqrtl은 SOC 전용이다.
9. **재시작/homogeneous**(L896–1017). 재시작은 **perold(반복 99), pilot4(57)에서 발동**하고 Rust에 옮겼다(`ipm.rs` `restart_due`):
   - 갱신 뒤 B+0xf8 = 0(아직 재시작 안 함)일 때 cg' = ncomp·compl/(|ctx|+1)(rp_rel, rd_rel < 1e-7이면 ×0.1), r = μ[it]/μ[it−3](it − f8 < 3이면 0, μ[it−3] = 0이면 1).
   - 조건: cg' > 1e-4이고, (step 곱 B+0x5e0/0x5e8/0x5f0 셋 다 < 0.001, r > 0.98, B+0x770, B+0x780 < 0.001) 또는 (B+0x5e0 = B+0x5e8 = 0).
   - 그러면 B+0x30 = max(1000, 10·B+0x38)(이동 하한), `2d0663` 재호출(분해 타이머 B+0x2a8에 1 + 0.75·F+0x168), B+0xf8 = it + 1. 이 반복은 "직전 step이 작으면 K = 10" 규칙을 건너뛴다.
   - homogeneous 전환(잔차가 1e15·max(1, ‖b‖∞)를 넘거나, 반복 ≥ 10에서 1e8배를 넘고 줄지 않음)은 64개에서 안 탄다(옮기지 않음).

## 5. 종료 `b2aecf` (Crossover=0이면 B+0x2a0 = 0)

- gap_rel = max(|p−d|/(|p−objcon|+1), |p−d|/(100|p|+1)), cg = compl·N/(|cᵀx|+1), mg = N·μ/(|p|+1)이다.
- fast = (μ > tol이고 μ[it] < 0.5·μ[it−1])다.
- pfeas = √(rp2² + ru2²) < ptol 또는 B1d8 < 10tol, dfeas = rd2 < dtol 또는 B1e0 < 10tol이다.
- OPTIMAL 조건(하나라도 맞으면):
  - (gap_rel < tol, mg < 100tol, pfeas, dfeas, κ < 100tol·τ, !fast)
  - (cg < tol, mg < 100tol, gap_rel < 0.1, pfeas, dfeas, !fast)
  - (gap_rel < tol, cg < tol, pfeas, dfeas, it > 1, μ[it] > 0.5·μ[it−1])
  - (μ < 1e-16, pfeas, dfeas)
  - (it > 10, gap_rel < 10tol, pfeas, dfeas, compl > 0.8·compl[it−3])
- 정체(13)·cutoff(6)·infeasible/unbounded 규칙은 에이전트 해독 결과를 [§8](#8-미해독확인-못-한-규칙)에 남겼다. 프로토타입은 13을 내지 않는다(64개 모두 2).
- 이력(`2c4689`, 반복 it 위치에 기록, 100개씩 늘림): 0x48 pobj, 0x50 rp2, 0x58 ‖r_p‖, 0x60 dobj, 0x68 rd2, 0x70 ‖r_d‖, 0x78 μ, 0x80 compl다.

## 6. 오라클 (`tools/bar_oracle.py`)

- 첫 기록: raw B(0xa00), F(0x200), lb·ub·obj·rhs(B), 유형(B+0x640), **D 스냅숏**(`snap_D`: 스케일·순열된 A, sense, rhs, bound, 스케일 배열, objscale, objcon).
- `--vec`: 반복마다 raw B, 이력 배열, 잔차·RHS·θ·방향 벡터(VEC 표). 방향은 직전 반복의 최종(합친) set 0과 마지막 corrector set 1이다.
- 저장된 덤프:
  - `bench/oracle/bar/<name>.jsonl.gz`: afiro, sc50a, adlittle은 전체, 25fv47은 0–3·20, perold는 0–3·50.
  - `bench/oracle/bar/it0/`: 64개의 반복 0 전체 기록.

## 7. 64개 결과 (`bar_ipm_check.py`, 반복 수 프로토타입(Gurobi))

모든 인스턴스에서 상태가 같고 목적 상대오차 ≤ 1e-6이며, 반복 0 점이 1e-12 안이다. 반복 수가 다른 8개는 굵게 표시했다.

25fv47 23(23), adlittle 12(12), afiro 7(7), **agg 17(18)**, agg2 18(18), **agg3 17(18)**, bandm 16(16), beaconfd 9(9), blend 8(8), **bnl1 48(49)**, boeing1 18(18), boeing2 14(14), bore3d 16(16), brandy 13(13), capri 22(22), **degen2 11(12)**, degen3 14(14), e226 17(17), etamacro 23(23), **fffff800 83(76)**, finnis 22(22), fit1d 15(15), fit1p 11(11), forplan 18(18), ganges 16(16), gfrd-pnc 15(15), grow15 12(12), grow22 12(12), grow7 10(10), israel 16(16), kb2 16(16), lotfi 17(17), **maros 20(19)**, modszk1 20(20), **perold 160(130)**, **pilot4 121(85)**, recipe 8(8), sc105 7(7), sc205 9(9), sc50a 7(7), sc50b 7(7), scagr25 15(15), scagr7 14(14), scfxm1 15(15), scfxm2 15(15), scfxm3 17(17), scorpion 10(10), scrs8 17(17), scsd1 8(8), scsd6 9(9), sctap1 16(16), sctap2 10(10), seba 12(12), share1b 19(19), share2b 9(9), shell 21(21), ship04s 13(13), stair 14(14), standata 10(10), standgub 10(10), standmps 15(15), stocfor1 9(9), tuff 10(10), vtpbase 50(50).

## 8. 불일치와 미해독/확인 못 한 규칙

| 인스턴스 | 첫 분기(1e-6) | 원인 추정 |
|---|---|---|
| agg, agg3 | 17, 16 (끝 반복) | 마지막 반복의 해법 정확도: Gurobi는 반복 16 rp = 1.6e-6, 프로토타입은 4.9e-8이다. 그래서 반복 17의 compl이 2.8배 차이 나고 종료가 한 반복 일찍 온다. 수치 분해 반올림 영역이다 |
| degen2 | 9 | 종속 행: 피벗 대체로 dy에 영공간 성분이 생기고, 그 크기가 반올림 잡음이다(반복 0부터 y가 Aᵀ(Δy) ≈ 1e-13인 벡터만큼 다름). 반복 4부터 z 오차가 반복마다 약 30배씩 커진다 |
| maros | 19 (끝) | agg와 같은 종류(종료 직전 반올림)로 추정한다 |
| bnl1 | 30 | 반올림 증폭: pobj·compl 오차가 반복 0의 5e-15에서 조금씩 커져 반복 20에 9e-9, 30에 2e-4가 된다. 한 반복에서 갑자기 뛰는 곳이 없다 |
| perold | 25 | 한 반복 비교와 K 수열이 반복 0–39 전부 1e-9로 맞는다. step이 0.02–0.3으로 작은 느린 궤적이라 해법 반올림이 쌓여 갈라진다. 자유 열이 있어 매 반복 refinement를 한다 |
| pilot4 | 17 | 오차가 반복 0의 3e-16에서 꾸준히 커진다(반복 9에 1e-7, 반복 25에 3e-4). 자유 열과 느린 궤적은 perold와 같은 종류다. 한 반복 오라클 비교는 하지 않았다 |
| fffff800 | 36 | 오차가 반복 0의 2e-13에서 꾸준히 커진다(반복 14에 4e-8, 반복 28에 1e-6). 규칙이 갈리는 반복은 보이지 않는다 |

8개 모두 오차가 1e-13 근처에서 시작해 반복마다 커지는 모양이다. 특정 반복에서 규칙이 달라지는 흔적(K 수열 차이, 한 반복 비교의 큰 오차)은 보이지 않았다. perold는 반복 0–39, degen2·25fv47은 전부 확인했다. 그래서 나머지 차이는 수치 분해(supernodal LDLᵀ, 합 순서) 비트 일치가 있어야 줄어든다고 본다.

확인 못 한 규칙:
- 정체·재시작(B+0x30 재설정, `2d0663` 재호출), homogeneous 전환(`2d35d7`, `2f8f0e`), 상태 13 규칙(b2aecf L180–285: iteration > max(f8, best) + 200, 30회 뒤 최선점 정체 등). 64개에서 발동한 흔적은 없지만 오라클로 확인하지 않았고, 프로토타입에도 없다.
- `2c4689`의 최선 해 선택(최종 해가 마지막 반복이 아닐 수 있음). 프로토타입은 마지막 반복의 pobj로 비교한다.
- `07d3ce`의 상한만 있는 열 뒤집기(64개에 해당 열이 없음).
- 반올림 순서(SIMD 8레인 합, long double 합)는 옮기지 않았다. 반복 0도 해법 때문에 비트 일치가 아니다.

## 9. Rust 포팅에 필요한 수치 분해 인터페이스

- **행렬**: 반복마다 M = A Θ Aᵀ + diag(θ_slack)(m×m, 등식 행 slack은 0)를 **한 번** 분해한다. 초기점에서 한 번 더(θ⁰) 분해한다. 패턴은 ordering 결과(F, supernode)와 같고 값만 바뀐다.
- **분해 출력**: L, D(작은 피벗 대체 규칙과 대체 횟수), F+0x100 = min|d/원 대각|(refinement 모드 판정)이 필요하다.
- **해법 횟수**(같은 인수로):
  - 초기점: 2번. rhs_y = b와 rhs_x = −u, 그리고 rhs_y = 0과 rhs_x = [c; 0].
  - 반복마다: predictor 1번(rhs_y = r_p, rhs_x = r_d − rxz/x + (rws − r_u s)/w), corrector k마다 1번(rhs_y = 0, rhs_x = −rxz/x + rws/w).
  - mode 1이면 해법마다 refinement 보정 해법을 최대 5번 더 한다.
- **인터페이스 제안**: `factor(theta) -> (minratio, nsmall)`, `solve(rhs_x, rhs_y) -> (dx, dy)`(2ee6c5 형태, 내부에서 A·v와 Aᵀy를 한다). Python 프로토타입의 `Normal` 클래스가 같은 모양이다.
- **work 계산**에 F+0x160(= 2·nnz(L))과 F+0x168(`2e7f38` 공식, symbolic 통계 F+0x48/0x50/0x60/0x68와 CPU 상수)이 필요하다. 지금 체크 도구는 이 둘을 반복 0 오라클 F에서 읽는다. ordering 쪽 `barrier_symbolic()`이 F+0x48/0x50/0x68도 내 주면 오라클 없이 돈다.
