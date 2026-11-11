# M3-04c-9 `054eef`·`056973` 작은 pivot 재진입 Rust

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다. §1–§5는 원본을 실행하기 전에 썼다.

## 1. 범위와 주장

- 작업 종류: 오라클 구축 + 구현. 기준 main `dcc8c0a`, branch `work/m3-04c-9`, worktree `../rsolver-m3-04c-9`. 미커밋 선행 변경 없음. 점유 `claude-m3-04c9-20261112`(사용자 "다음 M3 스프린트 진행해 너가 작업자야" 지시, M3 인계의 두 후보 중 범위가 닫힌 쪽).
- 새 행: 스프린트 목록에 M3-04c-9를 더한다. 04c-5 §7 후속("`054eef`·`056973` 재진입 Rust는 같은 방식으로 별도 행")이다.
- 완료할 동작: 위반 basis ratio `054eef@0xeefa0`와 재시도 ratio `056973@0xf0120`의 작은 pivot 재진입 패스. Rust `infeasible_ratio`·`retry_ratio`·`violated_ratio`가 지금 `Err(Reentry)`로 거부하는 조건을 원본대로 복원하고 `Reentry`를 없앤다.
- 포함 분기:
  - `054eef`: 1차 패스 끝 `0.1 > |pivot|`·`(tol·max)·coef > |pivot|` 판정, 계수 표 경계 1e4·1e7·1e10·1e13, 임계·0.1 동률, max의 홀수 꼬리·음수 alpha·NaN(`maxsd`), 새 tol `tol·(max·coef)`, 2차 패스 후보 선택(B 표시 음수 alpha 포함), 유한 cap flip, 2차 패스 후보 없음·후보 있는 상한 I에서 `12dd49` 요청 2와 반환 `ebx`(2차 패스 마지막 기록 위치, 없으면 1차 선택), 3차 패스 없음.
  - `056973`: 같은 판정·계수·새 tol(asm 명세 §6), 재진입 반복의 1단계·quickselect·2단계·대체 최소 ratio 전부, 2회째 후보 없음+기울기<0에서 `12dd49` > 16이면 요청 2와 **-1 반환**, 아니면 이전 선택 반환(theta·pivot 미기록), 2회째 기울기≥0 후보 없음의 대체(-1·theta=I·pivot 0), max scan의 NaN.
  - `violated_ratio`: `054eef` → (위치<0 또는 |pivot|<0.001) → `056973` 게이트는 그대로. 요청 2 플래그는 두 함수가 같은 `refactor`에 쓴다.
- 제외:
  - push 루프 연결. push는 ratio 반환값을 바로 `S+0x2f8`에 쓴다(`0x1c2655 mov %eax,0x2f8(%r14)`, 아래 §2.1). 그래서 `054eef`가 요청 2를 쓰고 위치≥0을 돌려주면 `S+0x2f8=-1`은 덮인다. 이 순서는 push 연결 작업에서 다룬다.
  - `056973` 재진입 2회째의 cap 종료(quickselect 뒤 `cap <= t`): 2회째 후보가 cap 미만으로만 걸러지는 표시에서는 t<cap이고, sparse N/U만 cap을 거르지 않는데 그 행은 1회째에도 같은 breakpoint를 만들어 1회째가 먼저 cap 종료한다. 1회째와 같은 코드이며 04c-7 `cap-exit`가 실행 근거다. 재진입 합성 사례로는 설계하지 않는다.
  - 강화 계수 경계 10: 04c-5 §2.3과 같은 이유로 판별 불가.
  - 자연 crossover의 재진입(합성만), 열길이 가중(두 함수에 없음).
- 선행: M3-04c-2(`054eef` 합성 57호출 `bench/oracle/m3_push_violated/ratio-{1,2,3}.json`), M3-04c-7(`056973` 합성 74호출 `bench/oracle/m3_push_retry/ratio-{1,2,3}.json`, asm 명세 [primal_push_retry_asm.md](primal_push_retry_asm.md)), M3-04c-5(같은 방식의 `055f68` 재진입).

## 2. 원본과 상태 경계

### 2.1 원본·ABI

- Gurobi 13.0.3 `../.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so`, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C `../re/decomp/c/000eefa0_PRIVATE0000000000054eef.c`(436줄), `../re/decomp/c/000f0120_PRIVATE0000000000056973.c`, `../re/decomp/c/001bb6f0_PRIVATE000000000012dd49.c`. asm `objdump -d --start-address=0xeefa0 --stop-address=0xef960`, `056973`은 [asm 명세](primal_push_retry_asm.md).
- ABI는 04c-2·04c-7 하네스와 같다. `xmm0..3`=h·tol·I·cap, `rdi`=S, `esi`=m, `rdx`=열, `rcx`=basic, `r8`=폭, `r9`=표시. 스택은 `054eef`가 후보 scratch·ratio scratch·theta*·pivot*·work*, `056973`이 theta*·pivot*·후보 scratch·ratio scratch·work*.
- push 호출점: `054eef` `0x1c2573`, 반환 `test %eax; jl 0x1c25b4` 또는 `0.001 > |pivot|`(`0x1c25aa`)이면 `056973` `0x1c263a`, 그 뒤 `0x1c2655`에서 반환값을 `S+0x2f8`에 쓴다.
- 시작: 함수 직접 호출 직전. 종료: 반환 직후. 호출마다 새 S(0x500바이트)·factor 두 객체.

### 2.2 asm으로 확인한 `054eef` 재진입

| 위치 | 내용 |
|---|---|
| `0xeefb0`·`0xeefb7` | `ebx`(반환 위치)=-1, `sil`(첫 패스)=1. 루프 머리 `0xef01a`는 후보 수만 0으로. `ebx`·theta·pivot은 패스를 넘어 남는다 |
| `0xef562` 등 | 후보를 기록할 때마다 `ebx`=그 위치 |
| `0xef349–0xef359` | pivot=`|val[ebx]|` 저장(패스마다) |
| `0xef35e–0xef37a` | `sil`=0이면 반환. `comisd |pivot|,0.1; jbe 반환` |
| `0xef380–0xef3f8`, `0xef5bd–0xef632` | max: sparse는 count개, dense는 m개. 짝마다 `maxsd`(dst>src?dst:src) 뒤 `comisd; ja` 유지, 홀수 꼬리 `maxsd` |
| `0xef3fc–0xef47f` | 계수 `10>max`→0.1 … `1e13>max`→1e-5, 그 밖(NaN 포함) 1e-6 |
| `0xef483–0xef493` | 임계 `(tol·max)·coef`, `comisd |pivot|,임계; jbe 반환` |
| `0xef499–0xef525` | 계수를 다시 골라 `max·coef`, `tol *= 그 값`(= `tol·(max·coef)`, C의 왼쪽부터와 다름), `sil=0`, 루프 머리 |
| `0xef63b–0xef696` | 상한 I 또는 후보 0: `sil`≠0이면 -1·theta=I·pivot 0. 아니면 theta·pivot을 쓰지 않고 `12dd49(S)`, `> 16`이면 `S+0x294=2`·`S+0x2f8=-1`. 반환은 `ebx` |

`055f68`(04c-5 §2.2)과 같은 구조다. work는 패스마다 더한다(sparse `2c+4n`, dense `2c+2t+n`).

### 2.3 `056973` 재진입(asm 명세 §2·§6 요약)

- 루프 머리 `0xf01b2`에서 기울기 누적·후보 수·chosen(-1)만 초기화. prev(`ebx`)=이전 선택, theta*·pivot*은 남는다.
- 1단계 후 후보 0이고 기울기<0: 1회째는 theta=`minsd(cap,I)`·pivot 0·-1. 2회째는 `12dd49(S)`, `> 16`이면 `S+0x294=2`·`S+0x2f8=-1`·**-1 반환**, 아니면 prev 반환(theta·pivot 미기록).
- 그 밖은 1회째와 같다(quickselect, cap 종료, 2단계, 대체). 끝에서 pivot=`|val[chosen]|`, 2회째면 그대로 반환.
- 1회째 끝 판정·max·계수·새 tol은 `054eef`와 같은 형식(`0xf081a`, `0xf083a–0xf08a6`·`0xf0db1–0xf0e32`, `0xf093d`, `0xf09d9`·`0xf09df`).

### 2.4 입력 상태와 보존

- 읽는 상태: 열(희소·dense), basic 값·폭·표시, h·tol·I·cap, `S+0x148`(0.25), `S+0x470`·`S+0x498`와 factor의 `+0x18c`·`+0x1a4`.
- 변경 상태: theta*·pivot*·work*, scratch(비교 안 함), 요청 2일 때만 `S+0x294`·`S+0x2f8`.
- 보존: 입력 배열 바이트, S(두 칸 제외), factor 두 객체.

## 3. 비교 계약

| 필드·사건 | 의미 | 비교 규칙 | 캡처·재구성 |
|---|---|---|---|
| `chosen`·`row` | 반환 위치·실제 행 | 정수 정확 | ctypes 반환 |
| `theta`·`pivot` | 출력 슬롯. 호출 전 -123.0 | f64 LE hex 비트 | 슬롯 값 |
| `work` | 시작 0, 계수 0.25 | 비트. 재진입이면 두 반복 합 | 슬롯 값 |
| `refactor`·`leaving` | `S+0x294`·`S+0x2f8`. 호출 전 0·7(대조군은 04c-2 0·0, 04c-7 0·7) | 정수 정확 | S 버퍼 |
| `inputs_preserved`·`workspace_preserved`·`factor_preserved` | 입력, S(두 칸 제외), factor | 참 | 호출 전후 bytes |

- 사전 기대: `cases()`에 사례마다 위치(행)·theta·pivot·요청·반복 수·work 단위(sparse·dense)를 asm으로 미리 정한다. 원본과 다르면 검사 실패이고 기대를 결과에 맞춰 고치지 않는다(실패로 기록).
- 재진입 확인: work = 0.25·Σ(반복 단위). 1반복과 2반복의 work가 다르게 설계해 원본 work 비트로 반복 수를 판정한다.
- NaN: 열 값에만. 출력 theta·pivot·work가 NaN이면 실패. ±0·I는 비트 비교.
- 시간·RNG·병렬: 해당 없음.
- 원본 재현성: 독립 프로세스 3회, 모든 사건 필드 일치.

## 4. 사례와 원본 자료

모두 **원본 합성** 입력이다. 목표 사례 기본값 h=1e-6, tol=1e-9, I=1e30. 사례마다 sparse·dense.

| 사례군 | 함수 | 노리는 분기 | 기대 |
|---|---|---|---|
| `control-inf-*` | 054eef | 04c-2 57호출 그대로 | 저장 `m3_push_violated/ratio-1.json` 9필드 동일 |
| `control-ret-*` | 056973 | 04c-7 74호출 그대로 | 저장 `m3_push_retry/ratio-1.json` 9필드 동일 |
| `inf-reentry-pick`·`-negB` | 054eef | 1차 2e-9 → tol 1e-8 → 2차 2e-8 행(N·B 음수) | 행 1, 2차 theta·pivot |
| `inf-reentry-negmax` | 054eef | max가 홀수 꼬리의 음수 alpha | 행 1(0.5) |
| `inf-reentry-flip` | 054eef | 2차 상한 ≥ 유한 cap | -1, theta=cap |
| `inf-reentry-inf-bound`(17)·`-keep`(3) | 054eef | 2차 후보 있는 상한 I | 위치=2차 후보, theta·pivot 1차 값 |
| `inf-reentry-none-*` | 054eef | 2차 후보 0, factor 470=17·16, 498=17, 둘 다 없음 | 1차 선택, 요청은 17만 |
| `inf-pivot-*-0.1`·`inf-thr-*`·`inf-scale-*` | 054eef | 0.1·임계 동률·계수 경계 | 경계 값은 1패스, 직전 값은 재진입+요청 |
| `inf-mul-order` | 054eef | 새 tol 곱 순서 | 2차가 THR 행 |
| `inf-nan-max-last`·`-mid` | 054eef | maxsd NaN | 마지막이면 1패스, 가운데면 재진입+요청 |
| `inf-second-bound-tie`·`inf-candidate-inf-bound` | 054eef | 1차 꼬리 경계 | 행 1 / -1·I |
| `ret-reentry-pick` | 056973 | t=0 선택 2e-9 → 2회째 대체 최소 ratio | 행 1, theta 1000 |
| `ret-reentry-qs` | 056973 | 1회째 t=max breakpoint, 2회째 quickselect·2단계 | 행 2, theta 5 |
| `ret-reentry-empty` | 056973 | 2회째 후보 0·기울기 0 | -1, theta=I, pivot 0 |
| `ret-reentry-none-*` | 056973 | 2회째 후보 0·기울기<0, factor 470=17·16, 498=17, 없음 | 요청이면 -1, 아니면 1회째 선택 0, theta·pivot 1회째 값 |
| `ret-reentry-negmax` | 056973 | 홀수 꼬리 음수 max | 행 1 |
| `ret-pivot-*`·`ret-thr-*`·`ret-scale-*` | 056973 | 0.1·임계·계수 경계 | 경계 값은 1회, 직전 값은 재진입 → -1·I |
| `ret-mul-order` | 056973 | 새 tol 곱 순서 | 2회째 THR 행 |
| `ret-nan-max-last`·`-mid` | 056973 | maxsd NaN | 1회 / 재진입 → -1·I |

- 입력: 새 도구 `tools/push_violated_reentry_oracle.py`의 `cases()`. 04c-2·04c-7 도구는 자기 해시로 고정돼 있어 고치지 않고 `cases()`를 import해 대조군으로 쓴다.
- 원본 자료: `bench/oracle/m3_violated_reentry/reentry-{1,2,3}.json`(스키마 `M3-04c-9-violated-reentry-v1`).

## 5. 재실행 명령

worktree 루트에서. Gurobi는 `tools/gurun.sh`로 한 프로세스씩.

```bash
D=bench/oracle/m3_violated_reentry
for i in 1 2 3; do tools/gurun.sh ../.venv/bin/python tools/push_violated_reentry_oracle.py --capture $D/reentry-$i.json; done
python3 tools/push_violated_reentry_oracle.py --controls $D/reentry-{1,2,3}.json     # 대조군: 04c-2 57·04c-7 74 불변
python3 tools/push_violated_reentry_oracle.py --acceptance $D/reentry-{1,2,3}.json   # 목표: 사전 기대·재진입·요청 2
python3 tools/push_violated_reentry_oracle.py --self-test $D/reentry-{1,2,3}.json    # 검사기 변조 거부
python3 tools/push_violated_reentry_rust.py --write && cargo test -p rsolver-lp --test push_ratio && python3 tools/push_violated_reentry_rust.py --compare
python3 tools/push_violated_reentry_rust.py --self-test
python3 tools/push_violated_reentry_rust.py --mutate-rust <새 경로>.json
```

회귀: 04c-2 `push_violated_rust.py --compare`·04c-7 `push_retry_rust.py --compare`(재진입 사례가 이제 거부 대신 원본과 비교돼야 한다), 04c·04c-5 재생, `cargo test -p rsolver-lp --test push_decision --test push_ratio && python3 tools/primal_push_decision_rust.py --compare`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`.

목표 합격 검사는 다음이면 0이 아닌 값으로 끝나야 한다: 함수별 재진입 사례 0, 함수별 요청 2 사례 0, 필수 필드 누락, 해시 0, 사건 배열 비움, 3회 미만, 합성 출처가 아닌 기록.
