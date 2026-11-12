# M3-04c-21h `1da0b2` 2×2 블록 단계 이식(capri 23 재분해) 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 원본 기준 자료는 [M3-04c-21e 지연 LU 연쇄 오라클](primal_push_lu_chain_oracle.md), 연결 대상은 [M3-04c-21g 재분해 생성](primal_push_refactor_rust.md)이다.

## 1. 범위와 주장

- 작업 종류: **구현**(블록 단계 입력 상태 3필드를 읽는 작은 캡처 확장 포함). 기준 main `aa2fcf0`(21g 병합), branch `work/m3-04c-21h`, worktree `../rsolver-m3-04c-21h`. 선행 미커밋 없음. main의 미추적 `.claude/settings.local.json`은 무관하며 보존한다. 사용자의 "다음 M3 스프린트 진행해 너가 작업자야" 지시로 이 세션이 직접 작업한다(작업자 선택 질문 생략).
- 완료할 동작: `1da0b2`의 2×2 블록 단계(C:686–1510)와 그 상태 갱신(C:1494–1510)을 M1 `Lu`에 이식한다. 블록 단계는 행 단계 A(개수 2 행을 같은 두 열로 짝지음, `1d9af1` 해시)와 열 단계 B(개수 2 열을 같은 두 행으로 짝지음), 각 블록 뒤의 행/열 singleton 스택, 해시 표 크기 `76fa2`(2·활성 이상의 소수), 수용 문턱 |Schur| ≥ 1e-10, L 버림 문턱 ZERO(param_1), F+0xe4 = 블록 전 단계 수(`local_88`)까지 포함한다. 입력 상태는 S+0x2c0(재분해 수, `05b257` C:1399에서 호출 직전 +1), S+0x4e0(블록 단계 실행 수), S+0x4e4(LU2X2BLOCK: −1 끔, 2e9 항상, 그 외 20단계 이상 피벗한 실행 수)이고 출력은 갱신된 S+0x4e0/0x4e4다.
- push 생성 모드(21g `--generate`)에서 첫 push 재분해의 S+0x2c0/0x4e0/0x4e4만 원본 캡처에서 넣고, 그 뒤 재분해는 **Rust가 갱신한 값**(0x2c0은 재분해마다 +1)을 이어 쓴다. 21g의 사전 명시 예상 실패(capri#813 재분해와 #821 FTRAN)를 닫아 **19/19 재분해·213/213 연산**의 비트 일치를 목표로 한다.
- M1 조율 결정(계약에서 선택): M1 simplex 경로는 **바꾸지 않는다**. 기존 `Lu::factor_tol`은 블록 단계를 끈 상태(S+0x4e4 = −1과 같음)로 호출해 지금과 같은 결과를 낸다. M1 simplex에서 원본처럼 켜려면 M1 spx의 S+0x2c0/0x4e0/0x4e4 수명(초기화 `02f7e1` C:270–281, 재분해마다 증가)을 원본 캡처로 확인해야 하므로 M1 후속으로 등록한다. M1 t(F+0x100) = s3 정정도 같은 이유로 M1 후속에 남긴다(21g 교훈).
- 포함: push 자연 7사례의 재분해 19회 전부. 각 재분해에서 블록 단계가 도는지(활성 수·S+0x4e4 조건), 도는 경우의 S+0x4e0/0x4e4 갱신, capri 23회째 블록 피벗(원본 140·141단계 = 행 {217,218} × 기저 위치 {152,206}).
- 제외 범위: 블록 단계의 풀 부족 반환 −1(C:765, C:806, C:977, C:1062, C:1338: 036ade 재할당 경로, 자연 0회 예상), 조밀 전환·1e99 상한·특이 기저(21g와 같음), work(`fw.q`는 블록 단계 L+U를 세지만 비교하지 않음), M1 simplex 활성화와 pathcmp 궤적 변화(M1 후속). 자연 사례에서 A·B 중 한 단계만 피벗하면 다른 단계는 C 근거만 있는 미검증으로 남긴다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C: `../re/decomp/c/000df800_PRIVATE00000000001da0b2.c` C:400–560(행 사본 재구성: 활성 행마다 단계 3 뒤 미피벗 열의 원소를 위치 오름차순으로), C:560–680(단계 4), C:681–1510(블록 단계), C:1494–1510(상태 갱신), C:1512–1542(F+0xb8…0xc4 기록). `000e2fa0_PRIVATE00000000001d9af1.c`(행 압축·키 = 작은 (개수, 번호) 열 + m·다른 열·체인 해시), `00109480_PRIVATE0000000000076fa2.c`(6k±1 소수 탐색), `000d4190_PRIVATE0000000000036ade.c` C:332·C:373(param_1 = ZERO, param_2 = S), `000f15c0_PRIVATE000000000005b257.c` C:1399(S+0x2c0 증가), `000d06f0_PRIVATE000000000002f7e1.c` C:270–281(LU2X2BLOCK → S+0x4e4: 0→−1, 1→2e9, 그 외 0). 0xdf800과 0x232460 두 사본 중 `036ade@0xd4190`의 callee는 0xdf800이다(`tools/dc.py`).
- 캡처 경계: 21e의 `036ade` 훅(0xd4190) 하나에 **호출자 구분 없이** 진입 S+0x2c0/0x4e0/0x4e4와 반환 S+0x4e0/0x4e4·반환값을 읽는 `lu:block` 사건만 더한다(같은 주소에 attach 하나). push 재분해 여부는 기존 gate(반환 주소 0xf34f2와 push 활성)로 표시한다. 읽기 전용이며 다른 사건은 그대로다.
- 시작 경계: push 재분해의 `036ade` 진입. Rust 입력: 21g와 같은 CSC·header·ZERO·Markowitz와 (S+0x2c0, S+0x4e0, S+0x4e4). 첫 push 재분해만 캡처값, 이후는 Rust 연쇄값.
- 종료 경계: `036ade` 반환의 S+0x4e0/0x4e4와 21g 비교 대상 전부(factor·순열·재번호·이후 연산).

## 3. 비교 계약

| 대상 | 규칙 |
|---|---|
| 캡처 재현성 | 7사례 × 독립 3회의 `lu:block` 사건 목록 정확 일치 |
| 캡처 비간섭 | 새 캡처에서 `lu:block` 사건만 뺀 사건 목록·결과·출력 = 21e 정식 `*-1` 캡처(JSON 전체 정확) |
| 블록 상태 연쇄 | push 재분해마다 Rust 진입 (0x2c0, 0x4e0, 0x4e4) = 원본 진입, Rust 반환 (0x4e0, 0x4e4) = 원본 반환(정확) |
| 재분해 factor와 이후 연산 | 21g 계약 §3 그대로(213연산 전부, 예상 실패 없음) |

- 정수·목록·순서 정확, f64는 ±0 포함 비트. 원본 상태는 첫 push 재분해의 진입값만 Rust 입력에 넣고 나머지는 비교기만 읽는다. push 재분해 사이에 push 밖 `036ade` 호출이 끼면 그 사실을 기록하고 연쇄 대신 그 재분해의 진입값을 넣는다(이 경우 그 재분해의 연쇄 비교는 통과로 세지 않는다).
- 판별력: (a) 캡처 검사기 변조(바이너리 해시, 빈 사건, 회차 차이, 투영 차이, 상태 3필드 각각), (b) 생성 trace 변조(블록 상태 진입/반환 필드), (c) Rust 소스 변이(임시 Cargo 사본): 블록 단계 끔(**반드시 거부**, capri#813), 피벗 행/열 |a| 비교 반전, 수용 문턱 1e-10 → 0, 해시 키 순서 반전, 표 크기 소수 → 2·활성, A 단계 생략, B 단계 생략, 블록 뒤 singleton 스택 생략, L 두 번째 열의 채움 순서, 실행 조건 50 → 49, 갱신 문턱 20 → 19, S+0x2c0 > 20 결정 반전, F+0xe4 = 블록 뒤 단계 수. 자연 사례가 지나지 않는 분기의 변이 생존은 따로 센다.
- 21g 생성 재생 결과와 21f 주입 재생(`--compare`)은 회귀로 유지한다.

## 4. 사례와 원본 자료

| 사례 | 재분해 | 비고 |
|---|---|---|
| arms·retry·interior | 4·3·4 | 21g 일치. 블록 조건은 캡처로 판정 |
| adlittle·scsd1 | 3·2 | 21g 일치 |
| capri | 3 (64·64·131) | 23회째 블록 피벗(21g 예상 실패) |

- 원본 자료: 새 캡처 `bench/oracle/m3_push_block/<사례>-{1,2,3}.json(.gz)`(Netlib은 결정적 gzip), manifest. 21e 정식 캡처를 비간섭 기준선으로 쓴다. 모델은 21e manifest와 같은 해시.

## 5. 재실행 명령

- 원본 캡처: `tools/hrun.sh --timeout 580000 -- 'python3 tools/push_block_runs.py'`(Gurobi 한 프로세스씩, 21회).
- 캡처 검사: `python3 tools/push_block_check.py --controls --acceptance --self-test`, manifest `--write-manifest`.
- 생성 재생: `python3 tools/push_lu_rust.py --generate --write` → `cargo test -p rsolver-lp --test push_lu` → `python3 tools/push_lu_rust.py --generate --compare`(213/213 기대), `--generate --self-test`, `--generate --mutate-rust <경로>`.
- 회귀: 21f `--compare`·`--self-test`, 21b/21d compare, `cargo test -p rsolver-lp`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, py_compile, `git diff --check`. M1 simplex 경로는 블록 단계를 끈 채라 수치 불변이므로 확장 설치·pytest·Netlib pathcmp는 workspace 회귀로 대신하고 이유를 남긴다(경로가 바뀌면 실행).

## 6. 실행 결과

(작성 예정)

## 7. 판정과 인수인계

(작성 예정)
