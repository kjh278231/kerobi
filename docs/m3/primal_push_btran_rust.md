# M3-04c-21d 지원 BTRAN Rust 수치 adapter 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 원본 기준 자료는 [M3-04c-21c 오라클](primal_push_btran_oracle.md), 같은 틀의 FTRAN 구현은 [M3-04c-21b](primal_push_ftran_rust.md)다.

## 1. 범위와 주장

- 작업 종류: **구현**. 기준 main `4c99d26`(21c 병합), branch `work/m3-04c-21d`, worktree `../rsolver-m3-04c-21d`. 선행 미커밋 없음. main의 미추적 `.claude/settings.local.json`은 무관하며 보존한다. 사용자의 "다음 M3 스프린트 진행해 너가 작업자야" 지시로 이 세션이 직접 작업한다(작업자 선택 질문 생략).
- 완료할 동작: crossover push 지원 라운드의 BTRAN 본체 `221173`(cache 인자 F+0x1d0)과 희소 Uᵀ 커널 `21a941`을 원본 factor 표현(F+0x20… 배열)을 그대로 받는 Rust adapter `crossover_btran`으로 구현한다. 21c `btran:enter` 상태(factor 전체·입력·out count·cache·F1d8·F+0x200·level bucket)에서 시작해 **커널 진입/반환의 in/out 벡터·별칭·scratch·cache·F1d8·F+0x200**, 본체 반환의 **raw out·입력 후상태·별칭·cache·F1d8·scratch·level bucket·F+0x200**을 원본 32호출과 비트 비교한다.
- 포함 분기(C 근거로 전부 구현, 자연 관측 수를 결과에 센다):
  - 본체 진입: cache 경로(U 행 r 빈 → 일반 경로, 비영 → F1d8=r·커널 강제·cache 복사), 일반 경로의 희소 Uᵀ 판정 `(int)(F108+F110)·count ≤ m`(asm `0x12dee6–0x12df33`), 조밀 Uᵀ(C:30–160), 커널 출력 0 조기 반환.
  - 커널 `21a941`: F108>5 순차 경로(C:59–158), heap 경로(C:160–339: U 위치 min-heap, `m−top ≤ 6·n`이면 순차 전환), 순차 꼬리(C:340–395).
  - eta: 없음(C:1231), 조밀 eta(희소 Uᵀ 출력에서 `m < F104·count`, 또는 조밀 Uᵀ 뒤; C:1237–1352·1496–1572), 희소 eta(F1a8 목록·`v ≠ 0`·`ZERO ≤ |w|` 압축, C:1353–1494).
  - Lᵀ(C:241–1189): 희소 판정 `count ≥ 0 ∧ F104·count ≤ m ∧ F140 < 0.1m`, 그 안의 heap 경로(F140<0, 행 max-heap, `top ≤ 6·n`이면 순차), level 경로(F140≥0, F1a8 bucket), 경로 B(`cnt_ge·(tail/2d)² > d`), 희소 출력의 F+0x20 치환. 조밀 Lᵀ(C:1573–1766)와 F+0x30 gather, 헤더 교환.
  - work: 커널·본체의 F+0x200 누적을 원본 연산 순서대로 계산해 비트 비교한다(21c에서는 기록만 했던 필드를 이번에 비교 대상으로 올림).
- 제외 범위: 다른 caller(일반 BTRAN `09e288`·simplex `9e376` 문맥), cache를 소비하는 `05b257` FT 갱신, 지연 LU·FT·재분해 연쇄(21e·21f), M1 `Lu`(자체 표현)와의 통합, solver 경로 연결·루프22·batch23·x87·P3. F108·F110·F104·F140 등 밀도 근사 필드는 캡처값을 입력으로 쓰며, Rust factor가 그 값을 만드는 규칙(M1-02k 일반 밀도 근사)은 이번 범위에서 **제외**한다(21f에서 factor 생성과 함께 다룬다). M1 공유 코드(`lu.rs`)는 바꾸지 않는다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 21c manifest의 바이너리·캡처·도구 해시를 그대로 검사한다(`push_btran_check.check_files`·`controls`).
- C(직접 읽음): `../re/decomp/c/0012de60_PRIVATE0000000000221173.c` 1–1766 전체, `00130b70_PRIVATE000000000021a941.c` 1–430 전체. asm은 21c `bench/oracle/m3_push_btran/source.asm`: 커널 인자 `0x12e567–0x12e586`(xmm0 = F+0x10 ZERO, rdx = F+0x20), FMA 명령 0개(스칼라 SSE `mulsd`/`addsd`/`subsd`/`divsd`).
- 커널의 factor 읽기(`param_4 = F+0x20` 기준): F+0x40 행→U 위치, F+0x48 U 위치→행, F+0x50 대각, U 행 F+0xb0/0xb8(시작/끝)·F+0xd0(열)·F+0xc8(값), F+0xf8 접두 표시, F+0x100 t, F+0x108(>5 판정 단독). 본체 추가: F+0x104, F+0x108+F+0x110, F+0xd8, F+0xe4, L 열 시작 F+0x58(꼬리 nnz `F58[m]−F58[e4]`), L 행 F+0x70/0x78/0x80, level F+0x130/0x138/0x140, eta F+0x180/0x184/0x198, F+0x1e0(work), scratch F+0x1a8(i32, level 목록과 bucket·희소 eta 목록)·F+0x1b0(i32 표시)·F+0x1b8(f64 값), cache F+0x1d0, F+0x1d8.
- 시작 경계: 21c `btran:enter`(wrapper가 F+0x200=0을 쓴 뒤 본체 진입). 종료: `btran:leave`. Rust 입력은 enter의 factor·입력 벡터·out count·cache(count·목록)·F1d8·F+0x200·bucket뿐이다. out 구조체의 idx/val 버퍼 내용, cache 버퍼의 count 밖 칸, F1a8[0..m)은 캡처되지 않으므로 **미지값**으로 시작하고, 비교 대상 영역에서 미지값을 읽으면 그 필드는 실패다(21b와 같은 방식).
- 버퍼 모델: 원래 in/out의 idx·val 버퍼 4개(길이 m)와 두 구조체(count, idx 버퍼, val 버퍼)를 원본처럼 교환한다. 별칭은 구조체가 가리키는 버퍼의 원래 소속(`in`/`out`)이다. 커널·Lᵀ heap이 out idx 버퍼 위쪽에 쓰는 heap 칸도 원본과 같은 위치에 쓴다.
- 원본 연산 규칙(구현에 그대로 옮김). 곱셈은 교환해도 비트가 같고, `-(a·b)+c`는 `c−a·b`와 비트가 같다(IEEE). 덧셈 결합 순서는 C 그대로 둔다.
  - work: 커널 `F200 = acc + F200`(acc는 C 순서의 지역 합). 조밀 Uᵀ `F200 = (c·2+m)+F200`, `F200 = (m+m)+F200`, 행마다 `len·2+F200`. eta는 지역 `x184·2+F200`을 조밀 Lᵀ로 넘기고, 희소 eta는 `(((2ne + 5c) + Σ3len) + 2(n−c)) + 3n + F200`(C:1488–1490 결합 순서). 희소 Lᵀ heap `l·7·(2m+F1e0)/(m+1)`, level `l·5·(F1e0+2m)/(m+1)`, B `3c + Σ2len + 2((d8−e4)+max) + …`. 조밀 Lᵀ `(dv + m) + m` 뒤 행마다 `len·2+F200`.

## 3. 비교 계약

| 대상 | 규칙 |
|---|---|
| 커널 사건 | 호출별 (enter, leave) 쌍의 유무가 원본과 같음. enter: in 구조체 count·목록·값, out count, 별칭, scratch(F1b0·F1b8의 0 아닌 칸 목록), cache(count·목록·값), F1d8, F+0x200. leave: 위 + out 구조체 |
| 본체 leave | raw out(count·목록·값), 입력 후상태(in 구조체), 별칭, cache, F1d8, scratch, level bucket(F140≥0일 때 F1a8[m..m+F140]), F+0x200 |

- 정수·목록·순서는 정확, f64는 ±0 포함 little-endian 비트. 미지값·NaN·무한대는 실패. 원본 기대값은 Python 비교기만 읽고 Rust 입력에는 넣지 않는다.
- 원본 재현성은 21c 정식 21회(7사례 각 독립3회 동일)를 재사용한다. 이번 작업의 원본 실행은 0회가 목표다.
- 비교기 판별력: trace 변조(사례마다 출력 값 비트·count·입력 후상태·별칭·cache·F1d8·scratch·bucket·work·커널 사건)와 Rust 소스 변이(임시 Cargo 사본에서 경로 판정·문턱·순서·교환·work 결합을 바꿈)를 실행한다. 변이 생존은 원본 자료에 그 경계가 없다는 뜻으로 따로 기록하고 통과로 세지 않는다.

## 4. 사례와 원본 자료

| 사례 | 출처(21c 정식 `*-1`, 2·3과 동일 검증됨) | 호출 | 노리는 분기 |
|---|---|---|---|
| arms | `arms-1.json` | 17 | U 행 빈 → 일반 희소 Uᵀ, eta0, 희소 Lᵀ |
| retry | `retry-1.json` | 4 | 같음 |
| interior | `interior-1.json` | 1 | 같음 |
| adlittle | `adlittle-1.json.gz` | 2 | 빈 행 1·cache 경로 1(조밀 Lᵀ) |
| scsd1 | `scsd1-1.json.gz` | 1 | cache 경로 희소 Lᵀ |
| capri | `capri-1.json.gz` | 7 | level 구조, 빈 행 2·비영 5, 희소/조밀 Lᵀ |

- 자연 미관측(21c §6): eta>0(희소·조밀 eta), 조밀 Uᵀ, 커널 출력 0·−1, 입력 count≥2·dense 입력. 이 분기의 Rust 코드는 C 근거 구현이며 결과에 **미검증**으로 센다.
- M1-02i(kb2 dual DSE의 같은 본체, 조밀 eta 17피벗)는 L·eta 입력을 Rust 덤프에서 가져온 재생이라 원본 상태만으로 입력을 만들 수 없어 이번 판정 자료로 쓰지 않는다.

## 5. 재실행 명령

- 기준 재검사: `python3 tools/push_btran_check.py --controls --acceptance --self-test`.
- 구현 재생: `python3 tools/push_btran_rust.py --write` → `cargo test -p rsolver-lp --test push_btran` → `python3 tools/push_btran_rust.py --compare`.
- 오류 검출: `--self-test`(trace 변조), `--compare --mutate-rust <새 결과경로>`(소스 변이).
- 회귀: `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, 21b `push_ftran_rust.py --compare`, 선행 push 재생(`push_round_rust.py --compare`, `push_step_chain_rust.py --compare`), py_compile, `git diff --check`. 확장·solver 경로는 바뀌지 않으므로 pytest·Netlib·P3는 제외하고 이유를 기록한다.

## 6. 실행 결과

(실행 후 작성)

## 7. 판정과 인수인계

(실행 후 작성)
