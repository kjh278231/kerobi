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

- 환경: x86-64 WSL2 Linux, Gurobi 13.0.3(위 SHA-256), Threads = 1·Seed = 0·Presolve = 0·Method = 2·LPWarmStart = 2. 계약 `b43f8d4`, 캡처·검사기 `3fcca1b`, Rust·비교기는 이 branch의 다음 commit.
- **범위 결정(탐색 뒤, Rust 비교 전)**: 1b·Netlib 자연 자료는 remaining 0·양수, first 0/1, 후보 0 직행만 지나고 retry·음수 remaining·자유 후보가 없었다. `+0x2ba0`가 Crossover 파라미터임을 노트(`env_param_offsets.tsv`)로 확인하고 ① Netlib 45를 **Crossover = -1(기본)**로 더했다 — 45 모두 first = 0·후보 0이라 임계값 분기는 자연으로 닿지 않았다. ② 원본 명령 합성 5종(`synthetic.json`)을 더했다: `inst_env_neg`(설치 진입에서 2ba0 = -1 → 임계값 음수·스냅), `inst_free`(남은 후보 bound를 ±I → 자유 후보 음수·스냅), `inst_retry`(설치 조건수를 1e15로 → attempt 0 retry), `inst_att1_keep`·`inst_att1_retry`(attempt 1 호출에서 1e20 → retry 없음, 1e31 → retry). 합성은 함수 출력·환경 값을 바꾼 실험이고 정상 실행 사례와 구분한다. 축소한 범위는 없다.
- 원본 실행: 정식 330회(1b 15 + 합성 5 + Netlib 45 + Crossover = -1 Netlib 45, 각 독립 3회) 모두 성공. 이 밖에 탐색 53회(훅 확인 2, Crossover = -1 45, 합성 5, grow7 21h agent 1)와, 합성 편집 기능을 넣기 전 agent로 찍었다가 근거로 쓰기 전에 버린 180회가 있다(총 563회).
- 통제(`push_install_check.py --controls`): 110사례 × 3회 사건열 동일, 1c 60사례 run 1 투영(install·icond 제거, push:leave 1c 필드)이 1c 자료와 같다(비간섭).
- C 규칙 재계산(`--acceptance`): push 호출 115, 설치 111(설치 중 `116fd7` 111회 1c `check_cond` 비트 재계산), 설치 없음 4(초기화 retry). 설치 진입 slot 별칭 11종·m·n·τ·I slot 값이 모두 맞고, x·slack·vstat·S+0x120·retry·remaining·condition 경로가 모두 일치했다. 분기 집계: first 기록 65·first = 0 46·후보 0 직행 18·PInf > 0 5·remaining 0 98·양수 9·임계값 음수 1·자유 후보 1·스냅 2·attempt 0 retry 1·attempt 1 비retry 1·1e30 retry 1. 합성의 사전 필수 분기 모두 경유.
- 검사기 판별력(`--self-test`): 변조 19종 거부(바이너리 해시·사건 비우기·run 간 차이·1c 비간섭·leave 필드·설치 입력·icond·별칭·header 보존). 첫 시도에서 `icond` 반복 0 FTRAN 출력 1비트 변조가 통과했다 — 다음 반복이 이어지면 그 값은 est 비교에만 쓰여 결과에 들어가지 않는다. 결과를 정하는 반복 1 출력으로 바꿔 거부를 확인했다(1c와 같은 방식). 규칙 변이 14종 중 필수 7종 포함 10종 거부, 생존 4: PInf 하한·τ·τ+w 경계의 동률(자료에 basic이 정확히 ±τ인 위치 없음), P = PInf(상태 2이면서 PInf > 0인 설치 없음).
- Rust (A) 설치 경계(`cargo test --test push_install` → `push_install_rust.py --compare`): **111호출·666필드 첫 비교부터 비트/정확 일치**(합성·Crossover = -1 포함).
- Rust (B) 연결(`--test push_init_ext init_install_trace`): 1c 62호출을 진입 → 초기화(1c Rust 조건수) → `push_loop`(SerialBackend) → 설치(루프 뒤 factor의 Rust `116fd7`)까지 원본 재주입 없이 재생했다. 1c 기대 줄 전체와 설치 쪽 626줄(install:enter·icond·push:leave)을 비교해 **설치까지 간 59호출 중 58호출 일치**(줄 1,583·필드 7,077), first = 0인 1호출(adlittle_x1)과 초기화 retry 2호출은 설치 연결 대상이 아니다.
- **실패 1(결과 확인 뒤 분류, 사전 명시 아님)**: `netlib/grow7` call 1의 설치 진입 header가 위치 89부터 다르다(원본 234, Rust 182). 기저 집합·x·S+0x120·반복 수는 같고 위치 순서와 basic 끝자리·폭·vstat(위치 값)이 다르다. 탐색 21h agent 캡처 1회를 24 하네스에 넣어 루프 911줄 중 906줄이 일치하고 **줄 907 마지막 round 재분해**(`036ade` 진입 S+0x2c0 = 8·0x4e0 = 7)에서 header·L/U 비영 수(원본 1424/1604, Rust 1128/1284)가 갈림을 확인했다. 설치 코드가 아니라 21g·21h 재분해 범위의 차이이며 후속 **M3-04c-21i**로 넘긴다. 비교기는 이 첫 차이 서명만 "분류된 실패"로 따로 세며, 다른 차이가 생기거나 grow7이 바뀌면 실패한다. 탐색 캡처는 기준 자료가 아니다(160MB, 저장하지 않음).
- Rust 판별력: (B) 기대 줄 변조 5,412종·(A) 666종 전부 거부(분류된 실패 호출 제외). Rust 변이 21종 중 15종 거부, 생존 6: PInf 하한·τ·τ+w 동률 3종(위와 같음), P = PInf(위와 같음), `cond ≥ 1e15`로 바꾼 retry 문턱(합성 조건수가 정확히 1e15라 구별 불가), 조건수 호출을 PInf 계산 앞으로(`116fd7`이 S+0x120을 읽지 않아 관측상 동치). 첫 변이 실행은 (B)를 분류 전 비교로 판정해 grow7 기존 실패만으로 6종을 거짓 거부로 셌고, 고친 뒤 다시 실행했다(`bench/validation/m3_push_install/rust_mutations.json`).
- 회귀: 1c `push_cond_check --controls --acceptance`(60사례·116fd7 64), `push_cond_rust --compare`(62호출·966줄·4,378필드), 1d `push_init_rust --compare`(17호출·132줄·727필드), 24 `push_loop_rust --lu --compare`(6사례·725줄·5,575필드), 1a `--init --compare`(6사례·731줄·5,743필드), `cargo test --release --workspace` 150통과·실패 0·ignored 12, `cargo clippy --workspace --all-targets -- -D warnings` 0(첫 실행에서 NaN 의미의 부정 비교 2곳 거부 → asm `comisd`/`jb` 의미 함수로 고침).
- 실행하지 못한 검사: P·cond > 1e16 항과 P' = PInf(status 2·PInf > 0 설치 없음), 임계값 양수(c ≥ max(5, S+0x46c/10), 남은 후보 5 이상 없음), NaN bound, x87·할당 실패.

## 7. 판정과 인수인계

- 작업 수행 상태: 완료.
- 동등성: **(A) 설치 경계 통과**(111호출, 비트/정확). **(B) 연결 범위 한정 통과**: 58/59 일치, grow7 1호출 **실패**(설치 이전 루프 재분해 차이, 결과 확인 뒤 분류). 실패는 통과 수에 넣지 않는다.
- 통과한 범위: double·Threads = 1에서 push 해 설치의 x·slack 기록, PInf·S+0x120, 설치 조건수(`116fd7`, 루프 뒤 factor — 1c가 남긴 "초기화 밖 116fd7 호출" 중 `0x1c4276`), retry(attempt 0 cond·1e30), remaining 부호(0·양수·임계값 음수·자유 후보 음수), vstat -1/-2 지정, 음수 스냅. 1c Rust 연결이 push 진입부터 반환까지 원본 재주입 없이 58호출에서 맞는다.
- 남은 한계: P 항·임계값 양수·±τ 동률 미관측(C·asm 근거만), grow7 재분해(21i), `0x1c8a0e` 근처 다른 `116fd7` 호출, cleanup 소비(`13f716` mode 선택·`0cda5b` 설치, 04f-2), x87·work·시간·할당 실패, solver 경로 연결.
- 후속: **M3-04c-21i**(grow7 마지막 round 재분해 피벗 순서), **M3-04f-2**(`13f716`의 remaining 소비와 `0cda5b` cleanup 진입·`0325b4` 상태 배열 설치).
- 재실행: `python3 tools/push_install_check.py --controls --acceptance && python3 tools/push_install_rust.py --write && cargo test -p rsolver-lp --test push_install --test push_init_ext && python3 tools/push_install_rust.py --compare`.
- 교훈: `docs/lessons/original.md`(crossover primal push: 해 설치 규칙, Crossover = -1 경로), `docs/lessons/tooling.md`(검사기: xfail과 변이 판정, `116fd7` 변조 위치, 함수 중간 훅 지점 선택).
