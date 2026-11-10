# M3-04b-3 primal push 거부·보류·미관측 이탈 arm 오라클

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다. §1–§5는 원본 대상 캡처 전에 고정했다. 탐색 실행은 §4 끝에 따로 적고 합격 근거로 쓰지 않는다.

## 1. 범위와 주장

- 작업 종류: 오라클 구축(+ 기존 Rust 판단의 저장 재생 확장). 기준선 main `e476024`(M3-04c-6 병합), branch `work/m3-04b-3`, worktree `../rsolver-m3-04b-3`, 미커밋 선행 변경 없음.
- 질문: double primal push `134a87`의 feasible basis 한 단계에서 04b-2·04c-6이 관측하지 못한 판단 arm이 실제 원본 실행에서 나오는가. 나오면 04b-2 판단 모델과 04c-6 Rust가 그 단계를 원본과 같게 재구성하는가.
- 대상 arm(각각 원본 사건으로 관측해야 한다):
  1. 거부 발동 두 arm: 방향 0(`r < -100h && dU < 100h`), 방향 1(`r > 100h && dL < 100h`). theta=I flip.
  2. pivot 보류(`pivot < 1e-6`). 이번에는 보류 지점 `0x1c3b7b`에 양성 사건을 붙인다(04b-2는 사건 부재로 추정했다).
  3. 이탈 상태: 고정폭 -4, `U`(a<0) -2, `U`(a≥0) -1·-2, `L`(a<0) -2·-1.
  4. 방향식: 첫 절(bound)만으로 만든 방향 1, 방향 1 거리 flip, 진입 slack 열(dL=slack, dU=I).
  5. 04c-6 생존 변조 경계: 방향식 계수 10·`<`·`-(r*dL)` 부호, 거리 flip `1e-10`·`<=`, 거부 arm 교환, pivot `1e-6`, 2h(K) 판단.
- 구조상 도달 불가로 분류할 것(실행 탐색 대상이 아님, §2.2 근거): 이탈 `A`·`B` arm과 a<0 기본 arm, `X` arm(고정폭에 가려짐), ±0 pivot arm, 거부 arm을 현재 `S+0x328`로 고르는 변조(재호출 경로에서만 다름).
- 입력: 원본 공개 API만 쓴다. 설계한 블록 대각 LP와 `PStart`·`DStart`(LPWarmStart=2, Method=2)로 crossover를 시작 벡터 경로(04a-1의 mode 1)로 연다. 내부 상태 주입·호출 대체는 없다. 따라서 "실제 원본 실행"이지만 LP와 시작 벡터는 arm을 노려 만든 것이다.
- 제외: 위반 basis 방향식·infeasible ratio(`A`·`B` arm 포함), theta=I 재호출, batch, x87, 이탈 뒤 basic 값·rhs 보정·factor, push 연결, work. `U`(a≥0)의 `<`와 `<=` 경계(|a|t==2h 정확히)는 1회 설계 시도 뒤 미검증으로 둔다(§4).

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c`. 판단식·세 슬롯은 [04b-2 §2](primal_push_decision_oracle.md#2-원본과-상태-경계), asm 비교 형태는 [04c-6 §2](primal_push_decision_rust.md#2-원본과-상태-경계)를 따른다.
- 시작 경계: `supply:leave`(candidate≥0). 종료 경계: `step:enter`(flip·pivot), 새 `step:defer`(보류), pivot이면 `step:applied`.

### 2.1 보류 사건 훅(`0x1c3b7b`)

objdump(`--start-address=0x1c0070 --stop-address=0x1c5310`)로 확인했다. 이 주소로 오는 명령은 승인 검사의 `ja` 두 개뿐이다: `0x1c2a99`(`1e-6 > pivot`), `0x1c2aaf`(`-1e-5 > theta*pivot`). 직전 명령 `0x1c3b79`는 무조건 `jmp 0x1c3b9f`라 fall-through가 없다. 지점의 첫 명령은 보류 목록 개수 `0x2c(%rsp)`를 읽고, `0x2fc(%r14)`(진입 j)를 목록에 넣은 뒤 실패 횟수를 올리고 공급자 `25615c`를 다시 부른다. r14는 workspace S다. 새 agent `tools/primal_push_arm_agent.js`는 04b-2 agent에 이 지점의 읽기 훅 하나만 더했다. 사건 `step:defer`는 selected·theta(`0x1eb8`)·pivot(`0x1ef8`)·보류 개수·상태·frame 7값(04b-2와 같은 슬롯)을 기록한다.

### 2.2 구조상 도달 불가 arm의 근거

- `A`·`B`: 표시를 쓰는 곳은 다섯 군데이고 모두 위반 수를 함께 맞춘다. 초기 분류는 `uVar25`를 0에서 세어(C 798–861줄) `local_1b0`에 둔다(868·904줄). rhs 보정 루프는 이전 표시가 `A`·`B`면 1을 빼고 새로 `A`·`B`면 1을 더한다(1688–1717줄). 재분해 뒤 재분류는 `local_40`을 0으로 두고(1905·1912줄) `A`·`B`마다 올린 뒤 `local_1b0`에 넣는다(1940–1978줄). 이탈 행 재분류는 2165·2202·2217줄, 마지막 재분류는 `uVar38`로 같은 일을 한다(2342–2405줄). 방향식과 feasible ratio `055f68`은 `local_1b0 == 0`일 때만 간다(1225·1547줄). 검사기는 이를 캡처로도 확인한다. feasible `ratio:enter`의 표시에 `A`·`B`가 있으면 실패다(04b-1·face5·arms 전 단계 통과). 따라서 feasible 경로의 이탈 행 표시는 `N`·`U`·`L`·`X`·`F`뿐이다. `A`·`B` arm은 infeasible ratio 경로에만 있다.
- a<0 기본 arm(`N`·`U`·`B`·`X`·`L` 밖): 남는 표시는 `A`·`F`다. `F`(자유 basic)는 feasible ratio에서 양수 alpha가 제외되고 음수 alpha는 폭<I 조건으로 제외된다(04c `feasible_ratio` 복원, [04c §2](primal_push_ratio_rust.md#2-원본과-상태-경계)). 그래서 feasible 경로에서는 도달하지 않는다.
- `X` arm: `X` 표시는 폭이 정확히 `0.0`일 때만 붙는다(C 847–848줄, 2209–2210줄). 이탈 상태 첫 검사 `ub-lb <= S+0x3b0`이 먼저이고, 모든 캡처에서 `S+0x3b0`은 `+0.0`이다. 그러면 `X`는 항상 고정폭 arm으로 가고 둘 다 -4다. `leave-X-1`·`leave-fixed-lt` 단독 변조는 서로를 가리므로 둘을 함께 바꾼 이중 변조로 판별을 확인한다. `S+0x3b0`이 0이 아닌 설정은 미확인이다.
- ±0 pivot: ratio가 고르는 pivot은 abs>0이라 a=±0인 이탈은 없다(04c ratio의 pivot 임계값).
- 거부 arm을 현재 `S+0x328`로 고르는 변조: 방향식 직후 `S+0x328`과 `r13b`는 같고, 둘이 갈리는 곳은 재호출 경로(`0x1c49e2`)뿐이다([04b-2 §2.2](primal_push_decision_oracle.md#22-판단식asm-확인)).

## 3. 비교 계약

04b-2 판단 모델(`tools/primal_push_decision_check.py` `decide`)과 비교 규칙을 그대로 쓴다. 바뀌는 점만 적는다.

| 필드·사건 | 규칙 | 출처 |
|---|---|---|
| 보류 | `ratio:leave` 다음 사건이 `step:defer`여야 한다. 사건 부재는 실패다 | 새 훅 |
| 보류 theta·pivot | `ratio:leave`의 theta·pivot과 f64 비트 일치 | 새 훅 |
| 보류 frame | r·dL·dU·상수 7값이 모델 계산과 f64 비트 일치(04b-2와 같은 슬롯) | 새 훅 |
| 시작 벡터 | 캡처의 `starts`(PStart·DStart)가 manifest와 같고 3회 같음 | 캡처 도구 |
| 판단·frame·이탈 상태 | 04b-2 §3과 같음 | 기존 |

NaN은 허용하지 않는다. I(`1e30`)와 ±0은 비트 그대로 둔다. Threads=1, 시간 분기 없음. 원본 재현성은 같은 설정 독립 3회의 캡처 JSON(사건·입력·결과·출력) 일치로 확인한다.

판별력: 04b-2 모델 변조 20종에 새 arm별 변조를 더해 저장 자료 전체(04b-1·face5·이번 LP)에 돌린다. frame을 뺀 "판단만" 실행에서 거부되는지 따로 센다. 상수 값 변조가 frame 대조로만 거부되는 경우는 판단 판별로 세지 않는다.

## 4. 사례와 원본 자료

입력: `tools/push_arm_lp.py`가 만드는 `bench/oracle/m3_push_arms/arms.lp`(42열×25행)와 `arms.extra.json`(ScaleFlag=0, PStart, DStart). 묶음은 서로 변수·행을 나누므로 alpha와 r이 묶음 안에서만 정해진다. 파라미터는 Threads=1, Seed=0, OutputFlag=0, Presolve=0, Method=2, LPWarmStart=2, ScaleFlag=0, Crossover=3(primal-first)·1(dual-first)·0(대조군).

| 묶음 | 설계 | 노리는 판단 |
|---|---|---|
| A | xa∈[0,10], c=-1, 시작 ub-5e-5. 행 slack 4.96e-5 | 거부 arm 0(dU<100h, theta<dU). flip 뒤 위반 4e-7은 허용오차 안 |
| A2 | 같은 꼴, dU=5e-3, slack 4.9e-3 | dU>100h라 승인 pivot(거부 상수 1e4h 판별) |
| B | xb, c=+1, 시작 lb+5e-5. 행 slack 4.96e-5 | 거부 arm 1(방향 1, both 절) |
| C | xc, c=0, 시작 lb+5e-5. 차단 행 slack 1 | 첫 절만의 방향 1(r=0) + 방향 1 거리 flip |
| E8 | c=-1e-3, 시작 lb+5e-5 | `-(r*dL)`=5e-8: e(1e-8) 이상이라 방향 0(0.1h 변조 판별) |
| H | xh 열 [1e-7, 1], 첫 행 slack 0 | pivot 1e-7 → 보류 |
| U | xu가 ub-5e-7에서 basic(U), 다음 wu 방향 1 | U(a<0) → -2 |
| P | U와 같고 wp 방향 0 | U(a≥0), abs(a)t≈10 → -1 |
| L | xl이 lb+5e-7에서 basic(L), 다음 wl 방향 0 | L(a<0), abs(a)t≈10 → -2 |
| K | 폭 2.4e-6, U basic b=1.5e-6 | U(a≥0), abs(a)t=1.5e-6∈[h,2h) → -2(2h 판단 판별) |
| K2 | 폭 2.4e-6, c=5e-5, L basic b=6e-7 | L(a<0), abs(a)t=1.8e-6<2h → -1 |
| X | 고정 xm과 pm(계수 1e-4)의 등식 행. 등식 slack이 basic | 고정폭 이탈 -4, pivot 1e-4∈[1e-6,1e-3) |
| SD | DStart(rt)=-1 | dual-first에서 slack 진입(방향 1 bound, 거리 flip) |
| T·T2 | dU/dL=9.5, 정확히 10 | 방향식 계수 10과 `<` |
| T3 | c=-1, dL=1, dU=20 | `-(r*dL)` 부호 |
| T4·T5 | theta=dU-5e-11, 정확히 dU-1e-10 | 거리 flip `1e-10`과 `<=` |

선택 기준: feasible ratio만, batch 0, x87 없음, 재호출 없음. 모든 단계가 판단 모델로 재구성돼야 한다.

### 4.1 탐색(합격 근거 아님)

이 계약 전에 단독 묶음과 결합 LP를 탐색 캡처로 돌렸다(`bench/oracle/m3_push_arms/history/`). 얻은 사실은 다음과 같다.

- 시작값이 bound에서 5e-7이면 push 전에 bound로 붙는다. 5e-5는 붙지 않는다.
- 차단 행이 없으면 ratio가 -1을 돌려 재호출 경로로 간다(범위 밖). 그래서 모든 묶음에 차단 행을 두었다.
- 등식 행에 내부 구조 변수가 있으면 crash basis가 그 변수를 고른다. 계수 0.1은 basic이 되고 0.01·1e-3·1e-4는 등식 slack이 남는다(`history/tX70`·`tX71`·`tX72`, 1e-4는 대상 LP).
- 단독 SD 묶음의 slack 진입은 primal-first에서 나오지 않았다. dual-first에서는 DStart(rt)=-1일 때 나왔고, 같은 묶음을 DStart=0으로 돌린 대조(`history/tSD0`)에서는 slack이 basic으로 남았다(`history/tSD`).
- 근거리 시작값만 준 첫 LP(t1)는 거부 flip 뒤 위반이 허용오차를 넘어 infeasible ratio로 갔다. 재호출로 간 묶음(tD, tSL)과 함께 후속 작업의 시작 사례로 남긴다.
- `U`(a≥0)의 `abs(a)*t == 2h` 정확 경계: rhs·시작값을 ulp 단위로 네 번 바꿨으나 theta가 2h의 ±1ulp에 떨어졌다. 미검증으로 둔다.

## 5. 재실행 명령

worktree 루트에서 실행한다. Gurobi는 한 프로세스만 쓰고, 캡처 도구는 기존 파일을 덮어쓰지 않는다.

```bash
D=bench/oracle/m3_push_arms
python3 tools/push_arm_lp.py $D/arms                  # 입력 재생성(기존 파일과 다르면 거부)
X="$(cat $D/arms.extra.json)"
for k in primal dual control; do for i in 1 2 3; do
  tools/gurun.sh node tools/primal_push_arm_capture.js $k $D/arms.lp $D/arms-$k-$i.json "$X"
done; done
tools/gurun.sh node tools/primal_push_arm_capture.js primal bench/oracle/m3_push_decision/face5.lp $D/face5-primal-noninterference.json
python3 tools/primal_push_decision_check.py --self-test --mutation <새 경로>.json   # 04b-1·face5·arms 전체
cargo test -p rsolver-lp --test push_decision && python3 tools/primal_push_decision_rust.py --compare
```

## 6. 실행 결과

- 환경: x86-64 Linux(WSL2), node·Frida는 04b-2와 같다. 원본 ELF 해시는 §2. 도구·입력·캡처·탐색 해시는 [manifest](../../bench/oracle/m3_push_arms/manifest.json)에 있다. 대상 캡처는 계약 commit `30b85d2` 뒤에 실행했다.
- 원본 실행: arm LP primal-first 3회(187사건), dual-first 3회(185사건), Crossover=0 대조군 3회(0사건), face5 비간섭 1회(109사건). 회당 1초 미만이다. 순서별 3회는 사건·입력·시작 벡터·결과·출력 줄 전체가 같다. 대조군은 push 사건이 없고 status 2다. 탐색은 단독 묶음·결합 LP 수십 회다(history는 일부만 보존, §4.1).
- 비간섭(검사기가 매번 확인): (1) 훅이 발동하지 않는 face5 primal에서 새 agent 캡처가 04b-2 `face5-primal-1.json`과 사건·결과·출력이 같다. (2) 훅이 발동하는 arm LP에서 두 순서 대상 캡처의 `step:defer`(각 3건)를 빼면 04b-2 agent로 찍은 탐색 캡처(`history/arms-probe-{primal,dual}.json`)와 사건·결과·출력이 같다. 탐색 캡처의 agent 해시가 04b-2 agent와 같음도 검사한다.
- 판단 재구성: 두 순서 각 25단계, 모두 feasible ratio·batch 0·x87 없음·재호출 없음. 판단 모델이 방향·alpha 반전·단계 종류·theta·frame 7값·이탈 상태를 모두 원본과 같게 재구성했다. 04b-1·face5 40단계도 함께 다시 통과했다(`--old`·전체 검사).
- 관측한 대상 arm(순서별 3회 모두, 단계 번호는 primal/dual):

| arm | 단계·변수 | 비고 |
|---|---|---|
| 거부 방향 0 | 3/3 `xa` | theta=I flip, 뒤 위반 4e-7은 허용오차 안 |
| 거부 방향 1 | 5/5 `xb` | both 절 |
| pivot 보류 | 17·19·25 / 같음, `xh` | 양성 사건 `step:defer`. pivot 1e-7 < 1e-6. 같은 입력의 재시도라 고유 1건. `theta*pivot < -1e-5` 쪽은 미관측 |
| 고정폭 -4 | 18/18 `pm`(등식 slack 이탈) | pivot 1e-4 |
| `U`(a<0) -2 | 24/24 `wu` | |
| `U`(a≥0) -1, -2 | 22 `wp`, 20 `wK` | -2는 abs(a)t=1.5e-6 |
| `L`(a<0) -2, -1 | 23 `wl`, 21 `wK2` | -1은 abs(a)t=1.8e-6 |
| 첫 절만의 방향 1 | 11·13 / 7·13·14 | 모두 slack 열(dU=I). 구조 열의 첫 절 참은 이번 대상에 없다 |
| 방향 1 거리 flip | 13 / 7·13 | slack `rT`(두 순서), `rt`(dual) |
| 진입 slack | 11·13 / 7·13·14 | 두 순서 모두 |

- 설계와 달라진 점: 결합 LP에서는 C(`xc`)·T(`xT`)의 구조 열이 crash basis에 들어가 push되지 않았다. 대신 그 행 slack(`cc`·`rT`)이 진입해 첫 절 방향 1·방향 1 거리 flip·진입 slack을 primal에서도 만들었다. SD의 `rt` slack 진입은 dual에서만이다. 방향식 계수 10·`<` 경계는 T 대신 T2(`xT2`, dU=10dL)가 판별했다. 대상 arm은 모두 관측해 범위는 줄지 않았다.
- 검사기 변조(`--self-test`, 종료 0, 43종): 04b-2 캡처 변조 11종, arm 캡처 변조 13종(해시·출처·설정·사건·x87·batch·frame·값·중복 실행·시작 벡터·보류 frame 누락), 대조군 사건 추가, arm LP 비간섭 사건 변조, 판단 변조 14종(arms-dual에 적용, A·B 표시 추가 포함), 보류 사건 삭제·pivot·frame 변조를 모두 거부했다. 보류 사건을 지우면 "사건 없음"으로 실패한다.
- 판단 모델 변조([기록](../../bench/oracle/m3_push_arms/model-mutation.json)): 04b-2의 20종과 새 arm별 12종, 총 32종. frame을 지운 "판단만" 실행에서 27종을 거부했다(full과 같은 수). 04b-2에서 상수 값으로만 거부되던 4종(e·거부 상수 2종·2h)도 이번엔 판단으로 거부된다(e는 E8, 10h는 K2, 1e4h는 A2, 2h는 K).
  - 생존 5종은 모두 §2.2 분류다: `reject-arm-current`(재호출만), `leave-fixed-lt`·`leave-X-1`(서로 가림, 이중 변조 `leave-fixed-lt+X-1`은 거부), `leave-neg-default-2`(`A`·`F`만), `leave-U-le`(정확 경계 미달성).
- Rust 재생: `tools/primal_push_decision_rust.py`에 arm 두 순서를 `arms` 역할로 더했다. 입력 `bench/oracle/m3_push_decision_rust/input.txt` 99단계(본문 SHA-256 `a507793d…`, 파일 첫 줄). `cargo test -p rsolver-lp --test push_decision && python3 tools/primal_push_decision_rust.py --compare` 종료 0. arm 50단계(고유 입력 43) 모두 방향·반전 alpha·단계 종류·theta(보류 포함)·frame·이탈 상태가 원본과 비트 일치했다. 집계는 방향 1 13, pivot 33, flip 11(거부 4 포함), 보류 6, frame 50이다. 연결 검사(Rust 반전 열의 `feasible_ratio`)도 99단계 모두 일치했다. pivot 1e-7 열도 재진입 없이 같은 위치를 골랐다. 기존 40+9단계 결과는 그대로다. `crates/lp`는 바꾸지 않았다.
- Rust 소스 변조([기록](../../bench/oracle/m3_push_arms/rust-mutation.json)): 04c-6의 24종에 arm별 12종을 더한 36종 중 30종을 거부했다(04c-6은 24종 중 9). 생존 6종: `leave-fixed-lt`·`leave-X-1`·`leave-neg-default-2`·`leave-U-le`(위와 같음), `sign-zero-neg`(±0 pivot 없음), `recall-no-theta`(재호출 경로, 범위 밖). `reject-arm-current`는 04c-6처럼 옮기지 않았다. 거부 30종 중 4종(`dir-e-0.1h`·`reject-10h`·`reject-1e4h`·`leave-k-h`)은 비교가 첫 차이에서 멈춰 face5 frame 상수로 기록됐다. 이 넷의 판단 수준 판별은 Python 모델의 판단만 실행(위)에서만 보였고 Rust에서는 따로 확인하지 않았다.
- 회귀: `primal_push_decision_check.py --old`, `push_slot_reach.py`, `primal_push_decision_rust.py --self-test`(변조 13종 거부), `cargo test -p rsolver-lp --test push_ratio && primal_push_ratio_rust.py --compare`(04c 47사례) 종료 0. `git diff --check main -- tools docs` 깨끗함.
- 실행하지 않은 검사: `cargo test --release --workspace`·clippy·Python 확장. Rust·Python 소스를 바꾸지 않았고 바뀐 것은 재생 입력 텍스트와 Python 도구뿐이다. 재생 테스트(`push_decision`)는 위에서 돌렸다.
- 기준 자료 갱신: manifest는 `python3 tools/push_arm_manifest.py`로 다시 쓴다(캡처·변조 기록 뒤, 검사 전). 04b-2 manifest의 검사기 해시와 04c-6 기록의 입력·재생 도구 해시는 이번 변경 전 값이다. 해당 문서에 이 기록으로 가는 안내를 더했다.
- 해시: `tools/primal_push_arm_agent.js` `3a066dda…`, `tools/primal_push_arm_capture.js` `049438a5…`, `tools/push_arm_lp.py` `b37ec3e4…`, `arms.lp` `b05832f6…`, `arms.extra.json` `7843373b…`(전체 값은 manifest).

## 7. 판정과 인수인계

- 작업 수행: 완료.
- 오라클 판정: 계약 범위 **통과**. 대상 arm 전부(거부 두 arm, 양성 사건 보류, 고정폭, `U`(a<0)·`U`(a≥0) 두 결과, `L`(a<0) 두 결과, 첫 절 방향 1, 방향 1 거리 flip, 진입 slack)를 원본 순서별 독립 3회로 관측했다. 04b-2 판단 모델과 04c-6 Rust가 그 단계를 원본과 같게 재구성했다(Rust: 50단계, 고유 43).
- 구조상 도달 불가(근거 §2.2, 실행 탐색 대상 아님): 이탈 `A`·`B` arm과 a<0 기본 arm(infeasible ratio 경로 전용), `X` arm(S+0x3b0=0에서 고정폭에 가려짐), ±0 pivot, 재호출에서만 다른 거부 arm 선택.
- 미검증: `U`(a≥0)의 `abs(a)t == 2h` 정확 경계(`<`/`<=`), 보류의 `theta*pivot < -1e-5` 쪽, 구조 열의 첫 절 참(slack 열만 관측), `S+0x3b0`이 0이 아닌 설정, 보류 뒤 동작(보류 목록·재시도 순서). 위반 basis·infeasible ratio·재호출·batch·x87·pivot 적용·push 연결·work는 범위 밖이다.
- 후속: infeasible ratio·위반 basis 방향식(`A`·`B` arm 포함)과 재호출 경로는 M3-04c-2(스프린트 목록에 새 행, 시작 사례는 `history/t1`·`tD`·`tSL`). M3-04c-5(재진입)는 그대로다. push 한 단계 연결은 pivot 적용 구현 뒤다.
- branch `work/m3-04b-3`, worktree `../rsolver-m3-04b-3`. 결과·인수인계·진행·로드맵은 이 branch에서 갱신해 `--no-ff`로 병합한다.
- 교훈: `docs/lessons/original.md`(시작값 붙임, crash와 등식 행, A·B 표시와 위반 수, X와 고정폭), `docs/lessons/tooling.md`(PStart·DStart로 push 상태 설계, LP 열 순서), `docs/lessons/process.md`(묶음 단독과 결합 LP의 crash 차이).
