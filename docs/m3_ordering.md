# M3 barrier ordering·symbolic 해독 기록

범위: Method=2의 구조 분석 단계(`2bf1ea` 94–175줄). AAᵀ 패턴 `303e2a` → ordering 드라이버 `300f28` → 커널 `309dd0`(이분할 `30b0f2`/`30d189`, 최소 차수 `3047bb`, 개수 `302921`) → symbolic 분해 `2e4f14`. 방식은 [10 §9](10_porting.md): 의사코드·objdump로 규칙을 읽고 Python 프로토타입을 오라클과 맞춘 뒤 Rust로 옮긴다.

| 파일 | 내용 |
|---|---|
| `tools/proto/bar_order_oracle.py` | 오라클. "Ordering time" MESSAGE 콜백에서 AAᵀ 그래프·순열·nnz(L)·ops를 뜨고 `309dd0`을 ctypes로 직접 부른다(mode 3, mode 1 seed 11–15). 첫 BARRIER 콜백에서 symbolic 뒤 F를 뜬다 |
| `bench/oracle/bar_order/*.json.gz` | 64개 덤프. `expected.tsv`는 Rust 테스트용 요약(Gurobi 값). `bisect/`는 `--bisect`로 `30b0f2`를 직접 부른 결과 |
| `tools/proto/bar_order.py` | 프로토타입. `python3 -I tools/proto/bar_order.py bench/oracle/bar_order` |
| `crates/lp/src/barrier/ordering.rs` | Rust 포트 `analyze(&Model) -> Symbolic` |

## 1. 일치율 (Netlib 64, Presolve=0, Method=2, Threads=1)

| 대상 | Gurobi 위치 | 프로토타입 | Rust |
|---|---|---|---|
| AAᵀ 패턴(AA' NZ = nnz/2) | B+0x100/0x110/0x118 | 64/64 | 64/64 |
| 최소 차수 순열(ordering 직후) | F+0x30 | 64/64 | (중간값, 비공개) |
| `302921` nnz(L)·ops | F+0x38/0x60(ordering 직후) | 64/64 | 옮기지 않음 |
| 최종 순열(postorder 뒤) | F+0x30(symbolic 뒤) | 64/64 | 64/64 |
| supernode·블록·열 길이·행 번호 | F+0x78/0xa8/0xc8 | 64/64 | 64/64 |
| Factor NZ / Factor Ops | F+0x38/0x60(symbolic 뒤) | 64/64 | 64/64 |
| F+0x48(Σ A 열 길이²) | F+0x48 | 64/64 | 64/64 |

단계별 기록: AMD만 53/64(`dec8e5d`) → symbolic 53/64(`66054a0`) → 이분할 64/64(`915ad7f`). 이분할을 타는 11개는 25fv47, bnl1, degen3, ganges, gfrd-pnc, maros, modszk1, perold, scfxm2, scfxm3, sctap2(AAᵀ에 행 600개 넘는 연결 성분).

Rust 검증: `cargo test -p rsolver-lp ordering`(`bench_lps_match_gurobi`가 64개 MPS를 읽어 `expected.tsv`의 Gurobi 값·체크섬과 비교).

## 2. 확정 규칙

### 2.1 AAᵀ 패턴 (`303e2a`)
- 모든 열(자유 변수 포함)의 행 쌍으로 만든 대각 없는 대칭 패턴, 각 열 안은 행 번호 오름차순. 64/64. 자유 변수 4개 모델도 같다.
- 로그 "AA' NZ"는 B+0x100/2.

### 2.2 ordering 드라이버 (`300f28`)
- 기준선은 항상 `309dd0(mode=3)`, RNG 상태는 model+0xb0(처음 0).
- ND 후보(mode 1, seed = Seed + i + 라운드·스레드 + 11)는 `BarOrder≠0`이고 `threads·(기준선 work)/ops < 0.1`일 때만 돈다. work는 시간이 아니라 결정적 계수다(`3047bb`의 5.25·elen, 3.75·len, 15·|Lme| 등, `30b0f2`의 (3·deg+2)·0.75). 64개 중 이 비가 0.1보다 작은 것은 israel(0.0028)·seba(0.0007)·fit1p(0)뿐이고, 셋 다 이분할이 없어 ND = 기준선이다. 이분할하는 11개는 비가 0.50–10.2라 ND를 돌지 않는다. 그래서 64/64에서 최종 순열 = 기준선이다(오라클: 직접 부른 mode 1이 더 작은 ops를 내는 25fv47 등에서도 채택되지 않음).
- 스레드 수 n<1000이면 1, Threads=1이면 F+0x18 = 1.

### 2.3 `309dd0` 라벨
- 밀집 임계 t = max(100, min((int)(50·nnz/n), 10·√n)). 차수 > t인 행은 -2(2단계), 나머지 -1, 밀집 열(n>m 부분)은 -3.
- mode≥1: 라벨 -1 노드의 연결 성분 중 크기 > max(600, n/100)인 것을 `30b0f2`로 이분할하고 구분자를 -2로 바꾼다. mode 1은 모든 성분이 작아질 때까지 반복, mode 2·3은 한 번만.

### 2.3a 이분할 `30b0f2` → `30d189`
- RNG `a7b4d6`: state = state·0x5851f42d4c957f2d + 0x14057b7ef767814f(64비트), 상위 32비트를 MT식으로 tempering해 31비트. `a7b54f` = 그 값·4.656612873077393e-10. 기준선의 state는 model+0xb0(처음 0)이고 성분을 차례로 이어 쓴다.
- 성분 추출: 라벨 -1 노드의 DFS 성분, 성분 안 노드는 번호 순서로 다시 매기고 이웃은 인접 목록 순서. 크기 > max(600, n/100)인 성분마다 한 번.
- coarsening(최대 19단계, 노드 > 99이고 줄어드는 동안): 노드를 차수 오름차순(안정)으로 놓고 같은 차수 구간마다 끝에서부터 Fisher–Yates(r = rand % 남은 길이). 그 순서로 아직 짝 없는 노드를, 짝 없는 이웃 중 차수가 가장 작은 첫 이웃과 묶는다(round = 3이면 첫 짝 없는 이웃; 기준선 round = -1). coarse 노드 번호는 방문 순서, 무게는 합, 인접은 만나는 순서(자기 자신 제외).
- 가장 거친 그래프는 전부 구분자(2)에서 시작. ratio = `a7b54f`·5 + 5. 각 단계에서 `30d189`로 정제하고 한 단계 아래로 투영한다.
- `30d189`(2면 FM, METIS `FM_2WayNodeRefine2Sided`와 같은 골격): 큐 k(이진 max-heap, 정수 키, 같은 키는 움직이지 않음)에 구분자 노드를 키 vwgt − edeg[1−k]로 넣는다. 패스 시작마다 구분자 노드를 번호 순서로 모아 끝에서부터 섞고(r = rand % (k+1), k = 0까지) 그 순서로 두 큐에 넣는다. 이동 방향: q0가 비면 1, q1이 비었거나 p1 > ratio·p0이면 0, p0 > ratio·p1이면 1, 아니면 두 후보의 이동 뒤 비용 |S|/(|P0|·|P1|)을 비교(같으면 1). 이동 수 − 최선 위치 > min(500, 5·초기 구분자 수)이면 멈춘다. 패스 끝에 최선 위치까지 되돌리고, 개선이 없거나 10패스면 끝.
- **Gurobi 특이점**: v를 옮길 때 구분자 이웃 u의 반대 큐 키 갱신 조건이 asm(0x3281eb)에서 `contains(q[other], v)`다. v는 방금 지웠으므로 갱신이 일어나지 않는다. 의사코드는 인자를 잃어 이 점이 보이지 않았고, 이것을 고치자 11/11이 맞았다(`30d189`를 ctypes로 직접 불러 같은 입력에서 RNG 소비 수가 다른 것을 보고 찾음. 재생 스크립트는 남기지 않았다).

### 2.4 최소 차수 `3047bb` (Seed=0)
AMD 계열 quotient graph. 목록 = 원소(elen개) + 변수.
- 초기 차수 = 원래 길이(밀집 행을 자르기 **전**). 길이 > t인 노드는 라벨 n+단계로 미뤄 단계 끝에 번호를 준다. 1단계 고립 노드는 맨 앞에 번호.
- 단계 = 남은 노드 라벨의 최소 단계. 단계마다 노드를 **n−1부터 0까지** 차수 버킷 머리에 넣는다(같은 차수면 작은 번호가 먼저 나온다). Seed≠0이면 이 순서를 섞는다(`a7b4d6`, 미구현).
- 피벗 = 가장 작은 비지 않은 버킷의 머리. Lme = 흡수 원소들의 변수 + 피벗의 변수(label<0, nv>0).
- w(e) = nv(e) − Σ nv(i∈Lme∩Le), 0이면 원소 흡수.
- 차수: d = min(degme + d_old − nv_i, Σ_e w(e) + Σ 변수 nv), 질량 소거 뒤 d = min(degme′ − nv_i + d, n−1). dsum + nv_i > 5(n−pos)이면 밀집으로 미룸.
- 질량 소거: 목록 길이가 1(새 원소뿐)인 같은 단계 변수.
- 해시 = label·n + Σ 남은 원소 번호 + Σ 남은 변수 번호(double). Lme를 해시로 `ab9789` 정렬(n<100: 간격 n/2, 이후 /2.2, 2→1인 shell sort)하고, 같은 해시 구간에서 라벨·len·elen이 같고 목록이 **순서까지** 같으면 앞의 것에 합친다.
- 마지막에 흡수된 변수는 대표 변수 앞 자리를 번호 순서로 받고 대표는 끝자리.

### 2.5 개수 `302921`
열 개수 c_j(대각 제외)로 nnz(L) = Σ(c_j+1), ops = Σ(c_j+1)². 로그 값이 아니다(로그는 §2.6).

### 2.6 symbolic `2e4f14`
- 열 구조를 A에서 직접 행 병합으로 만든다(결과는 보통 symbolic 구조와 같음).
- 최대 supernode: len[c+k] = len[c]−k이고 struct[c][k] = c+k인 동안 늘린다.
- 완화 병합(supernode 번호 순서): 자식 s(열 c, 행 r, b=r−c)를 부모 p(행 r_p)에 붙이면 늘어나는 Σk²가 [S(r_p+c)−S(r_p)] − [S(r)−S(b)] < (c<192 ? 3b : 0.1b)·b + 1000이면 합친다(S(k)=Σ_{i≤k} i²). r_p += c.
- 형제 단일 열 병합: 행 > 24인 단일 열 자식 뒤의 단일 열 형제가 대각 아래 구조가 같으면 합친다(24열까지).
- postorder: 뿌리를 자식 목록 순서로 스택에 넣고 pop하며 자식을 넣은 방문 순서를 뒤집는다. supernode 안 열 순서는 열 목록 순서(병합한 자식 열이 앞).
- supernode 행 수 = 열 수 + len(가장 큰 옛 번호 열) − 1. Factor NZ = Σ 사다리꼴, Factor Ops = Σ_열 (열 길이)².
- F+0x78: Threads=1이면 블록 상한 2e9(ops < 2·F+0x48이면 24), 블록 첫 열 길이 < 1920이면 192, 아니면 240으로 자른다(fit1p 627 = 192·3+51).
- F+0xc8: supernode마다 열과 아래 행을 새 번호 오름차순.

## 3. 오라클 메모
- "Ordering time" 메시지는 `300f28`이 `304736`(AAᵀ 해제) 전에 찍는다. OutputFlag=1, LogToConsole=0이면 MESSAGE 콜백이 온다. B·F는 스택에서 `(B+0x5c8, B+0x5b8, F)` 세 단어를 찾는다.
- `309dd0` 시그니처: (xmm0 ops 한도, model, B, F, mode, seed*, perm*, &nnzL, &ops, work*, thread, round, timer, aux*). 이분할 `30b0f2(env, n, colptr long*, rowind int*, part*, round, seed*, work*)`, 정제 `30d189(xmm0 ratio, env, seed*, n, xadj long*, adjncy int*, vwgt int*, where*, work*)`. 둘 다 env(model+0x100)만 있으면 콜백 밖에서도 부를 수 있다.
- BARRIER 콜백 시점에는 D의 행이 최종 순열로 바뀌어 있다(`2e9086`).

## 4. 미해독 지점
- `shortcut:` ND 후보(mode 1: 성분이 모두 작아질 때까지 반복 이분할, round별 seed)와 채택 판단의 work 계산. 64개에서는 결과에 영향 없음(§2.2). work가 작은 큰 모델에서 필요해지면 `3047bb`·`302921`·`30b0f2`·`30d189`의 work 증분부터 옮긴다.
- `shortcut:` `ab9789`의 n ≥ 100 quicksort 단계. 64개에서 |Lme| ≥ 100인 정렬이 없다.
- `shortcut:` 밀집 열(`2fd14c`, m ≥ 1000)과 그 라벨 -3, F+0x138. 64개 모두 Dense cols 0.
- Seed≠0일 때 `3047bb`의 노드 순서 섞기, model+0xb0의 초기값(Seed=0이면 0).
- Threads>1일 때 블록 상한(`2e4f14` L2160-2240).
