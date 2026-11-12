# M3-04c-21j push 재분해 `036ade` 풀 부족 재호출과 `local_70` 조밀 배열 Markowitz 모드 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 선행: [21i 조밀 전환](primal_push_dense_lu.md)(grow22 lu 392·431·455 사전 명시 실패), [21h 블록 단계](primal_push_block_rust.md), [21g 재분해 생성](primal_push_refactor_rust.md).

## 1. 범위와 주장

- 작업 종류: **구현**(원본 경계 캡처 확장 포함). 기준 main `2caf815`(21i 병합), branch `work/m3-04c-21j`, worktree `../rsolver-m3-04c-21j`. 선행 미커밋 없음. main 미추적 `.claude/settings.local.json`은 무관하며 보존한다. 사용자의 "다음 M3 스프린트 진행해 너가 작업자야" 지시로 이 세션이 직접 작업한다(작업자 선택 질문 생략).
- 계약 전 조사(판정 근거 아님): `tools/push_mode70_survey*`(21i 표지 agent + 풀 검사 입력·풀 부족 표지·반환≠0 기록)로 Netlib 91개 중 면허 크기 안 64개를 1회씩 돌렸다. `local_70` 모드와 `036ade` 반환 −1은 **grow22에서만** 나왔다: push 재분해 lu 392(n 211, 통과)·431(n 250, 첫 호출 풀 부족 −1 → 재호출 통과)·455(n 272), push 밖 1회(n 272). lu 431 첫 호출의 풀 검사: 필요 `n² + 0.3n²` = 81,250 > 여유 `local_1b8 − param_15[3]` = 193,384 − 126,850 = 66,534. 새 용량 291,984 = (193,844 + 14,716)·1.4가 C:2592–2620 식과 같다.
- 완료할 동작:
  1. **풀 계수**: `036ade`의 풀 위치 `local_1c0`(U, 0부터 위로), `local_1b8`(L, F+0x168 = F+0x158 − 1에서 아래로), `param_15[3]`·`local_138`(원소 노드 영역, `local_f8`부터 위로; 초기 활성 원소 노드와 4m칸 chunk, 정수 풀 쪽 덤 chunk)을 Rust 커널이 같은 규칙으로 센다(C:296–366, C:541–606, C:2440–2490). 노드는 32바이트(4칸)다.
  2. **`local_70` 진입 판정**(C:1750–1764, asm `0xd6a30`–`0xd6ab6`): Markowitz 탐색 직전 활성 n = (m − 단계) − 빈 열 수가 201 ≤ n ≤ 9999이고 `cidx·ridx > 0.1·n`, `채움 추정 < 0.25·n²`이면 `local_70 = 1`.
  3. **풀 검사·풀 부족 −1**(C:2572–2620): n' = 활성 − min(빈 행, 빈 열), 목록 상한 `iVar67 = max((n'² − 3n')/(2n'), max(3, (n'+2)/3))`, 정수 칸 `local_40`(n'² 또는 3n' + 2n'·iVar67)으로 세 조건을 검사하고, 실패하면 F+0x158 = F+0x168 = `(F+0x158 + max(m − 여유, 부족분)) · 1.4`로 두고 −1을 반환한다. `05b257` C:1479–1513은 풀을 새로 잡고(F+0x170 증가) 같은 재분해를 처음부터 다시 부른다(블록 단계 상태도 다시 갱신).
  4. **`local_70` 모드 본체**: 조밀 번호 배치(C:2621–2810: 활성 열 위치 → 조밀 열 `local_78`, 활성 행 → 조밀 행 `local_a8`, bucket 순환 목록 재구성, 행/열 목록 상한 iVar67·넘치면 전체 탐색 표시), 탐색(C:2813–3300: 조밀 전환 판정 재평가, 최소 bucket, 열·행 bucket 탐색과 수용 규칙 `1e-100 < |a|`·threshold·비용 동률), 피벗 실행(C:3686–3966: U 행 기록, L 열 기록과 갱신, `|old| ≤ 1e-100`인 칸의 새 비영 등록, `|new| < ZERO`이면 표지값 `0x2b2bff2ee48e0530`과 개수 감소, 빈 행·열 처리, 풀 검사 −1 두 곳)을 Rust로 옮긴다. 모드 중 조밀 전환(C:3405의 `local_70` 분기)은 C 근거만 이식하고 자연 0회면 미검증으로 남긴다.
- 포함: 21i 정식 5모델(grow7·grow22·perold·pilot4·stair)의 push 재분해 전부. 21i의 범위 안 28재분해는 그대로 일치해야 하고(회귀), grow22 lu 392·431·455가 이번 대상이다.
- **사전 명시 제외**: MKL `dgetrf`·전치 경로, 1e99 피벗 상한, 빈 열 조밀 전환, 블록 단계·일반 Markowitz의 풀 부족 −1(C:928·2140·2331·2459, 자연 0회), `LAB_001d53d9` 부분 조밀 경로(param_4), `local_70` 모드 안 조밀 전환, work(`param_19`·`auVar160`)·x87. 자연 0회이면 C만 남긴다. M1 `factor_tol`/`factor_block`은 21i처럼 끈 채 둔다(`factor_dense` 진입에서만 켬).

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. CPU 13th Gen Intel i7-13700F(WSL2).
- C: `../re/decomp/c/000d4190_PRIVATE0000000000036ade.c` C:296–366(풀 첫 채움), C:541–606(커널 노드 할당), C:1750–1764(진입 판정), C:2440–2490(채움 노드 chunk), C:2569–3300·C:3686–3966(모드 본체), `05b257` C:1427–1518(재호출). asm `0xd5010`(할당 뒤), `0xd6ab6`(판정), `0xd85b4`/`0xd85ee`(모드 진입·풀 검사 입력), `0xd881b`(풀 부족), `0xd9f5b`(모드 피벗), `0xdb025`(커널 끝).
- 스택 위치(asm 대조): `local_1b8` 0x180, `local_1c0` 0x178, `local_f8` 0x240, `local_138` 0x200, 활성 n 0x100, `local_60` 0x2d8, `local_f0` 0x248, 초기 노드 수 0x68. `param_15` = 036ade 15번째 인자(F+0x148).
- 캡처 경계(`tools/push_mode70_agent.js`): 21i agent 사본에 함수형 probe(창 안 착지점 0 확인)를 더한다. `pool:start`(0xd5010), `pool:mode70`(0xd85ee), `pool:short70`(0xd881b), `mode70:pivot`(0xd9f5b: 조밀 평탄 번호 esi, n r14d), `pool:end`(0xdb025). 풀 사건은 `local_1b8`·`local_1c0`·`param_15[2..4]`·`local_138`·`local_f8`를 담는다.
- 시작 경계: 21i와 같다(push 재분해 `036ade` 진입). 종료 경계: `036ade` 반환 순열과 재분해 `lu:leave` factor 전체, 재호출이면 마지막 호출의 반환.

## 3. 비교 계약

| 대상 | 규칙 |
|---|---|
| 캡처 재현성 | 5모델 × 독립 3회 사건열 정확 일치(pid·run_id 제외) |
| 캡처 비간섭 | 5모델·21i 대조군 7사례: 새 agent 사건열 − 새 사건(`pool:*`, `mode70:*`) = 21i 정식 같은 사례 사건열(JSON 정확) |
| (A) 풀 계수 | 5모델 push 재분해 `036ade` 호출 전부: Rust가 센 `pool:start`·`pool:end`(+ `pool:mode70`) 값 = 원본(정수 정확) |
| (B) 재분해 생성 | 5모델 push 재분해 전부(21i 범위 안 28 + grow22 3): 21h `--block` 입력 규칙으로 Rust가 만든 `036ade` 반환 순열·factor 전체·블록 상태 = 원본. 모드 재분해는 추가로 Rust 피벗 순서 = `mode70:pivot`, 풀 부족 재호출 수와 새 F+0x158 = 원본 |
| (C) 연결 | 21i `push_dense_rust.py --compare`(조밀 21회·28/28 유지), 04f-1 설치 59/59, 21h `--block` 219/219, 24·1a·1c·1d 회귀 |

- 정수·목록·순서 정확, f64는 ±0 포함 비트. NaN이 나오면 실패.
- 판별력: (a) 캡처 검사기 변조(바이너리 해시, 빈 사건, 회차 차이, 비간섭 투영, 풀 값 1, 피벗 교환), (b) 기대값 변조, (c) Rust 소스 변이(임시 Cargo 사본): 모드 끔(**필수 거부**, grow22), 풀 부족 끔(**필수 거부**), 판정 201→200·9999→10000·0.1→0.2·0.25→0.3, 1.4→1.5, iVar67 식, 표지값 대신 0, `1e-100` 경계, 비용 동률 규칙, 열/행 탐색 순서, chunk 4m → 4m+4. 생존은 따로 센다.

## 4. 사례와 원본 자료

| 사례 | 이번 대상 | 비고 |
|---|---|---|
| grow22 | push lu 392(n 211)·431(n 250, 재호출)·455(n 272) | 21i 범위 밖 3재분해 |
| grow7·perold·pilot4·stair | 풀 계수(A)와 회귀 | 모드 0회 |

- 자료: `bench/oracle/m3_push_mode70/<모델>-{1,2,3}.json.gz`(결정적 gzip), `controls/`(21i 대조군 7사례 × 1), manifest. 조사 자료 `bench/validation/m3_push_mode70/survey/`.
- 모델: `../re/bench/data/netlib/<모델>.mps.gz`. 설정 Threads = 1, Seed = 0, Presolve = 0, Method = 2, Crossover = 3, LPWarmStart = 2(21i와 같음).
- 적용하지 않은 범주: 면허 크기 초과 Netlib 27개(실행 불가), 위 §1 제외 경로(자연 0회).

## 5. 재실행 명령

- 조사: `python3 tools/push_mode70_survey.py OUT`.
- 원본 캡처: `python3 tools/push_mode70_runs.py`(5모델 × 3), `--controls`(7사례 × 1), Gurobi 한 프로세스씩.
- 캡처 검사: `python3 tools/push_mode70_check.py --controls --acceptance --self-test`.
- Rust: `python3 tools/push_mode70_rust.py --write` → `cargo test -p rsolver-lp --test push_lu mode70_chain_trace` → `python3 tools/push_mode70_rust.py --compare`, `--self-test`, `--mutate-rust <새 경로>`.
- 회귀: 21i `push_dense_rust.py --write/--compare`, 04f-1 `push_install_rust.py --write/--compare`, 21h `push_lu_rust.py --block --compare`, 21g `--generate --compare`, 21f `--compare`, 24 `push_loop_rust.py --lu --compare`, 1a `--init --compare`, 1c·1d, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`.

## 6. 실행 결과

(작업 중)

## 7. 판정과 인수인계

(작업 중)
