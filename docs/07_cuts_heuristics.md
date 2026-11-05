# 07. 절단 평면과 원시 휴리스틱

포팅 원본: [port_map.tsv](port_map.tsv)의 `07_cuts_heur`(394개). 진입 함수는 루트 루프 `64da9f`, separator 디스패처 `7b77cd`, cut 선택 `6d8986`/`6de7ca`, pool 정리 `6e289b`, 노드 휴리스틱 `8ad3a3`, sub-MIP `8e3397`/`8e4682`/`8e7530`이다. 근거: 노트 16 전체. 읽기 보조 문헌: Marchand–Martin–Weismantel–Wolsey 2002(서베이), Gomory 1960 / Balas et al. 1996(GMI), Cornuéjols–Li–Vandenbussche 2003(k-cut), Letchford–Lodi 2002, Marchand–Wolsey 2001(c-MIR), Gu–Nemhauser–Savelsbergh 1998/1999(lifted cover / flow cover), Atamtürk–Nemhauser–Savelsbergh 2000(clique), Caprara–Fischetti 1996 / –Letchford 2000({0,½}, mod-k), Balas–Ceria–Cornuéjols 1993(L&P), Witzig–Berthold–Heinz 2017(dual proof), Wesselmann–Suhl 2012(cut 선택), Fischetti–Glover–Lodi 2005 / Bertacco et al. 2007 / Fischetti–Salvagnin 2009(feasibility pump), Danna–Rothberg–Le Pape 2005(RINS), Berthold 2014(fix-and-propagate, RENS), Rothberg 2007(polishing), Hendel 2022(ALNS).

## 1. 루트 cut 루프 (`mip::root`) (노트 16 §3.1)

```
min_passes = min_root_cut_passes(기본 -1 → 2) ; max_passes = CutPasses == -1 ? 100 : CutPasses
(Cuts == -1 && 특정 플래그면 0 ; 집합 분할 등급이면 Gomory/L&P/ZeroHalf auto 끔)
루트 LP 전 휴리스틱(8ac5fe, Heuristics > 0.025), NoRel(설정 시), partition(bit 0x10)
pass = 0
while pass < max_passes:
   termination::check ; cut_pool.commit_new()
   degen = (pass==0 || Gomory auto -3 || 직전 개선 ≤ 1e-10) ? DegenMoves(-1 → 3) : 0
   lp::resolve_root(degen)  → 버퍼의 해 후보 제출 ; pass += 1 ; LP != OPTIMAL → break
   첫 패스(Heuristics > 0.025) 또는 (incumbent 없음 && Heuristics > 0.1): heuristics::root_first_pass
   pass ∈ {1,3} && DualReductions && Symmetry 정보: 대칭 고정 루프 (M6)
   --- 진전 판정 ---
   impr = (z − z_prev)/(|z_prev|+1)
   if impr가 세 기준(2e-5·(|z|+1)·(c+0.1), gap·c', 0.05·이전 개선)을 모두 넘으면 stall = 0
   elif 분수 정수변수 수가 10% 이상 줄지 않았고 pass ≥ min_passes:
        stall += 1 ; if stall > restarts && (pass/10 + 1 + 10c)·stall ≥ (restarts==0 ? 5 : 10): break   // (QC 모델이고 OA 위반 > root_qc_cut_thr면 대신 OA 전용 모드)
   sb_max_cands < 0 && 일반 정수 있음: 루트 분기 반복(627f3e 루프, [06 §5])
   LP 해 반올림 시도 → heuristics::scheduler(root, pass) → 적응 계수 갱신
   --- 이전 패스 cut 재선택 ---
   repeat { resolve ; n = pool.select_into_lp() ; pool.rebuild_matrix() } until n == 0 || 개선 < 임계
   separate(root, pass)                                        // §2
   헬퍼 스레드 병합 ; buffer → pool → LP (n1, n2) ; pool.trim() ; propagate
   if n1 == 0 && n2 == 0 && pool 크기 안 늘음: break
   이번에 고정된 변수 > (ncols − …)/5 + 10: 재시작 요청
루프 뒤: root_probe → probing(Learned cut) ; relax_heur ; pool.node_limit = m/5 + pool.len() ; 초기 bound가 gap 절반 안이면 Cover/Implied/InfProof auto를 1로
```

## 2. Separator 디스패처 (`cuts::separate`) (노트 16 §3.2)

- 조기 종료: `pool.len() ≥ m·gF + 1000`(gF = cut_growth 기본 8, 재시작 전 4/500) 또는 `pool.nnz ≥ nnz(A)·nF + 20000`(nF 6, 재시작 전 3/5000) 또는 nnz(A) > 1.33e9.
- 파라미터 해석: 개별 `*Cuts` -1 → `min(Cuts, 2)`; Cuts도 -1 → 종류별 기본값(§6 표). **값 ≥2: 루트+노드, 1: 루트만, ≤0: 끔**. ZeroHalf/ModK auto이고 열 > 10·행이면 pass > 2부터 끔. DualImplied는 DualReductions 필요. SubMIP/RelSubMIP는 pass 2–99. 
- GomoryPasses: 노드면 0; auto(-1): pool이 전부 Gomory/L&P이고 pass ∈ {1,2}이고 1e-4 이상 개선이면 pass+1, 아니면 (열 ≤ 10·행 || 정수 ≤ 10·연속) ? 2 : 0, maxRank 2; -2 → 1/2; -3(Cuts=3) → 15/5; G>0 → G, maxRank = G≤5 ? G : 5+(G−5)/2.
- 적응형 끄기(auto인 separator만): `success = 성공 호출/호출`, `s = max(0.01, 1e6/(k²+1e6))`; success < 0.01이면 `work_t > 1e7 && work_t > 0.1·s·총work + 1e7`에 끔, 아니면 `> 1e9 && > 0.2·s·총work + 1e9`. MIR·FlowCover·StrongCG·Mixing은 MIR 통계 하나로, BQP·PSD도 함께.
- **호출 순서**(고정): 준비(conflict graph, VUB 지도, network 추출) → Cover → Implied → ProjImplied → DualImplied → Clique → GUBCover → FlowPath → Knapsack → aggregation(MIR/FlowCover/StrongCG/Mixing/Cover/GUB/Knapsack) → Network → ZeroHalf → ModK(3, 5; zero-half가 실패했을 때만) → [close cut 재시도] → Gomory → WideSplit → LiftProject → PairVar → GRBCUT1 → InfProof → RLT → RelaxLift → BQP/PSD → MIPSep → SubMIP → RelSubMIP → (pass==4 보조) → Symmetry → WeakSymm → OA(2차/일반) → NL envelope. 각 호출 앞뒤 work 타이머, 뒤에 termination::check.
- close cut(`close_cuts=1`, 루트): 분리점 `0.8·x_LP + 0.2·x_inc`에서 먼저, 안 나오면 x_LP.

## 3. 버퍼 → pool → LP (노트 16 §3.3)

```rust
pub struct Cut { idx: Vec<i32>, val: Vec<f64>, sense: Sense /* ≤ 또는 = */, rhs: f64, kind: CutKind, rank: u8, lp_row: Option<usize>, viol_count: u32, forced: bool, hash: u64 }
pub struct CutPool { cuts: Vec<Cut>, node_limit, nnz_limit, by_hash: HashSet<u64>, aux: Vec<CutRef> }
```
1. separator → `buffer.add(cut, kind, rank)` (해 후보·bound 변경도 같은 버퍼).
2. `pool.insert`: 해시 중복 검사 → (mipcoestrength) 계수 강화 → 지배/무의미 검사 → `≤` cut이고 nnz ≤ 100이고 `cut_lift`면 lifting, 아니면 `cut_sparsify`면 sparsify → bound 변경 적용.
3. `pool.select_into_lp(x*)`:
```
orthTol = cut_orth_tol (0 → 0.75) ; denseFac = cut_dense_fac (0 → 재시작 전 && Cuts<2 ? 20 : 1000)
maxNnz = denseFac·nnz(A)/(m+1) + 1000 ; maxCuts = 0.05·(m + LP cut 수) + 1000 ; needViol = 루트 ? 0 : 2
eff(cut) = viol/‖a‖₂ ; incumbent에서 tight(a·x_inc − b ≥ −1e-6)면 ×100 ; nnz > maxNnz면 −1/eff ; forced면 1e100
정렬(내림차순) ; for cut: tol = (kind ∈ {OA, Envelope} || (루트 && pass<4 && kind ∈ {Gomory, MIR, StrongCG})) ? FeasTol : 1e-3
   viol ≤ tol → skip ; viol_count++ < needViol → skip ; 누적 nnz > 0.02·nnz(A) + 10000 → skip
   평행도: 비고정 열만 보고 이미 고른 cut과 cos(a,a') + 지지 겹침 비율 > 2·orthTol(1.5) → 거절
   채택 → engine.add_rows ; 채택 수 > maxCuts → break
```
4. `pool.trim`: 보조 목록이 한도 2배를 넘으면 LP 밖 cut을 `−1e6·viol_count + nnz` 순으로 정렬해 한도 아래까지 삭제.
5. LP 안 cut 정리: 원본 코드를 LP 행 관리(`6fbdc1`, `6c5f61`)와 simplex 쪽에서 찾아 포팅한다(노트 16 §6.2-1). 찾기 전에는 aging을 넣지 않는다. 원본에 없는 규칙을 넣으면 LP 행 수가 달라져 P1이 어긋난다.

## 4. 노드 cut (노트 16 §3.4)

`node.cut_target`일 때만 `separate(node, pass=100)`. 노드에서는 Gomory·L&P·close cut 없음, 값 ≥ 2인 종류만, pool cut은 2번 위반돼야 LP에. `root_probe`면 노드에서도 probing.

## 5. Separator 설계 (단계별)

표의 "단계"는 기본값(§6)이 0 이하인 종류에만 적용한다. **기본값이 1 이상인 separator는 전부 M2에 포팅한다**(flow cover, flow path, GUB, network, mod-k, strong CG, mixing, knapsack, relax-and-lift, dual implied, InfProof, L&P auto 포함). RLT·BQP·PSD는 2차·bilinear가 생기는 M4다. 루트 cut이 하나라도 빠지면 루트 bound와 노드 수가 달라진다.

| 종류 | 요지(노트 16 §3.5) | 단계 |
|---|---|---|
| Gomory(GMI) | LP optimal·분해 존재; 비등식 행 슬랙 추가(정수 행이면 정수 슬랙); 후보 = 기저 정수 변수 frac ∈ [1e-4, 1−1e-4], |x| ≤ 1e6, 점수(cutoff 없으면 (d+1e-6)(u+1e-6), 있으면 tableau penalty·bound 조임) 상위 200; `engine.tableau_row`; NumericFocus별 min/max 계수 거부; 비기저 bound 대입·보수화; **k = 1..7 배 행**으로 f0 ∈ [1e-4, 1−1e-4] 중 1/‖π‖ 최대(1.1배 이상 개선 시 교체); 강화 반올림(Letchford–Lodi) 조건부; 슬랙 역대입; 동적 범위 > 1e7(cut 행 썼으면 1e4) 거부; rhs += 1e-6; rank = 1 + max(사용 행 rank), rank > maxRank 건너뜀 | M2 |
| c-MIR aggregation | Marchand–Wolsey: 열 플래그(고정/이진/정수/연속, 내부/bound, VUB), 행 점수 `100·Σ0.5·dist + 3·개수 + nnz`, 내부 연속변수마다 소거 짝; 시작 행(혼합/정수/연속/'='×±1) → 최대 CutAggPasses(-1 → 2)번 합치며 생성기 시도: flow cover(레벨2 또는 VUB) / cover·GUB·knapsack(합친 행만) / **c-MIR**(VUB 대입 `u_j y* < 2x*`, 보수화, δ 후보 = |a_j| + 근사 GCD, δ/{2,3,4,5,8}, f0 ∈ [0.01,0.99], efficacy 최선, 보수화 뒤집기 개선, `F(a)=⌊a⌋+max(0,f_a−f0)/(1−f0)`) / strong CG(f 창 [0.069, 0.925]) ; 계수비 한도 100(NumericFocus≥1이면 10) | M2 (MIR, cover), M6 (flow cover, strongCG, mixing, knapsack 정확 분리) |
| Lifted cover(단일 행) | 모든 행(+1, '='면 −1), 루트는 pool cut도; knapsack 완화(비이진은 VUB/최소 기여); 이진 < 2·동일 계수·Σx* ≤ 1 거부; x* 내림차순 탐욕 cover(x* ≥ 1−IntFeasTol 고정), 극소화, sequence-independent lifting(누적합), 채택 Σαx* ≥ |C|−1+0.01 | M2 |
| Clique | 저장된 clique 사용 횟수 → 행에서 seed(a>0&x>0.05, a<0&x<0.95) → 상위 30 + 충돌 검사로 확장(합 > 1.05 유지) → 극대 clique로 늘려 전역 clique 표에 저장 → 표 전체에서 위반(합 > 1.01) clique 방출 `Σpos − Σneg ≤ 1 − |neg|`. probing과 표 공유 | M2 |
| Implied bound | 열/슬랙의 가장 위반된 VLB/VUB를 함의 `x_j ≤ cz + d`로, 위반 > 0.001이면 같은 bound 함의 이진들로 clique K: `x_j − Σ c_k z_k ≤ d` | M2 |
| Projected implied | 이진 z의 0/1 양쪽에서 함의·clique 표로 bound 조여 행을 z 분리에 투영, 위반 > 0.1 | M6 |
| Dual implied | 목적 ≠ 0 이진에 쌍대 논리, GCD 스케일, 위반 ≥ 0.05 | M6 |
| Zero-half / Mod-k | 예산 0.1·총work + 1e8; 행(±, pool 포함)에 연속변수 VLB/VUB·bound 대입, GCD 스케일, slack ∈ (−1e-6, 0.5); 홀수 계수만 남기고 보수화; 힙 기반 mod-2 XOR 소거; 결과 행(홀수 rhs, 홀수 열 ≥ 2, slack < 0.5)을 반올림/2. Mod-k는 k=3,5, slack < (k−1)−0.5 | M2 (zero-half), M6 (mod-k, 강화판) |
| Flow path / Network / 고정비용 | VUB 지도, 경로 탐욕 확장(길이 ≤ flow_path_passes 100), 흐름보존 행 union-find, `x − u·y ≤ 0` 합치기 | M6 |
| Lift-and-project | split 변수 선택(Gomory와 같음), **CGLP 보조 LP**(Threads=1, OutputFlag=-1)를 내부 `solve_core`로 풀고 결과를 c-MIR로 강화 | M6 |
| Wide split, pair-var, GRBCUT1, symmetry, weak symm, MIPSep, SubMIP, RelSubMIP | 숨은 종류. 설계만 자리 확보, 기본 꺼짐 | M6+ |
| Inf proof | 노드 LP 불가능·cutoff의 쌍대 증명(Farkas 조합 행)을 저장소에 넣고(`inf_proof` 모드 0/1/2 자동: 타이머 비율), 위반된 것을 재방출(1e-4/1e-6), c-MIR·flow cover로 강화 | M5 |
| RLT / BQP / PSD / OA / envelope | [08] | M4–M5 |
| Learned(probing) | 루트/노드 probing으로 바뀐 행을 투영, clique 확장 | M2 (루트), M6 (노드) |
| User / Lazy | `cbCut`/`cbLazy` → crush → pool 직접; 모델 `Lazy ≥ 1` 행은 presolve 모델에서 빼서 pool type Lazy("Extracted %d lazy constraints"), `Lazy = -1`은 user cut("Crushed %d out of %d user cuts to presolved model") | M2 |

"Cutting planes:" 통계는 **종료 시점 pool에 남은 종류별 수**(노트 16 §4.1). 출력 이름·순서는 노트 04 I 목록 그대로(25종).

## 6. 종류별 기본값 (Cuts=-1)

Cover 2, Implied 2, ProjImplied 0(Cuts≥2면 1), Clique 1, MIR 1, FlowCover 1, FlowPath 1, GUBCover 1, ZeroHalf 1, InfProof 2, Network 1, ModK 1, StrongCG 1, RLT 1, RelaxLift 1, BQP 1, PSD 1, LiftProject -1(auto), Mixing 1, DualImplied 1, Knapsack 1, SubMIP 0(Cuts≥2면 1), MIPSep 0, Gomory auto. `Cuts=2` → GomoryPasses 5, `Cuts=3` → -3.

## 7. 휴리스틱 스케줄러 (`heur::scheduler`) (노트 16 §3.6.2)

```
if Heuristics == 0 || status != Running: return
T = 총 work ; r = (bestBound − rootBound)/(incumbent − rootBound)
루트: fN = fS = Smult[크기 표 6칸]·H·root_heur_mult
H ≥ 0.25: fN = H/2 ; fS = max(H − 휴리스틱 시간/T, H/2)
else: fN = H ; fS = r<0.01 ? 4H : r<0.05 ? 2H : r<0.1 ? H : H/2 (+0.05·min(1,1e4/n) 최근 1e4 노드에 incumbent 갱신 시)
B = T·fN + (sub-MIP ? 1e5 : 1e7) ; Bs = T·fS + 상수 ; 정수 < 열의 1% → Bs를 0 또는 /10 ; Bd = Bs/3 ; Bs −= Bd
항상: 단순 반올림(ID6), 반올림2(ID7) ; 루트: ID 0xd(0.95), 개선 탐색(0x2a)
order = {0xf, 8, 0x10, 7, 0x12, 0x13, 0x14, 0x15, 0x25, 0x29, 0x37}을 누적 시간 오름차순
for id in order: if 휴리스틱 시간 < B && 노드 주기 규칙(id, n): run(id)
MIPFocus==4 && improve_method && incumbent && !루트: UCB1 bandit(§9) ; return
루트: BINHEUR, root-final 고정 휴리스틱(0.9), greedy ; VarHint 휴리스틱 ; FIXEDCHARGE
root-final && 없음: HAILMARY, int2bin, LURKBD ; zero-objective(예산 H·T·4, 재시작 뒤 30) ; !재시작 && PumpPasses && !pump_done: feasibility pump ; MinRelNodes>0: MinRel
노드: INFIMPROVE, BINHEUR, UNDERCOVER(4·H·T+1e6), NLPHeur ; RINS 블록(§8) ; partition/relax-and-fix ; SolutionLimit==1 → 즉시 고정
각 휴리스틱 뒤: termination::check, 예산 초과면 중단
```

노드 주기(소수): ID6 n%137==13; ID7 n≤1000 && n%91==0; ID8 루트/n%11==4(n≤1000)/n%97==4; 0xf n%97==0; 0x10 루트/n%11==7/n%97==7; 0x12 root-final/n≤100&n%9==0/n≤1000&n%101==0/n%1002==0; diving 3종 u=0,3,6: n≤100&n%8==u/n≤1000&n%94==u/n%997==u; 0x25 n%97==9/n%983==9; 0x29 n%93==3/n%979==3; 0x37 incumbent 없음 && (루트/n%11==5/n%97==5).

## 8. 개별 휴리스틱 (노트 16 §3.6.3–3.6.6)

표의 "단계"는 기본값에서 스케줄러(§7)가 부르지 않는 휴리스틱에만 적용한다. **기본값에서 불리는 휴리스틱은 전부 M2에 포팅한다**(presolve 전 휴리스틱, zero objective, RINS-dive 등 포함). 해 발견 시점이 달라지면 cutoff와 노드 수가 함께 달라진다.

| 휴리스틱 | 설계 | 단계 |
|---|---|---|
| sub-MIP 엔진(`8e3397`) | 고정 bound 전파 → 모두 고정이면 점 검사/연속부 LP → 자유 정수 > n/4+10000이면 "너무 큼"(8) → 최대 max(5, 100H)회: sub-model(단순/일반) + 파라미터(Threads=1, OutputFlag=-1, Seed += 0x12d8652, NodeLimit=SubMIPNodes, Cutoff=min(부모, inc+1e-6), subsolve_depth=1|2, PartitionPlace=0, 일부 cut 0) → `solve_core` → 해 제출. `subsolve_depth`가 재귀 폭주를 막는다 | M2 |
| 단순 반올림, 반올림2, 루트 반올림 | 매 노드/루트. LP 해를 lock 방향으로 반올림 | M2 |
| fix-and-propagate 계열 7개 | 변수 정렬(|obj| 등) → 반올림 고정(비율 0.8–1.0) → 전파 → 막히면 되돌림 → sub-MIP | M2 (2개), M5 (나머지) |
| LP diving 3종 | fractional / coefficient / guided (ID 0x13–0x15) | M2 |
| RINS | 조건: incumbent, 정수 > 29. 기준점(LP 해 또는 incumbent, gap에 따른 확률) ; 점수 `|x_LP − x_inc| + 1e-6·rand` ; 일치 변수부터 round(x_inc) 고정 ; 고정·전파 반복 → sub-MIP ; "너무 큼"이면 적응 카운터. 스케줄: RINS=-1 → 루트 1회, 노드는 마지막 뒤 54노드 이상; 단계 1 `rins(0.75)`, 단계 2 `rins(0.01)`; RINS=k → n%k==0 | M2 |
| RINS-dive, greedy, 주기형, 노드 sub-MIP, 장기 정체 sub-MIP(해 조합) | | M5 |
| Feasibility pump | 건너뜀 조건(PumpPasses<1, basis 없음, SOS/QC/NL) ; LP 복사 + basis ; 일반정수 보조변수 ; pass ≤ PumpPasses: x̃ = round(x*) ; 해시 순환 감지 → T=min(nInt,(rand+0.5)·20)개 뒤집기 ; 71패스 안이면 **전파 기반 반올림** ; 아니면 무작위 뒤집기 ; 거리 목적(하한 +1+1e-5·rand, 상한 −1−…, 일반정수 |x−x̃|) ; LP ; 정수면 sub-MIP로 마무리 ; 10패스마다 95%(100: 80%, 500: 50%) 정수면 고정 sub-MIP ; 로그 "Feasibility pump: pass %d, min int violation %.3f, total elapsed time %.0fs" | M2 |
| zero objective | 목적 0 복사본 재presolve(크기 50% 이상이면 포기), NodeLimit=ZeroObjNodes/SubMIPNodes(재시작 ×10), 해 찾으면 실제 목적으로 재연마; sub-MIP 불가능이면 원모델 INFEASIBLE 증명 | M5 |
| MinRel | 행마다 위반 슬랙(등식 2개), 목적 = Σ슬랙, NodeLimit=MinRelNodes | M5 |
| int2bin | 일반정수 ≥ n/2, LP값 ∈ [0,1]인 정수 bound를 [0,1]로 | M5 |
| NoRel | LP 없는 지역 탐색, phase 1(실행가능) / phase 2(개선), 연산자 8개 적응 가중치, lazy 처리. NoRelHeurTime/Work/Solutions 설정 시 | M6 |
| partition / relax-and-fix | `Partition` 속성 블록별 sub-MIP 병렬, PartitionPlace 비트(0x10 루트 전, 8/4 루트, 2 노드, 1 뒤) | M6 |
| HAILMARY, LURKBD, UNDERCOVER, INFIMPROVE, CARDTWOOPT, FIXEDCHARGE | 숨은 것. 기본값에서 스케줄러(§7)가 부르는 것은 M2에 포팅, 나머지는 M6 | M2/M6 |
| presolve 전 휴리스틱(`914d54`) | 원모델 구성·수리 패스, "Found heuristic solution: objective %s" | M5 |
| MIPCLEANUP(`484dab`) | 목적 ≠ 0 변수 고정 재풀이, "Cleanup yields a better solution" | M5 |

## 9. 개선 모드 (노트 16 §3.6.8)

MIPFocus=4(내부)에서 `improve_method`면 UCB1 `succ/calls + sqrt(2 ln N / calls)`로 개선 이웃 3개(SOS/QC/NL 있으면 6개)를 고른다. 이웃의 실제 구현은 원본 함수를 포팅한다. local branching을 따로 만들지 않는다(노트 16 §6.2-7: 전용 구현 미확인).

## 10. 포팅으로 확정할 지점 (노트 16 §6.2)

- LP 안 cut 정리(§3-5), `aabbe3` 정렬 방향.
- cut 종류 0x13의 생성 함수, 0xb(`811c50`, GRBCUT1)의 계열, `88d195`의 TYPE 인자.
- 내부를 추적하지 않은 생성기: mixing `88b0e9`, flow cover lifting `7fa759`, cover lifting(`7d129e` 530–560, `ab9789`), L&P의 CGLP 정규화, `8203f5`, `82dd5d` 후반, RLT·relax-and-lift·BQP·PSD.
- 적응형 끄기 식의 `k`(root mip+0x68)와 `5e8e36(…,-1)`, 루트 루프의 `mip[0x212]` 조건, `mip+0x3470` 플래그, `6e2f83` 조기 종료 조건.
- 휴리스틱: 루트 예산 배수 표(.rodata 6칸, §7의 `Smult`), 예산 필드 `mip+0x8c0/0x8c8/0x8d0/0x90`, `8c9e64`·`8cbd1f`·`8b5bdc`·`8faaa4`·`8d8fd1`·`9552a0`의 역할, NoRel 내부(`942283`, `8d84f3`), 비동기 sub-MIP(`8e5470`, `8e9318`).
- 숨은 env 오프셋 env+0x2458, +0x2498, +0x27d8, +0x28c8의 이름.
- `ROOTHEURMULT`(0x2928)를 읽는 함수 안의 라이선스 부분은 옮기지 않는다([10 §7](10_porting.md)).

## 11. 테스트

- 패리티 P1(합격 기준): `miplib_small` 38개(Threads=1)에서 "Cutting planes:" 종류별 수, 루트 패스별 bound, 루트 휴리스틱 해의 목적값과 순서가 Gurobi와 같다.

- separator 단위: 교과서 예제(각 논문의 running example)에서 cut이 나오고 위반·유효성(원 정수해가 cut을 만족) 확인. 유효성은 proptest로 무작위 작은 IP의 모든 정수점 열거.
- cut 선택: 평행도·밀도 필터 단위 테스트.
- 휴리스틱: `miplib_small` 인스턴스에서 루트 뒤 incumbent 존재 비율 기록(합격 기준 아님, 회귀 추적).
- 전체: Cuts=0/기본에서 `mas74`, `pk1`, `neos5`의 노드 수 비교(기본이 작아야 함).
