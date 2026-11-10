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

- `A`·`B`: 표시를 매길 때 `A`·`B`에서만 위반 수 `uVar25`를 올리고(C 839·856줄) 이를 `local_1b0`에 둔다(868·904줄). 이탈 뒤 표시 갱신도 `A`·`B`에서만 `local_1b0`을 늘리고 줄인다(2165·2202·2217줄). 방향식과 feasible ratio `055f68`은 `local_1b0 == 0`일 때만 간다(1225·1547줄). 따라서 feasible 경로의 이탈 행 표시는 `N`·`U`·`L`·`X`·`F`뿐이다. `A`·`B` arm은 infeasible ratio 경로에만 있다.
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
- 등식 행에 내부 구조 변수가 있으면 crash basis가 그 변수를 고른다. 계수 0.1은 basic이 되고 0.01·1e-3·1e-4는 등식 slack이 남는다.
- slack 진입은 primal-first에서 나오지 않았다. dual-first에서 DStart 비영 행으로 나왔다.
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

(캡처 뒤 작성)

## 7. 판정과 인수인계

(캡처 뒤 작성)
