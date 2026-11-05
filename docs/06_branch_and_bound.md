# 06. MIP 분기 한정(branch-and-cut) 트리

포팅 원본: [port_map.tsv](port_map.tsv)의 `06_bnb`(273개). 진입 함수는 `5bbf7f`, `5daa82`, `5bf638`, `5c239f`, `6e8eb3`, `5fb8ec`, `5ce67b`, `62a05d`, `62f909`, `646052`다. 근거: 노트 15 전체. 읽기 보조 문헌: Achterberg 2007, Achterberg–Koch–Martin 2005(reliability branching), Linderoth–Savelsbergh 1999, Bénichou 1971(pseudocost).

## 1. 구조 (노트 15 §1)

```
mip::bnb::solve(presolved, p, lim)                 // 5bbf7f: 새로 시작 또는 "Continuing optimization..."
  Mip::new (5daa82): 파라미터 사본 → M, cut·휴리스틱 설정 해석, lazy/user cut 추출, node file 준비, 루트 노드 생성
  root::process (64da9f): 루트 LP, cut 루프, 루트 휴리스틱, 루트 분기 반복   → [07]
  status = termination::check(M)
  tree::search (5bf638): threads==1 ? worker_loop(w0) : deterministic_rounds()
  정리: 상태 확정(§7), 결과 기록, "Cutting planes:" 통계, "NodeLP residual statistics"
```

```rust
pub struct Mip { m: PresolvedModel, p: Params /*사본*/, tree: Tree, pool: SolutionPool, cuts: CutPool, pc: PseudoCost, workers: Vec<Worker>, stats: Stats, work: WorkCounters, restarts: u32, root_bound, objective_step: Option<f64>, … }
pub struct Tree { heap: BinaryHeap<NodeRef /*key = bound + 2⁻⁴⁸ tie-break, 최소 힙*/>, lower_bound: f64, mem_bytes: u64, nodefile: Option<NodeFile> }
pub struct Node { id: u64, parent: Option<NodeId>, depth: u32, bound_changes: Vec<(Var, Dir, f64)> /*부모 대비*/, info: NodeInfo }
pub struct NodeInfo { lp_obj, bound, estimate, status: NodeStatus, branch_var, branch_val, branch_dir, plunge_depth, basis: Option<CompressedBasis>, lb/ub(압축), flags { excluded, cut_target, selected, inherited } }
pub enum NodeStatus { New, Integral, Fractional, Postponed, Infeasible, Cutoff, InfOrUnbd, Interrupted, Rejected, Refix, Unbounded }   // 0..11
pub struct Worker { id, state: AtomicU8 /*0 출발 1 실행 2 대기 3 종료*/, engine: SimplexEngine, heap: BinaryHeap, plunge: Vec<NodeRef>, deferred: Vec<NodeRef>, pc_local: PseudoCost, budget: f64, stats }
```

## 2. 노드 루프 (`worker_loop`) (노트 15 §3.4)

```
while status == Running:
   재시작 판정(§8) → 재시작이면 continue
   if MIPFocus == 3: bound-focus 전처리
   node = select(worker)                                    // §3. None → break
   if node.bound == +inf: mark Infeasible
   elif cutoff <= node.bound: mark Cutoff
   else:
      engine이 다른 노드를 들고 있으면 노드 LP 다시 적재 + 저장 basis 웜스타트
      process_node(node, worker)                            // §4
      if Integral: submit_incumbent(node.x)                 // 6fb9c9 → 6f602c
   if Fractional: heuristics::run_node(node)                // [07 §5]
   if status != Running: mark Interrupted
   store_basis(node)
   if needs_branch(node): branch(node) → children[2]; Integral이면 다시 submit
   for c in children: push_plunge(worker, c)                // 6e713c; cutoff/aspire/깊이 조건이면 deferred
   if node.Unbounded: status = UNBOUNDED
   requeue if Interrupted/Rejected/Refix ; finish_node(node) (콜백 where=MIP(3), MIPNODE(5); NodeCount++; 로그 행; 조상 감쇠; 메모리 회계)
   deferred 처리(cutoff 이하면 버림, 아니면 전역 힙)
   cut buffer → pool 동기화 ; nodefile::offload(tree)
   status = termination::check(M)                          // §7
```

## 3. 노드 선택 (`select`) (노트 15 §3.5)

1. **plunge 목록**(worker 지역)이 비어 있지 않으면 거기서 §3.2 규칙으로 고른다.
2. 비었으면 **힙**(단일 스레드: 전역 최소 힙, 병렬: worker 힙)에서 pop. pop 전에 node file의 최소 bound < 메모리 힙의 최소 bound면 파일에서 읽는다. 꺼낸 노드 검사: `Refix` 상태, 깊이 한도(`plunge_depth + 10·ncols`), `cutoff < bound`, `aspire < 1 && best + aspire·(inc − best) < bound` → deferred. 통과하면 plunge 목록에 넣고 `plunge_depth = depth`.

### 3.2 plunge 규칙 (선택 순번 k)

```
rule = 11 if objective_step 미정 && ncols == nint
     3 if VarHint 있음 && !IgnoreHints && k < 10
     6 if M.flag_848 ; 9 if BranchDir != 0
     2 if k > 8 && (k < 17 || 해 없음 || k % 11 == 0)
     (frac_ok ? 4 : 2) if k > 8 && k % 7 == 0 ; 5 if k > 8 && k % 13 == 0
     if k > 8 && k % 19 != 0: 7 if k % 43 == 9 ; k 홀수 → (rand < 0.9 ? 3 : 12) ; 짝수 → 3
     else (M.flag_808 == 0 ? 6 : 1)
     5 if MIPFocus == 4 (ImproveStart 내부 모드)
점수(최대 선택): 2 −est ; 3 depth − (est−minEst)/1e8 ; 12 depth + (est−minEst)/1e8 ; 9 10·depth − (est−minEst)/1e8 ± BranchDir 방향 ; 5 무작위(`a7b4d6`) ; 4·1·6·7·11은 `6e8eb3` 점프 테이블 대상(0x5b3e1e, 0x5b4274, 0x5b3bc1, 0x5b39fc, 0x5b37ca)과 조상 탐색 `5fb13c`를 objdump로 읽어 포팅한다. 3으로 대신하지 않는다. 선택 뒤 조상 깊이 계산(0x5b451b–0x5b4656)도 옮긴다
선택 뒤: VarHint가 있으면 hint와 맞는 방향의 자식 우선
```
요약: 전역 best-bound, 지역 depth-first(다이빙)에 best-estimate·무작위를 소수 주기로 섞는다.

## 4. 노드 처리 (`process_node`) (노트 15 §3.6)

```
node_presolve(node)  (node_pre≠0) 또는 단순 bound 일관성 검사(ub−lb < −FeasTol → Infeasible)
bound 변경을 engine에 반영 (engine.change_bounds)
cut_target 플래그 갱신: LP optimal 아님 || pool이 노드 한도 이상 || (incumbent: (z_inc−z_parent) < 10(z_node−z_parent)) || (없음: bound가 부모보다 1e-6 넘게 오름) → 끔
root_probe≠0 → probing (Learned cut)
DualReductions → 노드 쌍대 축약(시도 ≤ 1000 또는 성공 > 0)
cut_target → cuts::separate(node, pass=100)                   // [07 §4]
loop ≤ 100:
   lp::solve_node(node) → NodeStatus (OPTIMAL이고 분수 정수 0 → Integral, 아니면 Fractional; INFEASIBLE → Infeasible; CUTOFF → Cutoff; 한도 → Interrupted)
   루트면 tree.lower_bound = max(·, node.bound)
   termination::check
   Fractional && incumbent: reduced_cost_fixing (rcfix) ; 고정 생겼고 NodePre면 node_presolve 다시
   node_probe && NodePre && !루트 && Fractional: 통계로 빈도 조절 후 node probing; infeasible → Infeasible
   bound가 안 바뀌면 break
propagate (domain propagation worklist: [08 §5]) 는 node_presolve 안에서 호출
```

노드 LP(`lp::solve_node` = `5ce67b`): NodeMethod=-1이면 primal/dual 적응(기본 dual, 노드 LP가 dual infeasible로 시작하는 비율이 높으면 primal). 루트에서만 "Root relaxation: objective %.6e, %.0f iterations, %.2f seconds (%.2f work units)". 반복 한도는 `IterationLimit`와 병렬 라운드 예산.

## 5. 분기 (`branch`) (노트 15 §3.7)

### 5.1 VarBranch 해석
-1 → `flag_848 ? 1 : 0`. 루트 자동(`64da9f`): MILP이고 `|d_j| > 10·OptTol`인 reduced cost 수 ×5 ≤ ncols+nrows(쌍대 퇴화)면 1. 드라이버 조정(열 ≥ 5000, 이진 < 11)도 1.

| VarBranch | 경로 |
|---|---|
| 0 pseudo reduced cost | 후보(분수 정수 변수, `|x| > 1e6`는 뒤로, BranchPriority 최대만) → reliability branching(§5.2) |
| 1 pseudo shadow price | pseudocost 표가 `ncols+nrows` 크기라 행(슬랙)에도 값. 후보 평가 함수 집합이 다르므로 원본 경로(`62a05d` 디스패치)를 그대로 포팅 |
| 2 max infeasibility | `|x − round(x)|` 최대. 원본 분기 위치는 미확인(`63056b`/`634ddc` 안으로 추정) |
| 3 strong branching | 모든 후보 SB |

### 5.2 Reliability branching (§3.7.3)

```
pc = (cnt_down, cnt_up, sum_down, sum_up)[n+m] ; η(신뢰 임계) ; sb_cache {node_stamp, work_stamp, x, down_gain, up_gain}
gain(j, f): ψ⁻ = sum_down/cnt_down (0이면 0) … ; 같은 노드·work 차 ≤ 100·|x−x_cache| ≤ 0.01이면 캐시 사용 ; ψ = max(ψ, 1e-6) ; down = ψ⁻·f, up = ψ⁺·(1−f)
score(j) = sqrt((down + 1e-5)(up + 1e-5))                                  // product score
unreliable(j) = cnt_down < η || cnt_up < η (루트가 아니거나 캐시 없을 때)
η 자동: SB 누적 시간 < 전체 노드 시간·1% → η = ∞(전부 SB), SB iter 한도 = ∞
        아니면 cnt = 총 work / SB 1회 평균 비용 ; η = max(1, min(cnt, η_max))
for j in 후보 (점수 내림차순, ≤ sb_max_cands):
   if unreliable(j): (down, up) = strong_branch(j, iter_limit)   // dual simplex 웜스타트, 반복 한도 §5.3
       한쪽 infeasible → 변수 고정, 노드 Refix(다시 처리) ; 양쪽 → Cutoff
       캐시 갱신 ; lookahead(개선 없이 8개 연속이면 중단)
   best = argmax score
```

pseudocost 갱신: 자식 LP 결과로 `sum[dir][j] += Δobj/f, cnt++`를 하는 원본 지점을 `6fdec9`(5fb8ec:2584, 5fe9cd)와 `5ce67b` 후처리에서 찾아 포팅한다. 표준식으로 대신하지 않는다. 갱신 시점과 식이 다르면 노드 수가 달라진다. 조상 감쇠: 노드가 Infeasible/Cutoff면 조상 경로의 분기 변수에 `sum[dir][j] *= (1 − w/(cnt+1))`, w = 0.5에서 단계마다 절반, 0.001까지. 병렬은 worker 복사본을 라운드 끝에 합친다.

### 5.3 strong branching
- 순차: 같은 엔진에서 bound만 바꿔 반복 한도까지. "Warning: relaxation becomes infeasible in strong branch."
- 병렬: 엔진 복제 + VBasis/CBasis·dual norm 이식 후 스레드별 후보. M6.
- 반복 한도: `sb_max_iters ≥ 0 → max(1, ·)`; `flag_808 == 0 → 1`; 아니면 `it = (rootIters/3 + 10 + q/10)/(p+1)`, p는 MIPFocus 3 또는 노드>1e5면 /10, MIPFocus 2 또는 노드>1e4면 /3; p<100 → 무제한, 100≤p<1000 → it(10..30 범위면), 그 밖 ±.

### 5.4 자식 생성
down(`<`, floor) 먼저, up(`>`, floor+1). `child.bound = max(parent.bound, 추정, SB 결과)`, `child.estimate = max(child.bound − parent.lp_obj, pseudocost gain, 1e-6) + 누적`, 힙 키 = bound + 2⁻⁴⁸·|bound|·min(1, 비율). BranchDir는 생성 순서가 아니라 선택 규칙 9에서 쓴다. SOS 분기(`sos_brrat`)와 공간 분기(kind 5 → [08 §6])는 같은 디스패치.

## 6. incumbent, 해 풀, cutoff (노트 15 §4.4, 16 §3.6.7)

`submit(x)`: 목적값·실행가능성 검사(FeasTol·IntFeasTol, 원 presolved 모델) → 풀에 정렬 삽입(PoolSolutions, PoolGap) → incumbent 갱신 → cutoff = `min(Cutoff, inc − (|inc|+1)·1e-10)`, 목적 간격(objective step: 모든 목적 계수가 정수 배수면 그 gcd)이 있으면 다음 아래 간격값 → 로그 `*`(노드 LP 정수해) 또는 `H`(휴리스틱) 행, "Found heuristic solution: objective %s"(헤더 전) → 콜백 MIPSOL → SolFiles.

## 7. 종료 조건 (`termination::check`) (노트 15 §3.9)

```
if status != Running: return status
limits::check (MEM/terminate/Time/Work)
bound = tree.lower_bound(목적 간격 반올림) ; inc = pool.best
if 열린 노드 없음: return inc < ∞ ? OPTIMAL : INFEASIBLE
if bound ≥ ∞: INFEASIBLE ; if cutoff ≤ bound: OPTIMAL
if inc < ∞: MIPGap>0 && (inc−bound)/max(|inc|,1e-6) < MIPGap → OPTIMAL ; MIPGapAbs>0 && inc−bound < MIPGapAbs → OPTIMAL
if inc < BestObjStop+1e-10 || BestBdStop−1e-10 < bound: USER_OBJ_LIMIT(15)
NodeLimit → NODE_LIMIT ; Σ simplex iters ≥ IterationLimit → ITERATION_LIMIT ; 해 수 ≥ SolutionLimit → SOLUTION_LIMIT
ImproveStartGap/Nodes/Time/Work 도달 && incumbent → ImproveStart 전환 (내부 상태)
```
로그 gap은 `|inc−bound|/|inc|`(|inc|<1e-6이면 "-").

## 8. 재시작 (노트 15 §3.10)

조건(`mip_lim` 기본 510, ∞면 끔): 처리 노드 수가 `sqrt(threads)·mip_lim`(×0.5/2/5 변형)을 넘고, 루트 이후 고정된 변수 수 + cut 이득 + MIPFocus 배수가 `ncols/5 + 10`을 넘으며, 분기 비율 이동평균 ≥ 0.2, gap 개선이 `(inc − rootBound)·0.1` 대비 충분할 때. 실행: incumbent·bound 저장 → 루트 LP 재풀이 → 비기저 변수 strong branching 고정("Strongbranch on nonbasic variables: %d fixed") → 고정 반영 모델 → `mip::presolve` 재실행(postsolve 스택 한 층 추가) → 트리를 루트부터 다시. `restarts += 1`(cut 한도·밀도·정체 허용이 너그러워짐).

## 9. 병렬 결정적 라운드 (`deterministic_rounds`) (노트 15 §3.3, §3.12) — M6

```
worker 스레드 생성, 모두 alive 될 때까지 spin
while Running:
   재시작 판정 ; pseudocost 표를 worker에 복사 ; work 카운터 저장
   빈 worker에 전역 힙 best-bound 노드 배분 ; worker 힙 ↔ 전역 힙 재배분 ; node file 읽기
   라운드 work: q = 지난 노드 수/(threads·2500) → ×100, cap = min(1000, 5e8/(16(m+n))), len = clamp(max(q·100, 이전), 1, 0.9cap), 노드 안 늘면 이전×2 ; roundWork = max(len·노드당 평균 work, 1e7)
   worker.state = 출발 ; master는 worker0 몫 직접 ; 모두 대기 상태까지 기다림
   worker는 끝에서 pseudocost·cut pool·해 풀을 전역에 반영
   termination::check
```
시간이 아니라 work 예산이므로 스레드 수가 같으면 결과가 같다.

## 10. node file (노트 15 §3.11) — M5

`NodefileStart`(GB) 초과 시 힙에서 bound가 가장 나쁜 ≈4MB 분량을 quickselect로 골라 `"%s/grbnodes%d"`로 직렬화. 읽기는 §3. 디렉터리 `"%s/GRBnodefile%d"`, 끝에 삭제.

## 11. ImproveStart 단계 (`5be289`) — M5

ImproveStart* 도달 → Heuristics=0.5, RINS=10, MIPFocus=4(내부), cut 끔("Resetting heuristic parameters to focus on improving solution (using Heuristics=0.5, RINS=10 and disabling cuts)...") → **같은 트리로** 탐색 계속 → 끝에 복원.

## 12. Concurrent MIP (`9b2c8c`) — M6

ConcurrentMIP ≥ 2: 인스턴스마다 모델 복사, `Seed + i`, 스레드 분배, incumbent·bound를 `termination::check`에서 공유(CONCURRENTMODE==1). 먼저 끝난 인스턴스("Instance %d was solved")의 해 풀을 가져온다.

## 13. 로그 (노트 15 §3.13)

헤더 `    Nodes    |    Current Node    |     Objective Bounds      |     Work` / ` Expl Unexpl |  Obj  Depth IntInf | Incumbent    BestBd   Gap | It/Node Time`. 행 `" %5.0f %5.0f %10s %4d %4s %10s %10s %6s %5s %5s"`, 새 incumbent `*`/`H`. Obj 열 문자열: 값 | "postponed" | "infeasible" | "cutoff" | "-". Gap: ≥100 → "-", ≥0.995 "%.0f%%", <0.0995 "%.2f%%", 그 외 "%.1f%%". DisplayInterval 간격, OutputFlag≥2면 매 노드. 병렬이면 "ParUtil" 열.

## 14. MIP start (`6be95a`) (노트 15 §3.16)

StartNodeLimit=-3이면 생략. 각 start: 완전해면 검사·제출("Loaded user MIP start %d with objective %g"), 부분해면 "Completing partial solution with %d unfixed non-continuous variables"를 sub-MIP(StartNodeLimit/StartTimeLimit/StartWorkLimit)으로. "Processed %d MIP starts in %.2f seconds". 이전 solve의 해 풀은 새 solve의 시작 풀("MIP start from previous solve").

## 15. 포팅으로 확정할 지점 (노트 15 §6.2)

- pseudocost 일반 갱신 지점(§5.2), VarBranch=2 경로(§5.1), IntFeasTol로 분수 변수를 세는 함수(LP 정보 +0x80).
- 노드 선택 규칙 1, 4, 6, 7, 11의 점수와 `5fb13c`의 용도(§3.2).
- `I+0x20`(est)의 정의와 `627f3e`:300–308의 누적값.
- 재시작 `6a0962`(2차 판정), `6a3c81`(재시작 모델), 재배분 `6e7b32`.
- `M+0x600`, `M+0x808`, `M+0x848`, `M+0x40`의 의미.
- 숨은 파라미터 MIPLIM, ASPIRE, SBMAXCANDS 등의 기본값(정적 표), SB 후보 수 자동값과 후보 정렬 기준.

## 16. 테스트

- 패리티 P1(합격 기준): `miplib_small` 38개를 Threads=1로 Gurobi와 나란히 돌려 루트 LP 반복 수, 루트 bound, `NodeCount`, 해 발견 순서가 같다. 60초 안에 끝나는 인스턴스는 노드 수, 시간 한도에 걸리는 인스턴스는 같은 work 지점의 노드 수를 본다.
- 패리티 P3: 같은 38개에서 OPTIMAL까지 시간의 기하평균이 Gurobi의 1.2배 이내. `re/bench/results/mip.csv`의 상태(OPTIMAL 20, INFEASIBLE 2, TIME_LIMIT 16)와 같다.

- 단위: 힙 키·tie-break, plunge 규칙 표, pseudocost 갱신·감쇠, termination 표 전수, cutoff 계산(목적 간격 포함).
- 통합: `re/bench/data/miplib_small.csv`의 easy 인스턴스를 60초 한도로 풀어 `best_obj`와 비교(OPTIMAL이면 상대 1e-6, TIME_LIMIT이면 bound ≤ best ≤ inc). `neos859080`은 INFEASIBLE.
- 결정성: 같은 인스턴스를 Threads=4로 3회 → NodeCount·IterCount·ObjVal 동일.
- 콜백: MIPNODE에서 cbGet(MIP_OBJBND, MIP_NODCNT), MIPSOL에서 cbGetSolution, cbLazy로 TSP subtour 예제.
