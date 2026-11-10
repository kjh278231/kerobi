# M3-04c-2 primal push 위반 basis 방향식·infeasible ratio·`A`·`B` 이탈 arm

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다. §1–§5는 원본 대상 캡처 전에 고정한다. 탐색 실행은 §4.1에 따로 적고 합격 근거로 쓰지 않는다.

## 1. 범위와 주장

- 작업 종류: 오라클 구축 + 구현. 기준선 main `50d1b33`(M2-03d-5 병합, M3-04b-3 포함), branch `work/m3-04c-2`, worktree `../rsolver-m3-04c-2`, 미커밋 선행 변경 없음.
- 질문 1(오라클): double primal push `134a87`이 위반 basis(`local_1b0>0`)에서 고르는 방향과 infeasible ratio `054eef`의 결과, 그 뒤 거부·승인·보류·이탈 상태가 실제 원본 실행에서 어떻게 나오는가.
- 질문 2(구현): Rust `push_direction`(위반 표시 인자)과 새 `infeasible_ratio`가 같은 입력에서 원본과 비트 단위로 같은가.
- 포함 분기:
  1. 위반 방향식(C 1325–1366, asm `0x1c227c–0x1c23d1`)의 종단 10개: 거부 선행 두 절(`r>100h && 100h>dL`→1, `-100h>r && 100h>dU`→0), σ 절 두 개, |σ|<e 안의 bound 절 두 개와 r 부호 둘, σ 부호 둘. σ는 반전 전 alpha의 `B` 행 합에서 `A` 행 합을 뺀 값이다.
  2. `054eef` 첫 패스의 표시별 분기(양수 alpha: `N`·`U`·`L`·`X`는 x, `A`는 x-w, `B`·`F` 제외 / 음수 alpha: `B`는 x, `U`·`X`는 x-w, `L`·`N`은 w<I일 때 x-w, `A`·`F` 제외), Harris 상한, 2차 패스 최대 |alpha|·동률, 유한 cap, 후보 없음, 작은 pivot 통과, work. dense의 음수 alpha `U`·`X`만 `<`이고 나머지는 `<=`다(asm `ef826` `ja`, `ef6c1` `jae`).
  3. `054eef` 뒤 단계 판단: 거리 flip, 거부, 승인, 보류는 feasible 경로와 같은 코드(`0x1c290d` 이후)다. 위반 경로에서도 원본 사건으로 확인한다.
  4. 이탈 상태 `B`(a<0)→-1, `A`(a≥0)→-2.
- `056973` 게이트: push는 `054eef`가 -1을 돌려주거나 |pivot|<0.001이면 `056973`을 부른다(C 1447–1452, asm `0x1c258e–0x1c263a`). 이번 범위는 그 재시도가 붙지 않은 단계뿐이다. 재시도 여부는 새 훅 `retry:*` 사건으로 양성 확인한다.
- 구조상 이번 범위에서 도달하지 않는 것(근거 §2.3, 탐색 대상 아님):
  - 위반 경로의 theta=I 재호출: 재호출에 오려면 `054eef`가 -1이어야 하고 그러면 먼저 `056973`을 부른다.
  - `B`→-2, `A`→-1, a<0 기본 arm(`A`·`F`): `054eef`만 거친 pivot에서는 나오지 않는다.
- 제외와 후속:
  - `056973` 재시도, 위반 basis 재시도 뒤 재호출 → M3-04c-7(새 행).
  - `054eef` 작은 pivot 재진입 2차 패스 → Rust는 `Reentry`로 거부한다(feasible과 같음). 원본 합성 출력만 기록한다.
  - batch, x87, 이탈 뒤 basic 값·rhs 보정·factor, push 연결, 자연 work는 범위 밖이다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C: `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c`(push), `../re/decomp/c/000eefa0_PRIVATE0000000000054eef.c`(ratio), `../re/decomp/c/000f0120_PRIVATE0000000000056973.c`(재시도, 범위 밖).
- 시작 경계: `supply:leave`(candidate≥0). 종료 경계: `step:enter`·`step:defer`, pivot이면 `step:applied`. 04b-3과 같다.

### 2.1 위반 방향식(asm)

레지스터는 xmm4=r, xmm5=dL, xmm1=σ, `0x1ee0(%rsp)`=dU, `0x2080`=e(0.01h), `0x1ec8`=100h, `0x1eb0`=-100h, 상수 `0x2a61348`=10.0이다. 모든 분기는 `comisd` 뒤 `ja`(참일 때 이동)나 `jbe`(거짓·비순서일 때 건너뜀)다.

```text
r > 100h  && 100h > dL                       → 1   (0x1c227c)
-100h > r && 100h > dU                       → 0   (0x1c229a)
dU > 10dL && e > -(dL*σ) && e > -(r*dL)      → 1   (0x1c22c1)
dL > 10dU && e > dU*σ   && e > dU*r          → 0   (0x1c2311)
e > |σ|:                                           (0x1c235b)
    dU > 10dL && e > -(r*dL) → 1;  dL > 10dU && e > dU*r → 0;  r > 0 → 1, 아니면 0
아니면 σ > 0 → 1, 아니면 0                          (0x1c23c9)
```

방향 1이면 `S+0x328=1`, `r13b=0`이고 열 부호를 뒤집는다(`0x1c23d7`). 방향 0이면 `r13b=1`(`0x1c24ee`). 거부 arm은 이 `r13b`로 고르므로 feasible과 같다. r은 같은 루프에서 `r -= cost[header[i]]*alpha`, σ는 `B`면 `+alpha`, `A`면 `-alpha`를 희소 순서대로 누적한다(C 1329–1349). 호출 직전 `0x1c2569`에서 r·dL을 `0x1ed0`·`0x1ed8`에 저장하므로 04b-2의 frame 슬롯이 위반 경로에서도 같다.

### 2.2 `054eef` 인자와 꼬리

호출점 `0x1c24f1–0x1c2573`: xmm0=h(`S+0x380` 슬롯), xmm1=pivot 허용오차(`S+0x398`), xmm2=I(`S+0x370`), xmm3=xmm2(cap=I). rdi=S, esi=m, rdx=alpha, rcx=basic, r8=width, r9=표시, 스택=후보 scratch, ratio scratch, theta 슬롯(`0x1eb8`), pivot 슬롯(`0x1ef8`), work. `055f68`과 같은 배치다.

첫 패스 뒤 꼬리(`ef238` 이후)는 `055f68`과 같은 구조다. `cap<I && cap<=bound`→-1·theta=cap, `bound==I` 또는 후보 0→-1·theta=I, 2차 패스는 ratio≤bound 중 |alpha|가 엄격히 큰 것, pivot<0.1이면 최대 |alpha| 배율표로 재진입 여부를 정한다. work 단위는 sparse `2c+4·count`, dense `2c+2t+n`이다(c 후보, t 시험 수, `ef200`·`ef898`).

### 2.3 이번 범위에서 도달하지 않는 arm의 근거

- 이탈 `B`→-2: `054eef`는 `B` 행을 음수 alpha에서만 후보로 넣고 ratio를 `x/a`로 저장한다. 승인 뒤 이탈 상태는 같은 x와 같은 a로 `|x/a - t|`를 계산하고(C 2140–2143) t는 그 ratio 자체다. 차는 0이고 x<0, a<0이라 t>0이므로 `(t+1)k <= 0`이 거짓이다. 결과는 항상 -1이다.
- 이탈 `A`→-1: `A` 행은 양수 alpha에서만 후보이고 ratio는 `(x-w)/a`다. 같은 이유로 차가 0이고 결과는 -2다.
- a<0 기본 arm: 음수 alpha에서 후보가 되는 표시는 `B`·`U`·`X`·`L`·`N`뿐이고 모두 명시 arm이 있다.
- 위반 경로 재호출: `054eef`가 -1이면 `056973`을 먼저 부른다(C 1447). 재호출(C 1458–1590)에는 그 뒤에만 온다.

## 3. 비교 계약

04b-2·04b-3 판단 모델(`tools/primal_push_decision_check.py` `decide`)을 위반 경로로 넓힌다. 바뀌는 점만 적는다.

| 필드·사건 | 규칙 | 출처 |
|---|---|---|
| 경로 | `ratio:enter.name`이 `infeasible`이면 그 단계의 표시에 `A`·`B`가 있어야 하고, `feasible`이면 없어야 한다 | C 1225, 04b-3 §2.2 |
| 재시도 | 단계 구간에 `retry:enter`가 있으면 실패(범위 밖) | 새 훅 `0xf0120` |
| 방향 | §2.1 식으로 계산한 값과 정확 일치. 종단 이름을 집계한다 | asm |
| alpha 반전·거리 flip·거부·승인·보류·frame·이탈 상태 | 04b-2·04b-3과 같다 | 기존 |
| ratio(자연) | Rust `infeasible_ratio`의 위치·theta·pivot이 `ratio:leave`와 비트 일치(연결 검사) | 캡처 |
| ratio(합성) | Rust가 원본 직접 호출의 위치·행·theta·pivot·work와 비트 일치. 재진입 사례는 Rust 거부를 따로 센다 | 직접 호출 |

NaN은 허용하지 않는다. I(`1e30`)와 ±0은 비트 그대로 둔다. Threads=1. 원본 재현성은 같은 설정 독립 3회의 캡처 JSON(사건·입력·결과·출력) 일치로 확인한다.

판별력: 판단 모델 변조에 위반 방향식의 절별 변조 11종을 더해 저장 자료 전체에 돌린다. Rust 소스 변조도 같은 목록과 `054eef` 분기별 변조를 돌려 생존을 미검증 경계로 적는다.

합격 조건(하나라도 어기면 검사기 0이 아닌 종료):

- 위반 경로 단계가 순서별 3회 캡처에 하나 이상 있고, 계약 대상 arm(§4 표의 "필수")이 모두 관측된다.
- 대상 캡처의 모든 단계가 판단 모델로 재구성된다(재시도 사건 0).
- 대조군(Crossover=0)은 push 사건 0이다.
- 비간섭: 새 훅을 뺀 사건이 이전 agent 캡처와 같다(발동·비발동 각 한 LP).

## 4. 사례와 원본 자료

### 4.1 자연 위반 basis(실제 원본 실행, 설계한 LP)

입력은 `tools/push_violated_lp.py`가 만드는 블록 대각 LP와 PStart·DStart다(04b-3 방식). 파라미터는 Threads=1, Seed=0, OutputFlag=0, Presolve=0, Method=2, LPWarmStart=2, ScaleFlag=0, Crossover=3(primal-first)·1(dual-first)·0(대조군). 시작값이 행을 위반하면 crash basis의 basic slack이 `B`로 분류된다. 위반 수는 LP 전체에서 세므로 위반 행이 남아 있는 동안 다른 묶음의 단계도 위반 경로로 간다(σ=0이면 |σ|<e 절).

대상 LP `bench/oracle/m3_push_violated/violated.lp`(45열×25행)와 `violated.extra.json`은 `python3 tools/push_violated_lp.py <경로>`가 `TARGET` 순서로 만든다. 묶음은 서로 변수·행을 나눈다. S5·S6·D1–D4는 각자 위반 행을 갖는다.

| 묶음 | 설계 | 노리는 판단 |
|---|---|---|
| W | xw·yw를 상한 1에 두고 `xw+yw<=1.7`. 후보 없는 영구 `B` 행 | 위반 경로 유지(다른 묶음은 σ=0) |
| S5 | x5∈[0,100], c=-1e-9, 시작 5. 위반 행 `c5`에 계수 -5e-9 | σ=-5e-9: σ 절 탈락, \|σ\|<e bound 절 → 1(끄면 r 절 0) |
| S6 | x6∈[0,100], c=1e-9, 시작 95. `c6`에 계수 5e-9 | σ=5e-9: \|σ\|<e bound 절 → 0(끄면 r 절 1) |
| D1 | xD1, c=1, 시작 lb+5e-5. 자기 위반 행에 계수 -1(σ=-1) | 거부 선행 절 → 1(끄면 σ 부호 절 0) |
| D2 | xD2, c=-1, 시작 ub-5e-5. 위반 행 계수 +1 | 거부 선행 절 → 0(끄면 1) |
| D3 | xD3, c=1e-5, 시작 lb+5e-5. 위반 행 계수 -2e-8 | σ 절 → 1(끄면 \|σ\|>=e의 σ<0 → 0) |
| D4 | xD4, c=-1e-5, 시작 ub-5e-5. 위반 행 계수 2e-8 | σ 절 → 0(끄면 1) |
| D7 | xD7∈[0,10.5], c=0, 시작 1(dU/dL=9.5) | r=0 → 0(계수 10→9면 σ 절 1, r>=0이면 1) |
| S4b | x4b∈[0,10], c=-1e-5, 시작 ub-5e-5 | σ 절(방향 0), dL>10dU |
| S1 | x1, c=1, 시작 lb+5e-5, 차단 slack 1 | 거부 선행 절 → 1, 거리 flip |
| S1r | S1과 같고 차단 slack 1e-5 | 거부 선행 절 → 1, 거부 flip |
| S2 | x2, c=-1, 시작 ub-5e-5 | 거부 선행 절 → 0, 거리 flip |
| S3·S4 | c=0, 시작 lb+5e-5·ub-5e-5 | crash가 구조 열을 basic으로 골라 행 slack이 진입(σ 절 → 1) |
| S7·S7n | c=±1, 중앙 시작 | \|σ\|<e의 r 부호 → 1·0 |
| VB | `xv+yv<=4.8`, xv 시작 5 | σ=+1 → 1, 이탈 `B`(a<0) → -1 |
| S8n | `-x8+z8<=4.8`, z8 상한 | σ=-1 → 0, 이탈 `B` → -1 |
| SA | 등식 `pa+qa+ra=4.2`, pa·ra 상한 3 | crash basic이 3.2(`A`), σ=-1 → 0, 이탈 `A`(a≥0) → -2 |

필수 arm(검사기가 대상 두 순서의 합집합에서 확인하고, 하나라도 없으면 실패): 방향 종단 10개(`pre-reject1`, `pre-reject0`, `bound-sigma`, `bound-sigma0`, `small-bound`, `small-bound0`, `small-r`, `small-r0`, `sigma`, `sigma0`), 단계 원인 `distance`·`reject`·`accept`, 이탈 arm `neg-B`·`nonneg-A`. 대상 단계는 모두 `infeasible` 경로여야 한다.

보류는 위반 경로에서 이번 범위로 오지 않는다. pivot<1e-6이면 그보다 먼저 |pivot|<0.001 게이트로 `056973`을 부른다. `theta*pivot<-1e-5` 쪽은 `054eef`의 후보 ratio가 Harris 여유 안에서만 음수가 될 수 있어 설계하지 않았다(미검증).

비간섭: (1) 비발동: 대상 LP를 04b-3 arm 도구·agent로 두 순서 각 1회 캡처해 새 agent 대상 캡처와 사건·결과·출력이 같다. (2) 발동: `retry.lp`(W+SR, 차단 행 없음)를 새 agent·arm agent로 각 1회 캡처해 새 agent 쪽에 `retry:*`가 있고, 그것을 빼면 같다.

### 4.1 탐색(합격 근거 아님)

`bench/oracle/m3_push_violated/history/`. 계약 전에 단독·결합 LP를 새 agent로 돌렸다.

- `pVB`: 시작값이 행을 1 넘으면 basic slack이 `B`가 되고 σ=+1, 방향 1, `054eef`가 `B` 행을 고른다.
- `p1`: 결합하면 c=0인 구조 열(S3·S4)이 crash basic이 되고 그 행 slack이 진입한다. σ=0이면 σ 절이 r·dL 절과 같아져 \|σ\|<e 안 bound 절에 오지 않았다.
- `p2`: 5e-9 계수로 \|σ\|<e bound 절 둘이 나왔다. 묶음이 늘자 SA에서 crash가 ra를 골라 `A`가 사라졌다.
- `p3`(대상과 같은 묶음): pa·ra 상한을 같게 하자 두 순서 모두 14단계가 위반 경로로 재구성되고 위 필수 arm이 모두 나왔다. 재시도 0.
- `pR`(W+SR): `054eef`가 -1이고 새 훅에 `retry:enter`·`retry:leave`(-1)가 찍혔다.
- 판별력 점검: `p3`에서는 σ=0이라 거부 선행 절을 꺼도 σ 절이 같은 방향을 내는 등 절 변조가 살아남았다. 절마다 꺼졌을 때 다음 절이 반대 방향을 내도록 D1–D4·D7을 더하고 S5·S6 목적을 ∓1e-9로 바꿨다.
- `p4`(대상과 같은 묶음): 두 순서 각 19단계, 모두 위반 경로·재시도 0. 판단 모델 변조 중 위반 절 11종의 10종이 판단만으로 거부됐다. `v-sigma-ge0`는 \|σ\|>=e 가지라 σ=0이 될 수 없어 구조상 생존이다.

### 4.2 `054eef` 합성 직접 호출(합성)

`tools/push_infeasible_ratio_oracle.py`. 표시·부호별 29사례×sparse/dense(동률 역순은 sparse만), 총 57호출을 독립 3회 실행한다. 사례마다 설계 행을 두어 노린 분기에 들어갔는지 확인하고, 값의 기준은 원본 출력이다. 경계 사례: pivot 허용오차와 같은 alpha, 동률·역순, Harris 상한 안의 큰 pivot, 첫 후보의 상한과 정확히 같은 ratio의 음수 `U`·`X`(sparse 받음, dense 거름)와 `B`(둘 다 받음), 유한 cap, 빈 열, 0 alpha, -0, 작은 pivot 통과와 재진입.

## 5. 재실행 명령

worktree 루트에서 실행한다. Gurobi는 `tools/gurun.sh`로 한 프로세스만 쓴다. 캡처 도구는 기존 파일을 덮어쓰지 않는다.

```bash
D=bench/oracle/m3_push_violated
for i in 1 2 3; do tools/gurun.sh ../.venv/bin/python tools/push_infeasible_ratio_oracle.py --capture $D/ratio-$i.json; done
python3 tools/push_infeasible_ratio_oracle.py --check $D/ratio-{1,2,3}.json --selftest
python3 tools/push_violated_lp.py $D/violated            # 입력 재생성(기존 파일과 다르면 거부)
X="$(cat $D/violated.extra.json)"
for k in primal dual control; do for i in 1 2 3; do
  tools/gurun.sh node tools/primal_push_violated_capture.js $k $D/violated.lp $D/violated-$k-$i.json "$X"
done; done
python3 tools/primal_push_decision_check.py --self-test --mutation <새 경로>.json
cargo test -p rsolver-lp --test push_decision --test push_ratio && python3 tools/primal_push_decision_rust.py --compare && python3 tools/push_violated_rust.py --compare
```
