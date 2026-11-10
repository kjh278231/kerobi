# M2-03d-14 검증 계약과 결과: `10f448` 분할 Rust 재생

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. [M2-03d-13](mip_dive_split.md) §7의 후속 제안("분할 Rust 재생")이고, 사용자의 "다음 스프린트 진행" 지시로 착수했다(2026-11-11). M2가 미점유라 시작 절차 순서대로 골랐다.

## 1. 범위와 주장

- 작업 종류: 검증(L1 저장 기준 재생). Rust 코드는 바꾸지 않을 예정이고, 차이가 나오면 원인을 기록하고 수정 여부를 따로 판단한다.
- 완료 후 주장할 것: 03d-13 세 양성 사례 run 1의 재생성 4,306회 전부에서, 캡처한 진입 D와 목록을 `RowList::swap_in`(`crates/mip/src/dive_rows.rs`)에 넣은 결과의 열 ind·val·줄인 길이·rhs 변경 행이 원본 반환과 비트 단위로 같다. `swap_out`이 진입 길이·rhs를 돌려준다. 03d-10이 판별하지 못한 "두 포인터 교환을 안정 분할로" 변이가 거부된다.
- 포함 분기: 교환 있는 재생성(1,819)·없는 재생성, 목록과 겹치지 않는 열, MIP 깊이 1·2(sub-MIP), 전체 기록(479)과 참조 기록(3,827) 진입.
- 입력 전제와 한계(착수 전 고정):
  - 진입 rhs 값은 캡처에 없다. `s:build`는 목록 행 중 진입 rhs가 +0 비트가 아닌 행(`rhs_list_nonzero`), 사본에서 바뀐 행(`rhs_diff_rows`), +0이 아닌 변경(`rhs_diff_nonzero`)만 갖는다. 그래서 진입 rhs를 합성한다: 목록 행 중 `rhs_list_nonzero`가 아닌 행은 +0, 나머지 행은 1.0. rhs 비교는 "바뀐 행 집합 = `rhs_diff_rows`, 바뀐 값 = +0 비트"로 한정한다.
  - val의 원본 근거는 03d-13과 같다(`val_moved`=0 훅 비트 비교, 반환 val은 진입 val의 위치 치환으로 재구성).
  - 목록 상태는 재생성마다 새 `RowList`에 `deactivate(rows, x_list)`·`update()`로 만든다. 목록 순서는 행 오름차순이 되어 캡처 순서와 다를 수 있으나 `swap_in`은 소속만 읽는다. 목록 순서·`changed`·중첩 참조수(>1)는 이 재생이 시험하지 않는다(목록 순서는 03d-10이 대조).
- 제외 범위와 후속:
  - `bnb.rs` 연결은 하지 않는다(M2-03d-4).
  - `c3d0d`·`390989` 호출자의 재생성, 중첩 교체는 원본 실행 0회라 대조할 수 없다(03d-13 한계 그대로).
- 선행: main `7924343`(M1-02a-1 병합). 미커밋 선행 변경 없음. 03d-13 기준 자료 `bench/oracle/mip_dive_split/`(manifest `bench/oracle/mip_dive_split_manifest.json`).

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`(03d-13 캡처의 `binary_sha256`).
- 원본: `10f448` `0x1a31c0`(`re/decomp/c/001a31c0_PRIVATE000000000010f448.c`). C 동작은 [03d-13 §2](mip_dive_split.md)와 같다. 이번 작업은 원본을 다시 실행하지 않고 03d-13 캡처를 기준으로 쓴다.
- 시작 경계: `10f448` 진입(X≠0, 참조수 0, 만듦 표시 0). Rust는 새 `RowList`(만듦 0·참조수 0)의 `swap_in` 호출.
- 종료 경계: `10f448` 반환. Rust는 `swap_in` 반환, 이어서 `swap_out` 반환.
- 입력 상태: 진입 D의 열 수·열 시작(캡처 `beg`, 세 사례의 전체 기록 479개 모두 빈틈 없이 이어짐)·길이·열 ind·val, 행 수, 목록 X 행. 참조 기록은 같은 D 포인터의 앞선 재생성 반환 내용(03d-13 `rebuilds()`와 같은 해석).
- 출력: 열 ind·val(`swap_in` 뒤 D), 줄인 길이(`swap_in` 뒤 D 길이 배열), rhs 사본(`swap_in` 뒤 D rhs), `state()`(만듦 1·참조수 1), `swap_out` 뒤 길이·rhs·참조수 0.

## 3. 비교 계약

새 Rust 테스트 `crates/mip/tests/dive_split_l1.rs`(무시 테스트, 입력은 도구가 생성)와 비교기 `tools/mip_dive_split_rust.py`. 입력은 캡처처럼 압축한다: 전체 기록이면 D 전체, 참조 기록이면 참조할 재생성 번호만 적고 Rust가 자신의 그 재생성 반환 D(`swap_out` 뒤)를 이어 쓴다. 모든 재생성 반환을 비교하므로 첫 차이 위치는 그대로 드러난다. 트레이스는 재생성마다 바뀐 열(전체 ind·val 비트), 진입 길이와 다른 줄인 길이, 바뀐 rhs 행과 그 비트만 적는다.

| 검사 | 규칙 |
|---|---|
| M 입력 | 03d-13 manifest의 캡처 SHA-256과 같다 |
| X 입력 생성 | 입력은 진입 필드(`beg`·`len`·`ind`·`val_u`·`val_i`·`before_ref`·`rows`·`x_list`·`rhs_list_nonzero`)만 쓴다. 반환 필드(`perm`·`raw`·`xlen_diff`·`rhs_diff_rows`)는 쓰지 않는다 |
| P 열 | 재생성마다 Rust가 바꾼 열 집합 = 캡처 반환에서 진입과 다른 열 집합, 각 열의 ind·val 비트가 같다 |
| L 길이 | 진입 길이와 다른 Rust 줄인 길이(열, 값) = 캡처 `xlen_diff` |
| H rhs | Rust 사본의 바뀐 행 = `rhs_diff_rows`, 바뀐 값은 모두 +0 비트 |
| S 상태 | `swap_in` 뒤 `state()` = (만듦 1, 참조수 1) = 캡처 `built_after`·`ref_after`, `swap_out` 뒤 참조수 0이고 길이·rhs가 진입과 같다 |
| K 개수 | 사례별 재생성 수와 순번이 캡처와 같다. 합계 4,306, 교환 있는 재생성 1,819, 안정 분할과 다른 열 146,272를 Rust 트레이스에서 다시 세어 03d-13 고정값과 같아야 한다 |

- 정수·순서는 정확 비교, val·rhs는 u64 비트 비교. 허용오차 없음.
- 시간·work: `10f448`에 work 증가 없음(03d-13). 해당 없음.
- 재현성: 원본 재현성은 03d-13 B(3회 바이트 동일). Rust는 결정적이며 트레이스를 매번 지우고 다시 만든다.
- 코드 변이(`--self-test`, 작업 공간 사본을 고쳐 재생): 거부돼야 함 — 안정 분할(03d-10 변이 그대로), val 미교환, 줄인 길이 = 원래 길이, 목록 행 rhs 유지, 목록 판정 `>= 0`을 `> 0`으로. 결과를 그대로 적음 — 앞으로 찾아 바꾸기(목록 항목을 그 뒤 첫 비목록 항목과 교환).
- 트레이스 변조(`--self-test`, 각각 거부돼야 함): 바뀐 열 ind 한 칸, val 한 비트, 줄인 길이 +1, rhs 행 하나 추가, 재생성 하나 삭제, 사례 해시, 만듦 표시.
- 회귀: `mip_dive_split_check.py`(기준 자료 불변), `mip_dive_list_rust.py --self-test`·`mip_dive_gate_rust.py --self-test`(같은 `dive_rows.rs` 패턴), `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`. Gurobi는 실행하지 않는다. Python 경계는 바꾸지 않아 pytest는 돌리지 않는다.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 | 원본 출력 |
|---|---|---|---|---|
| nidda_H_s1 | 실제 | 교환 많은 재생성(깊이 1·2) | `bench/oracle/mip_dive_split/nidda_H_s1_1.json.gz` | 재생성 2,743, 교환 있는 1,354, 안정 분할과 다른 열 119,019 |
| gmu-35-40_default | 실제 | 기본 설정, sub-MIP 깊이 2 | `…/gmu-35-40_default_1.json.gz` | 재생성 1,318, 교환 있는 354, 22,800 |
| assign1-5-8_default | 실제 | 기본 설정 | `…/assign1-5-8_default_1.json.gz` | 재생성 245, 교환 있는 111, 4,453 |

- DynamicRows=0 대조군은 재생성 0이라 재생 입력이 없다.
- 합성 사례 없음(`dive_rows.rs` 단위 테스트 `row_list`가 손으로 만든 한 열을 이미 본다).

## 5. 재실행 명령

- 기준 자료 확인: `python3 tools/mip_dive_split_check.py`.
- 재생·비교: `python3 tools/mip_dive_split_rust.py`(입력 생성 → `cargo test --release -p rsolver-mip --test dive_split_l1 -- --ignored` → 비교), `--self-test`(변조·변이 추가).

## 6. 실행 결과

- 실행: 작업 branch `work/m2-03d-14`, 커밋 `5f80f95`(Rust 코드 `crates/mip/src/dive_rows.rs`는 main `7924343`과 같음). 도구 `tools/mip_dive_split_rust.py` SHA-256 `77d6247a…`, 테스트 `crates/mip/tests/dive_split_l1.rs` `c99876cd…`. 기준 자료는 03d-13 manifest의 run 1 캡처 세 개(M 통과). x86-64 Linux(WSL2). Gurobi는 실행하지 않았다.
- `python3 tools/mip_dive_split_rust.py` 종료 0:
  - X: 입력은 진입 필드만으로 만들었다(전체 기록 479, 참조 기록 3,827).
  - P/L/H/S: 세 사례의 재생성 4,306회 전부에서 Rust가 바꾼 열 집합·각 열 ind·val 비트, 진입과 다른 줄인 길이, rhs 변경 행(모두 +0)이 캡처와 같다. `swap_in` 뒤 `state()` = (1, 1), `swap_out` 뒤 참조수 0이고 길이·rhs가 진입과 같다.
  - K: nidda 2,743(교환 있는 1,354, 안정 분할과 다른 열 119,019), gmu 1,318(354, 22,800), assign 245(111, 4,453). 합계 4,306·1,819·146,272로 03d-13 고정값과 같다.
- `python3 tools/mip_dive_split_rust.py --self-test` 종료 0(약 1분 30초):
  - 트레이스 변조 7종 모두 거부: 열 ind 두 칸 교환(P), val 한 비트(P), 줄인 길이 +1(L), rhs 행 추가(H), build 2 삭제(K), 사례 해시(T), 만듦 표시 0(S).
  - 코드 변이 6종 모두 거부: 안정 분할(P, nidda build 1 열 0), val 미교환(P), 줄인 길이 = 원래 길이(L), 목록 행 rhs 유지(H), 목록 판정 `> 0`(L, 열 0 길이 222≠221), 앞으로 찾아 바꾸기(P). 03d-10에서 판별 안 되던 안정 분할이 이제 거부된다.
- 회귀: `cargo clippy --workspace --all-targets -- -D warnings` 0, `cargo test --release --workspace` 121 통과·0 실패(새 재생 테스트는 무시 테스트로 도구가 돌림), `python3 tools/mip_dive_split_check.py` PASS(기준 자료 불변).
- 회귀(계속): `python3 tools/mip_dive_list_rust.py --self-test`·`python3 tools/mip_dive_gate_rust.py --self-test` 종료 0(03d-10의 안정 분할 변이는 그 캡처로는 여전히 판별 안 됨으로 통과, 기대대로).
- 실행하지 않은 검사: pytest(Python 경계 변경 없음), Gurobi(원본 재실행 불필요, 03d-13 기준 자료 사용).
- 계약 변경: 판정 규칙은 없음. §3 변조 "바뀐 열 ind 한 칸"은 도구에서 두 칸 교환으로 만들었다(값 집합은 그대로 두고 순서만 바꿔 더 약한 변조).

## 7. 판정과 인수인계

- 작업 수행 상태: 완료(검증).
- 동등성: 통과(L1, 비트 비교). 범위는 03d-13 세 양성 사례 run 1의 `cda5b` 호출자 재생성 4,306회. Rust `RowList::swap_in`/`swap_out`은 원본 `10f448` 재생성의 열 ind·val·줄인 길이·rhs 변경 행과 같다. Rust 코드는 바꾸지 않았다.
- 남은 한계:
  - rhs는 변경 행 집합과 +0만 대조했다. 진입 rhs 값은 캡처에 없어 합성했다.
  - val 근거는 03d-13의 `val_moved` 훅 비교다.
  - 목록 순서·`changed`·중첩 참조수(>1)는 이 재생이 시험하지 않는다. `c3d0d`·`390989` 재생성과 중첩 교체는 원본 실행 0회다.
  - `bnb.rs`에는 연결하지 않았다.
- 후속: M2-03d-4(L2, 선행 M2-10b·M1 요청)가 남은 M2-03d 연결 작업이다. DynamicRows 선택·목록 유지·관문·분할이 모두 L1 통과 상태로 미연결이다. 선행이 막혀 있어 다음 착수 전 사용자 확인(03d-5부터의 두 질문)이 필요하다. 회귀 명령: `python3 tools/mip_dive_split_rust.py --self-test`(`dive_rows.rs` 분할을 바꿀 때).
- 작업 branch `work/m2-03d-14`. 커밋: 계약 `15a3cfd`, 도구·테스트 `5f80f95`, 결과·인수인계는 이 문서와 같은 커밋. 미커밋 선행 변경 없음.
- 갱신: `docs/handoff/M2.md`, `docs/11_progress.md`, `docs/12_roadmap_diagram.md`, `docs/14_sprint_plan.md`, `docs/m2/sprint_design.md`.
- 교훈: `docs/lessons/tooling.md`(참조 압축 캡처를 Rust 재생 입력에도 그대로 쓰기, 재생에 필요한 진입 값이 캡처에 없을 때의 한계).
