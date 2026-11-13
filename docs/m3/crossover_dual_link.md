# M3-04f-4b Rust push factor → `251b4f` 쌍대값 연결 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 선행: [04f-4a 쌍대값](crossover_dual_values.md), [04f-1 push 해 설치](primal_push_install.md), [21i 조밀 LU](primal_push_dense_lu.md).

## 1. 범위와 주장

- 작업 종류: **구현(연결)**. 사용자 지시 "다음 M3 스프린트 진행해 너가 작업자야"로 이 세션이 직접 작업한다(작업자 선택 질문 생략).
- 계획(14_sprint_plan)의 종료 확인은 "04f-1 하네스 사례의 0x1bf09e 호출 d·S+0x2d8 비트 일치"였다. **착수 전 조사로 경계를 바꾼다**(결과를 보기 전, 이유 기록).
  - 04f-1 연결 하네스의 사례는 1c 집합(Crossover = 3, primal-first, first = 1)이다. `13f716` C 1140–1215에서 Crossover 3은 primal push(`134a87`, 인자 1) 뒤 retry 0이면 dual push(`13b3d1`, param_9 = 0)를 부르고, 그 뒤에 0x1bf09e의 `251b4f`를 부른다.
  - 04f-4a 원본 자료 x3 45모델 run 1에서 dual push의 첫 `251b4f`(0x1c6b49)와 0x1bf09e 진입의 factor·header·vstat이 **같은 사례는 6개**(afiro·grow7·israel·kb2·scagr7·share1b)뿐이다. 나머지 39개는 dual push가 basis를 바꿨다. dual push 본체는 Rust에 없으므로 그 39개의 0x1bf09e는 이번 범위에서 Rust 상태로 만들 수 없다.
  - `13b3d1` C 150–451은 param_9 = 0에서 감산 후보 목록(lVar19)을 만들고 S+0x1d8 flag를 세운 뒤 곧바로 C 451의 `251b4f`를 부른다. 그 사이에 factor·header·vstat을 쓰는 호출이 없다(C 168 할당만). 그러므로 **dual push 첫 호출 0x1c6b49의 factor·basis = primal push 반환 상태**다.
- 완료 후 주장할 동작. Crossover = 3 자연 실행에서 1c 진입 상태로부터 Rust `push_init` → `push_loop` → `push_install`(04f-1 연결)이 남긴 `SerialBackend`의 factor(`LuState::btran_factor`, x1d8, bucket)와 vstat·header를 그대로 써서, Rust `crossover_dual::dual_values`가
  - (L1) dual push 첫 호출 **0x1c6b49**의 BTRAN 입력 c_B, BTRAN 반환(출력·입력 후상태·별칭·F+0x200·scratch), y, d, work를,
  - (L2) dual push가 basis를 바꾸지 않은 사례의 **0x1bf09e** 호출에서 같은 필드와 S+0x2d8 재계산을
  원본 3회 실행과 정확·비트 일치로 낸다. (L2)의 factor 쪽 진입 상태(x1d8·bucket)는 Rust L1 BTRAN이 남긴 값을 이어 쓴다.
- 연결 검사(입력 동일성): Rust 쪽 vstat(n+m)·header(m)와 factor 전체(04f-4a `factor_fields` 순서: perm 4종·diag·U 행·flag·x100·x104·x108·x110·xd8·xe4·L 꼬리·L 행·level 구조·bucket·x1e0·x184·eta)·x1d8을 원본 `dv:enter`·`dv:btran:enter`와 비교한다. 이 비교가 원본 factor 주입 제거의 근거다.
- 입력 출처.
  - Rust가 만든 것: factor·x1d8·bucket·vstat·header(push 연쇄). 비용과 행 방향 A는 1c 모델 기록(열 방향)을 전치해 만든다(원본 `rowA`와 같음을 입력 생성 단계에서 확인, 48사례 모두 같음).
  - 원본 캡처 입력으로 남기는 것: 0x1c6b49의 감산 목록·p7/p8 값(`13b3d1` C 168–240의 후보 생성, dual push 본체), S+0x3a8(drop tol), S+0x380(L2), work 누적 진입값, 출력 vector의 진입 count(S+0x228 머리).
- 포함 사례: 1c 연결 사례 중 04f-4a 원본 자료에 같은 모델·Crossover 3이 있는 것 = Netlib 45(x3) + arms·retry·interior(cases `_x3`) = 48사례. L2 대상은 run 1에서 0x1c6b49와 0x1bf09e의 factor·header·vstat이 같은 사례(입력 생성 단계에서 판정, 위 6 + 특수 3 예상).
- 제외와 후속: 1b의 adlittle·scsd1·capri(Netlib 판과 같은 모델, 중복), 합성 편집 사례(1c adjust·mk_*·quality*·slack_m2·col_m5*; 4a에 대응 자료 없음), adlittle_x1(first = 0). dual push 본체(감산 목록 생성, 반복, 0x1c8533·0x1c87a9)와 basis를 바꾼 39사례의 0x1bf09e(**04d 계열 dual push 이식 후**), Crossover −1(primal push가 dual push factor를 이어받음, first = 0), cleanup simplex 본체(04f-4c), x87, solver 경로 연결.
- 선행 commit: main `ce9ab8d`(Merge M3-04f-4a). 미커밋 선행 변경 없음(main의 미추적 `.claude/settings.local.json`은 무관).

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- `13f716` = `../re/decomp/c/001bc5d0_PRIVATE000000000013f716.c` C 1132–1226(push 순서와 0x1bf09e 호출), `13b3d1` = `../re/decomp/c/001c6440_PRIVATE000000000013b3d1.c` C 150–451(첫 `251b4f`까지), `251b4f`·`09e288`·`221173`은 04f-4a §2와 같다.
- 시작 경계: 1c `push:enter`(04f-1 연결 하네스와 같음). 종료 경계: L1은 0x1c6b49 `251b4f` 반환, L2는 0x1bf09e `251b4f` 반환과 `0x1bf2ee`의 S+0x2d8(편집 전).
- 원본 자료: 새 원본 실행 없음. 1c·04f-1 기준 자료(`bench/oracle/m3_push_install_rust/input.txt`, 그 원본 manifest들)와 04f-4a 원본 자료(`bench/oracle/m3_xo_dual/`, 121사례 × 3회 재현 확인)를 사례 이름으로 잇는다. 두 자료가 같은 실행 상태임은 위 연결 검사로 판정한다(다르면 실패).

## 3. 비교 계약

| 필드·사건 | 의미 | 비교 규칙 | 기대값 |
|---|---|---|---|
| vstat(n+m)·header(m) | Rust push 반환 상태 = `251b4f` 진입 | 정확 | `dv:enter` |
| factor 전체·x1d8 | Rust `SerialBackend` factor = BTRAN 진입 | 정확·비트(문자열 필드 열) | `dv:btran:enter.factor` |
| BTRAN 입력 c_B(m) | 감산 포함 | 비트 | `dv:btran:enter.input` |
| BTRAN 반환 | 출력·입력 후상태·별칭·F+0x200·scratch | 정확·비트 | `dv:btran:leave` |
| y(m)·d(n+m)·work | `251b4f` 출력 | 비트 | `dv:leave` |
| S+0x2d8 | L2만 | 비트 | 다음 `sel:enter.s2d8` |

- 오차 허용 없음. 원본 3회 각각과 비교한다(04f-4a에서 3회 동일 확인).
- 1c 연결의 기존 비교(04f-1 `push_install_rust.py --compare`)는 회귀로 함께 돌린다. 연결 사례의 Rust push 반환이 `leave` retry = 0으로 끝나지 않으면 그 사례는 실패다.
- 판별력: trace 변조 거부(`--self-test`)와 Rust 연결 변이(`--mutate-rust`: factor 전달·상태 전달·전치 순서 등) 거부.

## 4. 사례

| 사례 | 실제/합성 | 노리는 분기 |
|---|---|---|
| Netlib 45 × Crossover 3 | 실제 | L1(감산 목록 있음), L2(basis 불변 사례) |
| arms·retry·interior × Crossover 3 | 실제(1b 자연, mode 1) | L1, L2 |

## 5. 재실행 명령

- 입력 작성: `python3 tools/xo_link_rust.py --write`(04f-1 입력·04f-4a 원본 자료 검사 뒤).
- 구현 재생: `cargo test --release -p rsolver-lp --test push_init_ext dual_link_trace`.
- 비교: `python3 tools/xo_link_rust.py --compare`.
- 판별력: `python3 tools/xo_link_rust.py --self-test`, `--mutate-rust OUT`.
- 회귀: `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, 04f-1 `push_install_rust.py --write/--compare`, 04f-4a `xo_dual_rust.py --write/--compare`.

## 6. 실행 결과

- 환경: x86-64 WSL2 Linux, 계약 `7ece5ef`, 구현 `2252782`, 검증 기록 `6f2ca2b`. 원본 실행 0회(기존 자료 재사용). 입력 `bench/oracle/m3_xo_link_rust/link.txt` SHA-256 `90334fc6…`, 04f-1 연쇄 입력 `bench/oracle/m3_push_install_rust/input.txt` SHA-256 `e45f17d0…`(변경 없음), 04f-4a 원본 자료 manifest 검사(`xo_dual_check` 파일 해시·3회 재현)는 `--write`·`--compare` 때마다 다시 돈다.
- 입력 생성(`--write`): 48사례 모두 1c 모델의 n·m·비용이 원본 `dv:enter`와 같고 열 방향 A 전치가 원본 행 방향 A와 행·저장 순서까지 같았다. 원본 run 1에서 첫 `251b4f`는 48사례 모두 dual push 0x1c6b49였다. L1 48호출(감산 목록이 있는 호출 40), L2 9호출(dual push가 factor·header·vstat을 바꾸지 않은 arms·retry·interior·afiro·kb2·scagr7·share1b·israel·grow7). 나머지 39사례는 dual push가 basis를 바꿔 L2 대상이 아니다(§1).
- 비교(`--compare`): **첫 실행부터** 48사례 57호출 × 원본 3회 = 171 비교, 1,004,433필드(vstat·header·factor 전체·x1d8·scratch 0, c_B, BTRAN 출력·입력 후상태·별칭·F+0x200·scratch, y, d, work, S+0x2d8 9) 차이 0. Rust push 연쇄 48사례 모두 `leave` retry 0. BTRAN 경로는 57회 모두 일반 진입·조밀 Uᵀ·eta 없음·조밀 Lᵀ.
- 판별력: trace 변조 18종(vstat·header·x1d8·scratch·factor 두 위치·c_B·BTRAN 출력·F+0x200·y·d·work·L2 d·S+0x2d8·지점 이름·줄 삭제·Rust 오류 줄·retry 1) 모두 거부. Rust 연결 변이 9종 중 **거부 6·생존 3**: 거부는 install 전 vstat(−2 → −1), header 회전, x1d8 0, 비용 순서, 행 순서 뒤집은 전치, 행 마지막 항 누락. 생존 중 `transpose-col-reverse`(행 안 열 순서)는 한 행에 같은 열이 한 번만 있어 d 누적 순서가 바뀌지 않는 **동치 변이**다. `bucket-none`·`l2-fresh-btran-state`는 `--coverage`로 확인했듯 57호출이 모두 조밀 입력(count −1)·조밀 Lᵀ라 bucket을 읽는 level 경로와 x1d8 갱신(희소 입력 진입)을 타지 않았다(L1 factor 20개는 level 구조가 있지만 조밀 경로가 읽지 않음). 첫 시도의 `transpose-row-shift`는 범위 밖 인덱스 panic으로만 거부돼 판별력 근거가 약해 행 수를 보존하는 `transpose-rows-reverse`로 바꿨다(기록은 최종 9종).
- 회귀(`bench/validation/m3_xo_link_rust/regression.txt`): `cargo test --release --workspace` 164 통과·실패 0·ignored 12, clippy 0, 04f-4a `xo_dual_rust --write/--compare`(473호출·1,419비교·2,383,170필드 일치, `xo_dual.rs`를 공용 `tests/dual_io`로 옮긴 뒤), 04f-1 `push_install_rust --write/--compare`((A) 111호출, (B) 59/59), py_compile, `git diff --check` 모두 종료 0.
- 실행하지 못한 검사: pytest(Python 경계 변경 없음), 새 원본 캡처(기존 3회 재현 자료로 충분, 새 경계 아님), Netlib 전체 풀이(solver 경로 미연결). 독립 검토자는 쓰지 않았다.

## 7. 판정과 인수인계

- 판정: **구현 완료(계약 범위, 연결)**. double·Threads = 1·Crossover = 3 자연 실행 48사례에서 1c push 진입 상태 → Rust push 초기화·루프·설치 → Rust factor·vstat·header로 `251b4f`(c_B·BTRAN·y·d·work)가 dual push 첫 호출 0x1c6b49에서, dual push가 basis를 바꾸지 않은 9사례의 0x1bf09e에서는 S+0x2d8까지 원본 3회와 정확·비트 일치했다. 이 경계에서 원본 factor 주입이 사라졌다(감산 목록·S+0x3a8·S+0x380·work·출력 머리 count는 여전히 원본 캡처 입력).
- 계획 대비 변경: 계획의 "0x1bf09e 호출" 경계는 Crossover 3에서 dual push가 사이에 끼므로 dual push 첫 호출(0x1c6b49)과 basis 불변 9사례의 0x1bf09e로 착수 전에 바꿨다(§1 근거). basis를 바꾼 39사례의 0x1bf09e는 dual push 이식(04d·04e 계열) 뒤로 남는다.
- 미검증: 희소 입력 BTRAN의 level 경로·x1d8 갱신(자료 0회), eta가 있는 factor, dual push 감산 목록 생성(`13b3d1` C 168–240), dual push 본체·basis를 바꾼 사례, Crossover −1(primal push가 dual push factor를 이어받음), x87, solver 경로 연결.
- 다음: **M3-04f-4c**(cleanup simplex 본체 조사), **M3-04d**(dual push 분기 분할 조사; 감산 목록 생성이 가장 작은 첫 단위 후보), 04g(재시작). 재검사 `python3 tools/xo_link_rust.py --write && cargo test --release -p rsolver-lp --test push_init_ext dual_link_trace && python3 tools/xo_link_rust.py --compare`.
- 작업 branch `work/m3-04f-4b`(worktree `../rsolver-m3-04f-4b`), 병합은 main `Merge M3-04f-4b` 이력.
- 교훈: `docs/lessons/original.md`(Crossover 3에서 0x1bf09e 앞의 dual push, 13b3d1 첫 251b4f 전 factor 불변), `docs/lessons/process.md`(계획 경계를 착수 전 원본 자료로 확인).
