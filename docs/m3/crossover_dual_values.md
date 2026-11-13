# M3-04f-4a crossover 쌍대값 `251b4f`·S+0x2d8 재계산 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 선행: [04f-3 cleanup 진입 Rust](crossover_cleanup_rust.md), [04f-2 cleanup 진입 오라클](crossover_cleanup_entry.md), [21d 지원 BTRAN Rust](primal_push_btran_rust.md).

## 1. 범위와 주장

- 작업 종류: **오라클 구축 + 구현**. 04f-4(cleanup simplex 본체 → 최종 basis)는 범위가 커서 나눴다. 이번 04f-4a는 04f-3이 원본 값을 주입했던 쌍대값 d를 Rust가 직접 계산하게 한다. cleanup simplex 본체(`0cda5b` 반복)는 M1 simplex 궤적 일치에 의존하므로 04f-4b 이후(조사)로 둔다.
- 사용자 지시 "다음 M3 스프린트 진행해 너가 작업자야"로 이 세션이 직접 작업한다(작업자 선택 질문 생략).
- 완료 후 주장할 동작. `13f716` 문맥에서 `251b4f`가 불리는 순간의 원본 상태(S의 비용·상태 배열·basis header·행 방향 A·tolerance, 감산 목록, factor F 전체, work 값)에서 Rust가
  1. BTRAN 입력 c_B 구성(C 47–96: header 순 비용 수집, `[0, p3) ∪ [p4, p5)` 감산 목록의 `c_B[vstat[j]] −= (j < n ? p7[j] : p8[j−n])`),
  2. BTRAN `09e288` → `221173`(cache 인자 0, 입력 count = −1 조밀): 21d `crossover_btran::btran`으로 출력 y·입력 후상태·별칭·F+0x200, 그리고 wrapper의 `work = F200·F1f8 + work`(F1f8 = 1.0 임시, 반환 뒤 S+0x148로 복원),
  3. d(n+m) 계산(C 97–253·asm 0x1c9a39–0x1c9c5f): `d[j] = c[j]`(j < n), `d[n+i] = −y_i`, `tol(S+0x3a8) < |y_i|`인 행 i만 행 방향 A로 `d[col] = −(y_i·a) + d[col]`(행 순서·저장 순서), 끝으로 vstat = −2인 열의 d 부호 반전,
  4. push 뒤 S+0x2d8 재계산(`13f716` C 1225–1316·asm 0x1bf0a2–0x1bf17a·0x1bf951–0x1bfa08): 기저 `b += |d|`, 비기저 −3 건너뜀, −4는 0, −1·−2는 `v = (0 > −d) ? 0 : −d`(maxsd), `tol(S+0x380) > v`가 아니면 `nb += v`, 결과 `nb + b`,
  5. auto 쌍대 검사(0x1bfae2 지점): 04f-3 `dual_check`에 Rust d를 넣은 선택 bool r14
  를 원본과 정확·비트 일치로 낸다.
- 포함 분기: `251b4f` 호출 지점 4곳(`13b3d1` dual push의 0x1c6b49(감산 있음)·0x1c8533·0x1c87a9, `13f716` 뒤처리 0x1bf09e, 쌍대 검사 0x1bfae2). 조사 실행(아래 §6)에서 자연 Netlib 45 × Crossover 5값 1회에 953호출, 감산 목록이 있는 호출 533회, BTRAN 입력은 모두 조밀(count −1)이었다.
- 입력 전제: double(S+0x28 = 0), Threads = 1, Presolve = 0, Method = 2. 원본 factor는 캡처값을 입력으로 쓴다(밀도 근사 필드 F108·F110·F104·F140 포함, 21d와 같음).
- 제외와 후속:
  - push가 Rust로 만든 factor에서 이어 BTRAN하는 연결(push 연쇄 → 251b4f)은 하지 않는다. 시작 경계는 `251b4f` 진입의 원본 상태다(**04f-4b 후보**).
  - cleanup simplex 본체·최종 VBasis/CBasis(**04f-4c 이후**, M1 simplex 궤적 의존), dual push 본체 안에서 d를 소비하는 로직, x87 `251b4f` 변형(`0x29d3c0`), work 누적의 이후 소비, 시간.
  - `13b3d1` 안의 감산 목록·p7·p8의 생성 규칙은 비교하지 않는다(입력으로 캡처).
- 선행 commit: main `c1b2315`(Merge M3-04f-3). 미커밋 선행 변경 없음(main의 미추적 `.claude/settings.local.json`은 무관).

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 ELF file offset.
- `251b4f` = `../re/decomp/c/001c9730_PRIVATE0000000000251b4f.c` 전체(253줄), asm 0x1c9730–0x1c9c70. 인자: rdi S, rsi d, edx p3, ecx p4, r8d p5, r9 p6(목록), 스택 [rsp+8] p7, [+0x10] p8, [+0x18] p9(미사용), [+0x20] p10 = work 포인터(asm `mov 0xa8(%rsp),%rcx` → `09e288` 넷째 인자). 곱 누적은 `mulsd`·`xorps`(부호)·`addsd` = `−(y·a) + d`(FMA 없음).
- `09e288` = `../re/decomp/c/0012de10_PRIVATE000000000009e288.c`, asm 0x12de10–0x12de5d: F+0x200 = 0, 입력 count ≠ 0이면 `221173(F, in, out, 0)`(반환 주소 0x12de33), work가 있으면 `*work = F200·F1f8 + *work`. count = 0이면 out count = 0.
- `221173`: 21d와 같은 본체(`../re/decomp/c/0012de60_PRIVATE0000000000221173.c`), cache 인자 null.
- `13f716` 호출: C 1226(asm 0x1bf099, 반환 0x1bf09e, retry = 0·S+0x28 = 0일 때)과 C 1531(asm 0x1bfadd, 반환 0x1bfae2, 쌍대 검사). 두 곳 모두 p3–p8 = 0, p10 = `0x260(%rsp)`.
- 시작 경계: `251b4f` 진입. 종료 경계: `251b4f` 반환(d·F1f8·work·S+0x228 y), 0x1bf09e 지점은 이어서 `0x1bf2ee`의 S+0x2d8(편집 전), 0x1bfae2 지점은 `0x1bf6f2`의 r14.

## 3. 비교 계약

| 필드·사건 | 의미 | 비교 규칙 | 기대값 |
|---|---|---|---|
| BTRAN 입력 c_B(m) | `221173` 진입 입력 벡터(count −1) | 정확·비트 | `dv:btran:enter.input` |
| BTRAN 반환 | 출력·입력 후상태·별칭·F+0x200·scratch | 정확·비트 | `dv:btran:leave` |
| y(m) | S+0x228 값(반환 뒤 251b4f가 읽는 것) | 비트 | `dv:leave.y` |
| d(n+m) | `251b4f` 출력 | 비트 | `dv:leave.d` |
| work | `*p10` 반환 값 | 비트 | `dv:leave.work` |
| F1f8 | 반환 뒤 복원 값 | 비트 | `dv:leave.f1f8` |
| S+0x2d8 | 0x1bf09e 호출 뒤 재계산 | 비트 | 다음 `sel:enter.s2d8`(편집 전) |
| 쌍대 검사 r14 | 0x1bfae2 호출의 d로 `dual_check` | 정확 | `sel:decide.r14` |

- 오차 허용 없음. 부동소수는 little-endian 16진 비트(±0 포함). NaN·무한대가 나오면 그 사실을 기록하고 비트로 비교한다.
- 원본 재현성: 사례마다 독립 3회의 사건열(seq 제외 전체 필드)이 같아야 한다.
- 시간: 비교하지 않는다. work는 이 wrapper의 누적 한 단계만 비교한다.
- 판별력: 비교기 출력 변조와 Rust 규칙 변이(환경 변수 변이)가 거부되는지 확인한다.

## 4. 사례

| 사례 | 실제/합성 | 노리는 분기 |
|---|---|---|
| Netlib 45 × Crossover −1 | 실제 | dual push 3지점(감산 포함)·뒤처리 0x1bf09e. 조사에서 Crossover 1·2는 −1과 `251b4f` 사건이 같아 제외 |
| Netlib 45 × Crossover 3 | 실제 | primal 먼저인 경로의 같은 지점. 조사에서 Crossover 4는 3과 `251b4f` 사건(seq 제외)이 45모델 모두 같아 제외 |
| arms·retry·interior × Crossover 5값 | 실제(1b 자연) | 자연 쌍대 검사(retry auto), mode 1 |
| 04f-2 합성 16종 | 합성(0x1bf2ee slot 편집) | 쌍대 검사 강제(참·거짓). 편집은 `251b4f` 0x1bf09e 호출과 S+0x2d8 기록 **뒤**에 적용되므로 그 둘은 자연값 |

## 5. 재실행 명령

- 캡처: `tools/hrun.sh --timeout 590000 -- 'python3 tools/xo_dual_runs.py --crossover=-1,3 --reps 3 && python3 tools/xo_dual_runs.py --cases --reps 3'`(작업 worktree, Gurobi 한 프로세스).
- 원본 자료 검사: `python3 tools/xo_dual_check.py --controls`.
- 구현 재생: `python3 tools/xo_dual_rust.py --write` → `cargo test --release -p rsolver-lp --test xo_dual` → `python3 tools/xo_dual_rust.py --compare`.
- 판별력: `python3 tools/xo_dual_rust.py --self-test`, `--mutate-rust OUT`.
- 회귀: `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, 04f-3 `xo_cleanup_rust.py --write/--compare`, 21d `push_btran_rust.py --compare`.

## 6. 실행 결과

(작성 예정)

## 7. 판정과 인수인계

(작성 예정)
