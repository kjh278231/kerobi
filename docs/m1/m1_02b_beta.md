# M1-02b 검증 계약과 결과: DSE β 합산 수정(조밀 BTRAN 출력 압축)

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. [M1-02a](m1_02a_beta.md) §7의 후속 구현이고, 사용자의 "다음 스프린트 진행" 지시로 착수했다(2026-11-11). M2의 다음 후보 M2-03d-12는 이 세션이 먼저 점유했지만 다른 세션이 같은 worktree에서 진행해 넘겼다(사용자 확인). 그래서 시작 절차 순서대로 M1을 골랐다.

## 1. 범위와 주장

- 작업 종류: 구현(Rust 변경). 원본은 다시 캡처하지 않는다. 기준은 M1-02a 고정 캡처 `bench/oracle/m1_02a_beta/kb2_{1,2,3}.json.gz`와 저장 오라클 `bench/oracle/iter0-30/kb2.jsonl.gz`다.
- 완료 후 주장할 것:
  - `crates/lp/src/simplex.rs` dual 반복의 β=‖ρ_r‖²를 원본처럼 계산한다. BTRAN 출력이 조밀하면 \|ρ_i\|>ZERO(1e-13) 원소만 행 순서로 모아 `sum_squares`로 더하고, 희소하면 출력 그대로 더한다. 근거는 `28c93` C:149–163과 `8b473` C:105–222다.
  - 조밀 여부는 출력 길이가 아니라 `Lu` BTRAN이 어느 경로(조밀 Lᵀ)로 끝났는지로 정한다. kb2 피벗 0–21에서 Rust 조밀 표시가 원본 `09e376` 출력 nnz<0과 같다.
  - kb2(Method=1, Presolve=0, Threads=1)의 피벗 0–21에서 Rust β가 원본 저장 β와 비트까지 같다. 수정 전에는 피벗 11·12·16·18·20이 달랐다.
- 예상(판정 F, M1-02a §6.1 임시 실험): 반복 0–11은 그대로 같고 반복 12의 첫 차이는 w로 남는다. w 차이는 10개에서 2개(첫 var 58)로 줄고 피벗 분기는 21 그대로다. 남는 차이는 M1-02a-1 조사 범위다.
- 포함 분기: 조밀 출력(압축)과 희소 출력(그대로), 압축 뒤 n≤6·n>6. 합산 경로(순차·SIMD)는 M1-02a O가 판별했고 Rust `sum_squares`가 같은 묶음이다.
- 제외 범위와 후속:
  - `pivot_row`에 넘기는 ρ는 바꾸지 않는다. Rust는 지금 희소·조밀 모두 \|ρ_i\|>ZERO로 거르고, 원본은 희소 출력을 거르지 않는다(C:164–167). kb2 희소 출력 30원소 중 문턱 이하는 0개라 실행으로 판별되지 않는다(M1-02a §6.2). 행 work(`a171f`)도 같은 이유로 C만이다.
  - 초기 정확 노름(`dual_start`, `exact_weights`, `1bae9c` EXACTNORM)도 조밀 출력을 압축 없이 더한다. 원본 EXACTNORM 루프가 압축하는지는 읽지 않았다. 이번에는 바꾸지 않고 미검증으로 남긴다.
  - 문턱 0 대 1e-13, 첫 원소 떼기(정렬 8 mod 16), 별칭 순차 경로는 M1-02a대로 C만이다.
  - 반복 12에 남는 w 차이(M1-02a-1)와 Method=1 Netlib 64 재측정(M1-02c)은 다른 번호다. 이번 회귀 측정은 영향 확인용이며 M1-02c를 대신하지 않는다.
- 선행: main `c87c4fe`(M2-03d-12 병합). 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 모델 `../re/bench/data/netlib/kb2.mps.gz` SHA-256 `fd4e8937…`(M1-02a §2).
- 원본 근거는 M1-02a §2 표와 같다. `28c93` `0xc51d8` BTRAN 호출, C:149–163 조밀 출력 압축(`spx+0x3a8 < |x_i|`, i=0..m−1 순서), `8b473` `0x11f7ba`–`0x11fa4c` 합산·저장이다.
- Rust 대응:
  - `Lu::btran`의 조밀 경로는 두 곳이다. `btran`의 `!sparse_u || sparse_mult·nnz > m` 분기와 `btran_updated`의 `sparse_mult·nnz > m` 분기다. 둘 다 m개를 행 순서로 낸다.
  - 희소 경로는 `lt_sparse`(방문 순서)다.
  - 원본 `221173` 출력 nnz=−1에 대응하는 것은 조밀 경로다(`lu.rs:643` 주석).
- 시작·종료 경계: 각 dual 반복의 BTRAN 직후부터 β 계산 직후까지다. 스냅숏 j의 `beta`는 피벗 j−1의 값이다(M1-02a §2).
- 입력: BTRAN 출력(값·순서)과 조밀 표시. 출력: β(spx+0x310에 대응하는 `last_dse.2`)와 이번에 스냅숏에 더할 조밀 표시 `bt_dense`.

## 3. 비교 계약

`tools/m1_beta_check.py`의 R·F를 이번 주장에 맞게 바꾼다. M·B·C·O(원본 쪽)는 그대로 둔다. 하나라도 실패하면 종료코드 1이다.

| 검사 | 규칙 |
|---|---|
| M·B·C·O | M1-02a §3과 같다(캡처 3회 해시·재현·압축·원본 합산) |
| D 조밀 표시 | 피벗 0–21에서 Rust 스냅숏 `bt_dense` = (원본 `09e376` 출력 nnz < 0) |
| R Rust β | 피벗 0–21에서 Rust 스냅숏 β = 원본 저장 β(비트). 같은 피벗에서 Rust β = `sum_squares`(조밀이면 원본 BTRAN 출력을 \|x\|>spx+0x3a8로 행 순서 압축한 것, 희소면 그대로)도 확인한다. 즉 Rust 연산이 같은 입력에서 원본 연산과 같다 |
| F 최초 분기 | 저장 오라클과 Rust 스냅숏에서 반복 0–11의 vstat·x_B·d·w가 비트까지 같고 반복 12에 처음 w가 다르다. 다른 w는 2개이고 첫 변수는 58이다. 스냅숏 1–12의 β·α_r이 같다 |

- 비교는 모두 비트 정확 비교다. 허용오차 없음.
- 자기검사(`--self-test`): M1-02a의 원본 쪽 변조 8종은 그대로 둔다(바이너리 해시, 회차 차이, 파일 해시, 도구 해시, 빈 사건, 피벗 11 ρ 값 비트, 피벗 11 β 비트, 조밀 압축 원소 누락). 이번에 바꾸거나 더하는 것은 다음이다.
  - "Rust β를 원본 β로 대체"(이제 같으므로 의미 없음)를 "Rust β 한 비트 변조(스냅숏 12)"로 바꾼다.
  - "Rust 조밀 표시 뒤집기(피벗 11)"와 "Rust 스냅숏에 조밀 표시 없음"을 더한다.
  - "저장 오라클 반복 5 w 한 비트"는 그대로 둔다.
  - 모두 거부해야 한다.
- 코드 변이 확인(1회, 기록만): 수정 전 코드(main)는 R에서 거부돼야 한다. 피벗 11·12·16·18·20에서 β가 다르다.
- 관련 비교 도구: `tools/proto/statecmp.py kb2`. 반복 12 차이가 수정 전 w 10개·score 5개에서 줄어야 한다(M1-02a §6.1). `tools/proto/pathcmp.py kb2`의 피벗 분기 위치도 기록한다.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 | 원본 증거 |
|---|---|---|---|---|
| kb2 Method=1, Presolve=0, Threads=1 | 원본 실제 실행 캡처(M1-02a) | 조밀 28·희소 10 BTRAN 출력, 압축 뒤 n≤6·n>6 | `kb2.mps.gz` | `bench/oracle/m1_02a_beta/kb2_{1,2,3}.json.gz`(gz SHA-256 `caad5976…`), `bench/oracle/iter0-30/kb2.jsonl.gz` |

- 회귀(동등성 판정 아님): Method=1 Netlib(`tools/lp_check.py --methods 1`)의 상태·목적값·반복 수를 수정 전 main 빌드와 비교한다. 상태·목적값이 나빠진 모델이 없어야 한다. 반복 수 일치 개수의 변화는 기록만 하고, 판정은 M1-02c에서 한다.
- 적용하지 않은 범주: 다른 모델의 β 대조(원본 캡처가 kb2뿐), 희소 출력에 문턱 이하 원소가 있는 피벗(실행에 없음).

## 5. 재실행 명령

- 원본 캡처: 없음(M1-02a 고정 자료 재사용).
- 빌드: `tools/build_local.sh`(이 worktree의 `python/rsolver`에 격리 빌드).
- 판정: `PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_beta_check.py --self-test`.
- 비교 도구: `zcat bench/oracle/iter0-30/kb2.jsonl.gz > <tmp>/kb2.jsonl; PYTHONPATH=$PWD/python ../.venv/bin/python tools/proto/statecmp.py kb2 <tmp>/kb2.jsonl`, `PYTHONPATH=$PWD/python ../.venv/bin/python tools/proto/pathcmp.py kb2`.
- 회귀: `cargo test -p rsolver-lp`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `PYTHONPATH=$PWD/python ../.venv/bin/python tools/lp_check.py --methods 1 --timeout 120 --csv <tmp>`(수정 전·후).

## 6. 실행 결과

- 실행: worktree `../rsolver-m1-02b`, 기준 main `c87c4fe` + 계약 `85536e9` + 이 기록과 같은 commit의 Rust·검사기 변경, x86-64 Linux(WSL2). 입력은 M1-02a 캡처 3회(gz SHA-256 `caad5976…`, M에서 고정값 확인)와 저장 오라클이다. 원본은 다시 실행하지 않았다(저장 기준 재생).
- Rust 변경:
  - `Lu::btran_dense`가 출력과 조밀 표시를 함께 돌려준다. 조밀 Lᵀ 경로 두 곳이 참이고 `lt_sparse` 경로는 거짓이다. 기존 `btran`은 출력만 넘기는 얇은 함수로 남겨 다른 호출자는 그대로다.
  - `simplex.rs` dual 반복은 조밀이면 \|ρ_i\|>ZERO 원소만 더하도록 β 식을 바꿨다. 스냅숏에 `bt_dense`를 더했다(β와 같은 시점에 저장).
  - 잘못된 출처 주석(`1bae9c` L1971–2110)을 `8b473`·`28c93`으로 고쳤다.
- 수정 전 코드 거부(같은 검사기, 수정 전 main 빌드): 종료 1. 실패 35줄이다. R은 피벗 11·12·16·18·20에서 Rust β가 압축 합산·원본 β와 달랐다(예: 피벗 11 `…d34p+2` 대 `…d33p+2`). D는 `bt_dense` 없음으로 실패했다. F는 반복 12의 w 10개 차이와 β 차이로 실패했다.
- 판정(`PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_beta_check.py --self-test`, 수정 빌드, 종료 0):

```text
M: 3 runs, binary/model/params/output, pinned file sha, tool = current file ab414034c3e2f007
B: 38 pivots, 3 runs equal
C: dense outputs compacted in index order with |x| > spx+0x3a8 (28 dense, 10 sparse)
O: stored beta = sum in captured order for 38 pivots; paths [(('scalar', 0, False), 13), (('simd', 0, False), 25)]; chain combine also equal on 22/25 simd pivots
D: Rust bt_dense = original BTRAN output nnz < 0 on pivots 0-21 (12 dense)
R: Rust beta = sum_squares(dense output compacted to |x| > spx+0x3a8, sparse as is) = original beta on pivots 0-21
F: snapshots 0-11 equal (vstat, x_B, d, w), first difference at 12 is w on 2 variables from 58; beta and alpha_r equal 1-12
self-test: 12 tampers rejected (원본 쪽 8종 + Rust β 비트, Rust 조밀 표시, 조밀 표시 없음, 저장 오라클 w 비트)
PASS
```

- statecmp kb2(저장 오라클 반복 0–30):
  - 수정 전: 반복 12에 w 10개(첫 var 0)·score 5개가 다르고 반복 13도 같다.
  - 수정 후: 반복 0–11은 같고, 반복 12에 w 2개(첫 var 58)·score 2개, 반복 13에 w 4·score 3, 반복 14에 w 5·score 2, 반복 15에 w 6·score 1이 다르다.
  - pathcmp kb2: 수정 전후 모두 첫 피벗 분기 21이다(원본 진입 25, Rust 4, 이탈 78). §1 예상과 같다.
- 회귀:
  - `cargo test --release --workspace` 종료 0이다(16개 test result 줄 모두 ok, 실패 0).
  - `cargo clippy --workspace --all-targets -- -D warnings` 종료 0이다.
  - `tools/lp_check.py --methods 1 --timeout 120` Netlib 91개를 수정 전·후 같은 worktree 빌드로 비교했다. 정답은 90/91, IterCount 일치는 24/64로 둘 다 그대로다. 상태·정답 여부가 바뀐 모델은 없다.
  - 반복 수가 바뀐 모델은 3개다. degen2 694→639(원본 634), degen3 2184→2343(원본 2212, 목적값 마지막 자리만 바뀌고 정답 판정 그대로), greenbeb 5920→5946(원본 기준 없음)이다. 동작 판정이 아니며 M1-02c 재측정에서 첫 분기로 다룬다.
- 실행하지 못한 검사: 다른 모델의 β 원본 대조(원본 캡처가 kb2뿐), `pivot_row` 입력과 초기 정확 노름의 원본 대조(§1 제외), Python 통합 `pytest`(Python 경계를 바꾸지 않았다).
- 계약 변경: 없음. 수정 전 코드 거부를 §3 "기록만"대로 1회 실행했다.

## 7. 판정과 인수인계

- 작업 수행 상태: **완료(구현)**.
- 동등성: **통과**(구간 L1, 비트 정확). kb2 피벗 0–21에서 Rust BTRAN 조밀 표시가 원본 nnz 부호와 같고 Rust β가 원본 β와 비트까지 같다. 반복 12의 남은 w 2개 차이는 β 밖의 원인이며 M1-02a-1 범위다.
- 남은 한계:
  - C만: 희소 출력 무필터(Rust `pivot_row`는 여전히 \|ρ\|>ZERO로 거른 ρ를 받는다), 문턱 0 대 1e-13, 첫 원소 떼기, 별칭 순차 경로.
  - 미검증: 초기 정확 노름(`dual_start`·`exact_weights`)이 조밀 출력을 압축하는지(원본 EXACTNORM 루프 미독), kb2 밖 모델의 β.
- 후속: **M1-02a-1**(조사)은 kb2 반복 12 w 2개(첫 var 58)다. `8b473` DSE 갱신 지점의 α·τ 캡처부터 시작한다. **M1-02c**(검증)는 Method=1 Netlib 64 재측정이며, 이번 회귀의 24/64가 Presolve=0 도구 기준 새 출발점이다.
- 작업 branch: `work/m1-02b`. 검증 뒤 main에 `--no-ff`로 병합한다.
- 갱신 위치: [M1 인수인계](../handoff/M1.md), [스프린트 목록](../14_sprint_plan.md) M1-02b 행, [작업 단위](../13_work_units.md) M1-02, [진행 기록](../11_progress.md)·[로드맵](../12_roadmap_diagram.md) 맨 위.
- 교훈: [원본 사실](../lessons/original.md) "dual simplex DSE β" 항목에 조밀 판정 근거를 더했다. [절차](../lessons/process.md)에는 두 세션이 한 점유·worktree를 같이 쓴 사례를 적었다.
