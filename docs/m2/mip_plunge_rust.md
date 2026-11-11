# M2-06d 검증 계약과 결과: plunge 선택(`6e8eb3` 목록 경로) Rust 이식

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 원본 규칙과 기준 자료는 [M2-06c](mip_plunge_rule.md)(규칙 번호·RNG·규칙 3/12·꼬리)와 [M2-06c-2](mip_plunge_score.md)(규칙 1·2·5·6 점수, 모든 규칙 work)다. M2 인수인계의 첫 다음 작업이고, 사용자의 "다음 스프린트 진행해줘. 너가 작업자야" 지시로 착수했다(2026-11-11, 작업자는 이 세션).

## 1. 범위와 주장

- 작업 종류: 구현(L1 저장 기준 재생). 새 원본 캡처·Gurobi 실행 없음.
- 범위 결정: 스프린트 목록의 M2-06d 줄은 "06c의 규칙 하나"지만, M2 인수인계(06c-2 뒤)는 "06c·06c-2 규칙(번호·RNG·1·2·3·5·6·12·꼬리)"으로 넓혔다. 인수인계를 따른다.
- 완료 후 주장할 것: 새 `crates/mip/src/plunge.rs`에 06c 기준 자료 5사례 run 1의 **입력 필드만** 넣으면, 목록이 비지 않은 모든 최상위 `6e8eb3` 호출(9,864)에서 다음이 원본 캡처와 같다.
  - `seed`: 진입 k와 `W+0x28`로 만든 지역 RNG 슬롯 시드(뽑기 호출이면 `drw.st`, 아니면 `rule.st`).
  - `rule`: 규칙 번호(`rule.ecx`), hint 표시(`rule.r8b`), 규칙 단계 뒤 RNG 상태(`rule.st`), 뽑기 유무(`drw` 사건 유무). `5e6efd`는 원본이 부른 호출에서만 묻는다(그 밖에서 물으면 실패).
  - `pick`: 규칙 1·2·3·5·6·12의 선택 첨자(`pick.idx`)와 minEst 3n·본문 work 뒤 값(`pick.work`, 모든 규칙). 규칙 5는 Rust가 만든 RNG 상태에서 뽑는다(`rule.st`를 넣지 않는다).
  - `5fb13c` 조상(`ancestor`): 규칙 1·6의 후보 판정과 꼬리에서 Rust가 부모 조회로 걷는다. 결과(`anc.res`)는 넣지 않는다.
  - `tail`: 선택 노드(= `leave.node`), δ와 d′(`anc` sel의 d), 선택 노드 조상(`anc` sel의 res), 문턱(`thr.t20`, 조상이 null이면 `thr` 사건 없음), 항목별 유지/힙(단계 4 + 삽입)/버림(단계 0) 순서(`rm`·`ins`), 남은 목록 순서와 개수·work(`tend`).
- 입력(Rust에 주는 것): `pin`의 k·`W+0x28`·hint 입력·`M+0x848`·`Mx+0x808`·`Mx+0x40`·lp0 n/nint·BranchDir·MIPFocus, `s70`의 `5e70c1` 반환과(있으면) `5e6efd` 반환, `rule.esi`(pop 뒤 k), `rpre.work`(minEst 전 work), `pick.list`(id, est, depth, `I+0x41`, bound), `tail`의 `M+0x2c`·`T+0x138`·`T+0x120`, `thr`의 `5f9a41`·`5e7796`·`5e8239(M,2)`·`5e7204`(있을 때만, Rust가 조상이 null이 아닐 때 묻고 없으면 실패), 같은 호출 `anc` 사건 사슬에서 만든 부모 표(§2.2).
- 기대값으로만 쓰는 것: `rpre.ecx`, `rule.ecx`·`r8b`·`st`·`work`, `drw.st`, `pick.idx`·`work`, `anc.d`·`res`, `thr.t20`, `rm`·`ins` 순서, `tend`, `leave.node`.
- 선택 첨자를 입력(`given`)으로 쓰는 호출(정확한 수를 센다):
  - 규칙 4·7(385호출): 점수 입력(`I+0x34`·`+0x38`·`+0x40`, incumbent x)이 캡처에 없다. Rust는 이 규칙 점수를 옮기지 않고 work만 더한다. 꼬리는 그대로 비교한다. 후속 **M2-06c-3**.
  - 규칙 1·6 재생 불가(34호출): 06c-2와 같은 호출(꼬리 조상이 null이라 사슬이 없음)이어야 한다. Rust가 모르는 부모를 물으면 그 호출의 선택만 버리고 `given`으로 꼬리를 이어 간다. 꼬리에서 모르는 부모를 물으면 실패다.
    - 결과 뒤 변경(§6): 06c-2의 34호출 중 Rust가 모르는 부모 없이 계산한 호출은 `pick.idx`와 비교한다. 남은 재생 불가는 06c-2 34호출의 부분집합이어야 한다.
- 제외 범위와 후속:
  - 규칙 4·7·9·11 점수(Rust는 `None`): 4·7은 M2-06c-3, 9·11은 0회.
  - hint 경로(`r8b`)의 꼬리 앞 형제 교환(`0x5b44ba`–`0x5b4500`, `0x5b4b1b`–`0x5b4b90`): 해독 미완, 0회. Rust는 hint면 규칙 3만 정하고 교환은 하지 않는다(`shortcut:` 주석).
  - 병렬 worker 힙(`5e2919 ≠ 0`), `6e5802` 실패 뒤 압축, 힙 pop·전역 힙 경로(06b `NodeHeap`), 지연 목록·node file: 범위 밖.
  - `bnb.rs` 연결: 06b·06f-3과 같이 이식만 한다. 연결에는 노드 처리 순서 전체와 `W+0x28`·k 갱신이 필요하다.
  - `W+0x28` 쓰는 곳(06c 후속 질문): §2.3에 정적 확인 결과만 적는다. 실행 대조는 하지 않는다.
  - 0회 분기(asm대로 옮기고 미검증): 규칙 11·9 번호 절, MIPFocus 4 덮어쓰기와 δ = 3, δ = 10, 제외 항목(`I+0x41`)과 선택 노드 제외 시 문턱 −1e100, 규칙 5 순환 검색, `5e70c1` = 0·`5e6efd` = 0 절(06c 변종 생존).
- 선행: main `a25adca`(M2-06c-2b 병합 포함). 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`, 주소는 file offset. `6e8eb3` = `0x5b29d0`(본체와 조각 `0x5b4c50`까지). 식은 [06c §2](mip_plunge_rule.md#2-원본과-상태-경계)와 [06c-2 §2](mip_plunge_score.md#2-원본과-상태-경계)의 asm 해석을 그대로 옮긴다. 이번에 asm을 다시 해독하지 않았다.
- 시작 경계: `6e8eb3` 진입(시드)과 pop 뒤 규칙 단계(`0x5b3326`). 종료 경계: 꼬리 끝 probe `0x5b49d1`(`tend`)과 반환 노드.

### 2.1 Rust로 옮기는 식

| Rust | 원본 | 내용 |
|---|---|---|
| `seed(k_entry, w28)` | `0x5b29e1`–`0x5b2a5b` | x = (double)k + w28, x < 2⁶³이면 `cvttsd2si`(i64 → u64), 아니면 (x − 2⁶³)의 i64 \| 2⁶³ |
| `rule(&RuleIn, f6efd, &mut Rng)` | `0x5b3326`–`0x5b3553` | 06c §2.2 순서. 뽑기는 `Rng::next_f64() < 0.9`(`a7b54f`), MIPFocus 4면 5 |
| `pick(rule, items, rng, parent, work)` | `0x5b3541`–`0x5b44a0` | minEst(2개씩, `minsd` 뒤 `ja`) + 3n, 규칙별 본문(06c-2 §2), −1이면 0. 4·7·9·11은 work만, `None` |
| `ancestor(node, depth, d, parent)` | `5fb13c` | `(d \| depth − d) < 0`이면 null, 아니면 부모를 depth − d번(null에서 멈춤) |
| `tail(list, idx, &TailIn, thr, parent, removed, work)` | `0x5b44a9`–`0x5b49d6` | 선택 항목을 끝 항목과 맞바꿔 뺀 뒤 δ·d′·조상·문턱·항목별 판정, work 4n(조상이 있을 때만) |

- 규칙 1·6의 첫 반복은 asm에서 새 최댓값마다 `5fb13c`를 부르고 마지막 결과만 남는다. `5fb13c`는 읽기만 하는 순수 함수이므로 Rust는 첫 최심 항목에서 한 번만 부른다(같은 결과).
- `minsd`·`maxsd`는 `<`·`>` 선택으로 쓰고 `f64::min`·`max`를 쓰지 않는다(NaN·±0 처리 차이). work는 단계마다 `n·c + w` 한 번씩 더한다. log 분기는 `(depth as f64 * 0.2 + 1.0).ln() / 2f64.ln()`(glibc `log`, 06c에서 Python `math.log`와 같은 값 확인)을 i32로 자른다.
- 시드에 쓰는 k는 진입 k, 규칙·꼬리에 쓰는 k는 pop 뒤 k(`rule.esi`)다. 캡처에서 `tail.k` = `tail.esi` = `rule.esi`이고 `tail.m2c` = `pin.m2c`인지 입력 생성 때 확인한다(다르면 실패).

### 2.2 부모 표(입력)

같은 호출의 `anc` 사건마다 사슬 c(시작 노드부터 위로 id)와 필요 길이 L = depth − d + 1(d < 0 또는 depth − d < 0이면 사슬 길이 1)로 부모를 만든다. i < len(c) − 1이면 parent(c[i]) = c[i+1]. len(c) < L이면 parent(c[last]) = null(원본이 null에서 멈춤). len(c) = L이면 c[last]의 부모는 모른다. 사슬이 64개 상한이면 마지막 부모를 모른다. 같은 id에 다른 부모가 나오면 입력 생성 실패. Rust가 모르는 부모를 물으면 하네스가 그 질의를 표시한다(null로 바꾸지 않는다).

### 2.3 `W+0x28` 쓰는 곳(정적 확인만)

06c 캡처의 호출자는 모두 `5c239f`(`enter.ret`)다. `5c239f` C:104–105는 노드 루프 반복마다 `param_1[5] = *(P + 0x60)`(param_1 = W, P = `*W`)을 쓴다. 즉 `W+0x28`는 루프 시작 때 `(*W)+0x60`의 복사본이다. 다른 호출자 `5c37fd` C:307·`5c3438` C:341은 `W+0x28 = 이전 값 + 1.0`이다. `(*W)+0x60`을 쓰는 곳과 재시작 때 건너뛰는 이유는 찾지 않았다(미확인, 연결 작업에서 확인).

## 3. 비교 계약(첫 재생 전 고정)

`tools/mip_plunge_rust.py`가 판정한다. 하나라도 실패하면 종료 코드 1이고 첫 차이(사례·호출·줄·원본·Rust)를 출력한다.

| 검사 | 규칙 |
|---|---|
| M 입력 | 06c 5사례 run 1의 SHA-256이 `bench/oracle/m2_06c/manifest.json`과 같다(3회 동일은 06c C0) |
| X 입력 생성 | §1의 입력 필드만 쓴다. `tail.k` = `tail.esi` = `rule.esi`, `tail.m2c` = `pin.m2c`, `rpre.ecx` = `rule.ecx`(MIPFocus ≠ 4), work 포인터 null 0회, 부모 표 충돌 0. 각 사례 입력 끝 `end` 줄(없으면 Rust 실패) |
| R 규칙 | 호출마다 Rust 규칙 = `rule.ecx`, hint = `rule.r8b`, 시드 = `drw.st`(뽑기) 또는 `rule.st`(없음), 뽑기 유무 = `drw` 유무, 규칙 단계 뒤 상태 = `rule.st` |
| P 선택 | 규칙 1·2·3·5·6·12에서 Rust 첨자 = `pick.idx`. 모든 호출에서 Rust work = `pick.work`(비트) |
| T 꼬리 | 선택 id = `leave.node[0]`, d′ = `anc`(sel).d, 조상 = `anc`(sel).res, 문턱 = `thr.t20`(비트, 조상 null이면 `thr` 없음), 유지/힙/버림 순서 = `rm`·`ins` 사건 순서, 남은 목록 = `tend.list`, `tend.work`(비트) |
| K 개수 | 5사례 합계: 호출 9,864, Rust 선택 9,445(1·2·5·6 7,465 + 3·12 1,980), `given` 규칙 4·7 385, 재생 불가 34(06c-2와 같은 호출), 뽑기 963, 유지 32,979, 힙 6,512, 버림 5, 문턱 < 1e100 1,964, null 조상 58. 결과 뒤 변경(§6): 34호출은 Rust가 `pick`(원본과 비교) 또는 `unrep`을 내고, `unrep` 수 ≤ 34 |
| T 변조 | Rust 출력 변조가 각각 거부된다: 규칙 번호, RNG 상태, 시드, 선택 첨자, pick work, d′, 조상 id, 문턱, `rm` 줄 삭제, 힙/버림 맞바꿈, 남은 목록 순서. 입력 변조: 한 사례의 `end` 줄 삭제는 그 사례만 `panic`으로 거부되고 나머지는 통과한다 |
| V 코드 변이 | 작업 공간 복사본에 아래 변이를 넣고 사례별 거부가 표와 같아야 한다. 표는 06c §6.2·06c-2 §6의 같은 Python 변종 결과에서 옮겼다. 다르면 입력이 기대값을 새어 받거나 Rust가 다른 식을 쓴다는 뜻이므로 실패다 |

| 변이 | 대응 | 거부 사례(기대) |
|---|---|---|
| D1 규칙 3·12 부호 맞바꿈 | V1 | 5사례 |
| D2 시드를 pop 뒤 k로 | V5 | 5사례 |
| D3 뽑기 문턱 0.5 | V6 | 5사례 |
| D4 δ = 2 절 끔 | V7 | 5사례 |
| D5 δ = 6 절 끔 | V8 | 5사례 |
| D6 log 분기 반올림 | V9 | tr12-30 |
| D7 유지에서 조상 비교 무시 | V10 | 5사례 |
| D8 유지에서 bound 비교 무시 | V11 | pk1-pool |
| D9 제거는 항상 힙 | V12 | neos5·tr12-30 |
| D10 규칙 3·12 동률은 뒤 항목 | V4 | 없음(생존) |
| E1 규칙 1·6 조상 비교 끔 | W1 | 5사례 |
| E2 조상 깊이 maxd − 1 | W3 | 5사례 |
| E3 null 조상은 불일치 | W5 | pk1-pool |
| E4 규칙 1 depth 부호 + | W6 | markshare_4_0 제외 4사례 |
| E5 불일치 때도 best 갱신 | W8 | 5사례 |
| E6 규칙 2 동률은 뒤 항목 | W9 | neos5 |
| E7 규칙 5 시작 = trunc(u·n) | W10 | tr12-30 제외 4사례 |

- 수치: 모든 double은 비트 비교(16진 비트 문자열). 오차 허용 없음.
- 시간·병렬: Threads=1 캡처, 시간 분기 없음.
- 원본 재현성: 06c C0(사례마다 3회 gz 바이트 동일)를 재사용한다. 이번 판정은 run 1만 쓰고 M에서 해시를 확인한다.

## 4. 사례

06c 기준 자료 그대로(`bench/oracle/m2_06c`, 실제 사례 5개, 재캡처 없음): markshare_4_0·pk1·neos5·tr12-30(NodeLimit 2000)·pk1-pool(+PoolSearchMode=2). 분기별 실행 증거는 06c §6.2·06c-2 §6 표와 같다. 합성 사례는 없다.

## 5. 재실행 명령

- 판정: `python3 tools/mip_plunge_rust.py`(기대: M·X·K 통과, 사례별 R/P/T `rust = original`, `PASS`, 종료 0).
- 판별력: `python3 tools/mip_plunge_rust.py --self-test`(T 변조·입력 변조·V 변이 17종, `PASS (+ self-test)`, 종료 0). Herdr `tools/hrun.sh`로 실행한다.
- 06c·06c-2 검사기 회귀(검사기는 바꾸지 않음): `python3 tools/mip_plunge_score_check.py --self-test`.
- Rust 회귀: `cargo test -p rsolver-mip`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`. Python 패키지를 바꾸지 않으므로 pytest는 해당 없음(공유 `.venv`를 쓰지 않는다).

## 6. 실행 결과

- 실행 commit `55b2653`(코드·판정기), 기준 main `a25adca`. 판정기 SHA-256 `e6ea929c…`, `plunge.rs` `c6d58ad1…`, `plunge_l1.rs` `7abd00fa…`(앞 16자리). 입력은 `bench/oracle/m2_06c` run 1(manifest 해시 확인: markshare_4_0 `9d5aee70…`, pk1 `2e4dffbd…`, neos5 `8f75b832…`, tr12-30 `9c441f7e…`, pk1-pool `7a95437f…`). x86-64 WSL2, `cargo test --release`. Gurobi 실행 없음.
- 첫 실행의 차이 두 가지(코드 식 차이 아님):
  - `tend` 줄: 남은 목록이 비면 Rust 출력 끝에 공백이 붙었다(하네스 형식). 하네스를 고쳤다.
  - pk1 호출 213: 기대 `unrep 0`, Rust `pick 0`. 06c-2의 Python 재생은 꼬리 조상이 null인 호출에서 조상 질의가 생기면 모두 재생 불가로 쳤다. 그런데 질의 깊이가 항목 깊이와 같으면(steps = 0) `5fb13c`는 부모를 걷지 않고 노드 자신을 돌려준다. Rust는 이 경우와 부모 표로 닿는 경우를 계산했고 원본 첨자와 같다. 그래서 계약의 재생 불가 비교를 "06c-2 34호출에서 Rust는 `pick`(원본 비교) 또는 `unrep`, `unrep` ≤ 34"로 바꿨다(§1·§3 표시). 다른 비교 규칙은 그대로다.
- 판정(`python3 tools/mip_plunge_rust.py --self-test`, Herdr, 종료 0, `PASS (+ self-test)`):

| 사례 | 호출 | Rust 선택 (1·2·5·6 / 3·12) | `given` 4·7 | 06c-2 재생 불가 → Rust 선택 / 남음 | 뽑기 | 유지 | 힙 | 버림 | 문턱 < 1e100 | null 조상 |
|---|---:|---|---:|---|---:|---:|---:|---:|---:|---:|
| markshare_4_0 | 2000 | 707 / 1090 | 203 | 0 → 0 / 0 | 548 | 3492 | 406 | 0 | 0 | 2 |
| pk1 | 1984 | 1666 / 274 | 37 | 7 → 3 / 4 | 142 | 8001 | 1508 | 0 | 0 | 13 |
| neos5 | 1977 | 1567 / 325 | 77 | 8 → 4 / 4 | 106 | 6425 | 1252 | 2 | 0 | 14 |
| tr12-30 | 1922 | 1788 / 100 | 27 | 7 → 4 / 3 | 51 | 6916 | 1493 | 3 | 0 | 12 |
| pk1-pool | 1981 | 1737 / 191 | 41 | 12 → 6 / 6 | 116 | 8145 | 1853 | 0 | 1964 | 17 |
| 합계 | 9,864 | 7,465 / 1,980 | 385 | 34 → 17 / 17 | 963 | 32,979 | 6,512 | 5 | 1,964 | 58 |

  모든 호출에서 시드·규칙 번호·hint·뽑기 유무·RNG 상태·`5e6efd` 질의 유무, Rust가 계산한 선택 9,462(9,445 + 17), pick work, 선택 노드·d′·조상·문턱, 항목별 유지/힙/버림 순서, 남은 목록·work가 원본 캡처와 같다(출력 85,371줄). K 합계는 §3 표와 같다.
- T 변조 11종 모두 거부(규칙 번호, 뽑기 뒤 RNG 상태, 시드, 선택 첨자, pick work, d′, 조상 id, 문턱, `rm` 줄 삭제, 힙/버림 맞바꿈, 남은 목록 순서). 입력 변조(neos5 `end` 삭제)는 그 사례만 `panic`으로 거부되고 나머지 4사례는 통과했다.
- V 변이 17종: 사례별 거부가 §3 표와 모두 같다(D1–D5·D7·E1·E2·E5 5사례, D6 tr12-30, D8 pk1-pool, D9 neos5·tr12-30, E3 pk1-pool, E4 markshare_4_0 제외 4사례, E6 neos5, E7 tr12-30 제외 4사례, D10 생존). E2의 pk1-pool은 다른 첨자가 아니라 `unrep`(꼬리 사슬보다 위 부모를 물음)으로 거부됐다. 06c·06c-2의 Python 변종과 결과가 같으므로 입력이 기대값을 새어 받지 않는다.
- 회귀: `cargo clippy --workspace --all-targets -- -D warnings` 0, `cargo test --release --workspace` 125 통과·무시 11(새 무시 테스트 `plunge_l1` 포함), `python3 tools/mip_plunge_score_check.py --self-test` `ACCEPTANCE PASS (+ self-test)`(06c C0·R/G/S/T 포함).
- 실행하지 않은 검사: pytest(Python 패키지 미변경, 공유 `.venv` 미사용), 원본 재캡처(06c 자료 재사용, 3회 동일은 06c C0).

## 7. 판정과 인수인계

- 작업 수행 상태: 완료(구현 L1).
- 동등성: 통과(L1 저장 기준 재생, 비트 비교). 06c 5사례 9,864호출에서 §3 R/P/T 필드가 원본과 같다.
- 통과 범위와 한계:
  - 실행 확인(변이 거부까지): 진입 k 시드, 0.9 뽑기 문턱, 규칙 3·12 부호, 규칙 1·6 조상 비교·깊이 maxd − 2·일치할 때만 best 갱신·null == null, 규칙 1 −depth, 규칙 2 엄격 최소(동률 앞), 규칙 5 정수 r mod n, δ = 2·6·log 절(log는 tr12-30), 조상 비교·bound 문턱(pk1-pool)·힙/버림 갈림.
  - 실행 확인(판별 변이 생존): 규칙 3·12 동률(정확 동률 0, D10), 06c·06c-2의 생존 변종(사전식 점수, 첫/마지막 최심 항목, work 한 번 더하기, `5e70c1 == 0`·`5e6efd == 0` 절, 문턱 조건의 `5e8239` 비교).
  - 입력으로 둔 것: 규칙 4·7 선택 385호출(점수 미이식, work만 비교), 재생 불가 17호출(꼬리 사슬 위 부모 필요). `5e70c1`·`5e6efd`·`5f9a41`·`5e7796`·`5e8239`·`5e7204`·`W+0x28`·k·`M+0x2c`·`T+0x120`/`+0x138` 값.
  - asm만(0회, 코드는 옮김): 규칙 9·11 번호 절, MIPFocus 4와 δ = 3, δ = 10, 제외 항목과 선택 노드 제외 시 문턱 −1e100, 규칙 5 순환 검색, 규칙 8·10.
  - 옮기지 않음: 규칙 4·7·9·11 점수(`None`), hint 형제 교환·병렬 worker 힙(`shortcut:` 주석), 힙 pop 경로(06b `NodeHeap`), `bnb.rs` 연결.
  - `W+0x28`: `5c239f` C:104–105가 노드 루프마다 `(*W)+0x60`을 복사한다(§2.3). 그 값을 쓰는 곳은 미확인.
- 후속:
  - **M2-06c-3**(조사·오라클): 규칙 4·7 점수 입력 캡처. 끝나면 `plunge::pick`에 4·7을 더하고 이 판정기의 `given` 385를 줄인다.
  - **M2-06d-2**(구현·연결, 미정): `bnb.rs`에 `NodeHeap`·`NodeLife`·`plunge`를 잇는다. `(*W)+0x60` 쓰는 곳과 k(`W+0x30`) 갱신, plunge 목록 삽입 규칙이 먼저 필요하다.
  - 다음 명령: `python3 tools/mip_plunge_rust.py --self-test`(이 판정기), 06c 자료를 바꾸면 06c·06c-2 검사기 self-test부터.
- 작업 branch `work/m2-06d`, 기준 main `a25adca`. 커밋·병합 상태는 M2 인수인계를 따른다.
- 교훈: `docs/lessons/tooling.md`(Python 재생의 재생 불가 분류는 steps = 0 질의를 과하게 셈).
