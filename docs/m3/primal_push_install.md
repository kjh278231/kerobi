# M3-04f-1 primal push 해 설치 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 선행: [1c 조건수](primal_push_condition.md), [1d 외부 값 경로](primal_push_init_ext_rust.md), [24 실제 backend 루프](primal_push_loop_lu.md).

## 1. 범위와 주장

- 작업 종류: 오라클 확장(원본 내부 경계 캡처) + 구현(Rust 이식) + 검증. M3-04f(cleanup 진입과 basis 설치 한 경로)의 첫 하위 작업으로, push 함수 `134a87` 안의 **해 설치**(루프 종료 `LAB_002c4050` → 반환)를 맡는다. `13f716`의 cleanup 선택과 `0cda5b` 설치는 후속(04f-2 이후)이다.
- 완료할 동작: 루프가 끝난(또는 후보 0으로 초기화에서 바로 온) 상태에서 원본이 하는 일 — first≠0일 때 basic을 x·slack에 기록(열은 lb > −I면 lb 더함), 위치 순 위반 수·PInf 재계산과 S+0x120 기록, `12f282`(→ double `116fd7`) 재호출, retry 판정(attempt = 0이면 cond > 1e14·P > 1e10·P·cond > 1e16, 항상 cond > 1e30; P는 param_3 = 2일 때 PInf, 아니면 1), 남은 후보 수의 부호 결정(자유 후보가 있으면 음수, 없으면 `(S+0x4b8)+0x100 → +0x2ba0 = −1`일 때 `max(5, S+0x46c/10)`과 비교), 양수면 앞쪽 후보 vstat −1/−2 지정(lb+ub > 2x면 −1), 음수면 vstat −1/−2 전체를 bound로 스냅 — 를 Rust `push_install`로 옮긴다.
- 주장할 비교: (A) **설치 경계**: 원본 `install:enter` 상태 + 설치 중 조건수 출력만으로 Python C 규칙 재계산과 Rust가 `push:leave`의 x·slack·vstat·S+0x120·retry·remaining과 비트/정확 일치. (B) **연결**: 1d 하네스에서 push 진입 → 초기화(1c Rust 조건수) → `push_loop`(24의 `SerialBackend`) → 설치(조건수는 Rust `116fd7`, 루프 뒤 factor)까지 원본 재주입 없이 `install:enter`·설치 내부 `116fd7` 줄·`push:leave`가 일치. 루프 중간 줄은 24에서 검증한 범위라 이번 비교 대상이 아니며, 설치 진입 상태 일치가 루프 결과의 일치를 대신 확인한다.
- 포함 분기와 입력 전제: double(S+0x28 = 0), Threads = 1(batch 0), 1c 사례 집합의 자연 실행. 실제로 지나는 분기(first = 0/1, 후보 0 직행, retry, remaining 양수/음수/0, 자유 후보, 2ba0 = −1 여부)는 결과 §6에 관측 여부로 적는다. 연결(B)은 Rust factor가 있는 호출(first = 1 또는 이전 호출 factor 이어받기)만 대상이며, 루프가 미구현 경로(품질 helper `089f61`, factor 오류 등)를 만나면 그 호출은 **실패(미지원)**로 따로 집계하고 통과에 넣지 않는다.
- 제외와 후속: x87(`256781`·`23e604`), 로그 `d07d1`·work·시간, 할당 실패, `13f716`의 반환 소비(remaining 부호에 따른 cleanup mode 선택)와 `0cda5b`·`02f7e1`·`0325b4` 설치(04f-2), dual push(04d), 재시작(04g), solver 경로 연결.
- 선행 commit: main `085afe3`(1c 병합). 미커밋 선행 변경 없음(main의 미추적 `.claude/settings.local.json`은 무관).

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 ELF file offset(Ghidra = +0x100000).
- 디컴파일 `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c` C 2419–2582, asm `0x1c4050–0x1c44e3`·`0x1c4774–0x1c48af`. 초기화의 후보 0 직행은 C 917(`goto LAB_002c4050`), 루프 종료는 C 2065·2419.
- 스택 slot(prologue asm `0x1c0098–0x1c0284`로 확인): 0x1e90 남은 후보 수, 0x1fb0 remaining 작업값, 0x2050 param_7(first), 0x1fa8 x(param_5), 0x2058 slack(param_6), 0x1fa0 attempt(param_4), 0x1f98 param_3, 0x1fb8 후보 목록(S+0x208→+8), 0x1fc0 vstat(S+0x168), 0x2010 basic(S+0x1e8), 0x1fe8 header(F+0x18), 0x1ff0 폭((S+0x208)+0x10), 0x1f90 lb, 0x1f88 ub, 0x1f00 −I, 0x1f08 −τ, 0x1fc8 I(S+0x370), 0x2070 τ(S+0x380), 0x2068 m, 0x2060 n, 0x1f58 n+m, 0x20e0 &condition, 0x20e8 &retry.
- asm으로 정한 규칙:
  1. first ≠ 0: 위치 i = 0..m, v = header[i], b = basic[i]. v < n이면 `x[v] = (−I < lb[v]) ? lb[v] + b : b`(`comisd lb, −I; jb`), 아니면 `slack[v−n] = b`(0x1c40e6–0x1c412d).
  2. PInf(0x1c414d–0x1c420f): 위치 순. lb ≤ −I이고 ub ≥ I면 건너뜀. b < −τ면 수 +1, P −= b. 아니면 τ > b, w−τ > b, w ≥ I, τ+w > b 중 하나면 건너뜀, 아니면 수 +1, P += (b − w). m = 0이면 P = 0. S+0x120 = P.
  3. `12f282(S+0x4b8, &condition, env)` 무조건(0x1c4271). P' = (param_3 = 2) ? P : 1.0. attempt = 0이고 (cond > 1e14 또는 P' > 1e10 또는 P'·cond > 1e16)이거나 cond > 1e30이면 retry = 1, remaining = 남은 후보 수(양수 그대로), 설치 종료.
  4. 남은 후보 수 c = 0이면 remaining = 0. c > 0이면 목록 앞 c개를 순서대로 보며 첫 자유 후보(lb ≤ −I이고 ub ≥ I)가 있으면 remaining = −c. 없으면 `2ba0 = −1`일 때 t = max(5, trunc(S+0x46c/10))이고 c ≥ t면 c, 아니면 −c. `2ba0 ≠ −1`이면 c.
  5. remaining > 0: 목록 앞 remaining개 후보 v에 `vstat[v] = (lb[v]+ub[v] > 2·x[v]) ? −1 : −2`(x는 x·slack 연속 별칭, 0x1c4830–0x1c4877).
  6. remaining < 0이고 n+m > 0: 모든 j에서 vstat −1이면 x[j] = lb[j], −2면 x[j] = ub[j](0x1c439a–0x1c447e, 2개씩 펼침).
  7. `*param_10 = remaining`, 반환값 local_168(0).
- 시작 경계: `0x1c4050`(설치 진입). 종료 경계: `134a87` 반환. 설치 중 `116fd7`은 `0x1c4271` 직전~`0x1c4276` 직후 사이로 식별한다.
- 입력 상태: 후보 목록 앞 c개, header·basic·폭·vstat·x·slack, lb·ub, τ·I, first·attempt·param_3, S+0x46c, 2ba0, factor(조건수). 보존: header·basic·폭은 설치가 바꾸지 않는다.

## 3. 비교 계약

| 필드·사건 | 의미·좌표·순서 | 비교 규칙·근거 | 캡처·재구성 방법 |
|---|---|---|---|
| `install:enter` | c(0x1e90), 목록 앞 c개(+목록 포인터가 S+0x208→+8인지), header·basic·폭·vstat·x·slack, S+0x120·0x2a0·0x46c, 2ba0, first·attempt·param_3, τ·I | 정확·비트 | 새 agent `0x1c4050` 훅 |
| `install:condition` | 설치 중 `12f282` 출력 | 비트 | `0x1c4271`·`0x1c4276` 훅 |
| `icond:*` | 설치 중 `116fd7` 진입(F+0xc·정렬)·FTRAN/BTRAN 입출력·반환 | 1c와 같은 규칙(정확·비트) | 1c 훅을 설치 단계에도 켜고 이름만 `icond:`로 |
| `push:leave` | ret, condition, retry, remaining + vstat·x·slack·S+0x120·header·basic | 정확·비트 | 반환 훅에 전체 상태 추가 |
| C 재계산(검사기) | §2 규칙 1–6으로 `install:enter` → `push:leave`, `icond`는 1c `check_cond` | 비트·정확 | Python |
| Rust 연결 줄 | `inst`(설치 진입 상태)·`icnorm/icft/icbt/icest/icres`·`leave` | 비트·정확 | 1d 하네스 확장 |

- 수치 오차 허용 없음. NaN은 원본 자료에 나오면 실패로 기록.
- 시간·work·병렬: Threads = 1. 로그 `d07d1`의 위반 수는 레지스터라 비교하지 않는다(PInf와 같은 루프에서 계산; Rust는 수를 함께 낸다).
- 재현성: 1c와 같은 사례·설정으로 새 agent 독립 3회. 3회 같고, `install:*`·`icond:*`를 빼고 `push:leave`를 1c 필드로 줄인 투영이 1c run 1과 같아야 한다(비간섭).

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 |
|---|---|---|---|
| 1b 15사례(arms·retry·interior·adlittle·scsd1·capri·adlittle_x1 + 합성 8) | 실제 7 / 합성 8(1b 편집) | first 0/1, retry 재호출, 조정 | 1b·1c와 같음 |
| Netlib 소형 45 자연(Crossover=3) | 실제 | 루프 End·후보 0 직행·remaining 부호 | `push_init_runs.SURVEY45` |

- 자료: `bench/oracle/m3_push_install/`(사례 × 3, manifest). 탐색 1회 결과는 기준 자료가 아니다.
- 적용하지 않은 범주: x87, 할당 실패, threads > 1. 자연에서 나오지 않는 분기(예: 2ba0 ≠ −1, 자유 후보)는 미관측으로 기록하고 합성 편집을 추가할지는 탐색 뒤 Rust 비교 전에 결정한다.

## 5. 재실행 명령

- 원본 캡처: `tools/hrun.sh --timeout 580000 -- 'python3 tools/push_install_runs.py'` 및 `--netlib`(작업 worktree, Gurobi 한 프로세스).
- 검사기: `python3 tools/push_install_check.py --controls --acceptance`, `--self-test`, `--write-manifest`.
- Rust: `python3 tools/push_install_rust.py --write && cargo test -p rsolver-lp --test push_init_ext && python3 tools/push_install_rust.py --compare`, `--self-test`, `--mutate-rust OUT`.
- 회귀: 1c `push_cond_check.py --controls --acceptance`·`push_cond_rust.py --write/--compare`, 1d `push_init_rust.py`, 24 `push_loop_rust.py --lu`, 1a `--init`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`.

## 6. 실행 결과

(작성 예정)

## 7. 판정과 인수인계

(작성 예정)
