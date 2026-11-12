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

- 환경: WSL2 x86-64 Linux 6.6.87.2, Python 3.10, Rust stable(clippy 1.99). 계약 `9537aab`, adapter·재생 첫 동작 `0c87888`, M1 커널 재생 추가 `55b2bcf`. 원본 실행 **0회**(21a 정식 캡처와 M1-02j/02m 캡처 재사용). 증거 `bench/oracle/m3_push_ftran_rust/`(입력 `input.txt`·`m1_kernels.txt`, Rust `trace.txt`·`m1_trace.txt`, manifest, 비교·변조·변이·회귀 기록).
- **push FTRAN 50호출**(`--compare` 종료0): 공급42·재계산8(arms28·retry10·interior12) 전부 커널 사건 200개(이름·enter/leave 순서, in/out count·목록·값·별칭, F1c8 spike)와 본체 반환(raw out·입력 후상태·별칭·spike·scratch)이 원본과 비트/정확 일치. 비교 필드 5,520개. 첫 실행부터 일치했다.

| 분기 | 관측(push) | 비고 |
|---|---:|---|
| 입력 sparse / dense / count0 | 42 / 8 / 0 | dense는 재계산 |
| L `21b9de` A1 / `214344` | 33 / 17 | |
| R 없음 / dense / sparse 목록 / sparse→dense | 45 / 5 / 0 / 0 | eta 길이 3–4라 8-lane 본체 미실행 |
| spike save0 / dense / sparse | 41 / 8 / 1 | save는 0·1뿐 |
| U `2126cd` / `21686f` S1 / H2 | 17 / 3 / 30 | push의 희소 U는 모두 F_ec=0 |

- **M1 원본 커널 186호출**(같은 `21686f`·`21b9de`를 `09d112`에서 부른 degen2·kb2 캡처, 해시 고정): `21686f` 167호출(S1 9·S2 29·H1 1·H2 128)과 `21b9de` A1 19호출(kb2)의 출력 순서·값이 비트 일치하고 scratch가 0으로 돌아온다. degen2 `21b9de` 695호출은 레벨 경로 A2라 adapter 범위 밖으로 세기만 했다. 이 커널 입력은 `09d112` 문맥이며 2207de 본체의 버퍼 교환은 포함하지 않는다.
- **trace 변조**(`--self-test` 종료0): 1,493종 모두 첫 차이로 거부(사례 누락·순서·입력 해시0·빈 trace, 사례마다 출력 값 비트·count·입력 후상태 idx/val·별칭·spike count/값·scratch·미지값·오류 주입·커널 누락/이름/단계/입력 count/출력 값/순서, M1 사례마다 count·값·순서·scratch).
- **Rust 소스 변이**(임시 Cargo 사본, 26종, `rust-mutations.json`): 첫 차이 거부 15, 재생 거부 2(`r-no-swap`·`u-sparse-always`: 캡처되지 않은 칸을 읽어 adapter가 중단, trace 없음), **생존 9**. 생존은 모두 이번 원본 자료에 나타나지 않는 경계다.

| 생존 변이 | 판별에 필요한 원본 사례 |
|---|---|
| `sparse-l-boundary`(`<`→`≤`) | `count·F104 = m` 동률 입력 |
| `l-dense-nonstrict`, `u-dense-no-drop`, `eta-no-drop`, `spike-nonstrict`, `kahn-keep-small` | `|x| ≤ ZERO`인 0 아닌 값(문턱 필터가 실제로 거르는 값) |
| `eta-sequential` | 길이 ≥ 8(+peel)인 eta 레코드 |
| `spike-unscaled` | save ∉ {0, 1} (자연은 1뿐이라 `v·1 = v`) |
| `u-dense-scatter-prefix` | t 아래 비어 있지 않은 U 열(분해 직후 비어 있음, M1-02m P) — 동치 변이일 가능성 |

- 계약 변경(확대): 첫 변이 실행(26종 중 생존 12)을 보고, 같은 커널의 기존 원본 캡처(M1-02j/02m)를 두 번째 재생으로 추가했다. 그 결과 `u-sparse-ascending`·`u-sparse-h-always`·`u-sparse-no-input-compaction`·`kahn-lifo`가 원본 차이로 거부됐다. push 50호출 비교 규칙은 바꾸지 않았다. 첫 실행 결과는 이 기록과 커밋 `55b2bcf` 이전 이력에 남는다. M1 재생의 scratch 검사는 처음 assert(재생 중단)로 넣었다가 출력 필드 `scratch_clean`으로 바꿔 비교기가 첫 차이로 거부하게 했다.
- **회귀**([요약](../../bench/oracle/m3_push_ftran_rust/regression/summary.txt) 14명령 종료0): 21a `push_ftran_check.py --controls/--acceptance/--self-test`, `push_round` 재생·비교, `push_chain` 재생·`push_step_chain_rust.py --compare`, `push_ftran` 재생·비교·self-test, py_compile, `git diff --check`, `cargo test --release --workspace` **136통과·0실패·기존 ignored12**, `cargo clippy --workspace --all-targets -- -D warnings` 0. clippy 첫 실행이 복합 대입·`as_chunks` 6건으로 실패해 비트 동일 형태로 고친 뒤 재생·변이·workspace를 다시 실행했다.
- 실행하지 않은 검사: solver·확장·Python 경로를 바꾸지 않아(`lib.rs`에 모듈 등록만) 확장 설치·pytest·Netlib·P3는 제외했다. 원본 캡처 0회.

## 7. 판정과 인수인계

- 작업 수행: **구현 완료(계약 범위)**. 동등성: push 자연 50호출의 한 RHS FTRAN 커널 경계·raw·입력 후상태·별칭·spike·scratch가 원본과 **비트 일치(통과)**. `21686f` 네 경로와 `21b9de` A1은 M1 원본 186호출에서도 통과.
- 미검증(C 근거 구현, 원본 비교 0회): sparse 목록 R·sparse→dense R, count0 조기 종료, L 출력0, 8-lane eta 본체, `ZERO` 경계 값 처리(위 생존 변이), save ∉ {0,1}, 힙이 out idx에 남기는 칸(미지값으로 둠). 미구현(오류 반환): `21b9de` A2·B, `21574d`(k5/p6). work 누적은 비교하지 않았다.
- 범위 밖: M1 `Lu`와의 통합(이 adapter는 원본 배열 표현을 직접 받는다), factor 생성·갱신, BTRAN 21c/21d, 지연 LU·재분해 21e/21f, solver 경로 연결, 루프22, x87, P3.
- 다음: **M3-04c-21c**(지원 BTRAN 수치 오라클). 이 작업의 경계 미관측 분기를 원본으로 닫으려면 별도 오라클(ZERO 경계·긴 eta·sparse R가 나오는 모델 탐색 또는 원본 수치 명령을 직접 부르는 합성 경계)이 필요하며 21e 착수 전 후보로 둔다.
- 재실행: `python3 tools/push_ftran_rust.py --write` → `cargo test -p rsolver-lp --test push_ftran` → `python3 tools/push_ftran_rust.py --compare` / `--self-test` / `--compare --mutate-rust <새 경로>`.
- 교훈: `docs/lessons/original.md`(2207de 버퍼 교환과 입력 후상태, F_ec=0 희소 U는 H2), `tooling.md`(미캡처 칸을 미지값으로 두는 재생, 첫 일치 뒤 변이로 판별력 확인·기존 커널 캡처 재사용).
