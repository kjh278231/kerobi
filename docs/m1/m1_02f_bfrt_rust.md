# M1-02f 검증 계약과 결과: Rust BFRT 재훑기와 NONDEGENFLIP 상태

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. [M1-02e](m1_02e_bfrt.md)의 구현 후속이고, 사용자의 "다음 스프린트 진행" 지시로 착수했다(2026-11-11). M2는 처음 이 세션이 M2-06e를 점유했으나 같은 owner 이름을 쓴 다른 세션이 같은 worktree에서 작업 중이라, 사용자 지시로 M2-06e를 그쪽에 맡기고 M1을 골랐다.

## 1. 범위와 주장

- 작업 종류: 구현.
- 완료 후 주장할 것:
  - C1 `crates/lp/src/bfrt.rs`의 `bfrt`가 NONDEGENFLIP이 켜진 호출에서 02e 재훑기(`1c35c5` C:1500–1610)를 한다. 02e 코퍼스 `bench/oracle/m1_02e_bfrt`(Netlib 64, 3112호출)의 원본 입력을 Rust `bfrt`에 그대로 넣으면 진입·flip 목록이 원본과 같은 수(진입 3108/3112, flip 3112/3112)이고, 다른 4건은 02e가 비용 이동으로 설명한 tuff 31·33·36·37뿐이다. 3112호출 전부에서 Rust 출력이 프로토타입 `tools/proto/bfrt.py` 출력과 같다.
  - C2 `simplex.rs`가 dual의 재분해 뒤 재계산(`reinvert_with`, `48d5f` 전체 재계산 경로)에서 `48d5f` C:2106–2111의 조건(`spx+0x338 > 2` 또는 m < 3·반복)으로 NONDEGENFLIP을 켜고 계속 유지한다. kb2에서 켜지는 반복이 원본과 같다(02e 켜짐 캡처).
  - C3 kb2 경로: `pathcmp kb2`의 첫 차이가 피벗 21보다 뒤이고, `statecmp kb2` 반복 22가 같다. 새 첫 차이는 기록만 한다(이번에 고치지 않는다).
- 포함 분기: 재훑기의 교체 있음·없음, pow 항 있음(`spx+0x178 < 10`), slack·구조 열 후보.
- 제외 범위와 후속:
  - `spx+0x178`(수치 경고 누적)은 Rust에 대응 상태가 없다. 0으로 둔다(켜짐 조건의 `0x178 ≠ 0` 쪽과 pow 없는 분기 `0x178 ≥ 10`은 코퍼스·캡처에서 0회, C만). 코드에 shortcut 주석으로 남긴다.
  - tuff 4건의 RNG 비용 이동(C:1690–1740): 02e와 같이 미검증.
  - NONDEGENFLIP이 꺼지는 곳(원본의 0 초기화 위치)은 확인하지 않았다. Rust는 `Simplex` 생성 때 0(파라미터 기본값)으로 두고 다시 끄지 않는다(코퍼스 꺼짐 전환 0회).
  - reason 4(FLIP, `reinvert_with(true)`) 경로도 같은 재계산 함수에서 켜짐 검사를 한다. 원본 `48d5f`의 FLIP 경로가 C:2106에 닿는지는 확인하지 않았다.
  - Method=1 Netlib IterCount 패리티 판정: M1-02c. 이번에는 회귀(정답 수 유지)만 판정한다.
- 선행: main `e5c3eb8`(M1-02e 병합 포함). 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 이번에 Gurobi는 실행하지 않는다(02e 저장 캡처 재사용).
- 원본 근거(02e 기록 §6): 재훑기 `1c35c5` C:1500–1610, 보정 1000·OptTol(asm 부분 근거 + 코퍼스 적합), w′ = (열 원소 수 또는 slack 0.5 + ((θ_q·α_t − d_t)/OptTol)^0.25)·(0.3f + 0.7). 켜짐 `48d5f` C:2106–2111, 같은 함수의 perturb(C:1973 `0x338 = 3`) 뒤에 있다(`tools/dc.py 48d5f --grep '0x3fc|0x338|0x178'`).
- Rust 대응: `spx+0x338` = `Simplex::pt_lvl`(2 shift, 3 perturb, 기존), `spx+0x2a0` = `Simplex::iter`(피벗 전에 센다, `28c93` L820), m = `Simplex::m`.
- 시작·종료 경계: C1은 `bfrt` 호출 하나(02e 코퍼스의 진입 → 반환). C2·C3은 kb2 Method=1 Presolve=0 풀이 전체.

## 3. 비교 계약

| 필드·사건 | 비교 규칙 | 방법 |
|---|---|---|
| C1 진입 열·flip 목록(순서 포함) | 정확 일치. 원본과 다른 호출은 tuff 31·33·36·37이어야 하고, 프로토타입 출력과는 3112/3112 같아야 한다 | 새 `tools/m1_bfrt_rust.py`가 코퍼스를 텍스트로 바꿔 무시 테스트 `crates/lp/tests/bfrt_corpus.rs`에 넣고 출력 비교 |
| C1 판별력 | 재훑기 끔(NONDEGENFLIP을 0으로 넣음)이면 kb2 호출 21이 원본과 달라야 하고, pow 항 제거·보정 0 변이도 진입 일치 수가 줄어야 한다 | 같은 도구 `--self-test`(입력 변조로 변이) |
| C2 켜짐 반복 | kb2: Rust가 켜는 첫 반복 = 원본 켜짐 캡처의 반복 | `RSOLVER_DEBUG` 출력 또는 스냅숏 |
| C3 경로·상태 | pathcmp 첫 차이 > 21, statecmp 반복 22 OK | 기존 도구 |
| 회귀 | `lp_check --methods 1` 정답 90/91 유지, IterCount 일치 수와 모델별 반복 변화 기록 | 기존 도구 |

- 수치: 모두 원본 연산 순서. pow는 Rust `f64::powf`(libm `pow`)와 Python `**`(같은 libm)이 같다고 가정하고 C1 결과로 확인한다.

## 4. 사례와 원본 자료

| 사례 | 실제/합성 | 노리는 분기 | 자료 |
|---|---|---|---|
| 02e BFRT 코퍼스 3112호출 | 실제 | 재훑기 교체 있음(kb2 21, fit1d 12건, stocfor1 1건)·없음, NONDEGENFLIP 꺼짐 2815호출 | `bench/oracle/m1_02e_bfrt/*.jsonl.gz`, `SHA256SUMS`, 열 길이 `bench/oracle/iter0-5p` |
| kb2 켜짐 | 실제 | 재분해 직후 m < 3·반복 | `bench/oracle/m1_02e_bfrt_trigger` |
| kb2 전체 풀이 | 실제 | 피벗 21 이후 | `bench/oracle/path1/kb2.json.gz`, `bench/oracle/iter0-30/kb2.jsonl.gz` |

## 5. 재실행 명령

- C1: `python3 tools/m1_bfrt_rust.py`(종료 0, 위 수치), `python3 tools/m1_bfrt_rust.py --self-test`.
- C2·C3: `tools/build_local.sh`, `zcat bench/oracle/iter0-30/kb2.jsonl.gz > <tmp>/kb2.jsonl; PYTHONPATH=$PWD/python ../.venv/bin/python tools/proto/statecmp.py kb2 <tmp>/kb2.jsonl`, `PYTHONPATH=$PWD/python ../.venv/bin/python tools/proto/pathcmp.py kb2`.
- 회귀: `PYTHONPATH=$PWD/python ../.venv/bin/python tools/lp_check.py --methods 1 --timeout 120 --csv <tmp>`(수정 전후 같은 worktree 빌드), `cargo test -p rsolver-lp`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, worktree 빌드 pytest.

## 6. 실행 결과

(실행 뒤 작성)

## 7. 판정과 인수인계

(작성 전)
