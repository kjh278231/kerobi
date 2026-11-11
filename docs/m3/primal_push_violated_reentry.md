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

## 6. 실행 결과

- 환경: x86-64 Linux(WSL2, glibc 2.35), Gurobi 13.0.3(SHA-256 위와 같음, 체험판 라이선스. 직접 호출이라 라이선스 오류 없음), 기준 main `dcc8c0a`.
- 계약 뒤 바꾼 것: 없음. 사례·비교 규칙·사전 기대를 계약대로 유지했다.
- 첫 하네스 동작: scratch 시험 캡처 1회(근거로 쓰지 않음)에서 대조군 131사례가 저장 자료와 같고 목표 100사례가 사전 기대와 모두 일치했다. 기대값은 그 뒤에도 고치지 않았다.
- 원본 캡처: `tools/gurun.sh ../.venv/bin/python tools/push_violated_reentry_oracle.py --capture bench/oracle/m3_violated_reentry/reentry-{1,2,3}.json` 3회 순차, 각 종료 0. 세 파일 바이트 동일(SHA-256 `3bbbbac88a85176481d51f53b2564cbb8dfb0979f30ed50030084b66eef4417e`). 도구 `7aa92927…`.
- 대조군: `--controls` 종료 0. 04c-2 `054eef` 57사례와 04c-7 `056973` 74사례가 3회 모두 저장 `ratio-1.json`과 8필드 같고 S·factor 보존.
- 목표 합격: `--acceptance` 종료 0. 목표 100사례(054eef 27종·056973 23종 × sparse·dense) 전부 사전 기대와 일치. 재진입 054eef 36·056973 32사례(2반복 work 비트 일치), 요청 2 054eef 20·056973 4사례(`S+0x294=2`·`S+0x2f8=-1`), 나머지는 `0`·`7` 유지. 입력·S(두 칸 밖)·factor 보존 참. skip·누락 0.
  - 확인한 원본 동작: `054eef` 재진입이 `055f68`과 같다(2차 패스 선택·B 음수 alpha, 유한 cap flip, 후보 있는 상한 I이면 2차 마지막 기록 위치·1차 theta·pivot, 후보 없으면 1차 선택, 요청은 `12dd49`>16만, 3차 패스 없음, 계수 경계·임계·0.1 동률은 반환, 새 tol `tol·(max·coef)`, maxsd NaN). `056973` 재진입은 반복 전체(1단계·quickselect·2단계·대체)를 다시 돈다: 1회째 t=max breakpoint에서 작은 pivot → 2회째 quickselect·2단계 선택(`ret-reentry-qs`), 2회째 대체 최소 ratio(`ret-reentry-pick`), 2회째 후보 0·기울기 0이면 대체가 theta=I·pivot 0·-1(`ret-reentry-empty`·경계 직전 사례들), 2회째 후보 0·기울기<0이면 `12dd49`>16일 때 요청 2와 **-1**, 아니면 1회째 선택 0과 1회째 theta·pivot(`ret-reentry-none-*`).
- 검사기 변조: `--self-test` 종료 0, 30종 거부([기록](../../bench/oracle/m3_violated_reentry/self-test.json)): 해시 0(ELF·입력·도구·C·기반 도구), 사건 비움, 출처 natural, 필드 누락, 1회만 변경, 2회만, 대조군 theta·요청, 목표 위치·theta·1반복 work·NaN pivot, 요청 지움(두 함수), 요청에서 prev 반환, 유지에서 -1 반환, `S+0x2f8` 미기록, 거짓 요청, S·factor 변경, 함수별 재진입 0·요청 2 0(사례 정의와 기록에서 함께 빼 목표 검사만 거부).
- Rust: `infeasible_ratio`·`retry_ratio`·`violated_ratio`가 `factor_updates: &mut dyn FnMut() -> i32`·`refactor: &mut bool`을 받고 `i32`를 반환한다. `Reentry` 형식과 `ratio_tail`·`small_pivot`을 없애고 세 ratio가 공유 판정 `tightened()`(maxsd max, `(tol·max)·coef` 임계, 새 tol `tol·(max·coef)`)를 쓴다. `feasible_ratio`의 동작은 바꾸지 않았다(04c-5 재생 통과).
- Rust 재생: `python3 tools/push_violated_reentry_rust.py --write && cargo test -p rsolver-lp --test push_ratio && python3 tools/push_violated_reentry_rust.py --compare` 종료 0. 231사례(대조군 131 + 목표 100) 위치·행·theta·pivot·work·요청 6필드 비트·정확 일치, `12dd49` 조회 수(asm `0xef671`·`0xf0a32` 파생 기대) 일치. 입력 `bench/oracle/m3_violated_reentry/rust_input.txt` 본문 SHA-256 `c8e81e2b…`(첫 줄).
- 비교기 변조: `--self-test` 종료 0, 14종 거부(위치·theta·pivot·1반복 work·요청 누락·요청에서 prev·유지에서 -1·거짓 요청·조회 수·사례 누락·오래된 해시·빈 trace·입력 변경·갱신 수 변경).
- Rust 변이 21종([기록](../../bench/oracle/m3_violated_reentry/rust-mutation.json)): 21종 모두 거부, 모두 컴파일 성공(cargo 0)이며 모두 04c-9 재생이 거부했다. 공유 판정 8종(재진입 없음·0.1 `>=`·임계 동률·곱 순서·`f64::max`·홀수 꼬리 없음·계수 1e4·1e13)은 04c-5 재생도 거부한다. `054eef` 7종(3차 패스·옛 tol·`>=16`·-1 반환·theta 초기화·후보 있는 상한 I·work 한 번), `056973` 6종(3차 패스·요청에서 prev·유지에서 -1·`>=16`·1회째처럼 처리·prev 미갱신).
- 기존 재생의 재진입 사례: 04c-2 `push_violated_rust.py --compare`가 재진입 2사례를 이제 거부 대신 원본과 비교해 57사례 전부 통과. 04c-7 `push_retry_rust.py --compare`는 재진입 4사례(`reentry-pick`·`reentry-keep`)를 비교해 88사례 통과하고, 입력에 factor 갱신 수가 없어 `reentry-refactor` 2사례만 빼고 04c-9 재생이 비교한다. 두 도구의 self-test 변조 `refused-accepted`를 `reentry-theta`로 바꿨다.
- 회귀(모두 종료 0): `cargo test -p rsolver-lp --test push_decision --test push_ratio && python3 tools/primal_push_decision_rust.py --compare`(175단계·연결 불일치 0)·`--self-test`; 04c `primal_push_ratio_rust.py --compare`·`--self-test`; 04c-5 `primal_push_reentry_rust.py --compare`·`--self-test`(91사례); 04c-2·04c-7 `--compare`·`--self-test`; `primal_push_ratio_oracle.py`·`push_infeasible_ratio_oracle.py`·`push_retry_ratio_oracle.py --check … --selftest`; `cargo test --release --workspace`(129 통과·12 ignored); `cargo clippy --workspace --all-targets -- -D warnings`; `git diff --check`.
- 실행하지 않은 검사: maturin·pytest(`crates/py`·Python API 변경 없음, ratio 호출자 없음). 04c·04c-2·04c-5·04c-7 도구의 `--mutate-rust`는 재실행하지 않았다. 이번 리팩터링으로 그 변이 대상 문자열(`ratio_tail`·`small_pivot`·`let small = …` 등)이 코드에 없어 그대로는 돌지 않는다. 같은 판정 변이는 이번 기록이 네 재생을 함께 돌려 대신한다. 기존 변이 기록 파일은 당시 코드의 결과로 남긴다.
- 해시: `crates/lp/src/crossover.rs` `45a3987d…`, `crates/lp/tests/push_ratio.rs` `6046bae2…`.

## 7. 판정과 인수인계

- 작업 수행: 완료.
- 동등성: 계약 범위 **통과**(원본 합성 직접 호출, 저장 기준 재생, 비트·정확 비교). `054eef`·`056973` 재진입이 더는 Rust에서 거부되지 않는다.
- 미검증·제외: `056973` 재진입 2회째의 cap 종료(§1, 합성 설계 불가·1회째와 같은 코드), 계수 경계 10(판별 불가), `12dd49` 호출 여부 자체(asm 파생), 자연 crossover의 재진입(합성만), push 루프 연결(요청 2 뒤 push가 반환값을 `S+0x2f8`에 덮어쓰는 순서 포함).
- 후속: push 루프 연결(Rust push 루프. `Recall` 뒤 열 재반전·ratio·`push_recall_step`, `refactor`면 `S+0x294=2`, 반환값을 leaving으로 기록하는 `0x1c2655` 순서), 또는 M3-04d(dual push 조사).
- branch `work/m3-04c-9`, worktree `../rsolver-m3-04c-9`.
- 교훈: `docs/lessons/original.md`(crossover: `056973` 재진입 요청 반환 -1·push의 `S+0x2f8` 덮어쓰기), `docs/lessons/tooling.md`(공유 함수 리팩터링 뒤 변이 앵커 겹침).
