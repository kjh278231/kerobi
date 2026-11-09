# M2 라운드 끝 B: `423e35`, `5a1293`, `5aaaf7`, `838543`, `579e67` (+ 남은 것)

`3cd406` 라운드 끝 뒷부분(m2_spine_c.md §2, 카운트 9–21) 해독·포팅 노트. 코드는 `crates/mip/src/`의 `modk.rs`, `probe0.rs`, `roundend_b.rs`, `dualred.rs`, `disagg.rs`. 확인은 `tools/mip_dump_check.py <fn>`(38개 모델 모든 호출)과 `tools/sweep_check.py`.

| 함수 | 파일 | 덤프 비교(38개 모델) | 무엇을 바꾸나 |
|---|---|---|---|
| `838543` mod-k (k=2,3,5) | modk.rs | 126/126 | enlight_hard만(GF(2) 소거로 x 100개 고정, 다음 mod 3) |
| `423e35` PREPROBE0 | probe0.rs | 42/42 | bound·계수·고정(tr12-30, mas74 등) |
| `5aaaf7` | roundend_b.rs | 42/42 | 쌍대 고정 |
| `579e67` | roundend_b.rs | 42/42 | type 9 대입(neos-911970, markshare 등) |
| `5a1293` PREDUALREDUNDANT | dualred.rs | 42/42 | 38개에서는 작업량만(`39fb6d` 0회) |
| `5a04fc` PRESMARTDISAGG | disagg.rs | 42/42 | 38개에서는 작업량만(분해 0회) |
| `419ffc`, `5a4d8b`, `5a5eff` | stubs.rs | 안 함 | 38개에서는 작업량만(§7) |

레코드: `tools/mip_rec_check.py` 38/38(gmu 둘은 PRELIMIT=27, ran14x18은 71). clippy 0, cargo test·pytest 통과.

sweep: 35/38(시작 33/38). 남은 첫 불일치는 knapsack 2(gmu, `6b5e11`)와 ss2mip 1(ran14x18, `594c0c`)로 이 묶음 밖이다.

## 1. `838543(env, st, flag, k, W)` mod-k 소거

flag = mipctx 없음 → 1. W는 μ 없이 직접 더한다. 안에서 로컬 timer(`ade736`: 1 − W₀, 한도 `adf1a6(100·m + 1e7)`, `adf2be`)로 소거를 멈춘다.

1. `83c8d1(env, k, m, n, lb, ub, 0,0,0, &sys)`: cut 모듈의 희소 mod-k 행렬. 838543은 행 구성 추적 배열(+0xe0/+0xe8)을 곧바로 해제한다(그래서 `83e123`·`83e5f0`의 추적 분기는 안 돈다). 노드 0x20 B: 계수(double, 정수값) +0, 행 +8, 열 +0xc(−1 삭제), 행 다음 +0x10, 열 다음 +0x18. 둘 다 머리에 넣는다.
2. 행 i(`row_cnt > 0`, `=`): 고정 열(ub − lb < 1e-13)은 rhs −= a·lb. 나머지는 연속이거나 |a| > 2e9이거나 a가 정수가 아니면(1e-10) 행 포기. 계수 `floor(a+1e-10) mod k`(C 나머지 후 `(v>>31 & k)+v`), 0이 아니면 넣는다. W += 지나간 노드·22(포기면 그 노드 전까지). |res| ≤ 2e9이고 항목이 있으면 rhs = `floor(res+0.5) mod k`로 `83e123`, W += 항목·4.
3. `83e123`: 행 추가, W += 12·항목. 바로 앞 행과 개수가 같으면 표시 배열(+0x48)로 계수를 비교해(rhs는 비교 안 함) 같으면 새 행의 노드를 죽이고 count = −1. W: 23·len(새) + 23·len(앞) + 24·(불일치 위치 또는 len) [+ 23·len].
4. 힙(`a786fc`…, int 키 max-heap = lp barrier/ordering.rs Heap 복사): 행 r을 키 −cnt로 넣는다. 반복: timer 초과·힙 빔·cnt ≥ 51(flag=1)이면 끝. 행을 꺼내 열 count(sys)가 가장 작은 live 노드를 피벗(동률 앞), W += 행 노드·23. 피벗 열 리스트의 다른 행 q마다 mult = q.coef·(−p.coef⁻¹) mod k(표 `mod3/mod5.6415.0.27` = 음의 역원), W += (cnt_q + cnt_p)·10, `83e5f0(q += mult·p)`. 0이 된 행의 rhs ≠ 0이면 3(불가능). q가 힙에 있으면 cnt < 1이면 삭제, 아니면 키 갱신(`a790d9` = contains, 없으면 넣지 않는다). W += 열 노드·29.
5. `83e5f0`: scratch[col] = coef·mult(src), dst 행을 돌며 (scratch + coef) mod k, 0이면 노드를 죽이고 행 리스트에서 뺀다(열 리스트에는 남음). 남은 scratch는 새 노드로 dst에 넣는다. rhs_dst = (rhs_src·mult + rhs_dst) mod k. W: 23·len(src) + 23·len(dst) + 26·len(src).
6. 후처리(행 순서): 항목 1개 행 → 열이 live, 플래그 0, lb > −1e6, ub < 1e6, 아직 안 했으면 `574764(mod_subst, a, k, rhs)`. 항목 2개 행 → ub − lb == 1인 두 열의 네 꼭짓점 중 rhs를 맞추는 수: 0이면 3, 1이면 두 열을 `3de18e`로 고정, 2이면 `3baaef`로 묶기.

shortcut: 2개 꼭짓점 묶기(`3baaef`, 38개에서 0회)와 mod 7/11/13 표.

## 2. `423e35(st, env, T10, probe)` PREPROBE0

T10은 시작 전 복사본을 넘겨 안에서 `start`(다른 timer 포트와 같은 방식).

1. 두 bound 복사본(`424c23`, 각 W += 3n·μ): lb/ub, 바뀐 열 목록·위치(lb 쪽, ub 쪽), 원본 포인터(복원용).
2. 행 예산: `row_cnt > 0` 행의 키 = (이진 열 수)·row_cnt, W += 3μ·row_cnt. W += 4m·μ. 오름차순(`aaedd9` = sort_asc) 누적이 3·nnz + 1000(probe ≥ 2면 30·nnz + 10000)을 넘는 지점부터 행 skip. W += 2·목록·μ.
3. 후보: col_cnt > 0, 비연속, [0,1]. 키 = −Σ(skip 아닌 행의 row_cnt) · (1 + 1e-4·rand), rand = `a7b54f`(seed env+0x2cd0 = 0, probe_engine Rng). W += 4μ·col_cnt, 끝에 5n·μ. 내림차순(`aabbe3` = sort_desc).
4. 후보마다: 이미 고정(lb > 0.5 또는 ub < 0.5)이면 `3de18e`. 아니면 `426093(x=0)`·`426093(x=1)`:
   - `426093`: 복사본에 x 설정(불가능이면 바로 true), 열의 행(skip 제외)을 두 번 돈다. inf_min == 0이면 `427846(+1)`, `=`이고 inf_max == 0이면 `427846(−1)`. W += 5μ·노드.
   - `427846`: 복사본 bound로 행 L, U(±1e100 표시). slack = rhs − L(+1) 또는 U − rhs(−1), < −1e-4면 불가능. range6 = max((U−rhs)−(L−rhs), 100)·1e-6, tol = 1e-10·range6/1e-6. 다른 열마다 slack < (u − range6 − l)·|a'|이면 새 ub/lb(정수는 floor(+1e-5)/ceil(−1e-5)), 범위를 벗어나면 불가능. W: 5μ·노드, 4μ·지나간 노드.
   - 둘 다 가능: (a) 두 쪽 모두 바뀐 bound는 min/max로 조임(`3dd6f1`, +0x3f8, 추적). (b) x의 행마다(skip 제외) 두 복사본에서 U − rhs < −1e-4(한쪽에서 여유) 또는 `=`이고 L − rhs > 1e-4를 찾으면: `=` 행은 x를 다른 값으로 고정하고 (c)를 건너뛴다. `<` 행은 계수 강화(x=0 쪽: a −= d, rhs −= d; x=1 쪽: a += d), activity 갱신, |a| < 1e-9면 노드 죽임, +0x3fc. W: 5μ·행 노드, 끝에 4μ·col_cnt(바뀐 뒤). (c) 두 쪽 모두 bound에 붙는 열 중 lb == ub인 것은 고정, 아니면 `3baaef` 묶기.
   - 한쪽만 불가능: 반대 값으로 고정.
   - 복사본 복원(현재 st bound로), W += 4μ·목록.
5. 후보 끝마다 T10 `adf2be`면 멈춤. W − 마지막 > 1e7이면 `3ba9de`(예산). 끝에 W += 2·(본 후보 수)·μ.

shortcut: `426093`의 목적값 검사(`4f2c32` 플래그, cutoff 1e100이라 참이 될 수 없음), (c)의 `3baaef` 묶기(38개에서 0회). 위치 배열(local_40)은 후보 사이에 남는 표시까지 원본대로 둔다.

## 3. `5aaaf7(st, env)` (DualReductions·PREDUALREDUNDANT)

작업량 없음. price[m] = 1e100(state+0x398). 열 j(col_cnt > 0, |c| ≥ 1e-6, 플래그 0)의 live 항목이 `=` 행에 없고 부호가 모두 같고 c·a < 0이면: 행마다(inf_max == 0, act_max ≥ FeasTol) t = act_max/|a|(정수는 ceil(t − 1e-10)), t ≤ ub + tol − lb면 price = min(price, |c|·t). 그다음 정수 열(col_cnt > 1)마다 `=` 행이 없고 c·a > 0인 행의 price 합 ≤ |c| + 1e-10이면 싼 bound로 `3de18e`.

## 4. `579e67(st, env)`

`=` 행(row_cnt > 2), 모두 B/I, |a|가 모두 같지는 않음: g = `a7bea8`(iint.rs gcd_f64), 1e-3 ≤ g ≤ 1e4, |rhs/g| ≤ 1e6. 계수를 floor(a/g + 0.5)로 바꾸고 `a7c752`(knapsack.rs gcd_excl, W 포함)로 다른 계수들의 gcd d를 구한다. d ∉ {0, 1}, d < 1e8, 플래그 0인 열은 a·x ≡ floor(rhs/g + 0.5) (mod d) — 안의 대입 블록은 `574764`와 줄 단위로 같아서 `mod_subst`를 부른다. W += 8·항목(μ 없음).

## 5. `5a1293(st, env)` PREDUALREDUNDANT (timer 4)

예산 W₀ + 1000·nnz + 1e6(넘으면 W += 3μ·j 하고 끝). 열 j: col_cnt 2..1000, 플래그(0x40 제외) 0. 1차: `=` 행이 있거나 행 길이 합 > 1000이면 탈락, W += 3μ·지나간 노드. 2차(노드 e, c·a > 0, |a| ≥ 1e-4): x를 싼 bound에 두면 행 e가 남는지(inf_max 0/1 경우). 같은 열의 반대 부호 행 f마다: |b| < 1e-4면 탈락; 정수 열이면 행 f를 1/|b|로 나눈 rhs·계수가 정수(1e-10)이고 모두 정수 열이어야(W += 2μ·지나간 노드); `a7cec0`(knapsack.rs act_sum)으로 (e/|a| + f/|b|)의 최대 activity ≤ 두 rhs 합 + 1e-6. W: 2μ·row_cnt(f), 6μ·row_cnt(e), 6μ·row_cnt(f), 2μ·항목, 3μ·지나간 f, 8μ·e 노드, 끝에 3μ·n.

shortcut: 통과한 행의 `39fb6d`(레코드)·`415f4f`(행 삭제)는 38개에서 0회라 포팅 안 함.

## 6. `5a04fc(st, env)` PRESMARTDISAGG ≥ 2

1. 리터럴 행: `<`, row_cnt > 1, inf 0, act_max + act_min ≤ 1e-6. [0,1] 이진 열 중 |a| + act_min ≥ −1e-6인 것이 하나면 리터럴(a ≤ 0이면 j + n), 둘 이상이면 −2. W += 6μ·row_cnt, 끝에 5μ·m. 없으면 끝.
2. W += 2n·μ. 리터럴 행의 다른 리터럴(a ≥ 0이면 j) 사용 수. W += 5μ·row_cnt.
3. 리터럴 행에 ub − lb > 1e-10이고 사용 수 ≥ PRESMARTDISAGG(env+0x25b4, 기본 5)인 항목이 없으면 버린다. W += 5μ·row_cnt, 끝에 4μ·m. 남은 게 없으면 끝.
4. 리터럴별 행 CSR(행 오름차순). W += (5m + 6n)·μ.
5. 예산 W₀ + 1000·nnz + 1e6(넘으면 W += 14μ·i, 끝). 리터럴 행이 아니고 row_cnt > 1, inf 0인 행: 쪽 s(`=`면 −1 그다음 +1, `<`면 +1), d = act_max(s=−1) 또는 −act_min. |a·s| > 0.51·d + 1e-5인 [0,1] 이진 항목이 2개 이상이면(W += 6μ·row_cnt) 그 리터럴의 리터럴 행들을 돌며(W += 5μ·row_cnt, 리터럴마다 3μ·행 수) 다른 자유 리터럴의 도달 수를 센다. 도달 수 ≥ PRESMARTDISAGG면 분해(`3bde19` 새 행, `3bea24` 새 열, `370047`). W += 3μ·리터럴 수 + 3μ·본 행 수. 끝에 W += 14μ·m.

shortcut: 분해 자체(38개에서 0회).

## 7. 남은 것 (stub) → 포팅 완료(§7.1–7.3)

38개 모델에서 덤프 진입·복귀 상태를 비교하면 셋 다 작업량 말고는 바꾸는 것이 없다(덤프의 recs 차이는 레코드 패딩 바이트). stub은 작업량을 더하지 않으므로 덤프 비교는 W만 틀린다. sweep에는 드러나지 않는다(35/38).

| 함수 | 크기 | 하는 일 | 작업량 예(dW) |
|---|---|---|---|
| `419ffc` PRECOVERS(timer 23) | 7,235 B | 계수 ±1 이진 `<` 행(길이 2..499, rhs + 1 = 양수 항 수)을 리터럴 집합으로 보고, 64비트 서명 두 개와 정렬 병합으로 포함 관계를 찾아 행을 지우거나(`537da8`, row_cnt −2) 항목을 뺀다(rhs −1, +0x3fc, `370047`). 의사코드가 심하게 꼬여 asm으로 다시 읽어야 한다 | tr12-30 1,932, gmu-35-50 12,705 |
| `5a4d8b`/`5a5eff` PREDERIVED | 4,096 / 약 4 KB | LP `5b488d`의 MIP 판: col_cnt 2..19, 행 길이 ≤ 9, 이웃 ≤ 9, 정수 범위 곱 ≤ 1024. `5b2ea6`(행 식, 3,680회/13개 모델)과 `5b208f`(정수 꼭짓점 열거, 679줄, 1,186회/3개 모델)가 돈다. 성공(`3a008f` type 24, 0회)이면 col_cnt = −3, 행은 `537da8`+`415f4f` | tr12-30 81,068 / 20,884 |

### 7.1 `5a4d8b`(`derived.rs`, 덤프 42/42)

- 후보 열: col_cnt 2..19, col_flag 0. 열 리스트를 걸으며 살아 있는 노드의 행이 row_cnt > 9이면 W += 2·(지난 노드 수) 후 탈락. `=` 행 중 |a| 최대(> 0.001)가 피벗. 끝까지 가면 W += 2·노드 수. |c_j| > 1e-10인데 피벗이 없으면 탈락.
- 이웃: slot[j] = n으로 j를 막고 j의 행마다 행 리스트를 걸어 처음 보는 열을 목록에 넣는다(9개째에서 내부 break). 부호 수 neg(a < 0 또는 `=`)/pos(a > 0 또는 `=`). 행마다 W += 4·걸은 노드 + 3. slot 복구 W += 2·목록 길이. 목록 ≥ 9면 탈락.
- 열거 크기: prod = col_cnt[j]에서 시작, 양부호 이웃마다 ±(1e6, 1e30) 범위 bound면 1e100, 이웃 'C'면 ×2(j가 'C'가 아니면 1e100), j가 'C'면 ×2, 아니면 ×(ub + 1 − lb). 1024 초과에서 break. W += 5·목록 길이(break와 무관). 1024 초과면 탈락.
- 양부호 이웃을 앞으로(swap), W += 2·목록 길이. 앞 nk개로 `5b2ea6`.
- `5b2ea6(st, j, nk, list, slot, neg, pos, seg, ent_r, ent_c, v, sense, W)`: slot 지정 W += 2nk. 열 j의 살아 있는 노드(열 리스트 순)마다 1/a, 행 방향(`<`이고 1/a < 0이면 `>`), v = rhs/a에서 목록 밖 이웃을 최악 bound로 뺀다(neg/pos가 NULL이면 무시). 행마다 W += 6·걸은 노드 + 6. 누적합 W += nk, 항목 채우기(c = −a_q/a_j) 행마다 W += 8·걸은 노드 + 3, 되돌림 W += nk, slot 해제 W += 2nk.
- 행별 범위 검사(LP와 같다, 이웃마다 W += 7·항목 + 2). 실패면 탈락. 통과면 W += 5·nr.
- `5b208f(itol = env+0x2468, st, j, nk, list, &out, masks, seg, ent_r, ent_c, v, sense, W)`: x = 이웃 lb(W += 3nk), val 복사(W += 3nr), 무한 v 처리 후 W += 5nr, 초기 x 반영 후 W += 5·항목 수. 점마다 모든 행으로 lo/hi(조기 종료 없음), j가 정수면 lo = ceil(lo − itol), hi = floor(hi + itol). W += 2nr + 5. 나쁜 점(lo − hi > 1e-6, hi − lb_j < −1e-6, lo − ub_j > 1e-6)이면 masks가 NULL(`5a4d8b`)이면 out = 1로 끝, 아니면 비트(이웃 t는 비트 nk−1−t, x > 0.5)를 기록하고 W += 1.5nk. 다음 점: 마지막 이웃부터 ub − 1e-6 ≤ x면 lb로 되돌리고(자리올림), 이웃과 j가 모두 정수면 x + 1, 아니면 ub. 빼기·더하기 두 번 각 W += 3·항목 + 6(고정 이웃도).
- 성공(out = 0): `3a008f` type 24(아래), 비용 이동(피벗 행에서 j까지 W += 2·노드, 행 전체 W += 2·노드, objcon), j의 행마다 `537da8` + `415f4f`, W += 열 노드 수, col_cnt = −3, +0x408 + 1. 함수 끝에 W += 5n.
- type 24 데이터: +0 j, +8 lb, +0x10 ub, +0x18 vtype(u8), +0x1c −1, +0x20 0.0, +0x28 행 수, +0x30 →beg, +0x38 안 씀, +0x40 →sense, +0x48 →rhs, +0x50 →ind, +0x58 →val, +0x60 beg[행 + 1], ind는 +0x68 + Pb, val은 +0x70 + Pb + ((4·nz − 1) & !7), sense는 val 바로 뒤, rhs는 sense + 8 + ((행 − 1) & !7). Pb = (4·행 + 3) & !7. 38개에서는 성공이 없어 바이트 확인은 못 했다.

### 7.2 `5a5eff`(`derived.rs`, 덤프 42/42)

- 후보·이웃 수집은 `5a4d8b`와 같다(부호 대신 row_cnt > 1인 j의 행 수를 센다). 이웃이 모두 'B'여야 한다(W += 2·처음 'B'가 아닌 위치까지). 범위 곱·행별 검사 없이 nk = 이웃 전부, neg/pos = NULL로 `5b2ea6`, masks를 주고 `5b208f`.
- 나쁜 점이 둘 이상이면 큐브 병합: care 마스크가 같고 xor의 popcount가 1이면 그 비트를 둘 다에서 지우고, 병합한 큐브를 살아 있는 범위 끝에 세운다(다음 패스는 그 큐브끼리만). 바깥 단계마다 W += 4·(b − a) + nk.
- 큐브 수 ≤ (row_cnt > 1인 j의 행 수)면 성공: type 24, 비용 이동, 행 삭제(위와 같다), 그 뒤 열 리스트를 다시 걸으며 row_cnt < 0인 행을 앞에서부터 큐브마다 `<` 행(rhs −1)으로 되살린다: care 비트 중 1이면 +1(rhs + 1), 0이면 −1(`3bea24`), W += 2nk, +0x404 − 1, `370047`(그 행), `53763f`. 끝에 W += 4·걸은 노드, col_cnt = −3, +0x408 + 1. 38개에서 성공은 0회.

### 7.3 `419ffc`(`covers.rs`, 덤프 42/42)

- 2n + 1 > 2e9면 0. 후보 행: row_cnt 2..499, `<`. 행을 걸으며 neg(a ≤ 0)/pos를 세다가 'B'가 아니거나 ||a| − 1| > 1e-10이면 탈락. W += 3·(neg + pos)(탈락 지점까지). |rhs + 1 − pos| ≤ 1e-10이고 neg + pos ≥ 2면 후보. 끝에 W += m. 후보 < 2면 0.
- 한도 = min(+0x1f0 / 5, nnz·1000 + 1e6). 리터럴(a ≥ 0: j + n, a < 0: j), W += m + 4·nnz + 3k. `77aab`로 두 번 전치(행 안 리터럴 정렬). `77aab`의 작업량은 `7722f`와 다르다: 2·len, 3·nmaj, 2·nmin, 3·len(값 없음), 3·nmaj. 서명 (l & 63), (l mod 63), W += len. 키 = (행 리터럴의 최소 열 길이, 시작 2n + 1)·len, `aaedd9` 정렬, W += 7k.
- 행 r마다(len > 0): W − W0 > 한도, 또는 아직 성공이 없고 10(W − W0) > 한도면 끝. 최소·둘째 최소 열 길이 리터럴을 고르고, t = −1(그대로)과 t = 0..len−1(리터럴 t를 뒤집음, 서명 다시 W += len)마다 훑을 리터럴(뒤집은 리터럴이 최소면 둘째, 뒤집은 리터럴의 길이가 더 짧으면 그것)의 행 q(길이 < 5000일 때만)를 본다. q ≠ r, len_q ≥ len_r, 서명 포함이면 정렬 병합: 뒤집은 리터럴은 건너뛰고 나머지가 모두 q에 있어야 하며, 뒤집은 리터럴은 q 어디서든 보이면 된다. W += 2·끝 위치(성공·실패 모두). 성공: 뒤집지 않았으면 len_q = 0, 뒤집었으면 q에서 그 리터럴을 빼고(W += 3·새 길이, 서명 갱신), 비면 불능(+0x470 열, +0x478 q의 행, 반환 3). 리터럴의 행을 다 보면 W += 10·열 길이.
- 적용: len 0인 후보 행은 `537da8` + `415f4f` 꼴(row_cnt −2, +0x404 + 1), 길이가 준 행은 빠진 항목을 지우고(`5379f1`, a > 0이면 rhs − 1, 노드 값 0), +0x3fc + 1, `53763f`, `370047`. 38개에서는 성공이 없다(작업량만 맞춘다).
