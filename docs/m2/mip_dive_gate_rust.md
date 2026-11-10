# M2-03d-12 검증 계약과 결과: DynamicRows 관문 Rust 이식

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 원본 규칙과 캡처는 [M2-03d-11](mip_dive_gate.md)이다. M2 인수인계의 첫 제안 항목이고, 사용자의 "다음 스프린트 진행" 지시로 착수했다(2026-11-11).

## 1. 범위와 주장

- 작업 종류: 구현(L1 저장 기준 재생).
- 완료 후 주장할 것: `crates/mip/src/dive_rows.rs`의 `Gate::react`(C:526)와 `Gate::select`(C:1581–1610)가 03d-11 네 사례 run 1의 관문 평가 160,353회 전부에서 원본의 결과와 같다. 비교 대상은 C:526의 `10e662` 호출 여부, C:1581의 `a7f0ad`·`a7f923` 호출 여부와 선택 블록 진입 여부다. 원본 결과는 Python 규칙이 아니라 캡처의 양성 훅(`g:hit`) 열에서 읽는다.
- 포함 분기: dyn 참·거짓(대조군), 두 관문의 깊이 참·거짓, 상태 2·4·5, 통과, top-level과 중첩 MIP(gmu 깊이 2·3).
- 입력 전제: 술어 인자는 캡처의 원시 값이다(dyn, `I+0x2c`, `W+0x34`, `I+0x28`, `I+0xa0`·`I+8` null 여부, `*(param_3+0x18)`). `a7f0ad`·`a7f923`은 원본처럼 앞 절이 통과했을 때만 부르는 클로저로 받는다. 원본이 부르지 않은 평가에서 Rust가 부르면 재생이 실패한다(값이 없음).
- 제외 범위와 후속:
  - `bnb.rs` 연결, `W+0x34`를 쓰는 쪽(`6e8eb3`), 원본 상태 2와 Rust LP `Status`의 대응, DynamicRows −1의 해석·공개 파라미터는 M2-03d-4 몫이다. 이번 작업은 원시 값 술어만 둔다.
  - C:526 참일 때의 `RowList::reactivate_all` 호출과 그 결과는 03d-10 L1이 이미 대조했다. 이번에는 관문 판정만 본다.
  - `0x51305f` 도달(앞 세 절 통과 뒤 null 검사 진입)은 메모리 읽기뿐이라 의미 있는 상태 변화가 없다. 비교 대상에서 빼고 캡처 안의 일관성만 본다(§3 H).
- 선행: main `0e4f9ec`. 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 file offset이다.
- 원본(C 읽기, asm 대조):

| 위치 | 내용 |
|---|---|
| `5fb8ec` C:526–530 | `env+0x2b48 != 0 && I+0x2c == W+0x34`이면 `10e662(lp)`, 0이 아니면 반환 |
| `5fb8ec` C:1581–1584 | `env+0x2b48 != 0`, `I+0x2c == W+0x34`, `I+0x28 == 2`, `I+0xa0 != 0`, `I+8 != 0`(순서대로 단락 평가) |
| `5fb8ec` C:1608–1610, asm `0x513198`–`0x5131bc` | `a7f0ad`(eax≠0이면 `0x513b12`), `a7f923`(같음), `cmpl $0x2,0x18(%r15)` 뒤 `jge 0x513b12`(부호 있는 비교, i32 `< 2`) |
| `a7f0ad`·`a7f923` | 120바이트, 호출 없는 순수 판정(QP·QCP, `re/notes/11_driver_detail.md:65`) |

- 시작·끝: 관문 평가 한 번(03d-11 `g:526`·`g:1581` 사건)이 사건 하나다. 끝은 C:526의 `0x510961` 도달 여부, C:1581의 `0x5131c2` 도달 여부다.
- 입력: 위 원시 값과, 원본이 부른 경우의 `a7f0ad`·`a7f923` 반환(`g:hit` `51319d`·`5131af`의 eax).
- 출력: C:526 참·거짓. C:1581은 (`a7f0ad` 호출, `a7f923` 호출, 선택 진입) 세 값.

## 3. 비교 계약

`tools/mip_dive_gate_rust.py`가 판정한다. 하나라도 실패하면 종료코드 1이다.

| 검사 | 규칙 |
|---|---|
| M 입력 | 네 사례 run 1 캡처의 SHA-256이 `bench/oracle/mip_dive_gate_manifest.json`과 같다 |
| X 재생 입력 | `g:526`·`g:1581`의 원시 필드와 도달한 `51319d`·`5131af`의 eax만 쓴다. `g:hit` 도달 열·Python `predict`는 쓰지 않는다. 입력 끝에 `end` 줄을 둬 잘린 파일은 재생 실패 |
| H 캡처 일관성 | C:526의 hit는 `510961` 하나 이하. C:1581의 hit 열은 `51305f`,`513078`,`51319d`,`5131af`,`5131b7`,`5131c2`의 접두사다 |
| T 대조 | 사건마다 순서대로: C:526은 Rust `react` = `510961` 도달. C:1581은 Rust (`a7f0ad` 호출, `a7f923` 호출, `select`) = (`51319d` 도달, `5131af` 도달, `5131c2` 도달). 사례 머리의 캡처 해시가 같다 |
| K 개수 | 사례별 C:1581 평가 수와 `select` 참 수가 03d-11 기록과 같다: nidda 33,118·1,320, gmu 10,787·689, assign 3,088·84, 대조군 33,182·0. 네 사례 C:526+C:1581 평가 합 160,353 |
| V 변이 | 임시 복사본의 `dive_rows.rs`를 한 절씩 바꾼 변이를 재생한다. 거부되어야 하는 변이: `react`의 dyn 무시·깊이 무시, `select`의 dyn 무시·깊이 무시·상태 무시, `+0x18` 임계 1, `a7f0ad` 호출 빼기, `a7f923` 호출 빼기. 통과해야 하는 변이(실행에서 판별되지 않음, 03d-11 V와 같은 이유): `I+0xa0`·`I+8` null 무시, `a7f0ad`·`a7f923` 결과 무시(호출은 유지), `+0x18` 무시·임계 3. 예상과 다른 결과는 실패 |

- 정수·참거짓·순서는 정확 비교한다. 부동소수·허용오차는 없다.
- 자기검사(`--self-test`): trace 변조(react 뒤집기, select 뒤집기, `a7f0ad` 호출 표시 뒤집기, 줄 삭제, 머리 해시)가 모두 거부되어야 하고, 위 V가 예상대로여야 한다.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 | 원본 증거 |
|---|---|---|---|---|
| nidda H seed 1 | 원본 실제 실행(03d-11 run 1) | 깊이 참·거짓, 통과, 재시작 전후, 중첩 MIP | `bench/oracle/mip_dive_gate/nidda_H_s1_1.json.gz` | 03d-11 B·R·I 통과 |
| gmu-35-40 기본 | 같음 | 상태 4·5, 중첩 MIP 깊이 2·3 | `.../gmu-35-40_default_1.json.gz` | 같음 |
| assign1-5-8 기본 | 같음 | 상태 4·5 | `.../assign1-5-8_default_1.json.gz` | 같음 |
| nidda H + DynamicRows=0 | 같음 | dyn 거짓 | `.../nidda_H_s1_dr0_1.json.gz` | 03d-11 C 통과 |

- run 2·3은 run 1과 바이트 동일(03d-11 B)이라 재생하지 않는다.
- 적용하지 않은 범주: null 참·`a7f0ad`·`a7f923`≠0·`+0x18`≥2 쪽은 실제 실행에 없다(03d-11 §6.1 sweep 38모델 포함). 합성 입력은 만들지 않는다. 이 절들은 C·asm 근거만 있다.

## 5. 재실행 명령

- 원본 캡처: 03d-11 §5(이번에는 실행하지 않음).
- 재생·비교: `python3 tools/mip_dive_gate_rust.py`(입력 생성 → `cargo test --release -p rsolver-mip --test dive_gate_l1 -- --ignored` → 비교). 자기검사는 `--self-test`.
- 관련 회귀: `python3 tools/mip_dive_gate_check.py --self-test`, `python3 tools/mip_dive_rows_rust.py --self-test`, `python3 tools/mip_dive_list_rust.py --self-test`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`.

## 6. 실행 결과

- 실행 위치: worktree `../rsolver-m2-03d-12`(branch `work/m2-03d-12`, 기준 main `0e4f9ec`). 입력 캡처는 03d-11 run 1 네 개이고 SHA-256이 manifest와 같았다(M). Gurobi는 실행하지 않았다.
- 판정 출력(`--self-test`, 종료 0):

```text
M: run 1 captures match the M2-03d-11 manifest (4 cases)
H: every hit sequence is a prefix of its gate hook sequence
T/K: nidda_H_s1: 66237 evaluations (C:526 33119, C:1581 33118), select 1320: rust = capture
T/K: gmu-35-40_default: 21576 evaluations (C:526 10789, C:1581 10787), select 689: rust = capture
T/K: assign1-5-8_default: 6176 evaluations (C:526 3088, C:1581 3088), select 84: rust = capture
T/K: nidda_H_s1_dr0: 66364 evaluations (C:526 33182, C:1581 33182), select 0: rust = capture
K: 160353 gate evaluations
```

- trace 변조 5종(gmu react 뒤집기, assign select 뒤집기, nidda `a7f0ad` 호출 표시 뒤집기, 대조군 줄 삭제, gmu 머리 해시)을 모두 거부했다.
- V(변이, 예상과 모두 같음):

| 변이 | 결과 | 첫 차이·이유 |
|---|---|---|
| `react` dyn 무시 | 거부 | 대조군 평가 0(C:526, dyn 0): Rust 1, 캡처 0 |
| `react` 깊이 무시 | 거부 | nidda 평가 2(노드 2, 깊이 1, `W+0x34` 0) |
| `select` dyn 무시 | 거부 | 대조군 평가 1: 원본이 부르지 않은 `a7f0ad`를 불러 재생 panic |
| `select` 깊이 무시 | 거부 | nidda 평가 3: 같음 |
| `select` 상태 무시 | 거부 | gmu 평가 397: 같음 |
| `+0x18` 임계 1 | 거부 | nidda 평가 1(값 1): Rust 선택 0, 캡처 1 |
| `a7f0ad` 호출 빼기 | 거부 | nidda 평가 1: 호출 표시 0, 캡처 `51319d` 도달 |
| `a7f923` 호출 빼기 | 거부 | nidda 평가 1: 호출 표시 0, 캡처 `5131af` 도달 |
| `I+0xa0`·`I+8` null 무시 | 생존 | null은 앞 절이 거짓인 평가에서만 나옴(03d-11) |
| `a7f0ad`·`a7f923` 결과 무시(호출 유지) | 생존 | 반환이 모두 0(선형 MIP) |
| `+0x18` 무시·임계 3 | 생존 | 값이 언제나 1 |

- 관련 회귀(모두 종료 0): `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --release --workspace`(실패 0), `python3 tools/mip_dive_gate_check.py --self-test`, `python3 tools/mip_dive_rows_rust.py --self-test`, `python3 tools/mip_dive_list_rust.py --self-test`.
- 계약 변경 없음. rustfmt가 설치돼 있지 않아 `cargo fmt --check`는 실행하지 못했다.

## 7. 판정과 인수인계

- 작업 수행 상태: **완료(구현 L1)**.
- 동등성: **통과(L1, 계약 범위)**. 네 사례의 관문 평가 160,353회 전부에서 C:526 결정, C:1581의 `a7f0ad`·`a7f923` 호출 여부와 선택 진입이 원본 캡처와 같다.
- 판별된 절: dyn(두 관문), 깊이(두 관문), 상태=2, `+0x18`<2의 '값 1 통과' 쪽, 두 호출의 위치(앞 절 통과 뒤에만 부름).
- 남은 한계:
  - C·asm만: `I+0xa0`·`I+8` null 절, `a7f0ad`·`a7f923` 결과(QP·QCP), `+0x18`≥2 쪽(asm `jge`, 부호 있는 비교). null 절끼리와 앞 세 절의 순서는 순수 읽기라 판정 결과에 영향이 없고 비교하지 않았다. `a7f0ad`→`a7f923`→`+0x18`의 상대 순서도 반환이 늘 0, 값이 늘 1이라 실행으로 판별되지 않는다(C·asm 순서대로 옮김).
  - 미연결: `bnb.rs`는 관문·선택·목록 유지를 아직 부르지 않는다. 원본 상태 2와 Rust `Status`의 대응, `W+0x34`(다이브 시작 깊이) 유지, DynamicRows −1 해석은 M2-03d-4에서 정한다.
- 후속: M2-03d-4(L2)의 남은 선행은 M2-10b와 M1 요청(primal warm start·LP cutoff)이다. 분할 순서 오라클(03d-10 제안)은 그대로다.
- 재실행: `python3 tools/mip_dive_gate_rust.py --self-test`(release 빌드 포함 수 분, 변이마다 복사본 재빌드).
- 작업 branch: `work/m2-03d-12`. 검사 통과 뒤 main에 `--no-ff`로 병합한다.
- 교훈: [도구](../lessons/tooling.md)에 "부수효과 없는 원본 호출도 클로저로 받아 호출 여부를 재생에서 대조한다"를 추가했다.
