# M2-03d-9 검증 계약과 결과: DynamicRows 다이브 행 선택 Rust 이식(L1)

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. [M2-03d-8](mip_dive_rows.md) §7의 후속 제안("Rust 이식, `d:sel` 입력 재생으로 L1 대조")이고, 사용자의 "다음 스프린트 진행" 지시로 착수했다(2026-11-10).

## 1. 범위와 주장

- 작업 종류: 구현(L1, 저장 기준 재생). 원본을 다시 캡처하지 않는다. 기준은 03d-8 고정 캡처다.
- 완료 후 주장할 것: `crates/mip/src/dive_rows.rs`가 `5fb8ec` 선택 블록(`0x513078` 관문 통과 뒤 cutoff′ 계산 `0x5131f3`부터 선택 끝 `0x513951`까지)을 옮겼고, 03d-8 계약 사례의 모든 선택 사건에서 캡처한 입력으로 **원본 캡처의** 행 목록(개수·순서)과 `66661c` 호출 열(행, 반환값)을 정확히 재현한다.
- 포함 분기: 모델 행 플래그, `7889a5` 비트, maxact ≤ rhs+0.1·FeasTol, '=' 행 최소 활동값, cutoff 경로(DualReductions≠0, cutoff′<1e100, sense≠'=', `66661c`), cut 행 플래그·종류 0x14·`+0x40`·slack>1e-5, 순서(모델 행 오름차순 뒤 cut), 빈 목록.
- 제외 범위와 후속:
  - 관문(C:1581–1610의 DynamicRows, `I+0x2c`=`param_2+0x34`, `I+0x28`=2, `a7f0ad`·`a7f923`·`+0x18`)과 목록 유지(`10e5fc`·`edd71`·`10e662`, X=`D+0x140`)는 옮기지 않는다. M2-03d-8 인계는 "선택과 목록 유지"를 함께 적었지만 Rust B&B에 다이브·DynamicRows 구조가 없어 연결점이 없다. 목록 유지·관문은 후속(번호 미배정, 제안)으로 남긴다.
  - `bnb.rs` 연결과 노드 LP 입력(플래그 배열, cut 속성, slack·rc)의 Rust 생성은 M2-03d-4 범위다.
  - 03d-8 V에서 실행으로 판별되지 않은 절(cut 플래그, `7889a5`, '=' 최소 활동값, cutoff′<1e100, cut 0x14·`+0x40`)은 Rust도 C 근거로만 옮기고 미검증으로 보고한다.
  - work 증가(`a7cec0` 항목당 3, `66661c` 항목당 14·단계당 4, 모델 행 2·nrows, cut 3·ncuts)는 C로 옮기되 캡처에 없으므로 미검증이다.
- 선행: main `230b2de`(M2-03d-8 병합). 미커밋 선행 변경 없음. 기준 검사 `python3 tools/mip_dive_rows_check.py --self-test` 종료 0(이 worktree, 착수 시).

## 2. 원본과 상태 경계

- Gurobi 13.0.3, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`(03d-8 manifest).
- 원본 근거(C를 직접 읽음, 연산 순서는 03d-8의 asm 대조 Python과 교차 확인):

| 위치 | 이식 내용 |
|---|---|
| `5fb8ec` C:1625–1628 | cutoff′ = `5e7204()`, 1e100 미만이면 `I+0x10`을 뺀다 |
| C:1629–1680 | 모델 행 루프. `I+0xa0`[i]=0이고 `7889a5`=0일 때 maxact=−`a7cec0(−1)`. `0.1·FeasTol + rhs < maxact`이면 cutoff 경로, 아니면 '='은 minact=`a7cec0(+1)`, 그 밖은 1e100으로 `minact < rhs − 0.1·FeasTol`이면 cutoff 경로. cutoff 경로는 DualReductions(env+0x2430)≠0, cutoff′<1e100, sense≠'=', `66661c`≠0이면 선택. 루프 뒤 work += nrows+nrows |
| C:1681–1842 | cut 루프(16개 단위 펼침과 나머지). `I+0xa0`[nrows+j]=0, cut `+0x2c`≠0x14, `+0x40`=0, slack `(I+8)->+0x20`[nrows+j] > 1e-5이면 nrows+j. 뒤 work += 3·ncuts |
| `66661c` `0x554a30` C 전체 | 고정 열(ub−lb ≤ 1e-10)은 (a·0.5)·(l+u), 정수성(열 'C' 아님·a 정수), |a|≤1e-6·무한 bound 조기 0, rc 부호로 bound 쪽 합산, 나머지는 |rc/a| 오름차순 상위 5개 삽입. 정수 행이면 rhs=floor(rhs+9.999999999999999e-05)+1. loc < rhs−1e-6이면 비용을 쌓고(마지막 전 단계는 폭으로 자름) gap < cost이면 1. work += 14·항목, 4·단계 |
| `a7cec0` | 기존 `knapsack::act_sum` 재사용(M2 presolve에서 이미 이식) |

- 시작 경계: `0x513078` 이후 선택 블록 진입(관문 통과). 입력은 03d-8 `d:sel` 사건의 상태다.
  - 모델 행: `(mip+8)->D`의 행 우선 CSR·rhs·sense, vtype(`mip+0x650`). `d:model` 사건.
  - 노드 bound `I+0x80/0x88`(사건별 차분을 모델별로 누적), 플래그 `I+0xa0`(nrows+ncuts, 0 아닌 칸), `7889a5` 배열(개수와 0 이상인 칸), cut 종류·`+0x40`·slack, rc, FeasTol·DualReductions·cutoff 원값·`I+0x10`.
- 종료 경계: `0x513951`. 출력은 `0x48(%rsp)` 목록(개수·순서)과 `66661c` 호출 열. 상태 변경은 없다(목록은 다음 `10e5fc`의 인자).

## 3. 비교 계약

`tools/mip_dive_rows_rust.py`가 판정한다. 하나라도 실패하면 종료코드 1이다.

| 검사 | 규칙 |
|---|---|
| M 입력 | 03d-8 manifest(`bench/oracle/mip_dive_rows_manifest.json`)의 캡처 SHA-256과 run 1 파일이 같다(재생 입력의 출처 고정) |
| X 재생 입력 | 생성기는 `d:model`·`d:sel` 입력 필드만 쓴다. 기대 결과(`list`, `calls_66661c`, `count`)와 Python 규칙(`select`·`cutoff`)을 거치지 않는다. 입력 파일 머리에 사례 이름과 캡처 SHA-256을 적는다 |
| T trace | Rust trace 머리의 사례·캡처 SHA-256이 입력과 같다. 실행 전에 이전 trace를 지운다 |
| R 목록 | 모든 선택 사건에서 Rust 목록이 캡처 `list`와 개수·순서까지 같다 |
| A `66661c` | 모든 선택 사건에서 Rust 호출 열(행, 반환)이 캡처 `calls_66661c`와 같다 |
| K 개수 | 사례별 선택·행·호출·1 반환 수가 고정값과 같다: nidda_H_s1 1320·157,545·0·0, gmu-35-40_default 689·9,248·19,434·328, assign1-5-8_default 84·5,342·6,844·0, 합계 2,093·172,135·26,278·328 |

- 정수·목록은 순서까지 정확 비교. 부동소수 입력은 u64 비트로 넘기고 Rust는 원본 연산 순서(FMA 없음, `!(x >= y)` 형태 유지)로 계산한다. 허용오차 없음.
- 시간·work: work는 비교하지 않는다(캡처에 없음, §1 미검증).
- 재현성: 03d-8이 사례마다 3회 바이트 동일을 확인했다(manifest). run 1만 재생한다.
- 자기검사(`--self-test`):
  - trace 변조 거부: 목록 한 행 삭제, 목록 순서 교환, `66661c` 반환 반전, 선택 사건 한 줄 삭제, 캡처 SHA-256 변경.
  - 코드 변이 거부(임시 복사본, 본 코드 미변경): 0.1·FeasTol → 0, slack 1e-5 → 0, `66661c` 정수 반올림 제거, DualReductions 무시(호출 열로만 잡힘), cut을 모델 행보다 먼저. 변이마다 R 또는 A가 실패해야 한다.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력(03d-8 run 1) | 원본 증거 |
|---|---|---|---|---|
| nidda_H_s1 | 원본 실제 실행 캡처 | 플래그, maxact 절, cut slack, 중첩 MIP, DualReductions 0(호출 0) | `nidda_H_s1_1.json.gz` `6defab9c…` | 03d-8 §6.2 |
| gmu-35-40_default | 원본 실제 실행 캡처 | `66661c` 호출 19,434·1 반환 328, 정수 반올림·폭 상한 | `gmu-35-40_default_1.json.gz` `e1e4ccee…` | 03d-8 §6.2 |
| assign1-5-8_default | 원본 실제 실행 캡처 | '=' 행(최소 활동값 절, '=' cutoff 제외) | `assign1-5-8_default_1.json.gz` `bc45f038…` | 03d-8 §6.2 |

- DynamicRows=0 대조군은 선택 사건이 0이라 재생할 입력이 없다(관문은 범위 밖).
- 적용하지 않은 범주: 03d-8 V에서 C만인 절의 실행 사례(원본 실행에서 나오지 않음). 합성 입력으로 이 절을 "검증"하지 않는다. 단위 테스트의 손으로 만든 행은 이식 코드의 자기점검일 뿐 원본 대조가 아니다.

## 5. 재실행 명령

- 원본 캡처: 없음(03d-8 고정 자료 재사용, 재캡처는 `tools/mip_dive_rows_runs.sh`).
- 재생과 비교(작업 worktree 루트): `python3 tools/mip_dive_rows_rust.py --self-test`. 내부에서 입력 생성 → `cargo test --release -p rsolver-mip --test dive_rows_l1 -- --ignored` → 비교 → 변조·변이 순서로 실행한다. 비교만은 인자 없이 실행한다.
- 관련 회귀: `cargo test -p rsolver-mip`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `python3 tools/mip_dive_rows_check.py --self-test`.

## 6. 실행 결과

- 실행: worktree `../rsolver-m2-03d-9`, 코드 commit `7af3089`·`00e2893`(asm 비교 형태, 계약 `f0424d0`), x86-64 Linux(WSL2), `cargo test --release`. 입력은 03d-8 run 1 캡처 세 개(SHA-256 §4, manifest 일치)다. 원본은 다시 실행하지 않았다(저장 기준 재생).
- `python3 tools/mip_dive_rows_rust.py --self-test` 종료 0:

```text
M: run 1 captures match the M2-03d-8 manifest (3 cases)
X: replay inputs in target/mip_dive_rows_rust/in
R/A/K: nidda_H_s1 1320 sel 157545 rows 0 calls (0 =1)
R/A/K: gmu-35-40_default 689 sel 9248 rows 19434 calls (328 =1)
R/A/K: assign1-5-8_default 84 sel 5342 rows 6844 calls (0 =1)
T/R/A/K: every selection list (order) and 66661c call (row, result) = capture
rejected: list row dropped: R nidda_H_s1: sel call 74 node 1: rust 6 rows != capture 7; ...
rejected: list order: R assign1-5-8_default: sel call 135 node 0: rust 2 rows != capture 2; ...
rejected: 66661c result: A gmu-35-40_default: sel call 72 node 0: 66661c calls differ, first ([3, 1], [3, 0]) (rust, capture)
rejected: selection deleted: K gmu-35-40_default: 688 selections in the trace, capture 689
rejected: capture sha256: T assign1-5-8_default: trace is for assign1-5-8_default ...
mutant rejected: 0.1*FeasTol -> 0: R nidda_H_s1: sel call 186 node 3: rust 8 rows != capture 10; ...
mutant rejected: slack 1e-5 -> 0: R nidda_H_s1: sel call 2173 node 1067: rust 42 rows != capture 41; ...
mutant rejected: 66661c no integral rounding: A gmu-35-40_default: sel call 117 node 3: 66661c calls differ, first ([27, 0], [27, 1]) (rust, capture)
mutant rejected: DualReductions ignored: A nidda_H_s1: sel call 74 node 1: 66661c calls differ, first (302, 0) (rust, capture)
mutant rejected: cuts before model rows: R nidda_H_s1: sel call 74 node 1: rust 7 rows != capture 7; ...
PASS
```

- 통과 2,093/2,093 선택, 행 172,135개, `66661c` 호출 26,278건(1 반환 328)이 캡처와 정확히 같다. skip·xfail 없음. 첫 실행부터 일치했고 이식 코드를 결과에 맞춰 고친 일은 없다.
- 변이 5종의 첫 차이는 03d-8 V의 Python 변종 첫 차이(nidda call 186·2173·74, gmu call 117)와 같은 사건이다. DualReductions 변이는 목록이 아니라 호출 열(A)로만 잡혔다.
- 관련 회귀: `cargo test -p rsolver-mip` 종료 0(단위 `dive_rows::tests::clauses` 포함, L1 테스트는 ignored 1로 표시), `cargo test --release --workspace` 종료 0, `cargo clippy --workspace --all-targets -- -D warnings` 종료 0. `python3 tools/mip_dive_rows_check.py --self-test`는 착수 시 이 worktree에서 종료 0이었고 이후 03d-8 파일은 바꾸지 않았다.
- 실행하지 않은 검사: pytest(`crates/py`·`python/rsolver` 미변경), Gurobi(원본 재캡처 불필요).
- 계약 변경: 없음.

## 7. 판정과 인수인계

- 작업 수행 상태: **완료(구현 L1)**.
- 동등성: **통과(L1, 저장 기준 재생)**. 03d-8 세 사례의 모든 선택 사건에서 목록·순서와 `66661c` 호출 열이 원본 캡처와 정확히 같다. 부동소수 입력은 비트로 넘겼고 허용오차는 없다.
- 남은 한계:
  - 실행으로 판별되지 않은 절(cut 플래그, `7889a5`, '=' 최소 활동값, cutoff′<1e100, cut 0x14·`+0x40`)은 C 근거로 옮겼고 미검증이다. 원본 사례에서 참이 된 적이 없다.
  - work 증가는 C로 옮겼고 미검증이다(캡처 없음).
  - NaN 입력: 비교식은 asm 분기 형태(비순서 포함)를 따랐다. `66661c`의 asm(`0x554bee`–`0x554c34` rc 부호 분기, `0x554dde` 폭 자르기 `minsd`)은 디컴파일 C 표현(`!(a<=0 || 1e-10<r)`, `W <= step`)과 NaN에서만 다르다. 처음 작성은 C 표현이었고 asm 확인 뒤 고쳐 다시 재생했다(결과 동일). NaN 분기는 캡처에 없어 미검증이다.
  - 관문·목록 유지(`10e5fc`·`edd71`·`10e662`)와 `bnb.rs` 연결은 하지 않았다. Rust B&B에는 다이브 깊이, 행 플래그 배열, cut 속성, 노드 LP slack·rc를 이 함수에 넘길 구조가 아직 없다.
- 후속:
  - 목록 유지·관문 이식(번호 미배정, 제안): `10e5fc`/`10e662`/`edd71`의 X 목록과 D 교체(03d-7 `10f448`/`10f5ae`)를 옮긴다. 03d-7 X 목록 633단계 재생이 비교 기준이다.
  - M2-03d-4(L2)의 다이브 선택 선행은 이 작업으로 채워졌다. 남은 선행은 M2-10b, M1 요청(primal warm start·LP cutoff), 위 목록 유지다.
- 재실행: `python3 tools/mip_dive_rows_rust.py --self-test`(release 빌드 포함 수 분, 변이는 `target/mip_dive_rows_rust/mutant/`의 작업 공간 사본에서 빌드).
- 작업 branch: `work/m2-03d-9`. 검사 통과 뒤 main에 `--no-ff`로 병합한다.
- 교훈: [도구](../lessons/tooling.md)(생성 입력을 쓰는 Rust 재생 테스트는 `#[ignore]` + 드라이버, 사본 작업 공간 코드 변이), [원본 사실](../lessons/original.md)(`66661c` C와 asm의 NaN 분기 차이)에 추가했다.
