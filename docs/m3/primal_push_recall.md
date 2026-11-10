# M3-04c-8 primal push theta=I 반대 방향 재호출

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다. §1–§5는 원본 대상 캡처 전에 고정한다. 탐색 실행은 §4.4에 따로 적고 합격 근거로 쓰지 않는다.

## 1. 범위와 주장

- 작업 종류: 오라클 구축 + 구현. 기준선 main `c87c4fe`(M2-03d-12 병합, M3-04c-7 포함), branch `work/m3-04c-8`, worktree `../rsolver-m3-04c-8`, 미커밋 선행 변경 없음.
- 영역 선택: 사용자의 "다음 스프린트 진행" 지시. 시작 절차 순서는 M2·M1·M3이지만 M1은 다른 세션이 M1-02b를 점유해 진행 중이고, M2는 미점유이나 직전 M2 세션(M2-03d-12 병합)이 열려 있어 "세 세션이면 M1·M2·M3를 각각 맡는다" 규칙에 따라 M3를 골랐다.
- 질문 1(오라클): double primal push `134a87`에서 ratio 결과 theta가 I일 때 진입 열이 자유이거나 방향 0·ub≥I이면 방향을 뒤집어 ratio를 다시 부른다(C 1458–1590). 그 재호출의 입력·결과와 뒤따르는 단계(거리 flip·승인·자유 열 flip 상태)가 feasible(`055f68`)·위반(`054eef`→`056973`) 두 경로에서 원본 실행으로 어떻게 나오는가.
- 질문 2(구현): Rust `push_step`이 재호출 뒤 단계를 원본처럼 판단하고, 새 flip 상태 함수가 진입 열의 flip 뒤 상태(-1·-2·-3)를 원본과 같게 내는가.
- 포함 분기(asm `0x1c28ee–0x1c2994`, `0x1c497e–0x1c4db2`, `0x1c2994–0x1c2a2b`):
  1. 재호출 조건: theta == I(`ucomisd`, 같을 때만)이고 `lb[j] <= local_1d8`·`I <= ub[j]`(자유) 또는 방향 0·`I <= ub[j]`. 방향 1·자유 아님은 재호출 없이 거리 검사로 간다.
  2. 재호출: 방향 `S+0x328 = 1-d`, alpha 부호 다시 반전, feasible은 `055f68`(-1이면 theta=I, `S+0x2f8`에 쓰지 않음), 위반은 `054eef`와 게이트·`056973`(결과를 `S+0x2f8`에 씀).
  3. 재호출 뒤: theta가 I이고 자유이면 `S+0x2f8=-1`·위치 -1. 그 밖은 결과 그대로. 거리 검사는 **새 방향**의 gap(`0x1c290d`: 방향 1이면 dL, 0이면 dU), 거부 arm은 방향식 직후의 `r13b`(원래 방향), 승인·보류는 기존과 같다.
  4. flip 상태(C 1596–1612, `0x1c29cd–0x1c2a2b`): `local_1d8 < lb[j]` 또는 `ub[j] < I`이면 현재 방향 0→-2, 1→-1, 아니면 -3(자유 열은 superbasic으로 남음). 재호출이 없는 기존 flip 단계에도 같은 식이 적용되므로 기존 자료에도 검사를 넓힌다.
  5. `local_1d8`: C 602 `local_1d8 = -local_110`, C 195 `local_110 = S+0x370`. 새 agent가 단계 frame에서 `0x1f00(%rsp)`(위 세 곳이 읽는 슬롯)를 읽어 -I 비트와 같은지 본다.
- 구조상 판별할 수 없는 것(근거 §2.3, 탐색 대상 아님):
  - `reject-arm-current`(거부 arm을 현재 `S+0x328`로 고르는 변조). 스프린트 행은 이 변조를 재호출로 판별한다고 적었으나, 재호출 단계에서는 두 arm 모두 거부가 성립하지 않아 판별할 수 없다. 범위를 줄이는 것이 아니라 행의 기대를 C·asm 근거로 바로잡는다.
  - 자유 열 재호출 뒤 theta=I의 `S+0x2f8=-1`·위치 -1 대입(`0x1c4d7d`). 그 뒤 거리 검사가 언제나 flip(`0x1c293b`)으로 가 단계 판단에 영향이 없다. 쓰기는 `step:enter`의 `leave`가 -1로만 보인다.
- 제외와 후속:
  - `055f68` 작은 pivot 재진입(M3-04c-5), `056973` 재진입 2차 패스 Rust(거부 유지).
  - flip 적용 뒤 x·rhs 보정(C 1614 이후, `dVar49`), batch, x87, pivot 적용 뒤 factor, push 연결, 자연 work 전체.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C: `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c`(push, 195·602·1320–1330·1446–1612줄), `../re/decomp/c/000ef960_PRIVATE0000000000055f68.c`, `../re/decomp/c/000eefa0_PRIVATE0000000000054eef.c`, `../re/decomp/c/000f0120_PRIVATE0000000000056973.c`.
- asm: `tools/dc.py 134a87 --asm --range 0x1c28ee,0x1c2a00`와 `--range 0x1c497e,0x1c4dc0`으로 대조했다.
- 시작 경계: `supply:leave`(candidate≥0). 종료 경계: `step:enter`·`step:defer`, flip·pivot이면 `step:applied`. 04c-7과 같다.

### 2.1 재호출 분기(asm)

```text
0x1c28ee  theta(0x1eb8) == I(0x1fc8)?            아니면 0x1c28ff: ecx=S+0x328, esi=S+0x2f8 → 0x1c290d
0x1c497e  local_1d8(0x1f00) >= lb[j] && ub[j] >= I → 0x1c49e2 (자유)
          그 밖: S+0x328 != 0 → esi=S+0x2f8, 0x1c2911(방향 1 거리 검사)
                  ub[j] < I   → esi=S+0x2f8, 0x1c2924(방향 0 거리 검사)
0x1c49e2  S+0x328 = 1 - S+0x328, alpha 부호 반전(8개씩 묶음 + 꼬리, 04c-2와 같은 xor)
          0x1f28(%rsp)(위반 수) != 0 → 054eef, -1이거나 |pivot|<0.001이면 056973 → 0x1c4d0c
          == 0 → 055f68(edx=0) != -1 → 0x1c4d0c / == -1 → 0x1c4d99: theta=I, esi=S+0x2f8 → 0x1c4d1c
0x1c4d0c  S+0x2f8 = esi
0x1c4d1c  theta == I && 자유 → S+0x2f8=-1, esi=-1 / 아니면 → ecx=S+0x328, 0x1c290d
0x1c290d  ecx != 0 → theta >= dL(xmm5)-1e-10이면 flip / ecx == 0 → theta >= dU(0x1ee0)-1e-10이면 flip
0x1c2948  esi < 0 → flip. 거부 arm은 r13b(방향식 직후 값)
0x1c29cd  local_1d8 < lb[j] || ub[j] < I → ecx==0: -2, 아니면 -1 / 아니면 -3
```

`comisd` 분기라 NaN은 각 비교를 거짓으로 보낸다(이번 사례에는 없음).

### 2.2 기존 사건으로 보는 상태

새 agent `tools/primal_push_recall_agent.js`는 04c-7 agent에서 step frame에 `ninf`(`0x1f00(%rsp)`)만 더했다. 재호출은 한 단계 안의 두 번째 `ratio:enter`·`ratio:leave`(위반이면 각 호출 뒤 `retry:*`)로 보인다. 두 번째 `ratio:enter.selected`의 `direction`은 1-d, `leave`는 feasible이면 첫 `ratio:enter`의 값(055f68 -1은 쓰지 않음), 위반이면 첫 호출의 최종 위치(`0x1c2655`에서 씀)다.

### 2.3 `reject-arm-current` 구조상 판별 불가

거부는 거리 검사를 통과한 뒤에만 본다. 재호출은 자유 열(dL·dU가 I 근처) 또는 방향 0·ub≥I(dU가 I 근처)에서만 온다.

- 자유 열: dL·dU ≥ 100h가 아니면 자유가 아니므로 두 arm(`r>100h && dL<100h`, `r<-100h && dU<100h`) 모두 거짓이다.
- 방향 0·자유 아님: 원래 arm 0은 dU<100h가 필요해 거짓이다. 현재 arm 1(`r>100h && dL<100h`)이 참이려면 r>100h·dL<100h인데, 그러면 feasible 방향식 첫 절(10dL<dU, -(r·dL)<e)과 위반 방향식 첫 절(`r>100h && 100h>dL`)이 방향 1을 내므로 방향 0일 수 없다.

## 3. 비교 계약

04c-7 판단 모델(`tools/primal_push_decision_check.py` `decide`)을 넓힌다. 바뀌는 점만 적는다.

| 필드·사건 | 규칙 | 출처 |
|---|---|---|
| ratio 호출 수 | 재호출 조건이 참이면 정확히 2, 아니면 1. 위반 경로의 게이트·재시도는 호출마다 따로 본다 | §2.1 |
| 재호출 입력 | 두 번째 `ratio:enter`: 경로 이름 같음, `selected.direction`=1-d, alpha 값이 첫 호출 alpha의 비트 부호 반전, basic·width·표시 같음, `selected.leave`는 §2.2 규칙 | `0x1c49e2`, C 1327·1460 |
| 재호출 결과 | feasible -1이면 theta=I. theta=I·자유이면 위치 -1. 단계 입력은 재호출 결과 | `0x1c4d99`·`0x1c4d7d` |
| 거리 검사 | 재호출 단계는 새 방향의 gap | `0x1c290d` |
| 거부 arm | 방향식 결과 d(r13b) | `0x1c294c` |
| 방향 보존 | `step:enter`·`step:defer`의 `selected.direction`이 단계의 현재 방향(재호출이면 1-d) | `0x1c49ee` |
| flip 상태 | 모든 flip(거리·거부)의 `step:applied` `vstat[j]`가 §1 포함 분기 4와 같다(기존 자료 포함) | C 1596–1612 |
| `ninf` | 새 agent frame의 `ninf`가 -I(`S+0x370` 부호 반전) 비트와 같다 | C 602·195 |
| 그 밖 | 방향·반전·게이트·재시도·승인·보류·frame·이탈 상태는 04c-7과 같다 | 기존 |
| Rust | `push_step`(첫 호출)·`push_recall_step`(재호출 뒤)·`push_flip_state`가 같은 원시 입력에서 단계 종류·theta·flip/이탈 상태·반전 alpha를 비트 일치로 낸다. 재호출의 Rust ratio(`feasible_ratio`·`violated_ratio`)도 재반전 열에서 원본 두 번째 결과와 같아야 한다(연결 검사) | 캡처 |

NaN은 허용하지 않는다. I(`1e30`)와 ±0은 비트 그대로 둔다. Threads=1. 원본 재현성은 같은 설정 독립 3회의 캡처 JSON(사건·결과·출력) 일치로 확인한다.

판별력: 판단 모델 변조를 더한다. `recall-free-only`(방향 0·ub 절 끔), `recall-up-any-dir`(방향 무시), `recall-free-lt`(자유 판정 `lb < local_1d8`), `recall-gap-original`(재호출 뒤 원래 방향 gap), `recall-no-gate`(재호출 안 게이트 끔), `recall-alpha-keep`(재반전 안 함), `recall-stale-leave`(feasible -1도 `S+0x2f8`에 씀), `flip-free-state`(자유 arm 없음), `flip-dir-swap`(방향별 상태 반대). `reject-arm-current`는 이제 재호출 단계에 적용되며 §2.3에 따라 생존이 기대값이다. Rust 소스 변조도 같은 이름으로 돌린다.

합격 조건(하나라도 어기면 검사기 0이 아닌 종료):

- 두 대상 LP(feasible·위반) 각각 두 순서 독립 3회 캡처가 같고, 모든 단계가 판단 모델로 재구성되며 feasible LP는 feasible 경로만, 위반 LP는 위반 경로만 있다.
- 필수 재호출 arm(LP별 두 순서 합집합). 태그는 `<조건>-<결과>`이고 조건은 `free0`(자유·방향 0)·`free1`(자유·방향 1)·`up0`(자유 아님·방향 0·ub≥I), 결과는 `pivot`·`none`(재호출도 -1, flip)·`distance`(재호출 결과가 새 방향 gap 이상), 재호출 없이 theta=I·방향 1·ub≥I인 단계는 `none-up1`이다.
  - feasible: `free0-pivot`, `free0-none`, `up0-pivot`, `none-up1`.
  - 위반: `free0-pivot`, `free0-none`, `up0-pivot`, `up0-distance`, `free1-pivot`, `none-up1`, 그리고 재호출 안의 `056973` 재시도 1건 이상.
  - flip 상태 -3·-1(재호출 단계)과 feasible 재호출 `selected.leave`가 -1이 아닌 이전 값인 사례 1건 이상.
- 대조군(Crossover=0)은 push 사건 0이다.
- 비간섭: (1) 대상 두 LP를 04c-7 agent로 두 순서 각 1회 캡처해, 새 agent 캡처에서 `frame.ninf`를 빼면 같다. (2) 04c-7 `retry.lp` primal을 새 agent로 1회 캡처해 같은 투영이 04c-7 `retry-primal-1`과 같다.
- 기존 04b-1·face5·arm·04c-2·04c-7 자료가 넓힌 판단 모델(flip 상태 포함)로 다시 통과한다.

## 4. 사례와 원본 자료

### 4.1 대상 LP(실제 원본 실행, 설계한 LP)

입력은 `tools/push_recall_lp.py`가 만드는 블록 대각 LP와 PStart·DStart다(04c-2 `build` 그대로). 파라미터는 Threads=1, Seed=0, OutputFlag=0, Presolve=0, Method=2, LPWarmStart=2, ScaleFlag=0, Crossover=3(primal-first)·1(dual-first)·0(대조군). 무한 bound는 LP 파일에 ±1e30으로 쓰며 `push:enter` 모델의 lb는 -1e30이다.

`bench/oracle/m3_push_recall/feasible.lp`(`FR`·`FN`·`FU`·`FD`)와 `violated.lp`(`W`·`VR`·`VN`·`VU`·`VD`·`VDd`·`VS`·`VRg`):

| 묶음 | 설계 | 노리는 판단 |
|---|---|---|
| FR·VR | 자유 열, r=0, 계수 -1인 N 행(위로 막지 않음) | 방향 0 → -1 → 자유 재호출 → 방향 1 pivot(`free0-pivot`) |
| FN·VN | 자유 열, 빈 열 | 두 방향 모두 -1 → flip 상태 -3(`free0-none`) |
| FU·VU | lb=0·ub=I, r=0, 계수 +1 N 행(아래로 막지 않음) | 방향 1 → -1 → 재호출 없이 거리 flip -1(`none-up1`) |
| FD | lb=0·ub=I, 목적 -1, 위쪽은 \|a\|=1e-8(허용오차 미만) 행만 | feasible 방향 0 → -1 → 재호출(자유 아님) → pivot(`up0-pivot`). 1e-8 행이 LP를 유계로 둔다 |
| VD | lb=0·ub=I, 자기 B 행 계수 -2e-8(σ<0) | 위반 방향 0(σ 부호 절) → `up0-pivot` |
| VDd | VD와 같되 시작값 0.5 < 재호출 theta 1 | 새 방향 1 gap으로 거리 flip(`up0-distance`) |
| VS | 자유 열, 자기 B 행 계수 +2e-8(σ>0) | 위반 방향 1 → -1 → 자유 재호출 → 방향 0 pivot(`free1-pivot`, 자유 절 단독) |
| VRg | VR과 같되 N 행 계수 -5e-4 | 재호출의 `054eef` pivot이 0.001 미만 → 재호출 안 `056973` |

### 4.2 대조군·비간섭

대조군은 두 LP Crossover=0 각 3회. 비간섭은 §3 합격 조건의 두 가지다.

### 4.3 Rust 재생

`tools/primal_push_decision_rust.py`에 `recall` 사례(두 LP 두 순서 run 1)를 더한다. 입력 줄에 두 번째 ratio 결과(위치·theta·pivot, 위반이면 재시도 뒤)를 넣고, Rust는 첫 결과로 `push_step`을 부른 뒤 `Recall`이면 alpha를 다시 반전하고 `push_recall_step`을 부른다. flip이면 `push_flip_state`를 낸다. 기존 153단계의 판정은 바뀌지 않아야 한다.

### 4.4 탐색(합격 근거 아님)

`bench/oracle/m3_push_recall/history/`. 계약 전에 같은 묶음 LP(`eF`·`eV`)를 04c-7 agent로 두 순서 각 1회, 새 agent로 위반 primal 1회 돌렸다.

- feasible 4단계·위반 7단계가 위 설계대로 나왔다. `FN`·`VN`은 flip 뒤 `vstat`가 -3, `FU`·`VU`는 재호출 없이 -1, `VDd`는 재호출 theta 1에서 거리 flip -1, `VRg`는 재호출 안에서 `056973`을 불렀다. 두 순서의 push 단계는 같았다.
- feasible `FN`의 재호출 `ratio:enter.selected.leave`는 0(이전 값)이었고 위반 LP의 재호출에서는 첫 호출 결과 -1이었다.
- 새 agent의 `ninf`는 7단계 모두 -1e30이었고, `ninf`를 빼면 04c-7 agent 캡처와 사건·결과·출력이 같았다.

## 5. 재실행 명령

worktree 루트에서 실행한다. Gurobi는 `tools/gurun.sh`로 한 프로세스만 쓴다. 캡처 도구는 기존 파일을 덮어쓰지 않는다.

```bash
tools/push_recall_runs.sh                                   # 두 LP × primal·dual·control 각 3회, 비간섭 5회
python3 tools/push_recall_manifest.py
python3 tools/primal_push_decision_check.py --self-test --mutation <새 경로>.json
cargo test -p rsolver-lp --test push_decision && python3 tools/primal_push_decision_rust.py --compare
python3 tools/primal_push_decision_rust.py --mutate-rust <새 경로>.json
```

## 6. 실행 결과

- 환경: x86-64 Linux(WSL2), node·Frida는 04c-7과 같다. 원본 ELF 해시는 §2. 도구·입력·캡처·기록 해시는 [manifest](../../bench/oracle/m3_push_recall/manifest.json)에 있다. 대상 캡처는 계약 commit `1860e32` 뒤에 `tools/push_recall_runs.sh`로 실행했고 캡처 commit은 `8740f7b`다.
- 원본 실행: feasible LP primal-first 3회(42사건), dual-first 3회(40사건), 대조군 3회(0사건, 상태 2). 위반 LP primal-first 3회(87사건), dual-first 3회(85사건), 대조군 3회(0사건, 상태 2). 비간섭 5회(두 LP × 두 순서 04c-7 agent, 04c-7 `retry.lp` primal 새 agent). 순서별 3회는 사건·입력·시작 벡터·결과·출력 줄 전체가 같다.
- `local_1d8`: 새 agent의 모든 단계 frame `ninf`가 -1e30(-I) 비트였다(C 602·195 해석 실행 확인).
- 비간섭(검사기가 매번 확인): 새 agent 캡처에서 `frame.ninf`만 빼면 04c-7 agent 캡처와 사건·결과·출력이 같다(대상 두 LP 두 순서, 04c-7 `retry.lp` primal).
- 판단 재구성: feasible LP 두 순서 각 4단계, 위반 LP 두 순서 각 7단계. 두 순서의 단계 순서·진입 변수가 같다. 판단 모델이 ratio 호출 수, 재호출 방향·재반전 alpha·진입 `S+0x2f8`, 호출별 게이트·재시도, 단계 종류·theta·frame·`ninf`, 이탈 상태와 flip 상태를 원본과 같게 재구성했다. 기존 04b-1·face5·arm·04c-2·04c-7 153단계도 flip 상태 검사를 더한 모델로 다시 통과했다.

| LP | 단계 | 묶음 | 방향식 | 첫 호출 → 재호출 | 단계 | 태그 |
|---|---|---|---|---|---|---|
| feasible | 1 | FN | 0(r=0) | -1 → -1 | flip, 상태 -3. 재호출 진입 `leave`=0(이전 값) | `free0-none` |
| feasible | 2 | FR | 0 | -1 → 행 2 theta 1 | pivot, 이탈 `N`→-1 | `free0-pivot` |
| feasible | 3 | FU | 1(bound 절) | -1, 재호출 없음 | 거리 flip, 상태 -1 | `none-up1` |
| feasible | 4 | FD | 0(r=-1) | -1(1e-8 행 무시) → 행 1 theta 1 | pivot | `up0-pivot` |
| 위반 | 1 | VN | 0(small-r0) | -1(056973 -1) → -1(056973 -1) | flip, 상태 -3 | `free0-none` |
| 위반 | 2 | VR | 0 | -1 → 행 1 theta 1 | pivot | `free0-pivot` |
| 위반 | 3 | VU | 1(bound-sigma) | -1, 재호출 없음 | 거리 flip, 상태 -1 | `none-up1` |
| 위반 | 4 | VRg | 0 | -1 → `054eef` pivot 5e-4 → 재호출 안 `056973` 행 5 theta 0.2 | pivot | `free0-pivot` |
| 위반 | 5 | VDd | 0(sigma0) | -1 → theta 1 ≥ dL 0.5 | 새 방향 gap으로 거리 flip, 상태 -1 | `up0-distance` |
| 위반 | 6 | VS | 1(sigma, σ=+2e-8) | -1 → 방향 0 행 2 theta 1 | pivot | `free1-pivot` |
| 위반 | 7 | VD | 0(sigma0) | -1 → 행 3 theta 1 | pivot | `up0-pivot` |

- 검사기 변조(`primal_push_decision_check.py --self-test`, 종료 0): 04c-7까지의 93종에 재호출 판단 변조 10종(재호출 삭제, 재호출 방향, 재반전 없음, feasible 진입 `S+0x2f8`을 -1로, 단계 방향, flip 상태, 재호출 안 재시도 삭제, 자유 열 lb 유한화, VDd 시작값, `ninf`), 재호출 캡처 변조 12종(기존 11종+`ninf` 누락), 비간섭 변조 3종, 필수 태그 누락 6종을 더한 124종을 모두 거부했다.
- 판단 모델 변조([기록](../../bench/oracle/m3_push_recall/model-mutation.json)): 04c-7의 46종과 재호출·flip 9종, 총 55종 중 48종 거부(frame을 지운 판단만 실행도 같다). 새 9종은 모두 거부됐다. 생존 7종은 04c-7과 같다(`reject-arm-current`, `leave-fixed-lt`, `leave-neg-default-2`, `leave-X-1`, `leave-U-le`, `v-sigma-ge0`, `gate-no-neg`). `reject-arm-current`는 이제 재호출 단계에도 적용되지만 §2.3대로 판별되지 않았다.
- Rust 재생(구현): `cargo test -p rsolver-lp --test push_decision && python3 tools/primal_push_decision_rust.py --compare` 종료 0. 175단계(기존 153 + 재호출 LP 22, 고유 입력 11)가 방향·frame·단계 종류·theta·이탈/flip 상태·재호출 여부·최종 alpha 비트 일치했고, 연결 검사(Rust 반전 열의 ratio, 재호출 18단계는 재반전 열의 두 번째 ratio까지)도 175단계 불일치 0이다. 이번부터 모든 flip 단계에서 `push_flip_state`를 비교한다. `--self-test` 종료 0(변조 15종 거부, 연결 변조 2종 검출).
- Rust 소스 변조([기록](../../bench/oracle/m3_push_recall/rust-mutation.json)): 04c-7의 54종에 재호출·flip 6종(`recall-free-lt`, `recall-gap-original`, `recall-free-pos`, `flip-free-state`, `flip-dir-swap`, `reject-arm-current` 이식)을 더한 60종 중 49종 거부, 연결 검사로만 검출 2종(`gate-off`·`gate-pivot-1e-4`), 생존 9종. 04c-7에서 생존이던 `recall-free-only`·`recall-up-any-dir`가 거부됐다. 생존은 04c-2의 6종(`leave-fixed-lt`, `leave-neg-default-2`, `leave-X-1`, `leave-U-le`, `sign-zero-neg`, `v-sigma-ge0`), `gate-no-neg`(구조상), `reject-arm-current`·`recall-free-pos`(§1 구조상 판별 불가)다.
- 회귀: 기준선 `c87c4fe` 위 작업 commit `64ae4d5`에서 `cargo test --release --workspace` 통과, `cargo clippy --workspace --all-targets -- -D warnings` 종료 0, `cargo test -p rsolver-lp --test push_decision --test push_ratio`, `push_retry_rust.py --compare`(합성 84 비교·재진입 6 거부), `push_violated_rust.py --compare`(55), `primal_push_ratio_rust.py --compare`(자연 18·합성 29), `primal_push_decision_check.py --old`, `push_slot_reach.py` 종료 0. main(M1-02b 병합 `7a4af88`)을 합친 `49bdd96` 위에서 `cargo test --release --workspace`(121 통과, 실패 0)·clippy·`push_decision` 재생 `--compare`·검사기 `--self-test`를 다시 통과했다.
- 실행하지 않은 검사: Python 확장·pytest. Rust 변경은 호출자가 없는 `crossover` 모듈과 그 재생 테스트뿐이고 Python 경계를 바꾸지 않았다.
- 계약 변경: 없음. 사례 이름은 04c-2 `violated-primal`과 겹쳐 manifest 사례를 `recall-` 접두어로 지었다(캡처 파일 이름은 그대로). 필수 flip 상태 -3·-1은 계약 문구대로 두 LP 합집합으로 검사한다(feasible LP에는 재호출 뒤 -1 flip이 없다).
- 04b-3·04c-2·04c-7 manifest의 검사기·재생 도구·`input.txt` 해시는 이번 변경 전 값이다(캡처 도구·agent 해시만 검사에 쓰이며 그대로다).
- 기준 자료 갱신 순서: `tools/push_recall_runs.sh` → `python3 tools/primal_push_decision_rust.py --write` → 변조 기록 → `python3 tools/push_recall_manifest.py`(검사 전).

## 7. 판정과 인수인계

- 작업 수행: 완료.
- 오라클 판정: 계약 범위 **통과**. theta=I 반대 방향 재호출을 feasible(`055f68`)·위반(`054eef`→`056973`) 두 경로에서 원본 순서별 독립 3회로 관측했다. 재호출 조건의 세 절(자유·방향 0, 자유·방향 1, 자유 아님·방향 0·ub=I)과 재호출 없는 방향 1·ub=I, 재호출 결과 pivot·-1(자유 flip -3)·새 방향 gap 거리 flip, 재호출 안 `056973`, feasible -1의 `S+0x2f8` 미기록, `local_1d8`=-I를 확인했다.
- 구현 판정: 계약 범위 **통과**. Rust `push_step`·`push_recall_step`·`push_flip_state`가 재호출 22단계와 기존 153단계에서 원본과 비트 일치했다.
- 미검증·구조상 판별 불가: `reject-arm-current`(§2.3), 자유 열 재호출 뒤 위치 -1 대입(`recall-free-pos`, 뒤 거리 검사가 언제나 flip), 재호출 뒤 거부·보류(재호출 조건과 거부·보류 조건이 함께 서기 어려움, 관측 없음), dense 열 재호출, NaN. flip 적용 뒤 x·rhs 보정과 push 연결은 범위 밖이다.
- 후속: M3-04c-5(`055f68` 재진입) 그대로. push 연결(Rust push 루프가 `Recall` 뒤 재반전·ratio·`push_recall_step`을 부르는 일)은 연결 작업에서 한다.
- branch `work/m3-04c-8`, worktree `../rsolver-m3-04c-8`. 결과·인수인계·진행·로드맵은 이 branch에서 갱신해 `--no-ff`로 병합한다.
- 교훈: `docs/lessons/original.md`(재호출 경로와 `local_1d8`, feasible -1 미기록, flip 상태), `docs/lessons/process.md`(스프린트 행 기대의 구조상 도달 불가 판정, 사례 이름 충돌).
