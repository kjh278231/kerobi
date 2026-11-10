# M3-04b-2 primal push 방향·ratio 뒤 거부·이탈 상태 오라클

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다.

## 1. 범위와 주장

- 작업 종류: 오라클 구축. 기준선 main `1e6a937`(M3-04c 병합), branch `work/m3-04b-2`, worktree `../rsolver-m3-04b-2`, 미커밋 선행 변경 없음.
- 질문: double primal push `134a87`의 한 단계에서 (1) 방향식, (2) ratio 뒤 거부 판정, (3) pivot 이탈 변수 상태 선택이 읽는 `0x1ec8(%rsp)`·`local_228`·`local_250`의 저장 출처는 무엇인가. 그 값과 이미 캡처한 상태로 세 판단을 원본과 같은 결과로 재구성할 수 있는가.
- 완료할 것:
  1. 세 슬롯의 저장 출처를 asm과 CFG로 확정한다(§2).
  2. 판단 모델(Python)로 04b-1 저장 캡처 9단계×2순서의 방향·flip/pivot·이탈 상태를 재생 대조한다(Gurobi 미실행).
  3. 새 agent로 판단 지점의 frame 값(r·dL·dU·세 슬롯·e)을 직접 캡처한다. `interior.lp`에서 새 필드를 뺀 사건이 04b-1 저장 사건과 같음을 보여 비간섭을 확인한다.
  4. 목적이 있는 LP에서 방향 1 단계를 포함한 자연 캡처를 순서별 독립 3회 얻고 같은 모델로 대조한다.
- 포함 분기: feasible basis(`local_1b0==0`)의 방향식 두 절, 방향 0·1의 거리 flip(`1e-10`), 거부 두 arm, pivot 승인(`1e-6`, `-1e-5`), 이탈 상태 -4(고정폭·`X`)·-1·-2의 표시별 분기.
- 관측 시 기록만 하는 분기: 거부 실제 발동, 이탈 상태의 개별 arm, pivot 보류. 목표 수를 채우려고 탐색을 무한히 늘리지 않는다. 일정 탐색에서 나오지 않으면 미관측으로 남기고 후속 번호로 분리한다.
- 제외: 위반 basis 방향식(1333–1454줄)과 infeasible ratio `054eef`·`056973`, theta=I 반대 방향 재호출(`0x1c497e`–`0x1c4d0c`), batch, x87 `256781`, 이탈 뒤 basic 값·rhs 보정·factor, Rust 구현. 이 경로를 밟는 LP는 `history/`에 남기고 합격 근거로 쓰지 않는다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`(이번 objdump 대상 파일에서 재확인).
- C `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c` 921–926줄(루프 준비), 1209–1240줄(방향식), 2060–2177줄(거부·승인·이탈 상태). asm은 `objdump -d --start-address=0x1c0070 --stop-address=0x1c5310`.

### 2.1 세 슬롯의 저장 출처

h=`S+0x380`은 push 진입 `0x1c0284`에서 `0x2070(%rsp)`에 한 번 저장되고 함수 안에서 다시 쓰이지 않는다. 단계 루프 준비 `0x1c1f09–0x1c1fae`(C 921–926줄)는 다음 셋을 쓴다. 상수는 rodata에서 읽었다.

| 슬롯 | 값 | C 이름 | 루프 뒤 읽는 곳 |
|---|---|---|---|
| `0x1ec8(%rsp)` | `100.0*h`(`0x2a61358`) | `local_210` | 위반 방향식 `0x1c227c/87/a9`, 거부 `0x1c2955/64`, `0x1c4961` |
| `0x1eb0(%rsp)` | `-100.0*h`(`0x2a613f8`) | `local_228` | 위반 방향식 `0x1c229a`, 거부 `0x1c494e` |
| `0x1e88(%rsp)` | `h+h` | `local_250`(이 구간의 double) | 이탈 상태 `0x1c2c45`, `0x1c2ca6`, `0x1c2d27`, `0x1c2d5f` |

`0x1ec8`은 진입 `0x1c0271`에서 `S+0x1f8` 포인터로, `0x1e88`은 루프 전 정수 카운터로도 쓰인다. C의 `local_250` `CONCAT` 표현은 이 재사용 때문이다. `tools/push_slot_reach.py`가 직접 jmp·fall-through CFG에서 각 쓰기가 닿는 읽기를 계산한다. 루프 준비의 쓰기만 루프 안 읽기에 닿고, 진입에서 쓰기 없이 닿는 읽기는 없다. 간접 jmp는 없다. call은 fall-through로 본다. 스택 주소를 넘기는 lea는 theta `0x1eb8`, pivot `0x1ef8`, `0x1f28`, `0x2090/98/9c`, `0x8`, `0x1868`뿐이라 세 슬롯과 겹치지 않는다. 인자 push로 rsp가 이동한 6개 구간의 스택 쓰기는 frame `0x1ed0/0x1ed8`(xmm4·xmm5 보존)에만 닿는다.

### 2.2 판단식(asm 확인)

- r·dL·dU: `0x1c2164–0x1c26a9`. 구조 열 j는 `dU=ub-x`, `dL=x-lb`, slack은 `dU=I(S+0x370)`, `dL=slack[j-n]`. `r=cost[j]`에서 희소 위치 순서로 `r=r-alpha_k*cost[header[idx_k]]`. alpha는 방향 반전 전 FTRAN 값이다. `xmm4=r`, `xmm5=dL`, `0x1ee0=dU`, `0x2080=e=0.01*h`.
- 방향 1(`0x1c26ab–0x1c2733`): `(10*dL < dU && -(r*dL) < e) || ((dL <= 10*dU || e <= dU*r) && 0 < r)`. 참이면 `S+0x328=1`, `r13b=0`, alpha 부호 비트 반전. 아니면 `r13b=1`.
- ratio 반환 -1이면 theta=I로 `0x1c497e` 재호출 경로(제외).
- 거리 flip(`0x1c290d–0x1c293b`, 현재 `S+0x328`): 방향 0은 `dU-1e-10 <= theta`, 방향 1은 `!(theta < dL-1e-10)`이면 leave=-1.
- 거부(`0x1c294c`·`0x1c2a72`·`0x1c494e`): **방향식 직후의 `r13b`**로 arm을 고른다. 재호출 경로 `0x1c49e2`는 `S+0x328`만 뒤집고 `r13b`를 쓰지 않는다. `r13b=0`: `r > 100h && dL < 100h`. `r13b=1`: `r < -100h && dU < 100h`. 참이면 theta=I(`S+0x370`), leave=-1로 flip 경로에 합류한다. 04c 문서의 "`0x1c294c–0x1c2989`"는 방향 1 arm만이다.
- pivot 승인(`0x1c2a84–0x1c2aaf`): `1e-6 <= pivot && -1e-5 <= theta*pivot`. 아니면 `0x1c3b7b` 보류(훅 없음).
- 이탈 상태(`0x1c2bc2–0x1c2d7f`): leave=header[row]. `ub-lb <= S+0x3b0`이면 -4. 아니면 부호 있는 a=alpha[p](반전 뒤), t=theta, mark=marks[row], b=basic[row], w=width[row], K=2h로
  - `0 <= a`: `L`·`N`→-1, `X`→-4, `A`는 `abs((b-w)/a-t) < (t+1)*K`이면 -2, `U`는 `abs(a)*t < K`이면 -2, 나머지 -1.
  - `a < 0`: `N`·`U`→-2, `B`는 `(t+1)*K <= abs(b/a-t)`이면 -2, `X`→-4, `L`은 `K <= abs(a)*t`이면 -2, 나머지 -1.

## 3. 비교 계약

| 필드·사건 | 비교 규칙 | 출처 |
|---|---|---|
| 방향(`ratio:enter.selected.direction`) | 정수 정확 | 모델 대 캡처 |
| alpha 부호 반전 | `supply:leave`→`ratio:enter` 값 비트가 방향 1이면 부호 비트만, 0이면 동일. 인덱스 동일 | 캡처 |
| 단계 종류 flip·pivot·보류와 flip 원인(거리·거부) | 정확 | 모델 대 `step:enter.kind`·`selected.leave`·theta |
| 이탈 상태 | 정수 정확 | 모델 대 `step:applied.state.vstat[leave]` |
| r·dL·dU(새 캡처) | f64 비트 | 모델 재계산 대 frame `0x1ed0/0x1ed8/0x1ee0` |
| 100h·-100h·2h·e(새 캡처) | f64 비트 | h에서 계산 대 frame `0x1ec8/0x1eb0/0x1e88/0x2080` |

NaN은 허용하지 않는다. I(`1e30`)와 ±0은 비트 그대로 둔다. 시간·병렬 분기는 없다(Threads=1). 원본 재현성은 같은 설정 독립 3회의 사건 JSON 전체 일치로 확인한다.

판별력: 모델 변조(상수·부등호·arm 선택·합산 순서·표시 대응)를 하나씩 넣어 저장 자료가 거부하는지 기록한다. 살아남은 변조는 해당 경계 미검증으로 적는다. 목적 0인 `interior.lp`는 r=+0이라 방향식이 `10*dL < dU`로 줄어 r 관련 임계값과 거부 두 arm을 가르지 못한다.

## 4. 사례와 원본 자료

| 사례 | 실제/합성 | 노리는 분기 | 자료 |
|---|---|---|---|
| `interior.lp` primal·dual 각 3회(04b-1) | 실제 | 방향 0, 거리 flip, pivot 7 | `bench/oracle/m3_primal_push_step/` |
| `interior.lp` 새 agent 1회 | 실제 | 비간섭, frame 값 | `bench/oracle/m3_push_decision/` |
| 목적 있는 LP primal·dual 각 3회 | 실제 | 방향 1(r>0 절 포함), 가능하면 거부·이탈 arm | 같은 위치 |

선택 기준: feasible ratio만, batch 0, x87 없음, 재호출 없음. 후보 LP는 탐색 기록을 `history/`에 남기고, 모델 변조를 가장 많이 거부하는 LP를 고른다.

## 5. 재실행 명령

worktree 루트에서 실행한다. Gurobi는 한 번에 한 프로세스만 실행하며 캡처 도구는 기존 파일을 덮어쓰지 않는다.

```bash
D=bench/oracle/m3_push_decision
python3 tools/push_slot_reach.py                       # 슬롯 쓰기→읽기 도달, 종료 0
python3 tools/primal_push_decision_check.py --old      # 04b-1 저장 캡처만 재생(Gurobi 불필요)
tools/gurun.sh node tools/primal_push_decision_capture.js primal $D/interior.lp $D/<새>.json
python3 tools/primal_push_decision_check.py --noninterference $D/<새>.json bench/oracle/m3_primal_push_step/primal-1.json
for k in primal dual; do for i in 1 2 3; do
  tools/gurun.sh node tools/primal_push_decision_capture.js $k $D/face5.lp $D/face5-$k-$i.json
done; done
python3 tools/primal_push_decision_check.py --self-test --mutation <새 경로>.json
python3 tools/primal_push_decision_check.py --probe $D/history/<이름>-<kind>-1.json   # 탐색 캡처, 합격 근거 아님
```

캡처 도구의 넷째 인자는 추가 파라미터 JSON이다. 이번 캡처에서는 쓰지 않았다.

## 6. 실행 결과

- 환경: x86-64 Linux(WSL2), node·Frida는 04b-1과 같다. 원본 ELF 해시는 §2. 도구·C·캡처·탐색 해시는 [manifest](../../bench/oracle/m3_push_decision/manifest.json)에 있다.
- 슬롯: `push_slot_reach.py` 종료 0. 도구가 §2.1 표의 쓰기→읽기 대응을 assert한다(읽기 주소 하나를 바꾸면 종료 1). 간접 jmp 0, 진입에서 쓰기 없이 닿는 읽기 0, 이동 구간 쓰기는 frame `-0x30…-0x10`(인자)과 `0x1ed0/0x1ed8`뿐이다.
- 04b-1 저장 재생(`--old`): primal·dual 각 9단계 모두 일치. 방향 0×9, 거리 flip 2, 승인 pivot 7, 이탈 arm `N`(a≥0 5·a<0 2). r은 9단계 모두 +0이다.
- 비간섭: `interior-primal-noninterference.json` 87사건에서 `frame`을 빼면 04b-1 `primal-1.json` 사건 JSON 전체와 같다. 9단계의 frame r·dL·dU와 100h·-100h·2h·0.01h가 모델 계산과 비트 일치했다. 세 슬롯 값의 실행 확인이다. 잠금 대기로 7분 46초 걸렸다.
- 목적 LP 탐색(합격 근거 아님, `history/`): 9개 LP×primal-first 1회, 일부 dual-first 1회. 모두 판단 모델과 일치했다.

| LP | 목적 | 결과 |
|---|---|---|
| obj2·obj2b·obj3 | 2–3개 변수 | 단계 6–10, r=0(목적 변수가 최적에서 bound nonbasic) |
| objall | 12개 변수 무작위 | push 단계 0 |
| face1·face3·face5 | 제약 c1·c3·c5 행 최대화 | 방향 1 6·2·3건, r≠0 9·6·7건 |
| narrow5·fixed5 | face5+좁은 bound·고정 변수 | 방향 1 2·3건. 거부·고정폭·`X` 미발동 |

  history 단일 실행에서만 본 분기: face1 두 순서의 방향 1 거리 flip 각 2건, narrow5의 첫 절(bound)로 만든 방향 1 1건과 두 절 모두 참 1건(거리 flip). 독립 3회가 아니라 합격 근거가 아니다.

  face5를 골랐다. 변조 거부 수(8)가 가장 많고 `L` arm과 음수 theta의 `-1e-5` 승인 경계가 있다.
- face5 원본 독립 3회: primal-first(Crossover=3) 109사건, dual-first(Crossover=1) 107사건. 순서별 사건 JSON이 3회 같다. 두 순서 모두 11단계, 방향 0×8(r<0 4·r=0 4)·방향 1×3(모두 r>0 절, 첫 절 0건), r≠0 7건, 승인 pivot 11, flip 0, 이탈 arm `N`(a≥0 5·a<0 5)·`L`(a≥0 1). frame 7개 값이 11단계 모두 모델과 비트 일치. 첫 단계 theta는 음수(`bd64300000000000`)이고 `-1e-5 <= theta*pivot`로 승인됐다.
- 판정 집계(`--self-test --mutation`, 종료 0): 04b-1 2사례와 face5 2사례, 총 40단계 불일치 0.
- 캡처 변조 11종(바이너리·agent·모델 해시, 합성 표시, 파라미터, 사건 비움, x87 사건, batch, frame 누락, 사건 값, run 중복)과 판단 변조 13종(방향, alpha 부호, 단계 종류, theta, cost, 이탈 상태, frame 7종)을 모두 거부했다. 첫 차이의 사례·단계·필드·기대·실제값을 출력한다.
- 모델 변조 20종([기록](../../bench/oracle/m3_push_decision/model-mutation.json)) 중 8종이 거부됐다. 성격이 다르므로 나눠 센다.
  - 판단으로 거부 3종: `0<r`→`<=`(04b-1, r=0 단계), `-1e-5`→0(face5 음수 theta), `L`(a≥0)→-2.
  - 중간값 비트로 거부 1종: r 합산 역순(frame r).
  - 상수 값으로 거부 4종: e·거부 상수 2종·2h. 검사기가 변조 상수로 기대 frame을 만들기 때문이며 기본 frame 대조와 같은 정보다. 그 상수를 읽는 판단이 갈렸다는 뜻이 아니다.
- 살아남은 12종은 저장 자료가 판별하지 못하는 경계다. 방향식 계수 10과 `<`·`-(r*dL)` 부호, 거리 flip의 `1e-10`·`<=`, 거부 arm 선택 두 종, pivot `1e-6`, 고정폭 `<=`, a<0 기본 -1, `X`→-4, `U`의 `<`.
- 회귀: 04b-1 `step.check`(primal·dual 3회)는 `--old`·전체 검사 안에서 실행돼 통과했다. `node --check` 두 JS, `py_compile`, `git diff --check` 통과. 04b-1·04c 도구와 저장 자료는 바꾸지 않았다.
- 실행하지 않은 검사: Cargo·clippy·Python 확장. Rust·Python 코드 변경이 없다.

## 7. 판정과 인수인계

- 작업 수행: 완료.
- 오라클 판정: 계약 범위 **통과**(방향 1은 r>0 절 3건). 세 슬롯은 h의 상수배이고(asm·CFG·frame 비트), 판단 모델이 실제 LP 2개×2순서×3회의 40단계에서 방향·alpha 반전·단계 종류·theta·이탈 상태를 원본과 같게 재구성했다. 방향 1과 r≠0 절은 실행 관측했다.
- 미관측·미검증: 거부 판정 발동(9개 LP 0건, 100h·-100h는 매 pivot 단계에서 읽히지만 조건이 참인 적 없음), pivot 보류, 이탈 상태 `A`·`U`·`B`·`X`·고정폭·`L`(a<0) arm, 위 생존 변조 12경계. `0x1e88`(2h)을 읽는 네 곳은 모두 미관측 arm(`L` a<0·`B`·`A`·`U`)에 있어 2h는 값만 확인했고 판단에 쓰인 것은 미관측이다. 거리 flip의 방향 1과 첫 절(bound)로 만든 방향 1은 대상 캡처에 없고 history 단일 실행에만 있다. theta=I 재호출에서 거부 arm이 `r13b`를 쓰는 것은 asm만이다. 위반 basis·infeasible ratio·batch·x87은 범위 밖이다.
- 후속: M3-04c-6(Rust 방향식·거리 flip·승인·이탈 상태, 이 오라클 재생)은 착수 가능하다. 거부·보류·미관측 arm은 퇴화 basic을 강제한 합성 사례나 다른 LP 탐색으로 M3-04b-3에 둔다. M3-04c-5(재진입)는 그대로다.
- branch `work/m3-04b-2`, worktree `../rsolver-m3-04b-2`. 결과·인수인계·진행·로드맵은 이 branch에서 갱신해 `--no-ff`로 병합한다.
- 교훈: `docs/lessons/original.md`(목적과 r, 세 슬롯·거부 arm), `tooling.md`(재사용 pane의 이전 끝 표시).
