# M2-06d-3 검증 계약과 결과: plunge 규칙 4·7·9 Rust 이식과 본문 조상 질의

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 앞선 작업은 [M2-06d](mip_plunge_rust.md)(Rust `plunge`, 규칙 4·7 385호출과 재생 불가 17호출은 원본 선택을 입력으로 씀)와 [M2-06c-3](mip_plunge_r47.md)(규칙 4·7·9 식, 본문 `5fb13c` 질의, 규칙 5 뒤 RNG를 새 캡처로 확정)이다. 사용자의 "다음 스프린트 진행해. 너가 작업자야" 지시로 M2 인수인계의 첫 다음 작업을 맡았다(2026-11-11, 작업자는 이 세션).

## 1. 범위와 주장

- 작업 종류: 구현(L1 저장 기준 재생). 새 원본 캡처·Gurobi 실행 없음(06c-3 캡처 재사용).
- 완료 후 주장할 것: `bench/oracle/m2_06c3` 7사례 run 1의 **입력 필드만** 넣으면 목록이 빈 호출을 뺀 모든 최상위 `6e8eb3` 호출(13,830)에서 06d 계약 §3의 R·P·T 항목과 아래가 원본 캡처와 같다.
  - 규칙 4·7·9 선택 첨자(`pick.idx`)를 Rust가 계산한다(06d의 `given` 385호출 → 0, BranchDir 사례 3,966호출 추가).
  - 규칙 1·6 본문의 `5fb13c` 질의: Rust가 묻는 순서와 인자(시작 노드, depth, d)가 `ranc` 사건과 같다. 답은 `ranc`의 결과를 순서대로 준다(06d의 재생 불가 17호출 → 0).
  - 선택 뒤 RNG 상태: 규칙 5면 `r5.st`, 그 밖에는 `rule.st`(pick이 슬롯을 건드리지 않음).
  - 규칙 4가 x를 읽는 횟수 = `x4` 항목 수. `x4`는 원본의 실제 읽기 기록이 아니라 캡처 도구가 `5e6f57` 반환 때 `I+0x41 = 0`·`I+0x34 ≥ 0` 필터로 만든 목록이므로, 이 비교는 asm 건너뛰기 조건을 옮긴 도구 필터와의 일치다. x는 `x4`에 있는 값만 주고 그 밖의 칸을 읽으면 그 사례가 실패한다.
- 06d에서 바꾸는 계약(이유: 06c-3 캡처가 06d가 입력으로 쓰던 06c 자료를 포함하고 점수 입력을 더 담는다):
  - 입력 자료를 `bench/oracle/m2_06c`(5사례)에서 `bench/oracle/m2_06c3`(7사례)로 바꾼다. 06c-3 검사기 C0이 5사례 run 1의 투영 = 06c run 1을 이미 확인했다.
  - `given` 줄과 `unrep` 출력을 없앤다. `pick`이 `None`(규칙 11)을 내면 그 사례는 panic으로 실패한다.
  - `pick`의 `parent` 인자를 본문 `5fb13c` 질의 클로저 `anc(node, depth, d)`로 바꾼다. 첫 반복은 asm대로 새 최댓값마다 묻는다(06d는 첫 최심 항목에서 한 번만 물었다. 결과는 같지만 질의 순서를 비교하려면 원본과 같이 물어야 한다). 꼬리(`tail`)는 그대로 부모 표를 걷는다.
  - K 합계와 V 변이 표를 7사례로 넓힌다(§3).
- 입력(Rust에 주는 것): 06d §1 입력 + `ents`(항목별 `I+0x34`·`I+0x38`·`I+0x40`), `x4`(변수 첨자 → x 값, 규칙 4 호출만), `ranc` 결과(순서대로), BranchDir(`pin.bdir`; `rule.r10d`와 같은지 입력 생성 때 확인).
- 기대값으로만 쓰는 것: 06d와 같고, `ranc`의 시작 노드·depth·d(질의 비교), `r5.st`, `x4` 항목 수를 더한다.
- 제외 범위와 후속:
  - 규칙 11(`I+0x90` 점수, 0회): `None`으로 남긴다.
  - 판별되지 않은 절(06c-3 §7): 규칙 4 나눗수(100 대 10)·동률·루트 항목 건너뛰기, 규칙 7 depth 계수, 규칙 7·9 동률, 방향 바이트 부호 확장, 규칙 9 `== −1`(`< 0`과 구분 안 됨), 제외 항목(`I+0x41`). 검토에서 추가: 규칙 4 반올림 판정이 `'<'`(0x3c)인지만 보는 것(규칙 4 목록의 바이트가 60·62뿐이라 "`'>'`면 ceil, 그 밖 floor"와 구분 안 됨), 규칙 9의 연산 순서(`(10·depth) − q`에 바이트를 double로 ±; 정수로 먼저 더하는 꼴과 구분 안 됨). asm대로 옮기고 캡처 일치로 보고하지 않는다(변이 F3·F5·F6·F9 생존으로 표시).
  - hint 형제 교환, 병렬 힙, `bnb.rs` 연결(가칭 M2-06d-2): 06d와 같이 범위 밖.
- 선행: main `8569900`(M2-06c-3 병합). 미커밋 선행 변경 없음(main의 `.env`는 추적 밖, 무관).

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`, 주소는 file offset(첫 LOAD 세그먼트 offset = vaddr, `readelf -lW`).
- 이번에 objdump로 다시 읽은 범위(`tools/dc.py 6e8eb3 --asm --range …`):
  - 규칙 9 `0x5b3910`–`0x5b39f7`: `q = (est − m)/1e8`, `base = cvtsi2sd(10·depth) − q`, `xmm7 = base − cvtsi2sd(movsbl I+0x40)`, `xmm5 = base + 같은 값`, `r10d == −1`(`cmp $0xffffffff`)이면 xmm7 아니면 xmm5. `comisd` 옛 best, `cmova` 첨자, `maxsd`. 제외 항목 건너뜀. work += n·5.0.
  - 규칙 7 `0x5b39fc`–`0x5b3bbc`(2개씩 풀어 쓴 루프와 홀수 꼬리): `sc = cvtsi2sd(movsbl I+0x40 + 10·depth) − q`(정수로 더한 뒤 변환), 엄격 최대, work += n·5.0.
  - 규칙 4 `0x5b3e1e`–`0x5b3fa6`, `0x5b4a68`: `5e6f57` 한 번(반환 x에 null 검사 없음), 항목마다 `I+0x41 ≠ 0` 또는 `movslq I+0x34 < 0`이면 건너뜀, `I+0x40 == 0x3c`면 `floor@plt` 아니면 `ceil@plt`(`I+0x38`), `sc = cvtsi2sd(depth) − (andps(x[I+0x34] − b, 마스크))/100.0`, `comisd`·`maxsd`·`cmova`. work += n·7.0.
- 바이너리에서 읽은 상수: `0x2a6e380` = `ffffffffffffff7f 0…`(하위 레인 `0x7fffffffffffffff`, 절댓값), `0x2a6e3d0` = 100.0, `0x2a6e418` = 1e8, `0x2a6e430` = 5.0, `0x2a6e438` = 7.0.
- 첫 엄격 최댓값과 `maxsd`: 규칙 2·3·12와 같은 꼴이다. Rust는 4·7·9에 공용 `argmax`(엄격 `>`로 첨자, `maxsd`로 best, 제외 항목 건너뜀)를 쓴다.
- 본문 `5fb13c` 질의 지점과 순서는 [06c-3 §2](mip_plunge_r47.md#2-원본과-상태-경계)(반환 `0x5b3c2c`·`0x5b3d1f`·`0x5b42e6`·`0x5b4402`), 질의 조건은 06c-3 §7 QA(첫 반복 새 최댓값마다 d = 새 maxd − 2, 둘째 반복 sc > best인 항목마다 d = maxd − 2).

### 2.1 Rust 변경

| Rust | 원본 | 내용 |
|---|---|---|
| `Item.var`·`val`·`dir` | `I+0x34`(s32)·`I+0x38`(f64)·`I+0x40`(i8) | 방향 바이트는 `movsbl`이라 i8 |
| `pick(rule, items, branch_dir, rng, anc, x, work)` | `0x5b3541`–`0x5b44a0` | 규칙 4·7·9 본문 추가. 1·6 첫 반복이 새 최댓값마다 `anc` 호출. 규칙 11만 `None` |
| `argmax` | 위 세 본문 | 비제외 항목의 엄격 최대 |

## 3. 비교 계약(첫 재생 전 고정)

`tools/mip_plunge_rust.py`가 판정한다. 하나라도 실패하면 종료 코드 1이고 첫 차이(사례·호출·줄·원본·Rust)를 출력한다.

| 검사 | 규칙 |
|---|---|
| M 입력 | 7사례 run 1의 SHA-256이 `bench/oracle/m2_06c3/manifest.json`과 같다(3회 동일·06c 투영 일치는 06c-3 C0) |
| X 입력 생성 | 06d 조건 + `rule.r10d` = `pin.bdir`, `ents` 항목 수 = 목록 수, `x4`는 규칙 4 호출에만 있고 null 아님, `r5`는 규칙 5에만, `ranc`는 규칙 1·6에만, 같은 변수의 x 값은 하나 |
| R 규칙 | 06d와 같다 |
| A 본문 조상 | 호출마다 Rust 질의 줄 `anc <시작 id> <depth> <d> <결과>`의 순서·개수 = `ranc` 사건(결과는 입력에서 순서대로 받음). 질의 지점(`ranc.at`)은 비교하지 않는다. 검토 뒤 추가: 꼬리 사슬로 모든 부모를 아는 질의는 Rust `ancestor()` 결과가 주입한 답과 같아야 한다(다르면 panic) |
| P 선택 | 모든 호출(규칙 1–7·9·12)에서 Rust 첨자 = `pick.idx`, 선택 뒤 RNG 상태 = `r5.st`(규칙 5) 또는 `rule.st`, 규칙 4 x 읽기 수 = `x4` 항목 수, work = `pick.work`(비트) |
| T 꼬리 | 06d와 같다(Rust 첨자로 이어 간다) |
| K 개수 | 7사례 합계(캡처에서 계산, Rust 실행 전 고정): 호출 13,830, 규칙 1·2·5·6 7,499 + 3·12 1,980 + 4·7·9 4,351, `ranc` 28,245, x 1,023, `r5` 239, 뽑기 963, 유지 41,985, 힙 8,834, 버림 5, 문턱 < 1e100 1,964, null 조상 89. 5사례 부분은 06d K(유지 32,979·힙 6,512·null 58)와 같다 |
| T 변조 | 06d 11종 + 규칙 9 첨자(pk1-bd1), 선택 뒤 RNG 상태, 본문 조상 결과, 본문 질의 줄 삭제, 규칙 4 x 읽기 수. 입력 `end` 삭제는 06d와 같다 |
| V 코드 변이 | 작업 공간 복사본에 아래 변이를 넣고 7사례별 거부가 표와 같아야 한다 |

변이 기대값의 출처: D·E는 06d 표(5사례)를 그대로 쓰되, 이번에 새로 비교하는 호출이 결과를 바꿀 수 있는 E3과 BranchDir 두 사례 열은 Rust 실행 전에 Python 재생으로 구했다(D: 06c 검사기 변종 V1·V5–V12·V4를 BranchDir 투영에, E3: `ranc` 답으로 06c-3 규칙 1·6 재생에 "null 조상은 일치 안 함"을 넣음). BranchDir 사례는 규칙 9뿐이라 규칙 1·2·3·5·6·12 변이는 통과한다. F는 06c-3 변종 X1–X15 결과에서 옮겼다(Rust 하네스가 같은 호출의 첨자와 질의 순서를 비교하므로 거부 사례가 같아야 한다).

| 변이 | 대응 | 거부 사례(기대) |
|---|---|---|
| D1 규칙 3·12 부호 맞바꿈 | V1 | 5사례 |
| D2 시드를 pop 뒤 k로 | V5 | 7사례 |
| D3 뽑기 문턱 0.5 | V6 | 5사례 |
| D4 δ = 2 절 끔 | V7 | 7사례 |
| D5 δ = 6 절 끔 | V8 | 7사례 |
| D6 log 분기 반올림 | V9 | tr12-30 |
| D7 유지에서 조상 비교 무시 | V10 | 7사례 |
| D8 유지에서 bound 무시 | V11 | pk1-pool |
| D9 버림도 힙으로 | V12 | neos5, tr12-30 |
| D10 규칙 3·12 동률 뒤 항목 | V4 | 없음 |
| E1 규칙 1·6 조상 비교 끔 | W | 5사례 |
| E2 조상 깊이 maxd − 1 | W | 5사례 |
| E3 null 조상 불일치 | W | **5사례**(06d는 pk1-pool만. 질의 순서와 06d 재생 불가였던 호출이 더해져 늘어남, 사례별 1–2호출) |
| E4 규칙 1 depth 부호 + | W | pk1, neos5, tr12-30, pk1-pool |
| E5 불일치에도 best 갱신 | W | 5사례 |
| E6 규칙 2 동률 뒤 항목 | W | neos5 |
| E7 규칙 5 시작 trunc(u·n) | W | markshare_4_0, pk1, neos5, pk1-pool |
| F1 규칙 4 floor/ceil 맞바꿈 | X1 | 5사례 |
| F2 규칙 4 반올림 없음 | X2 | 5사례 |
| F3 규칙 4 나눗수 10 | X3 | 없음 |
| F4 규칙 4 depth 항 없음 | X4 | markshare_4_0, pk1, neos5, tr12-30 |
| F5 4·7·9 동률 뒤 항목 | X5·검토 | 없음 |
| F6 규칙 4 루트 항목 건너뛰지 않음 | X6 | 없음(루트 항목이 규칙 4 목록에 0개) |
| F7 규칙 7 방향 바이트 무시 | X7 | markshare_4_0 |
| F8 규칙 7 방향 바이트 부호 − | X8 | markshare_4_0 |
| F9 규칙 7 depth 계수 1 | X9 | 없음 |
| F10 규칙 9 부호 반대 | X10 | pk1-bd1, pk1-bd-1 |
| F11 규칙 9 방향 바이트 무시 | X11 | pk1-bd1, pk1-bd-1 |
| F12 첫 반복 마지막 최심 항목 | X12 | 5사례 |
| F13 둘째 반복 동률 `>=` | X13 | 5사례 |
| F14 둘째 반복 모든 항목 질의 | X14 | 5사례 |
| F15 규칙 5 슬롯 진행 안 함 | X15 | markshare_4_0, pk1, neos5, pk1-pool |

"5사례"는 06c 5사례(markshare_4_0, pk1, neos5, tr12-30, pk1-pool), "7사례"는 여기에 pk1-bd1, pk1-bd-1.

- 수치: 모든 double은 비트 비교. `floor`·`ceil`·`abs`는 정확한 연산이라 libm과 Rust가 같다.

## 4. 사례와 원본 자료

06c-3 §3.2 7사례(Threads=1, Seed=0, NodeLimit 2000; pk1-pool PoolSearchMode=2; pk1-bd±1 BranchDir ±1), run 1만 입력. 기준 자료 `bench/oracle/m2_06c3/`(manifest 해시), 캡처 도구 `tools/mip_plunge_score.js`. 적용하지 않은 범주: 규칙 11·8·10, 제외 항목, hint, MIPFocus 4(캡처 0회).

## 5. 재실행 명령

- 판정: `python3 tools/mip_plunge_rust.py`(기대: M, X 7사례, K, `R/A/P/T` 7사례, `PASS`, 종료 0).
- 판별력: `tools/hrun.sh --timeout 1800000 -- 'python3 tools/mip_plunge_rust.py --self-test'`(변조 16종·입력 변조·변이 32종, `PASS (+ self-test)`).
- 회귀: `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `python3 tools/mip_plunge_r47_check.py --self-test`(자료·검사기 미변경 확인). pytest는 Python 패키지 미변경이라 해당 없음.

## 6. 실행 결과

- 실행 commit `1b26f93`(계약·코드, 첫 재생 전 커밋), main `8569900` 위. x86-64 Linux(WSL2), Rust release, Threads=1 캡처. 입력 `bench/oracle/m2_06c3` run 1(manifest 해시 일치).
- `python3 tools/mip_plunge_rust.py` 종료 0(`PASS`), 첫 실행에서 전부 일치. K 합계가 §3과 같고 7사례 출력 줄(17,896·23,920·22,326·24,651·25,170·16,975·17,044)이 모두 원본과 같다.
  - 규칙 4·7·9 4,351호출(06d `given` 385 + BranchDir 3,966)의 선택, 06d 재생 불가 17호출을 포함한 규칙 1·6 4,346호출의 선택, 본문 `5fb13c` 질의 28,245개의 순서·인자, 규칙 5 239호출의 뽑기 뒤 RNG 상태, 규칙 4 x 읽기 1,023회가 원본과 같다. `given`·`unrep` 0.
  - BranchDir 두 사례의 꼬리(유지 9,006·힙 2,322·null 조상 31)도 같다.
- `tools/hrun.sh --timeout 580000 -- 'python3 tools/mip_plunge_rust.py --self-test …'` 종료 0(`PASS (+ self-test)`, 로그 `target/m2_06d3_selftest.log`는 추적 밖):
  - 변조 17종 모두 거부(06d 11종, 새 5종: 규칙 9 첨자, 선택 뒤 RNG, 본문 조상 결과, 본문 질의 삭제, x 읽기 수; 입력 `end` 삭제는 neos5만 panic으로 거부되고 나머지 통과).
  - 변이 32종의 사례별 거부가 §3 표와 모두 같다. F13·F14는 각 사례 call 0(루트 하나, est = 1e100이라 −est = best 동률)에서 추가 질의로 거부됐다. 06c-3 X13이 질의 순서로만 거부된 6호출과 같은 종류다.
- 회귀(Herdr): `cargo clippy --workspace --all-targets -- -D warnings` 0, `cargo test --release --workspace` 통과 125·실패 0·무시 11(06d와 같은 수), `python3 tools/mip_plunge_r47_check.py --self-test` `ACCEPTANCE PASS (+ self-test)`(자료·검사기 미변경). pytest는 Python 패키지 미변경이라 실행하지 않았다.
- 독립 검토(새 에이전트, 결론 없이 코드·asm 범위·계약만 전달): 디스패치 표(`0x2a6e350`)·상수·세 규칙 식(movsbl, 변환 위치, 엄격 비교, `maxsd` 순서, work)과 규칙 1·6 질의 조건을 asm에서 다시 확인해 차이 없음, 정답 누출 경로 없음. 지적과 처리:
  - [중간] 규칙 4 `'<'` 판정과 규칙 9 연산 순서가 판별되지 않는다(검토자가 복사본에 손 변이를 넣어 7사례 통과 확인). §1 한계에 추가했다.
  - [낮음] `x4`는 도구 필터 목록이라 x 읽기 수 비교는 그 필터와의 일치다. §1 문구를 고쳤다.
  - [낮음] 본문 질의 답을 주입하므로 그 질의에서 Rust `ancestor()`를 대조하지 않았다. 하네스에 사슬로 계산되는 질의의 대조 assert를 더하고 판정과 self-test를 다시 돌렸다: `python3 tools/mip_plunge_rust.py` PASS, `--self-test` 종료 0(변조 17·변이 32 표와 같음), clippy 0, cargo 125·무시 11. 입력 `q` 답 하나를 바꾼 손 변조(pk1 `q 12 0 0` → 100000)는 `ancestor(12, 2, 0) = Some(0), captured Some(100000)` panic으로 거부됐다. 질의 지점 비교 없음은 한계로 남긴다.
- 계약 변경: 검토 뒤 A 검사에 `ancestor()` 대조 assert를 더했다(판정을 엄하게 하는 쪽, 기대표 변경 없음).

## 7. 판정과 인수인계

- 작업 수행 상태: 완료(구현 L1).
- 동등성: 통과(06c-3 7사례 run 1 저장 기준, 비트 비교). 13,830호출 전부에서 시드·규칙·선택·본문 질의·선택 뒤 RNG·work·꼬리가 원본 캡처와 같다. 원본을 다시 실행하지는 않았다(06c-3 캡처 3회 동일을 재사용).
- 남은 한계:
  - asm대로 옮겼지만 캡처로 판별되지 않은 절(변이 생존): 규칙 4 나눗수(F3)·동률(F5)·루트 항목 건너뛰기(F6)·`'<'`만 보는 반올림 판정, 규칙 7 depth 계수(F9), 규칙 7·9 동률(F5), 규칙 9 연산 순서, 방향 바이트 부호 확장, 규칙 9 `== −1`(BranchDir ±1만), 제외 항목(`I+0x41`, 0회). 본문 질의 지점(`ranc.at`)은 비교하지 않았다. x의 열 공간은 미확인.
  - 규칙 11(`None`), 8·10, hint 형제 교환, MIPFocus 4: 0회, 06d와 같다.
  - `bnb.rs` 미연결. 연결 때 `anc`는 `|n, dep, d| ancestor(n, dep, d, &mut parent)`, x는 해 풀 최선해(`5e6f57`, `M+0x3220`)로 준다.
- 후속: `bnb.rs` 연결(가칭 M2-06d-2, `(*W)+0x60`·k 갱신·plunge 목록 삽입 규칙 확인 뒤), M2-06e-2(표 B·C), M2-06g(SB 입력).
- 다음 명령: `plunge.rs`를 바꾸면 `python3 tools/mip_plunge_rust.py`와 `tools/hrun.sh --timeout 580000 -- 'python3 tools/mip_plunge_rust.py --self-test'`.
- 작업 branch `work/m2-06d-3`, 기준 main `8569900`. 커밋·병합 상태는 M2 인수인계를 따른다.
- 교훈: `docs/lessons/tooling.md`(주입한 질의 답의 순서 비교, 변이 기대표를 넓힐 때 Python 재생으로 다시 구함), `docs/lessons/original.md`(규칙 4 x null 검사 없음, 상수).
