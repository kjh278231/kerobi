# M1-02a 검증 계약과 결과: kb2 Method=1 최초 분기(DSE β 합산)

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. [스프린트 목록](../14_sprint_plan.md) M1-02a("Method=1 불일치 모델 하나의 최초 분기")이고, 사용자의 "다음 스프린트 진행" 지시로 착수했다(2026-11-11). M2의 다음 후보(M2-03d-11)는 다른 세션이 먼저 점유해 진행 중이어서, [시작 절차](../handoff/README.md) §1-3·§1-5 순서대로 M1을 골랐다.

## 1. 범위와 주장

- 작업 종류: 조사. 산출물은 [M1 설계](sprint_design.md) §2대로 최초로 달라지는 반복, 필드, 그 필드를 마지막으로 쓴 원본 함수, 같은 입력의 Rust 연산이다. Rust 솔버 코드는 바꾸지 않는다.
- 질문: kb2(Netlib, Method=1, Presolve=0, Threads=1, Seed 0)에서 Rust dual simplex가 원본과 처음 다른 값을 내는 반복과 필드는 무엇이고, 원본의 어느 연산과 Rust의 어느 연산이 다른가.
- 완료 후 주장할 것(§6.1 탐색에서 정한 가설, §3 검사로 판정):
  - 반복 0–11의 비교 필드(vstat·x_B·d·w, 가격 상태는 `statecmp`)는 같고, 처음 다른 값은 피벗 11(스냅숏 12)의 β=‖ρ_r‖²(spx+0x310) 마지막 비트다. 같은 피벗의 α_r(spx+0x300)과 ρ_r 원소 값은 같다.
  - 원본: `28c93` C:139–163이 BTRAN `09e376`의 출력이 조밀(nnz<0)이면 인덱스 오름차순으로 \|ρ_i\| > spx+0x3a8인 원소만 앞으로 모아 희소로 바꾼다. 희소 출력이면 그대로 둔다. `8b473` C:105–222(asm `0x11f7be`–`0x11fa4c`)가 그 벡터를 저장 순서대로 8레인 합산해 spx+0x310에 쓴다.
  - Rust: `crates/lp/src/simplex.rs:980`이 `lu.btran` 출력(조밀이면 0을 포함한 m개)을 거르기 전에 `sum_squares`로 더한다. 조밀 출력에서 8레인 묶음이 달라 마지막 비트가 갈린다.
- 제외 범위와 후속:
  - 수정은 M1-02b(구현)다. 이번에는 Rust를 바꾸지 않는다.
  - β를 원본 규칙으로 바꾼 임시 실험(§6.1)에서도 같은 반복의 DSE 가중치 2개가 남고 피벗 분기는 21 그대로였다. 이 두 번째 원인은 이번에 특정하지 않는다(새 하위 번호 제안, §7).
  - Method=1 Netlib 64 재측정은 M1-02c다.
- 선행: main `2bc84de`. 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 file offset(= objdump 주소)이다. 모델 `../re/bench/data/netlib/kb2.mps.gz` SHA-256 `fd4e89375a650a3a5eb25c1d57c293e1cf9b1791e0d6fa61a0d586b76ba5d516`.

| 위치 | 내용 |
|---|---|
| `28c93` `0xc51d8` | `09e376(LU, in=e_r, out)` 호출, 반환 `0xc51dd`. C:143 |
| `28c93` C:149–163 | out nnz<0이면 `i=0..m-1` 순서로 `spx+0x3a8 < \|x_i\|`인 원소를 `(idx, val)`에 앞에서부터 채우고 nnz를 센 수로 바꾼다. 희소면 `a171f`로 행 work만 더한다 |
| `8b473` `0x11f7ba` | rbx = spx+0x218 벡터(C:73 `puVar7`). ecx = nnz(음수면 `cmovl`로 m) |
| `0x11f7e1` | `cmp $6,%ecx`. 6 이하이면 `0x11f98b` 순차(2개씩) 경로 |
| `0x11f80a`·`0x11f81a` | 별칭 검사: spx−val ≥ 8n 또는 val−spx ≥ 0xf20이면 SIMD, 아니면 `0x11f98b` 순차 |
| `0x11f838`–`0x11f84f` | val&0xf≠0이면 8바이트 정렬일 때 첫 원소 1개를 먼저 더한다(아니면 순차) |
| `0x11f974` / `0x11f9dd`·`0x11f9f8`·`0x11fa4c` | SIMD 경로 / 순차 경로의 β 저장 |

- 시작 경계: 각 dual 반복의 `28c93` BTRAN 직후. 종료 경계: 같은 반복의 `8b473` β 저장 직후.
- 입력: BTRAN 출력(raw nnz, 조밀이면 m개 값), spx+0x3a8, `8b473`이 읽는 벡터(nnz·idx·val), val 주소의 정렬과 spx와의 거리(경로 판정에만, 주소값은 저장하지 않는다).
- Rust 쪽 경계: `RSOLVER_SNAP` 스냅숏(반복 j 줄의 `beta`는 피벗 j−1의 값)과 저장 오라클 `bench/oracle/iter0-30/kb2.jsonl.gz`(spx_oracle, 반복 0–30).

## 3. 비교 계약

`tools/m1_beta_check.py`가 판정한다. 하나라도 실패하면 종료코드 1이다.

| 검사 | 규칙 |
|---|---|
| M 입력 | 캡처 3회 각각 정상 종료(`DONE 2 38`), 바이너리·모델·도구 해시, 파라미터, 출력 파일 덮어쓰기 금지 |
| B 재현 | 3회의 사건 목록(주소값 제외, 경로·정렬 판정 포함)이 정확히 같다 |
| C 압축 | 모든 피벗에서 `8b473` 벡터가 `09e376` 출력과 §2 규칙으로 같다. 조밀이면 idx = {i : \|x_i\| > spx+0x3a8}(오름차순)이고 값 비트가 같다. 희소면 그대로다 |
| O 원본 합산 | 모든 피벗에서 저장된 β 비트 = 캡처 순서의 합산(SIMD: 8레인 `((0+2)+(4+6))+((1+3)+(5+7))` 뒤 나머지 순차, 첫 원소 떼기 반영 / 순차: 차례로) |
| R Rust 연산 | 피벗 0–21에서 Rust 스냅숏 β = `sum_squares(09e376 출력)`(조밀이면 0 포함 m개, 희소면 그대로)다. Rust β ≠ 원본 β인 피벗을 모두 나열하고, 그 피벗이 모두 조밀 출력이며 거기서 압축 합산이 원본 β와 같다 |
| F 최초 분기 | 저장 오라클과 Rust 스냅숏에서 반복 0–11의 vstat·x_B·d·w가 비트까지 같고, 반복 12에 처음 w가 다르다. 스냅숏 1–11의 β는 같고 12에서 다르며, 12의 α_r은 같다 |

- 비교는 모두 비트(정수·u64) 정확 비교다. 허용오차 없음.
- 판별되지 않는 절은 C만으로 보고한다: 압축 문턱(0과 1e-13), 첫 원소 떼기(정렬), n>6의 별칭 순차 경로, 8레인 합치기의 짝 묶음 대 사슬 묶음.
- 자기검사(`--self-test`): 바이너리 해시, 회차 차이, 빈 사건, 피벗 11 ρ 한 값 비트, β 한 비트, 조밀 압축에서 한 원소 누락, Rust β를 원본 β로 대체, 저장 오라클의 반복 5 w 한 비트 변조를 모두 거부해야 한다.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 | 원본 증거 |
|---|---|---|---|---|
| kb2 Method=1, Presolve=0, Threads=1 | 원본 실제 실행(읽기 전용 Frida 훅) | 희소·조밀 BTRAN 출력, SIMD·n≤6 순차 경로 | `kb2.mps.gz`(위 해시) | `bench/oracle/m1_02a_beta/kb2_{1,2,3}.json.gz` |

- 적용하지 않은 범주: 다른 Netlib 모델(이번 질문은 모델 하나), 정렬 8 mod 16 버퍼, 별칭 경로(실행에서 나오지 않으면 C만).

## 5. 재실행 명령

- 원본 캡처(작업 worktree 루트, Gurobi 한 프로세스씩): `for r in 1 2 3; do tools/gurun.sh node tools/m1_beta_path.js ../re/bench/data/netlib/kb2.mps.gz bench/oracle/m1_02a_beta/kb2_$r.json.gz; done`.
- Rust 빌드: `tools/build_local.sh`(이 worktree `python/rsolver`에 격리 빌드).
- 판정: `PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_beta_check.py --self-test`.
- 기존 도구(Presolve=0 수정 뒤): `PYTHONPATH=$PWD/python ../.venv/bin/python tools/proto/statecmp.py kb2 <kb2.jsonl>`, `tools/proto/pathcmp.py kb2`.

## 6. 실행 결과

### 6.1 탐색(계약 확정 전, 판정 근거 아님)

- 기존 비교 도구 `tools/proto/statecmp.py`·`pathcmp.py`·`alphacmp.py`·`tools/lp_check.py`가 rsolver에 Presolve=0을 주지 않았다. LP presolve가 들어온 뒤 기본값으로 presolve된 LP를 Presolve=0 원본 기준과 비교했다(statecmp는 행 수가 달라 실패). 네 도구에 `Presolve=0`을 넣었다. `bench/pathcmp.txt`(2026-11-06)는 presolve 전 코드로 만든 것이다.
- 고친 statecmp로 kb2를 다시 보니 2026-11-06 기록([포팅 가이드](../10_porting.md) §9.8)과 같았다. 반복 0–11 일치, 반복 12에 w 10개·score 5개 마지막 비트 차이, 피벗 분기 21.
- 스냅숏의 마지막 피벗 값: 스냅숏 12에서 α_r은 같고 β만 다르다(원본 `0x1.46f4755d4fd33p+2`, Rust `…d34p+2`). 스냅숏 13도 β만 다르다.
- `tools/spx_oracle.py`의 `tau`(spx+0x218)는 조밀(nnz=−1)이면 버렸다. 조밀도 저장하게 고쳤다. 그러나 콜백 시점의 spx+0x218은 ρ_r도 τ도 아닌 섞인 버퍼였다(스냅숏 12에서 idx 15·18·26만 피벗 11 ρ와 같고 0–13은 다른 값). 그래서 이 필드로 ρ·τ를 비교하지 않는다.
- 반복 11 콜백에서 원본 BTRAN `9e288`을 직접 불러(`tools/proto/btran_oracle.py` 변형, 스크래치) e_6의 출력을 얻었다. 조밀(nnz=−1)이고 비영 11개다. Rust `lu.btran` 출력(임시 계측, 커밋 안 함)과 값이 비트까지 같다. 이 조밀 배열 43개를 8레인으로 더하면 `…d34`(Rust), 비영 11개만 더하면 `…d33`(원본)이다.
- Frida 탐색(스크래치 `beta_path.js`, 1회): 실제 반복의 `8b473`은 피벗 11에서 n=11 희소 벡터를 SIMD 경로(정렬 0 mod 16)로 더했다. 38피벗 모두 val&0xf=0이었고, n≤6 피벗은 순차 경로, n>6은 모두 SIMD였다(별칭 경로 0회). Rust ρ를 ZERO(1e-13 또는 0)로 거른 뒤 같은 합산을 하면 피벗 0–21의 β 22개가 개수·비트까지 원본과 같았다. Rust β가 원본과 다른 피벗은 11·12·16·18·20이다.
- `28c93` C:139–163에서 압축 코드를 확인했다(§2). 반복별 β는 `8b473`이 쓴다. Rust 주석이 가리키던 `1bae9c` L1971–2110은 초기 정확 노름(EXACTNORM) 루프다.
- 스냅숏 1–13의 spx+0x308(행 α_rq)과 Rust `alpha[q]`(`last_aq_row`)는 절댓값 비트가 모두 같다. 부호는 스냅숏 1부터 저장 관례가 다르다(이 실행에서 원본 값은 항상 음수, Rust는 α 부호 그대로). 최초 분기와 무관하고 원본 부호 규칙은 확인하지 않았다.
- 임시 실험(커밋 안 함): Rust β를 "조밀이면 거른 뒤 합산"으로 바꾸면 반복 12의 w 차이가 10개에서 2개(첫 차이 var 58)로 줄지만 피벗 분기는 21 그대로다. 남은 차이의 원인은 특정하지 않았다.

### 6.2 고정 캡처

- 실행: worktree `../rsolver-m1-02a`, 계약 커밋 `3bdf88b` 뒤의 `tools/m1_beta_path.js`(sha256 앞 16자리 `ab414034c3e2f007`, 캡처 파일에 기록). 세 회 모두 `DONE 2 38`, 오류 없음. gz 파일 SHA-256이 세 회 모두 `caad5976f259e668d473e9e992b4c1b69e44254cbce7b63eda778c7b479e48b1`로 같다.
- 첫 시도에서 다른 세션이 Gurobi를 쓰고 있어 잠금 오류 출력으로 파일이 먼저 써졌고, `gurun.sh` 재시도가 덮어쓰기 금지에 걸렸다. 잠금 메시지면 파일을 쓰지 않고 끝내도록 도구를 고친 뒤 고정 캡처를 돌렸다(탐색 probe는 스크래치).
- Rust 쪽: 같은 worktree에서 `tools/build_local.sh`로 빌드한 main `2bc84de` Rust(임시 계측·실험 되돌림, `git diff crates/` 비어 있음).
- 판정 출력(`PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_beta_check.py --self-test`, 종료 0):

```text
M: 3 runs, binary/model/params/output, pinned file sha, tool = current file ab414034c3e2f007
B: 38 pivots, 3 runs equal
C: dense outputs compacted in index order with |x| > spx+0x3a8 (28 dense, 10 sparse)
O: stored beta = sum in captured order for 38 pivots; paths [(('scalar', 0, False), 13), (('simd', 0, False), 25)]; chain combine also equal on 22/25 simd pivots
R: Rust beta = sum_squares(uncompacted BTRAN output) on pivots 0-21; differs from the original on pivots [11, 12, 16, 18, 20] (all dense outputs, compacted sum equals the original there)
F: snapshots 0-11 equal (vstat, x_B, d, w), first w difference at 12; beta equal at 1-11, differs at 12; alpha_r equal 1-12
self-test: 10 tampers rejected (binary sha, run 3 differs, file sha, tool sha (all runs), empty events, pivot 11 rho
  value bit, pivot 11 beta bit, dense compaction drops one element, Rust beta replaced by original beta,
  stored oracle w bit at iteration 5)
PASS
```

- 판별력: spx+0x3a8은 세 회 모두 1e-13이다. 문턱 0과 1e-13은 이 실행에서 같은 결과다(\|x\|≤1e-13인 비영 원소 없음). 모든 피벗의 val이 16바이트 정렬이라 첫 원소 떼기는 실행되지 않았다. n>6 피벗 25개는 모두 SIMD였고 별칭 순차 경로는 0회다. 사슬 묶음은 SIMD 25피벗 중 3피벗에서 저장된 β와 달라 짝 묶음이 실행으로 판별됐다.
- 희소 출력 30원소 중 \|x\|≤spx+0x3a8인 원소는 0개다. "희소 출력은 거르지 않는다"(C:164–167)는 실행으로 판별되지 않아 C만이다.
- 계약 변경(결과 확인 뒤, 기록만): (1) 짝 대 사슬 묶음은 §3에서 C만 후보로 적었지만 실행에서 판별됐다(위). (2) 독립 검토 뒤 M을 강화했다. 캡처 gz SHA-256 고정과 캡처의 도구 해시 = 현재 파일 검사를 더했고, 변조가 8종에서 10종(file sha, tool sha 추가)이 됐다. 판정 규칙 C·O·R·F는 바꾸지 않았다.
- 실행 확인: Presolve=0을 넣은 `tools/proto/alphacmp.py kb2 25`(호출 0–21 α 순서·값 같음, 22에서 이탈 행 다름)와 `tools/lp_check.py --methods 1 kb2`(OK, 반복 38/38)를 돌렸다.
- 실행하지 못한 검사: 다른 모델, `cargo test`·clippy(Rust 미변경), `tools/lp_check.py` Presolve=0 수정의 재측정(M1-02c 범위).

## 7. 판정과 인수인계

- 작업 수행 상태: 완료(조사).
- 답: kb2 Method=1의 최초 분기는 피벗 11(스냅숏 12)의 DSE β(spx+0x310)다. 원본 `28c93` C:149–163은 조밀 BTRAN 출력을 \|x_i\|>spx+0x3a8 원소만 인덱스 순으로 압축하고, `8b473` C:105–222가 그 희소 벡터를 8레인으로 더한다. Rust `simplex.rs:980`은 압축 전 조밀 출력 m개를 같은 8레인으로 더해 묶음이 달라진다. ρ_r 값과 α_r은 같다(캡처·C).
- 통과 범위: kb2 한 모델, 피벗 0–37(원본 규칙), 피벗 0–21(Rust 연산 대조). 그 뒤는 피벗 경로가 갈려 같은 입력이 아니다.
- 남은 한계: 문턱 값(0 대 1e-13)·첫 원소 떼기·별칭 경로·희소 출력 무필터는 C만이다. Rust `lu.btran`이 피벗마다 조밀·희소를 원본 nnz 부호와 같게 고르는지는 비교하지 않았다(β만 비교). β만 원본 규칙으로 바꾸어도 반복 12에 w 2개 차이가 남고 피벗 분기 21은 그대로다(§6.1 임시 실험). 그 원인은 미확인이다.
- 후속:
  - **M1-02b**(구현): 조밀 판정은 출력 길이(`len == m`)가 아니라 원본처럼 BTRAN 경로의 조밀 표시로 해야 하고, 그 판정이 원본 nnz 부호와 같은지 먼저 대조한다. Rust β를 "조밀 출력이면 \|ρ_i\|>ZERO 원소만 인덱스 순으로 모아 `sum_squares`, 희소면 그대로"로 바꾼다. 검증은 `tools/m1_beta_check.py`의 R을 "Rust β = 원본 β(피벗 0–21)"로 바꾼 판정과 `statecmp` 반복 12 차이 감소다. `simplex.rs:979` 주석의 `1bae9c` L1971–2110 출처도 `8b473`·`28c93`으로 고친다. 행 work(`a171f`)와 `pivot_row`가 받는 ρ가 원본과 같은지(희소 출력은 거르지 않음)도 함께 확인한다.
  - **M1-02a-1**(조사, 새 행): β를 고친 상태에서 kb2 반복 12의 남은 DSE w 차이 2개(첫 var 58)의 원인. 콜백 시점 spx+0x218은 섞인 버퍼라 쓰지 않는다. `8b473`의 DSE 갱신 지점에서 α·τ를 훅으로 캡처한다.
  - Method=1 Netlib 64 재측정(M1-02c)은 Presolve=0을 넣은 `tools/lp_check.py`·`pathcmp.py`로 한다. `bench/pathcmp.txt`와 24/64는 presolve 도입 전 측정이다.
- 작업 branch·commit: `work/m1-02a`, worktree `../rsolver-m1-02a`. 미커밋 없음(이 기록 커밋 기준). 병합 상태는 [M1 인수인계](../handoff/M1.md)와 main 이력으로 확인한다.
- 갱신 위치: [M1 인수인계](../handoff/M1.md), [스프린트 목록](../14_sprint_plan.md) M1-02a·M1-02a-1 행, [작업 단위](../13_work_units.md) M1-02, [진행 기록](../11_progress.md)·[로드맵](../12_roadmap_diagram.md) 맨 위.
- 교훈 기록: [원본 교훈](../lessons/original.md) "simplex DSE β", [도구 교훈](../lessons/tooling.md) "M1 비교 도구 Presolve=0", "콜백 시점 spx+0x218", "gurun 재시도와 덮어쓰기 금지".
