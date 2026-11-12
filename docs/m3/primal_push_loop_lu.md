# M3-04c-24 실제 Rust 직렬 backend를 push 루프에 설치: 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다. §1–§5는 구현과 비교를 실행하기 전에 썼다.

## 1. 범위와 주장

- 작업 종류: **구현 연결**. 기준 main `6340a20`(M3-04c-22 병합), branch `work/m3-04c-24`, worktree `../rsolver-m3-04c-24`. 선행 미커밋 변경 없음. main의 미추적 `.claude/settings.local.json`은 무관하며 보존한다. 사용자의 "다음 M3 스프린트 진행해 너가 작업자야" 지시로 이 세션이 작업자로 수행한다(작업자 선택 질문 생략). 점유 owner `claude-m3-04c24-20261112`.
- 완료할 동작: 22의 `push_loop`에 tape 대신 **실제 Rust 직렬 수치 backend**(`crossover_backend::SerialBackend`)를 넣는다. backend는 21b–21h의 `crossover_lu::LuState`(FTRAN `2207de`·BTRAN `221173`·FT 갱신 `05b257` 요청 0·재번호)와 `crossover_refactor::refactor`(`036ade`+C:2570–2760, `1da0b2` 블록 단계 포함)를 다음 경계에 연결한다.
  - 공급 FTRAN(`09d008`): spike 인자가 있으면(지원 0) wrapper clear 뒤 save=1·spike F1c8, 없으면 save=0. raw = 출력 header(count·목록·값).
  - 지원 BTRAN(`09e376`→`221173`): cache F+0x1d0, F+0x200 work 0. raw = 출력 header.
  - `05b257` 요청 0: header를 넣고 FT 갱신(이탈 위치 S+0x2f8). 요청≠0: S+0x2c0 블록 상태로 `refactor`, 이어 LuState 재번호(C:2006–2090)와 같은 순열을 루프에 돌려준다.
  - 재계산 rhs FTRAN(`09cf9b`, count -1, save 0, spike 없음): raw = dense 출력 m값.
  - factor 판단 통계를 backend 상태에서 만든다: F+0x18c·F+0x188·F+0x184는 LuState 값, S+0x46c는 시작값 고정(루프 안에서 쓰는 곳 없음, 자연 불변), **F+0x190**은 FT 갱신마다 작업 열 S+0x210(= wrapper가 넣은 현재 alpha)의 `count+1`(dense면 1+|v|>S+0x3a8 개수)을 더하고(`05b257` C:144–251) 재분해에서 0(C:1379), **F+0x1e0 = m + F+0x58[m]**, **F+0x1e8 = F+0xb0[m] + m**(재분해 끝 C:1978–1986; F+0xb0[m]은 압축된 U 행 끝 = U 비영 수로 둔다).
- 초기 backend 상태: 루프 전 유일한 수치 연산인 초기화 factor(요청 10, 자연 6사례 모두 `lu:enter` 1회)를 21h 첫 G와 같은 입력(호출 전 header·S+0x2f8·vstat, ZERO, Markowitz, 할당 4필드, 첫 `036ade` 진입 블록 상태)으로 Rust가 만든다. 그 뒤 원본 factor·수치 출력 주입은 **0회**다.
- 루프 시작 push 상태(모델·후보·basis·rhs 등)는 22와 같은 `supply:enter`/`pass:enter` 입력이다. Rust 초기화 factor의 재번호 header가 이 시작 header와 같아야 시작한다(같지 않으면 실패).
- `PushLoopBackend::factor`는 push 상태(header·S+0x2f8·vstat)와 작업 열을 받도록 서명만 넓힌다. 루프 제어 동작은 바꾸지 않는다.
- 제외와 후속: 품질 helper `089f61`(15a, 자연 0회; backend는 호출되면 실패), factor 반환 1·12·오류와 FT 실패·재할당(`cd9cc`·`1f2069`), pricing 객체, batch(23), x87, 종료 검사·콜백, work·시간, 초기화의 나머지(후보 구성·정렬, C 1143 이전)와 해 설치, 기존 solver 경로 연결. 수치 카운터 S+0x178/0x184/0xf0은 자연 경로에서 `05b257`이 바꾸지 않으므로 그대로 둔다(바꾸는 실패 경로는 범위 밖).

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so`, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 ELF file offset.
- `../re/decomp/c/000f15c0_PRIVATE000000000005b257.c`: 요청 0 FT 경로 C:144–1335(F+0x190 누적 C:144–251, F+0x18c++ C:1327), 재분해 C:1337–(F1f0 = F190/k, F190 = 0 C:1372–1379, S+0x2c0++ C:1399), 재번호 C:2006–2090, L/U 통계 C:1978–1986, 공통 출구 C:2765–2770. 나머지 조각의 근거는 21b·21d·21f·21g·21h 문서 §2.
- 캡처 확인(21e run 1, 6사례): 루프 전 수치 연산은 `lu:enter` 요청 10 한 번. 공급 FTRAN 107회는 save = spike F1c8 여부(1/0 = 51/56), 재계산 13회는 count -1·save 0·spike 없음이고 rhs는 바뀌지 않으며 raw = `ftran:leave` dense 출력, 공급 raw = 출력 header(107/107 일치). BTRAN 32회 모두 cache F1d0·x200 0. BTRAN 출력 header의 진입 count는 dense Lᵀ에서 입력 후상태 count로만 옮겨지고 출력에는 쓰이지 않으므로 0을 넣는다. factor:leave의 numeric·failure·status·q는 모든 호출에서 진입과 같다.
- 시작 시점: 초기화 `lu:enter`(factor 생성)와 첫 `roundctl:pass:enter`(루프). 종료 시점: `roundctl:end`.

## 3. 비교 계약

22의 canonical 사건열(줄·순서·필드)을 그대로 쓰고 두 가지를 더한다.

| 줄 | 비교 필드 | 원본 출처 | 규칙 |
|---|---|---|---|
| `init`(사례 첫 줄) | 초기화 재분해 뒤 header·S+0x2f8·vstat | 초기화 `lu:leave` | 정확 |
| `fcall`에 `stats` | 호출 뒤 k(F18c)·limit(F188)·q(S+0x46c)·C(F190)·E(F184)·L(F1e0)·U(F1e8) | `factor:leave.factor_ctl.state` | 정확 |

- 22에서 tape이던 값(공급 FTRAN raw, BTRAN raw, 재분해 순열, 재계산 raw)은 이제 Rust가 만든 값이며, 각각 `supply`(alpha)·`btran`(raw)·`fcall`(hdr2·vstat·leave2)·`recompute`(basic·width·marks·PInf) 줄에서 원본과 비트/정확 비교된다. tape 입력 파일에는 수치 출력이 없다(입력 줄: case·model·cfg·state·round·stats(초기화 뒤)·init).
- 줄 수·순서·태그가 원본과 같아야 하며, backend가 실패(미구현 경로·unknown 셀 읽기)하면 그 사례는 실패다.
- 수치는 ±0 포함 f64 비트 비교. 허용오차 비교 없음.
- 원본 재현성: 21e 캡처(독립 3회·비간섭·변조 54 거부)와 21h 블록 상태 캡처(독립 3회·21e 비간섭)를 재사용한다. 도구는 두 검사기의 controls·acceptance를 먼저 통과시킨다. 새 원본 실행은 하지 않는다.

## 4. 사례

22와 같은 자연 6사례(arms·retry·interior·adlittle·scsd1·capri), 집계 고정값도 22 §4와 같다(패스14·공급119·단계107·BTRAN32(dense2)·판단67·`05b257` 루프 호출49·재계산13·round12·종료6) + init 6. 수치 분기: FT 갱신 36(행 eta 포함), 재분해 13(초기화 6 + 루프 13 = 19, capri 23회째 B 단계 블록), FTRAN A2·spike dense/sparse, BTRAN 희소/dense Lᵀ·cache.

미관측(판정 안 함): 22 §4의 목록, FT cache 재사용(F1d8=r), BTRAN eta>0, 품질 helper.

## 5. 재실행 명령

- 입력: `python3 tools/push_loop_rust.py --lu --write`(21e·21h 검사 선행).
- 재생·비교: `cargo test -p rsolver-lp --test push_loop_lu && python3 tools/push_loop_rust.py --lu --compare` 종료 0.
- 판별력: `python3 tools/push_loop_rust.py --lu --self-test`(trace 변조·구조 변조 전부 거부), `--lu --mutate-rust <새 경로>`(backend·루프 변이; 자연 경로를 지나는 변이는 거부되어야 하며 생존은 캡처로 분류).
- 회귀: 22 tape 재생(`--write`·`push_loop`·`--compare`), 21h `push_lu_rust.py --block` 재생 비교, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`. 기존 solver 경로는 바꾸지 않으므로 확장 설치·pytest·Netlib·P3는 제외한다.

## 6. 실행 결과

(작성 예정)

## 7. 판정과 인수인계

(작성 예정)
