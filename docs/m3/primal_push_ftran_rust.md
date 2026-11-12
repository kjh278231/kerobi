# M3-04c-21b 한 RHS FTRAN Rust 수치 adapter 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 원본 기준 자료는 [M3-04c-21a 오라클](primal_push_ftran_oracle.md)이다.

## 1. 범위와 주장

- 작업 종류: **구현**. 기준 main `40e1280`(21a 병합), branch `work/m3-04c-21b`, worktree `../rsolver-m3-04c-21b`. 선행 미커밋 없음. main의 미추적 `.claude/settings.local.json`은 무관하며 보존한다. 사용자의 "다음 M3 스프린트 진행해 너가 작업자야" 지시로 이 세션이 직접 작업한다(작업자 선택 질문 생략).
- 완료할 동작: crossover push의 한 RHS FTRAN 본체 `2207de`를 원본 factor 표현(F+0x20… 배열)을 그대로 받는 Rust adapter `crossover_ftran`으로 구현한다. 같은 `ftran:enter` 상태(factor 전체·입력 벡터·spike·인자)에서 시작해 **커널 진입/반환마다 in/out 두 벡터(count·목록·값)·버퍼 별칭·spike**, 본체 반환의 **raw out·입력 후상태·별칭·spike**, 그리고 scratch(F1b0/F1b8)가 0으로 돌아오는지를 원본 50호출과 비트 비교한다.
- 포함 분기(C 근거로 구현): 입력 dense gather/sparse 치환, count0 조기 종료, L `214344`(dense)·`21b9de` A1(Kahn FIFO), R eta 0 교환·dense R·sparse 목록 R·sparse→dense R, spike save0/dense 문턱 필터/sparse 복사, U `2126cd`·`21686f`(S1/S2/H1/H2). 각 분기의 자연 관측 수를 결과에 센다. 자연 0회 분기는 **C 근거 구현·원본 비교 미검증**으로 표시하며 통과로 세지 않는다.
- 제외 범위: `21b9de` 레벨 경로 A2(F128≥0, 레벨 배열 미캡처)와 경로 B(`m−F_e4`가 큰 L 꼬리)는 구현하지 않고 adapter가 오류를 반환한다. 두 번째 RHS `21574d`(k5/p6 비null)도 오류다. work 누적값은 21a처럼 기록만 하고 비교하지 않는다. M1 `Lu`(자체 표현)와의 통합·factor 생성/갱신·BTRAN(21c·21d)·지연 LU/재분해 연쇄(21e·21f)·기존 solver 경로 연결·루프22·x87·P3는 범위 밖이다. M1 공유 코드(`lu.rs`)는 바꾸지 않는다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 21a manifest의 바이너리·캡처 해시를 그대로 검사한다.
- C(직접 읽음): `../re/decomp/c/00120740_PRIVATE00000000002207de.c`(본체 1–1051), `0012bd30_…214344.c`(dense L), `0012a990_…21b9de.c` C:1–257(A1)·894–899(반환), `00121ef0_…2126cd.c`(dense U), `001223e0_…21686f.c`(희소 U 전체). 호출 인자는 21a §2와 `tools/dc.py 0x120740 --asm`(`0x1208b6`·`0x120922`·`0x121ca1`·`0x121ccc`)를 따른다.
- 원본 연산 규칙(구현에 그대로 옮김):
  - 본체: count0이면 out count0. dense 입력은 `out[i]=in[F20[i]]` 뒤 두 구조체 idx/val 포인터 교환·둘 다 count −1, `214344`. sparse는 `idx=F30[idx]` 제자리 치환 뒤 `count·F104 ≤ m`이고 `0.1m > F128`이면 `21b9de`, 아니면 `214344`. L 출력 count0이면 종료.
  - R: eta0이면 in/out 교환(in=L 출력). eta>0·L dense면 교환 뒤 eta 전부 dense. sparse·`F104·count ≤ m`이면 F1b8/F1b0 scratch에 흩고 `min eta_oldest[행]`부터 eta를 계산해 새 행은 out idx 뒤에 붙이고, out 목록 순서로 `ZERO ≤ |v|`만 in에 압축(0개면 out count0 종료). 아니면 in val을 0으로 채우고 흩은 뒤 `min eta_oldest`부터 dense eta, in count −1. eta 값은 `{r,len,idx,val}` 레코드 순서로, 8-lane 누적(값 포인터 mod16=8이면 1개 peel, `n<8`·`n<peel+8`이면 순차)과 `|v| ≤ ZERO → +0`.
  - spike(save≠0, spike=param7 또는 F1c8): in count<1이면 `i` 오름차순으로 `ZERO < |v|`인 위치를 목록, 값 `v·save`. 아니면 count·목록 복사(포인터가 다를 때), 값 `v·save`를 행 위치에.
  - U: in count ≥ 0이고 `(m−F100)+F_ec+count < m` 또는 `(F108+F110)²·count < 2m`이면 `21686f(m,F+0x20,in,out,mark,vals)`, 아니면 `2126cd`. `2126cd`는 dense 입력이면 구조체 교환, sparse면 out val을 0으로 채운 뒤 흩고 U 위치 m−1→t는 `|x|≤ZERO` 또는 `|x/d|≤ZERO`면 +0, 아니면 나누고 열 [F88,F90)을 흩는다. t−1→0은 나눗셈만. `21686f`는 M1-02m 경로(S1·S2·H1·H2)와 같은 순서·값이며 출력 `ZERO < |x/d|`, 입력 idx의 꼬리 행 제자리 압축(S1 제외), out idx의 구간2(m−F_ec부터) 쓰기까지 옮긴다. 힙↔순차 전환은 처리 순서가 같아(내림차순 U 위치) 순차로 구현하고, 힙이 out idx 버퍼(m−F_ec−1부터 아래)에 남기는 값은 **미지값**으로 표시한다.
- 시작 경계: 21a `ftran:enter`(wrapper clear 뒤 본체 진입). 종료: `ftran:leave`. 입력은 enter의 factor·input·args·F1c8 spike만이다. out 구조체의 진입 count·버퍼 내용은 캡처되지 않으므로 **미지값**으로 시작하고, 비교 대상 영역에서 미지값을 읽으면 그 필드는 실패다.
- 버퍼 모델: 원래 in/out의 idx·val 버퍼 4개(길이 m)와 두 구조체의 (count, idx 버퍼, val 버퍼)를 원본처럼 교환한다. 별칭은 구조체가 가리키는 버퍼의 원래 소속(`in`/`out`)이다.

## 3. 비교 계약

| 대상 | 규칙 |
|---|---|
| 커널 사건 순서 | 호출별 (커널 이름, enter/leave) 열이 원본 `ftran:kernel` 열과 정확히 같음 |
| 커널 enter | in 구조체 count·idx 목록·val(count 길이, dense는 m)·별칭, F1c8 spike(count·목록·목록 위치 값) |
| 커널 leave | 위 + out 구조체 |
| 본체 leave | raw out, 입력 후상태(in), 별칭, F1c8 spike |
| scratch | Rust가 반환 시 F1b0 표시·F1b8 값 m개 모두 0임을 직접 출력(원본 21a 규칙7과 같은 조건) |

- 정수·목록·순서는 정확, f64는 ±0 포함 little-endian 비트. 미지값·NaN·무한대는 실패. 원본 기대값은 Python 비교기만 읽고 Rust 입력에는 넣지 않는다.
- work는 비교하지 않는다(21a 계약과 같음).
- 원본 재현성은 21a 정식 12회(각 독립3회 동일)를 재사용한다. 이번 작업의 원본 실행은 0회가 목표다.

## 4. 사례와 원본 자료

| 사례 | 출처 | 호출 | 노리는 분기 |
|---|---|---|---|
| arms | 21a 정식 `arms-1.json`(2·3과 동일 검증됨) | 공급25·재계산3 | sparse L A1·sparse U·dense 재계산 |
| retry | `retry-1.json` | 공급8·재계산2 | sparse spike 복사1 |
| interior | `interior-1.json` | 공급9·재계산3 | dense L·eta>0 dense R·dense spike 8 |

- 자연 미관측(21a §7): sparse R, eta>0의 sparse 목록, count0, L 출력0, `21b9de` A2/B, 접두 목록 있는 `21686f`의 H2/S2 일부, sparse spike≥2, ±ZERO 경계. 이 분기의 Rust 코드는 C 근거 구현이며 결과에 미검증으로 센다.

## 5. 재실행 명령

- 기준 재검사: `python3 tools/push_ftran_check.py --controls --acceptance --self-test`.
- 구현 재생: `python3 tools/push_ftran_rust.py --write` → `cargo test -p rsolver-lp --test push_ftran` → `python3 tools/push_ftran_rust.py --compare`.
- 오류 검출: `--self-test`(trace 변조: 필드·값 비트·순서·별칭·spike·scratch·누락 사례·입력 해시), `--compare --mutate-rust <새 결과경로>`(임시 Cargo 사본에서 소스 변이를 빌드해 원본 차이로 거부; 빌드 실패·생존을 따로 센다).
- 회귀: `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, 선행 push 재생(`push_round_rust.py --compare`, `push_step_chain_rust.py --compare`), py_compile, `git diff --check`. 확장·solver 경로는 바뀌지 않으므로 pytest·Netlib·P3는 제외하고 이유를 기록한다.

## 6. 실행 결과

(실행 후 작성)

## 7. 판정과 인수인계

(실행 후 작성)
