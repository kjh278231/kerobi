# M3-04c-7 primal push 위반 basis 재시도 ratio `056973`

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다. §1–§5는 원본 대상 캡처 전에 고정한다. 탐색 실행은 §4.4에 따로 적고 합격 근거로 쓰지 않는다.

## 1. 범위와 주장

- 작업 종류: 오라클 구축 + 구현. 기준선 main `c383966`(M2-03d-7 병합, M3-04c-2 포함), branch `work/m3-04c-7`, worktree `../rsolver-m3-04c-7`, 미커밋 선행 변경 없음.
- 범위 분할: 스프린트 행은 `056973` 재시도와 theta=I 반대 방향 재호출(위반·feasible)을 묶었다. 두 계약은 코드 경로를 공유하지 않는다(feasible 재호출은 `056973`을 거치지 않는다). 그래서 1주 범위에 맞춰 재호출을 새 행 **M3-04c-8**로 나눈다([인수인계 안내](../handoff/README.md) §1-6).
- 질문 1(오라클): double primal push `134a87`이 위반 basis에서 `054eef` 결과가 위치<0 또는 |pivot|<0.001일 때 부르는 `056973`의 결과와, 그 결과로 가는 단계(거리 flip·승인·보류·이탈 상태)가 실제 원본 실행에서 어떻게 나오는가. `056973` 내부 분기(아래 포함 분기 2)는 원본 직접 호출로 고정한다.
- 질문 2(구현): Rust `retry_ratio`(`056973`)와 게이트 `violated_ratio`(`054eef` → `056973`)가 같은 입력에서 원본과 비트 단위로 같은가. Rust `push_step`이 재시도 결과와 theta=I 비재호출 단계를 원본처럼 판단하는가.
- 포함 분기:
  1. 게이트(C 1447–1452, asm `0x1c258e–0x1c25ae`): `054eef` 위치<0 이거나 `0.001 > |pivot|`(ordered)이면 `056973`. 위반 경로에서만 있다.
  2. `056973`(asm 명세 §2): 1단계 breakpoint·기울기(표시·부호별, sparse N/U(a>0)의 cap 미검사), 후보 없음+기울기<0, 기울기≥0의 t=0, quickselect(Hoare 분할·차선 합), 2단계 잔차 선택(×2 유무, 두 번 시도하는 A·B, |a| 엄격 비교), 최소 ratio 대체, 작은 pivot 재진입과 `12dd49` 요청 2, work.
  3. 재시도 뒤 단계: 거리 flip(theta=I 비재호출 포함), 승인, 보류, 이탈 `B`(a<0)→-2·`A`(a≥0)→-1. 판단 코드는 04c-2와 같은 `0x1c290d` 이후다.
- 제외와 후속:
  - theta=I 반대 방향 재호출(위반·feasible, C 1458–1586) → M3-04c-8. 이번 판단 모델은 재호출 조건(자유 열, 또는 방향 0·ub≥I)이 참인 단계를 실패로 거부하고, 대상 LP는 그 조건을 피한다.
  - `056973` 재진입 2차 패스(작은 pivot, `12dd49`)의 Rust → `Reentry`로 거부한다(054eef·055f68과 같음). 원본 합성 출력만 기록한다.
  - 위치<0·유한 theta(재진입 요청 2 반환)의 단계 판단은 자연 실행에서 오지 않는다. Rust `push_step`은 C 순서(거리 검사 뒤 flip)로 두고 미검증으로 적는다.
  - batch, x87, pivot 적용 뒤 basic·rhs·factor, push 연결, 자연 work 전체는 범위 밖이다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C: `../re/decomp/c/000f0120_PRIVATE0000000000056973.c`(재시도, 879줄), `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c`(push), `../re/decomp/c/001bb6f0_PRIVATE000000000012dd49.c`(factor 갱신 수 `*(S+0x470)+0x18c`).
- asm 명세: 함수 전체를 objdump로 대조해 [asm 판독 명세](primal_push_retry_asm.md)에 주소별로 남기고 아래로 요약했다(디컴파일과 다른 점은 2.3). dense 1단계는 `0xf1343–0xf15b8`에 있어 `tools/dc.py 056973 --asm` 기본 창 밖이다(`--range 0xf1340,0xf15c0`).
- 시작 경계: `supply:leave`(candidate≥0). 종료 경계: `step:enter`·`step:defer`, pivot이면 `step:applied`. 04c-2와 같다.

### 2.1 ABI(호출점 `0x1c25b4–0x1c263a`)

xmm0=h(`S+0x380`), xmm1=pivot 허용오차(`S+0x398`), xmm2=I(`S+0x370`), xmm3=cap(=I). rdi=S, esi=m, rdx=alpha(+0 count, count<0이면 dense, +8 인덱스, +16 값), rcx=basic, r8=width, r9=표시. 스택은 theta*, pivot*, 정수 scratch, 실수 scratch, work* 순이다. **`054eef`와 순서가 다르다**(그쪽은 scratch가 먼저). A(a>0)·B(a<0)는 행마다 breakpoint 둘을 넣으므로 scratch는 2m이 필요하다. 정수 scratch에는 sparse 위치 k(dense는 행)가 들어가고 반환값·pivot도 그 위치로 읽는다. x·w·표시는 행으로 읽는다.

### 2.2 제어 흐름

```text
loop: acc9=acc10=0, ncand=0, chosen=-1
 1단계: a > tol → A: cand((x-w)/a), cand(x/a), acc9 -= a | B: acc9 += a | L,X: acc10 += a | N,U: x/a (sparse는 cap 미검사)
        -tol > a → A: acc9 -= a | B: cand(x/a), I>w면 cand((x-w)/a), acc9 += a | L,N: I>w면 cand((x-w)/a) | U,X: acc10 -= a
        cand(r): cap > r일 때만. work: sparse ((4c+n)+n)·coef, dense ((4c+nnz)+m)·coef
 slope = acc10 + acc9
 ncand==0 && 0>slope: 첫 패스면 theta=minsd(cap,I), pivot=0, -1. 재진입이면 12dd49(S)>16일 때 S+0x294=2, S+0x2f8=-1, -1,
                      아니면 이전 선택(theta·pivot 미기록)
 0>slope: Hoare quickselect(pivot 값 dscr[lo], 엄격 비교, 두 scratch 함께 교환). 분할 뒤 0>slope면 왼쪽 |a| 합을 더하고
          아니면 오른쪽을 뺀다. 8개 이상은 차선 A(0,1) B(2,3) C(4,5) D(6,7) 누적에 slope를 A 0번 차선에 두고 (A+B)+(C+D)
          뒤 low+high, 나머지는 순차. 0>slope면 lo=j+1, 아니면 hi=j. t=dscr[lo]. I>cap && cap<=t면 theta=cap, -1
 아니면 t=0
 2단계: 잔차 |num - t·a|(곱 뒤 뺄셈, FMA 없음). N,U(a>0)·A·B·L,N(a<0)는 ×2 뒤 <=h, L,X(a>0)·U,X(a<0)는 ×2 없음.
        a>best(엄격)일 때 theta를 바로 쓴다. A(a>0): (x-w) 다음 x, B(a<0): x 다음 I>w면 (x-w). work sparse 4n·coef, dense 3m·coef
 선택 없음: theta=I, scratch 순서(분할 뒤 순서) 최소 ratio(엄격 <, 시작 I). 후보 없으면 pivot=0, -1
 pivot=|val[chosen]|. 재진입이거나 0.1<=pivot면 반환. big=max|val|, 배율 0.1…1e-6, (tol·big)·scale<=pivot면 반환,
 아니면 tol = tol·(big·scale), 재진입 패스로 반복
```

### 2.3 디컴파일과 다른 점(asm)

차선 결합은 C의 ((A+B)+C)+D가 아니라 (A+B)+(C+D)다(`0xf0b42–0xf0b58`). 재진입 허용오차는 C의 (tol·big)·scale이 아니라 tol·(big·scale)이다(`0xf09d9`·`0xf09df`, 판정식은 (tol·big)·scale). 첫 패스 후보 없음의 theta는 `minsd`(cap<I ? cap : I, `0xf0a1f`)다. sparse N/U(a>0)의 cap 미검사는 `0xf024c–0xf026a`, dense의 검사는 `0xf13c3`, 재진입 `12dd49` 분기는 `0xf0a32–0xf0a54`다. 비교는 모두 `comisd`라 NaN이면 진행 조건이 거짓이다.

### 2.4 theta=I 뒤 분기(push C 1458–1590)

`054eef`·`056973` 결과 theta가 I이면, 진입 열이 자유이거나(lb≤`local_1d8` && I≤ub. `local_1d8 = -local_110`(C 602), `local_110 = S+0x370`(C 195)이라 -I로 읽었다. C만이고 참인 쪽은 관측하지 않았다) 방향 0이고 I≤ub일 때 방향을 뒤집어 ratio를 다시 부른다(M3-04c-8). 그 밖은 `0x1c2911`·`0x1c2924`의 거리 검사로 가고 theta=I라 거리 flip이다(04c-2 `pR` 탐색에서 관측). 위치<0·유한 theta는 거리 검사 뒤 `0x1c2948`에서 flip이다.

## 3. 비교 계약

04c-2 판단 모델(`tools/primal_push_decision_check.py` `decide`)을 넓힌다. 바뀌는 점만 적는다.

| 필드·사건 | 규칙 | 출처 |
|---|---|---|
| 재시도 횟수 | 위반 경로에서 게이트가 참이면 정확히 1, 아니면 0 | asm `0x1c258e–0x1c25ae` |
| 재시도 입력 | `retry:enter`의 alpha·basic·width·표시가 `ratio:enter`와 같고 theta·pivot 슬롯이 `054eef` 결과와 같다 | 호출점 인자 |
| 단계 입력 | 재시도가 있으면 단계 판단은 `retry:leave`의 위치·행·theta·pivot을 쓴다 | C 1452–1455 |
| theta=I | 재호출 조건이 참이면 실패(범위 밖), 거짓이면 거리 flip | C 1458–1586 |
| 방향·반전·거리 flip·거부·승인·보류·frame·이탈 상태 | 04c-2와 같다 | 기존 |
| 재시도 결과(자연) | Rust `violated_ratio`가 `ratio:enter` 입력에서 `retry:leave`의 위치·theta·pivot과 비트 일치. work는 `ratio:leave` work에서 시작해 `retry:leave` work와 비트 일치 | 캡처 |
| 재시도 결과(합성) | Rust `retry_ratio`가 원본 직접 호출의 위치·행·theta·pivot·work와 비트 일치. 재진입 사례는 Rust 거부를 따로 센다 | 직접 호출 |

NaN은 허용하지 않는다. I(`1e30`)와 ±0은 비트 그대로 둔다. Threads=1. 원본 재현성은 같은 설정 독립 3회의 캡처 JSON 일치로 확인한다.

판별력: 판단 모델 변조에 게이트 변조 3종(pivot 임계값 1e-4, 위치<0 조건 끔, 재시도 결과 무시)을 더해 저장 자료 전체에 돌린다. Rust 소스 변조는 게이트·`056973` 분기별(차선 합 순차, ×2 유무, sparse cap, 엄격 비교, 대체 시작값 등)로 돌려 생존을 미검증 경계로 적는다.

합격 조건(하나라도 어기면 검사기 0이 아닌 종료):

- 대상 LP 두 순서 각 독립 3회 캡처가 같고, 모든 단계가 위반 경로·재시도 1회이며 판단 모델로 재구성된다.
- 필수 arm(두 순서 합집합): 재시도 결과 `changed`(054eef와 다른 위치·theta)·`same`·`theta-I`, 원인 `distance`·`accept`·`accept-fail`, 이탈 `neg-B-2`·`nonneg-A-1`.
- 대조군(Crossover=0)은 push 사건 0이다.
- 비간섭: (1) 발동: 대상 LP를 04c-2 agent로 두 순서 각 1회 캡처해, 새 agent 캡처의 `retry:*`를 04c-2 필드로 투영하면 같다. (2) 비발동: 04c-2 `violated.lp`를 새 agent로 1회 캡처해 04c-2 `violated-primal-1`과 사건·결과·출력이 같다.
- 합성 직접 호출 3회가 같고 모든 사례가 설계 행·S 변경(재진입 요청 2만 `S+0x294=2`·`S+0x2f8=-1`)에 들어간다.

## 4. 사례와 원본 자료

### 4.1 자연 재시도(실제 원본 실행, 설계한 LP)

입력은 `tools/push_retry_lp.py`가 만드는 블록 대각 LP와 PStart·DStart다(04c-2 형식과 `build`를 그대로 쓴다). 파라미터는 Threads=1, Seed=0, OutputFlag=0, Presolve=0, Method=2, LPWarmStart=2, ScaleFlag=0, Crossover=3(primal-first)·1(dual-first)·0(대조군). 열 계수를 1e-3보다 작게 두어 `054eef` pivot이 게이트에 걸린다.

대상 `bench/oracle/m3_push_retry/retry.lp`(18열×10행)와 `retry.extra.json`:

| 묶음 | 설계 | 노리는 판단 |
|---|---|---|
| W | 04c-2와 같은 영구 `B` 행 | 위반 경로 유지 |
| TF | N 두 행, 계수 1e-4·5e-4, ratio 0.5·0.5+1e-6 | 기울기 0 → t=0 선택 없음 → 최소 ratio. `054eef`는 Harris 안의 큰 \|a\|를 골라 위치가 달라진다(`changed`) |
| TB2 | 등식 행 basic z1∈[0,2e-4]·z2가 하한 아래(B), 계수 5e-4·4e-4 | 기울기 -9e-4, z1 상한 breakpoint 0.6에서 0 이상 → B 두 번째 시도, 이탈 `B`→-2 |
| TA2 | TB2의 거울(상한 위 A) | A 두 번째 시도, 이탈 `A`→-1 |
| TZ | basic slack이 하한(0)인 행, 계수 5e-4 | 기울기 +a → t=0, 2단계 L(×2 없음) |
| TD | 계수 5e-7 | 재시도 pivot<1e-6 → 보류 |
| SR | 04c-2와 같은 차단 행 없는 열(방향 1, ub 유한) | `054eef`·`056973` -1, theta=I 비재호출 → 거리 flip(`theta-I`) |

### 4.2 `056973` 합성 직접 호출(합성)

`tools/push_retry_ratio_oracle.py`. 표시·부호별과 경계 38사례×sparse/dense(역순 사례는 sparse만), 총 74호출을 독립 3회 실행한다. h=1e-6, 허용오차 3e-8은 자연 캡처값이다. theta·pivot 슬롯은 -123, `S+0x2f8`은 7로 두고 호출해 원본이 쓰지 않는 경로를 구분한다. 사례별 설계 행과 S 변경을 검사한다.

- 1단계: 각 표시×부호(A·B·L·N·U·X·F, 허용오차 경계), w=I인 음수 N(a=-1, 그리고 \|a\|>1이라 (x-I)/a가 cap보다 작은 `negN-winf-big`. 뒤의 것은 Rust 변조 생존을 보고 추가, §6), -0 theta.
- ×2 판별: `posN-x2`(×2가 없으면 큰 \|a\| 행), `posL-nox2`·`negU-nox2`(×2를 붙이면 다른 행).
- 동률: t=0 같은 \|a\|(정·역순), 대체 최소 ratio 동률(정·역순).
- cap: `cap-sparse-N`(sparse는 cap 너머 2를 고르고 dense는 -1), `cap-no-cand`(theta=cap), `cap-exit`(sparse N breakpoint가 cap과 같아 t=cap 종료. Rust 변조 생존을 보고 첫 대상 캡처 뒤 추가, §6).
- quickselect: B 5행+L(중간 교차), `lanes`(B 10행+L, 첫 분할이 11원소 벡터 합. 원본 차선 순서면 t=7, 순차 합이면 t=8 — 설계 보조 모델로 찾았다).
- 재진입: `reentry-pick`(강화 허용오차로 큰 행), `reentry-keep`(factor 갱신 3 → 1차 선택 유지), `reentry-refactor`(갱신 20 → 요청 2, -1, theta·pivot은 1차 값).

### 4.3 대조군·비간섭

대조군은 대상 LP Crossover=0 3회. 비간섭은 §3 합격 조건의 두 가지다.

### 4.4 탐색(합격 근거 아님)

`bench/oracle/m3_push_retry/history/`. 계약 전에 묶음 단독·결합 LP를 새 agent로 돌렸다.

- 첫 TB2·TA2는 basic z를 bound에서 시작해 crash가 등식 slack(폭 0)을 골랐고 LP가 infeasible(상태 3)이었다. z를 내부에서 시작하고 완충 열 v를 더하자 z가 basic(폭 2e-4·10)이 되고 상태 2가 됐다.
- 결합 LP(대상과 같은 묶음)는 두 순서 모두 8단계가 재시도 경로였다. TF는 `054eef` 행 5(0.500001)→`056973` 행 6(0.5), TB2·TA2는 0.2→0.6, TD는 세 번 보류, SR은 -1→거리 flip. 판단 모델이 모든 단계를 재구성했다.
- 합성 탐색 1회(`ratio-explore.json`): 70사례가 모두 설계 행에 들어갔다.

## 5. 재실행 명령

worktree 루트에서 실행한다. Gurobi는 `tools/gurun.sh`로 한 프로세스만 쓴다. 캡처 도구는 기존 파일을 덮어쓰지 않는다.

```bash
tools/push_retry_runs.sh                                  # 합성 3회, 대상 primal·dual·control 각 3회, 비간섭 3회
python3 tools/push_retry_ratio_oracle.py --check bench/oracle/m3_push_retry/ratio-{1,2,3}.json --selftest
python3 tools/primal_push_decision_check.py --self-test --mutation <새 경로>.json
cargo test -p rsolver-lp --test push_decision --test push_ratio && python3 tools/primal_push_decision_rust.py --compare && python3 tools/push_retry_rust.py --compare
```

## 6. 실행 결과

- 환경: x86-64 Linux(WSL2), node·Frida는 04c-2와 같다. 원본 ELF 해시는 §2. 도구·입력·캡처·기록 해시는 [manifest](../../bench/oracle/m3_push_retry/manifest.json)에 있다. 대상 캡처는 계약 commit `db37218` 뒤에 `tools/push_retry_runs.sh`로 실행했다. Gurobi는 `tools/gurun.sh`로 다른 세션과 순차 실행했다.
- 원본 실행: 합성 74호출×3회(사건 필드 9개 일치), 대상 LP primal-first 3회(80사건), dual-first 3회(78사건), 대조군 3회(0사건, 상태 2), 비간섭 3회(대상 LP 04c-2 agent 두 순서, 04c-2 `violated.lp` 새 agent 1회). 순서별 3회는 사건·입력·시작 벡터·결과·출력 줄 전체가 같다.
- 합성 사례: 74호출이 모두 설계 행과 S 변경에 들어갔다. 재진입 요청 2 사례만 `S+0x294=2`·`S+0x2f8=-1`이고 theta·pivot은 1차 패스 값(0.1, 1e-7) 그대로다. `cap-sparse-N`은 sparse가 cap(1) 너머 theta 2를, dense는 -1을 돌려 asm의 sparse 미검사를 원본이 확인했다. `lanes`는 원본 t=7(행 7)이다. `negL-t0`·`negU-at-bound`·`negX`의 theta는 -0이다.
- 비간섭(검사기가 매번 확인): 새 agent 대상 캡처의 `retry:*`를 04c-2 필드로 투영하면 04c-2 agent 캡처와 사건·결과·출력이 같다(두 순서). 04c-2 `violated.lp` 새 agent 캡처는 04c-2 `violated-primal-1`과 같다(재시도 0).
- 판단 재구성: 두 순서 각 8단계, 모두 위반 경로이며 재시도 1회다. 같은 단계 순서와 진입 변수다. 판단 모델이 방향·alpha 반전·게이트·재시도 입력·단계 종류·theta·frame·이탈 상태를 원본과 같게 재구성했다. 04b-1·face5·arm·04c-2 위반 128단계도 함께 다시 통과했다.

| 단계 | 묶음 | `054eef` → `056973` | 단계 | 재시도 태그 |
|---|---|---|---|---|
| 1 | TZ | 행 8 theta 0 → 같음 | pivot, 이탈 `L`→-1 | same |
| 2·7·8 | TD | pivot 5e-7 → 같음 | 보류(accept-fail) | same |
| 3 | SR | -1 → -1, theta=I(방향 1, ub 유한) | 거리 flip | theta-I |
| 4 | TF | 행 5 theta 0.500001 → 행 6 theta 0.5 | pivot, 이탈 `N`→-1 | changed |
| 5 | TB2 | theta 0.2 → 0.6(둘째 breakpoint) | pivot, 이탈 `B`(a<0)→**-2** | changed |
| 6 | TA2 | theta 0.2 → 0.6 | pivot, 이탈 `A`(a≥0)→**-1** | changed |

- 검사기 변조(`primal_push_decision_check.py --self-test`, 종료 0): 04c-2까지의 65종에 재시도 판단 변조 8종(재시도 사건 삭제, 게이트 거짓인데 재시도, 재시도 theta, B 두 번째 breakpoint, 재시도 입력·진입 슬롯, 자유 열로 재호출 조건 참, 054eef 결과로 단계 진행), 재시도 캡처 변조 12종(해시·출처·설정·사건·x87·batch·frame·값·중복 실행·재시도 입력 누락), 비간섭 변조 3종, 필수 arm 누락 5종을 더한 93종을 모두 거부했다.
- 판단 모델 변조([기록](../../bench/oracle/m3_push_retry/model-mutation.json)): 04c-2의 43종과 게이트 3종, 총 46종 중 39종 거부(frame을 지운 판단만 실행도 같다). 생존 7종은 04c-2의 6종과 `gate-no-neg`다. `054eef`는 -1을 돌릴 때 pivot도 0으로 쓰므로(C·합성 캡처) 위치<0 절은 \|pivot\|<0.001 절에 포함돼 구조상 생존이다.
- Rust 재생(구현):
  - `056973`: `python3 tools/push_retry_rust.py --compare` 종료 0. 합성 68호출과 자연 16호출(두 순서 각 8)이 위치·행·theta·pivot·work 비트 일치했다. 자연 work는 `retry:enter`의 work(054eef 뒤)와 `S+0x148`에서 이어 계산했다. 재진입 6호출은 Rust가 설계대로 `Reentry`로 거부했다(입력 `bench/oracle/m3_push_retry/ratio_input.txt`).
  - 판단·연결: `python3 tools/primal_push_decision_rust.py --compare` 종료 0. 153단계(04c-2까지 137 + 재시도 16, 고유 7)가 방향·반전 alpha·단계 종류·theta·frame·이탈 상태 비트 일치했고, V 경로 연결(`violated_ratio` = 054eef → 게이트 → 056973)도 153단계 모두 캡처 결과와 같았다. SR의 theta=I 비재호출 단계는 Rust `push_step`이 거리 flip으로 판단했다(전에는 `Recall`로 거부).
- Rust 소스 변조:
  - `056973`([기록](../../bench/oracle/m3_push_retry/ratio-rust-mutation.json)): 분기별 22종 중 21종 거부. 차선 합 순차·결합 순서·중간 pivot 분할은 `lanes`, sparse cap은 `cap-sparse-N`, cap 종료 비교는 `cap-exit`, w=I 검사는 `negN-winf-big`, ×2 유무는 `posN-x2`·`posL-nox2`가 거부했다. 생존 `posA-order`(A 두 breakpoint의 scratch 순서)는 두 항목이 같은 위치라 선택은 같고 분할 경로·합 순서의 반올림에만 영향이 있어, 판별 사례가 없다(미검증 경계).
  - 판단·게이트([기록](../../bench/oracle/m3_push_retry/rust-mutation.json)): 04c-2의 48종에 게이트 3종·재호출 조건 3종을 더한 54종(미적용 `reject-arm-current` 1 별도) 중 43종 거부, 거부되지 않은 11종은 생존 9종과 연결 검사로만 검출한 2종이다. 04c-2의 `recall-no-theta`는 새 코드에 맞춰 대상 문자열을 바꿨다. `recall-no-theta`는 04c-2에서 생존이었으나 SR 단계가 거부했다. `gate-off`·`gate-pivot-1e-4`는 단계 판단 입력이 캡처값이라 판단 대조는 통과하지만 연결 검사가 6단계 불일치로 검출한다(`--compare` 종료 3). 생존은 04c-2의 6종(`leave-fixed-lt`, `leave-neg-default-2`, `leave-X-1`, `leave-U-le`, `sign-zero-neg`, `v-sigma-ge0`), `gate-no-neg`(구조상), `recall-free-only`·`recall-up-any-dir`(재호출 조건이 참인 사례는 04c-8 범위)다.
  - 위 `056973` 판정은 세 번째 실행이다. 첫 `056973` 변조 실행에서 `hoare-le`(왼쪽 스캔 `>=`)가 분할 루프를 끝내지 못해 테스트가 46분 멈췄다. 테스트 프로세스를 끝내 기록을 폐기하고, 종료하는 중간 pivot 변조 `hoare-mid`로 바꾸고 도구에 600초 제한을 넣어 다시 돌렸다. 같은 실행에서 `gate-off`는 게이트를 거치지 않는 직접 호출 도구에 있어 판단 재생 변조로 옮겼고, `cap-exit` 생존을 보고 합성 사례 `cap-exit`를 더해 합성을 다시 3회 캡처했다(이전 70사례 3회는 `history/ratio-toolv1-*`, 공통 70사례의 사건이 같다). 두 번째 실행의 생존 `negLN-inf`는 w=I 음수 N 사례가 a=-1뿐이라 (x-I)/a=I가 cap에서 우연히 걸러진 것이어서, a=-2 사례 `negN-winf-big`을 더해 합성을 다시 3회 캡처했다(72사례 3회는 `history/ratio-toolv2-*`).
- 회귀: 공유 꼬리를 `small_pivot`으로 나눈 뒤 `push_violated_rust.py --compare`(55사례)·`primal_push_ratio_rust.py --compare`(자연 18·합성 29) 종료 0, 두 도구의 변조 대상 문자열을 새 코드에 맞췄다. 리팩터 뒤 04c `--mutate-rust` 14종(10 거부)·04c-2 `054eef` 19종(17 거부)의 분류가 기존 기록과 같다([04c](../../bench/oracle/m3_push_retry/ratio04c-rust-mutation-rerun.json), [04c-2](../../bench/oracle/m3_push_retry/ratio04c2-rust-mutation-rerun.json)). `primal_push_decision_check.py --old`, `push_slot_reach.py`, `push_infeasible_ratio_oracle.py --check … --selftest` 종료 0. `cargo test --release --workspace` 통과(119개), `cargo clippy --workspace --all-targets -- -D warnings` 종료 0. `git diff --check` 깨끗함.
- 실행하지 않은 검사: Python 확장·pytest. Rust 변경은 호출자가 없는 `crossover` 모듈과 그 테스트뿐이고 Python 경계를 바꾸지 않았다.
- 04b-3·04c-2 manifest의 검사기·재생 도구·`input.txt` 해시는 이번 변경 전 값이다(캡처 도구·agent 해시만 검사에 쓰이며 그대로다).
- 기준 자료 갱신 순서: `tools/push_retry_runs.sh` → `python3 tools/push_retry_rust.py --write`·`python3 tools/primal_push_decision_rust.py --write` → 변조 기록 → `python3 tools/push_retry_manifest.py`(검사 전).

## 7. 판정과 인수인계

- 작업 수행: 완료.
- 오라클 판정: 계약 범위 **통과**. 게이트 뒤 `056973`의 결과와 그 뒤 거리 flip(theta=I 비재호출 포함)·승인·보류·이탈 `B`→-2·`A`→-1을 원본 순서별 독립 3회로 관측했다. `056973` 1단계·t=0·quickselect·차선 합·2단계 ×2 유무·두 번 시도·동률·대체 최소 ratio·cap·재진입·`12dd49` 요청 2를 합성 직접 호출 3회로 고정했다.
- 구현 판정: 계약 범위 **통과**. Rust `retry_ratio`·`violated_ratio`가 합성 68·자연 16호출에서, `push_step`이 재시도 16단계와 기존 137단계에서 원본과 비트 일치했다. 재진입 6호출은 설계대로 거부한다.
- 미검증: `056973` 재진입 패스의 Rust(원본 합성 출력만 있음), `gate-no-neg`(구조상), 위치<0·유한 theta의 `push_step` flip(재진입 요청 2 뒤에만 옴), 자연 실행의 dense 열과 quickselect 8원소 이상(합성만), 재시도 뒤 거부(관측하지 않음), `S+0x3b0`≠0. NaN 처리 차이(asm `minsd`·`maxsd`)는 사례가 없다.
- 후속: **M3-04c-8**(새 행) theta=I 반대 방향 재호출(위반·feasible). Rust `PushEntry::recall`이 조건을 계산하고 `push_step`이 `Recall`로 거부한다. 재호출 뒤 방향은 바뀌지만 거부 arm은 방향식 직후의 `r13b`로 고르므로 오래 생존한 `reject-arm-current` 변조를 처음 판별할 수 있다. feasible 재호출이 -1이면 `S+0x2f8`은 이전 값을 유지한다(C 1549–1554). M3-04c-5(재진입)는 그대로다.
- branch `work/m3-04c-7`, worktree `../rsolver-m3-04c-7`. 결과·인수인계·진행·로드맵은 이 branch에서 갱신해 `--no-ff`로 병합한다.
- 교훈: `docs/lessons/original.md`(`056973` ABI·sparse cap·차선 합, 재진입 no-cand, 게이트 위치<0 중복과 자연 설계), `docs/lessons/process.md`(asm 명세 subagent와 임시 모델로 판별 사례 탐색, 계약 전 분할), `docs/lessons/tooling.md`(hrun 즉시 124).
