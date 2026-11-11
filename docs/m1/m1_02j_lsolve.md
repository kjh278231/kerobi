# M1-02j 검증 계약과 결과: degen2 첫 차이(희소 L 출력 순서)

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. [M1-02h](m1_02h_aq_order.md) §7의 후속이다. 사용자의 "다음 스프린트 이어서" 지시로 착수했다(2026-11-11). M2는 다른 세션이 직전에 M2-06c를 점유해 M1을 골랐다([시작 절차](../handoff/README.md) §1-3).

## 1. 범위와 주장

- 작업 종류: 조사(새 읽기 전용 캡처 포함). 구현은 §8의 조건을 만족할 때만 같은 branch 둘째 단계로 넣는다.
- 조사 질문: degen2(Method=1, Presolve=0, Threads=1) `pathcmp` 첫 차이 피벗 288(진입 509 같음, 이탈 원본 164 대 Rust 738)의 원인. 처음 다른 입력과 그 원본 규칙은 무엇인가. 02h가 근사로 둔 희소 L(`21b9de`)·행 eta(`21f83a`, 인라인 희소) 출력 순서의 원본 규칙은 무엇인가.
- 포함 분기: dual DSE 피벗(`8b473` ← `28c93`)의 두-RHS FTRAN `09d112`가 부르는 `21b9de`(네 호출 지점)와 `21f83a`(두 지점). degen2 피벗 0–633, kb2 피벗 0–37.
- 제외 범위와 후속:
  - 반복 263 w 차이(§6 (II), τ 값 끝자리): 별도 후속 번호.
  - `21b9de` 경로 B(LU+0xe4 위 꼬리를 순차로 훑는 경로): degen2·kb2에서 0회면 C 구조만 기록하고 미검증.
  - primal `2207de`·`9d0b7`의 `21b9de` 호출: 이 캡처 범위 밖.
- 선행: main `faae34d`. 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca2…0e98f9c`. 모델 `../re/bench/data/netlib/degen2.mps.gz`, `kb2.mps.gz`.

| 위치 | 내용 |
|---|---|
| `21b9de` `0x12a990` C:60–80 | 인자 `(tol, LU, m, L = LU+0x20, in, out, mark, work, bucket, &work)`, 입력·출력은 L 인덱스 공간(`09d112` C:670–677이 LU+0x30으로 바꿈). 경로: `m − e4 < 6` 또는 `10(m − e4)² ≤ m` 또는 `beg[m] − beg[e4] ≤ 10(m − e4)`(e4 = LU+0xe4)이면 A, 아니면 B. A 안에서 LU+0x128 < 0 또는 ≥ 0.1m이면 A1, 아니면 A2 |
| C:81–257 (A1) | Kahn FIFO. 목록 = 입력 순서, 각 입력 차수 +1, 목록을 앞에서 훑으며 L 열 원소(저장 순서)의 차수 +1, 처음 닿은 행을 덧붙임. 값 단계: 입력 순서대로 x = 값, 차수 −1, 0이면 큐. 큐 FIFO: `|v| ≥ tol`이면 출력하고 열 원소에 `x_i = −(l·v) + x_i`, 차수 −1, 0이면 큐. 작으면 차수만 내림 |
| C:259–605 (A2) | 레벨 버킷. LU+0x118 레벨, LU+0x120 레벨 시작. 입력을 입력 순서로 각 레벨 버킷에 넣고 최소 레벨부터 올라간다. 버킷 안은 넣은 순서. `tol ≤ |v|`이면 출력·전파, 처음 닿은 행은 그 레벨 버킷 끝에 넣는다. 작은 값은 전파하지 않는다 |
| C:607–893 (B) | e4 아래만 Kahn(`|v| > tol` 출력), e4 이상은 닿은 최소 인덱스부터 m−1까지 순차(`|v| > tol`) |
| `036ade` C:5764–5830 | L 레벨: `LU+0x118 ≠ 0`이고 m ≥ 250이고 m > LU+0x104일 때 열 j = 0..m−1을 앞으로 훑어 `lev[i] = max(lev[i], lev[j]+1)`, LU+0x128 = 최대 레벨, 시작 = 레벨별 수의 누적합. 아니면 LU+0x128 = −1 |
| `09d112` C:1376–1553 (인라인 희소 R) | 입력 행 표시, 입력 행들의 LU+0x1a0(그 행을 읽는 첫 eta) 최솟값부터 최신 eta까지 `x_r = η·x`. `|v| ≤ tol`이면 x_r = 0, 아니면 표시 안 된 r을 목록 끝에 덧붙인다. 끝에 목록 순서로 `|x| ≥ tol`만 남긴다 |

- 시작 경계: 피벗 k의 `09d112` 호출 진입(`8b473`에서, 반환 주소 `0x120200`). 종료 경계: 그 `09d112` 반환.
- 읽는 상태: `21b9de` 입력 벡터(인덱스·값·순서), L(열 시작·행·값, 레벨·레벨 시작·LU+0x128, LU+0xe4, 행→L 순열 LU+0x30). tol은 xmm0이라 훅에서 읽지 못한다. 모델은 Rust `ZERO`(1e-13)를 쓰고, 출력 일치로 간접 확인한다.

## 3. 비교 계약

| 검사 | 규칙 |
|---|---|
| M 입력 | 새 캡처 degen2·kb2 각 3회 gz SHA-256 고정·바이트 동일, 정상 종료, 바이너리·모델·도구 해시 |
| N 비간섭 | 새 캡처 `aq_out` 인덱스 = 02a-1 형식 캡처(`tools/m1_tau_path.js`, degen2 새 1회·kb2 `bench/oracle/m1_02a1_tau/kb2_1`)의 같은 피벗 `aq_out` |
| L 커널 | 모든 `21b9de` 호출: §2 모델(경로 판정 포함)이 캡처 L·입력으로 계산한 출력 인덱스 순서와 값 = 캡처 출력(비트) |
| R 순서 | 모든 `21f83a` 호출: 희소 쪽 출력 = L 출력 순서에서 작아진 행을 빼고 새 행을 끝에 덧붙인 것(앞부분 순서 보존). 덧붙는 행의 eta 순서는 캡처에 eta가 없어 Rust 연결 검사(§8)로만 판정 |
| −1 행 | 희소 U(`21686f`·`21841f`)이고 α_q 쪽 L이 희소인 피벗: 최종 α_q의 LU+0xf8 = −1 행 순서 = L 출력의 −1 행 순서 |
| T 변조 | (`--self-test`) 바이너리 해시 0, 빈 사건, 출력 두 원소 교환, 값 한 비트, A2를 A1로 바꾼 모델, 버킷 안 역순 모델. 모두 거부 |

- 값은 IEEE 비트 비교다. 순서는 정확 비교다.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 | 원본 출력 |
|---|---|---|---|---|
| degen2 Method=1 Presolve=0 Threads=1 | 원본 실제 실행 | m = 444 ≥ 250: A2(레벨), `21f83a` 두 지점, 인라인 희소 R | `degen2.mps.gz` | `bench/oracle/m1_02j/degen2_ls_{1,2,3}.json.gz`, `degen2_tau_1.json.gz`(02a-1 도구), `degen2_260_300.jsonl.gz`(spx_oracle 반복 260–300) |
| kb2 같은 설정 | 원본 실제 실행 | m < 250: A1(Kahn) | `kb2.mps.gz` | `bench/oracle/m1_02j/kb2_ls_{1,2,3}.json.gz` |

- 캡처 도구 `tools/m1_lsolve.js`(새, 읽기 전용): `8b473`·`09d112` 경계는 `tools/m1_tau_path.js`와 같고, `21b9de`·`21f83a` 입출력과 L 덤프(바뀔 때마다)를 더했다.
- 적용하지 않은 범주: 경로 B(두 모델에서 0회), A1의 작은 값 분기(실행 여부는 결과에서 셈).

## 5. 재실행 명령

- 원본 캡처(작업 worktree 루트, Gurobi 한 프로세스씩): `for r in 1 2 3; do for m in degen2 kb2; do tools/gurun.sh node tools/m1_lsolve.js ../re/bench/data/netlib/$m.mps.gz bench/oracle/m1_02j/${m}_ls_$r.json.gz; done; done`, `tools/gurun.sh node tools/m1_tau_path.js ../re/bench/data/netlib/degen2.mps.gz bench/oracle/m1_02j/degen2_tau_1.json.gz`.
- 판정: `python3 tools/m1_lsolve_check.py --self-test`.
- 상태 대조: `zcat bench/oracle/m1_02j/degen2_260_300.jsonl.gz > <tmp>/d.jsonl; PYTHONPATH=$PWD/python ../.venv/bin/python tools/proto/statecmp.py degen2 <tmp>/d.jsonl`.

## 6. 실행 결과(조사)

(작성 중)

## 7. 판정과 인수인계

(작성 중)
