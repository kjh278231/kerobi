# M1-02v 반복 커널 work 검증 계약과 결과

## 1. 범위와 주장

- 종류: 조사·구현. 기준 main `27bbda9`, 미커밋 선행 변경 없음(main의 미추적 `.env`는 사용·변경하지 않음).
- 대상: `a4109` 1, `1c35c5` 7, `28c93` 1, `1c8477` 3, `2896a3` 3, `15d2d` 4지점의 work 저장. 합계 19지점.
- 입력 범위: 기존 M1-02p·02q의 kb2, degen2 피벗 0–355. 기존 검사기의 정렬 경계·분해 입력 검사를 유지하고 지점·k·양을 비교한다.
- 제외: degen2 피벗 356 이후, quad·병렬·다른 호출자, LU 풀이(02t), 48d5f·8b473(02u), work 재분해 판정 연결. 관측 0회 분기는 코드 근거와 미검증을 구분한다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- 원본: `../re/decomp/c/00134d90_PRIVATE00000000000a4109.c`, `000c74a0_PRIVATE00000000001c35c5.c`, `000c4fe0_PRIVATE0000000000028c93.c`, `000cd720_PRIVATE00000000001c8477.c`, `000ccbc0_PRIVATE00000000002896a3.c`, `000bdc10_PRIVATE0000000000015d2d.c`와 대응 asm. 주소는 ELF file offset, 저장 뒤 trap pc는 [식 부록](m1_02p_formulas.md) 1부.
- 경계: 각 work 저장 직전 원본 work를 입력으로, Rust가 같은 반복 상태에서 계산한 양을 원본 연산 순서로 더한 직후 값. 기존 검사기 W와 동일하며 절대 work 전체 일치를 주장하지 않는다.
- 상태: 같은 모델·Method=1·Presolve=0·Threads=1, 원본 후보·행·열·basis·pricing 표시·목록 순서에 대응하는 Rust 상태. F1 basis 위치 차이는 기존 계약대로 집합만 검사하고 명시한다.

## 3. 비교 계약

| 필드 | 규칙 | 방법 |
|---|---|---|
| 저장 지점·k·실행 순서 | 정수·순서 정확 일치 | 02p watchpoint W와 Rust Work 로그 |
| work 저장값 | double 비트 일치 | 원본 직전 값 + Rust의 계산 항(곱 결합·carry 유지) |
| 분해 입력·항·0x158 | 기존 A·K·F 계약 유지 | 02q 캡처 재생 |
| 경로 회귀 | 기존 기준선과 동일 | Netlib pathcmp·lp_check 출력 비교 |

NaN·무한대는 사례에 없음. x148=1.3. 원본 캡처는 각 사례 독립 3회 바이트 일치한 02p·02q 자료를 재사용한다. 새 Gurobi 실행 없이 저장 자료를 비교하며, 이번에 새 원본 경계를 만들지 않는다. 시간 성능은 측정하지 않는다.

## 4. 사례와 기준 자료

- 실제 LP: `../re/bench/data/netlib/{kb2,degen2}.mps.gz`. 기준 자료 `bench/oracle/m1_02p/*_w_{1,2,3}.json.gz`, `bench/oracle/m1_02q/*_fw_{1,2,3}.json.gz`. 바이너리·입력·도구·자료 해시는 기존 검사기 M으로 고정한다.
- 분기: 단일/희소/조밀 pivot row, BFRT 후보·quickselect·재훑기, mode 0 재구성·mode 1 선택/재탐색, flip 희소/조밀 경로. 실제 도달은 결과에서 지점별로 보고하며 0회는 미검증이다.
- 오류 검출: 새 지점의 양·지점·k·누락을 변조하고, 원본 해시·목표 0회·필수 상태 누락의 거부를 유지한다. Rust 식 변이는 관측 가능한 계수·분할 길이 변경을 사용한다.

## 5. 재실행 명령

- `python3 tools/m1_work_check.py --self-test`
- `tools/build_local.sh`(실제 cargo 종료와 산출물 별도 확인), `PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_factor_work_check.py --self-test`
- `cargo test -p rsolver-lp`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`
- worktree 전용 확장으로 관련 M1 검사기·pytest·Netlib 기준선 회귀. 세부 명령은 실행 뒤 기록한다.

## 6. 실행 결과

- 저장 비교는 19지점 모두 도달했고 kb2 758·degen2 6,459저장이 지점·순서·k·값 비트까지 일치했다. 기존 02q·02r·02s 지점을 포함한 수다. 새 지점만 kb2 251·degen2 2,341이다. 재분해 진입 work는 기존처럼 원본 값으로 주고 각 저장은 원본 직전 값 + Rust 양으로 비교했다.
- 변조 28종 모두 거부. Rust 변이는 실행 중이다. 원본 Gurobi 실행 0회다.
- 최초 실패와 수정: `bench/oracle/m1_02v/first_failure.txt`. 후보 1개는 `1c35c5` C:788의 `uVar46 != 1` 바깥으로 가서 0xc82ee·0xc8b67·0xc92bd를 저장하지 않는다. `1c8477` C:239–256은 FTRAN의 반환 표현이 조밀할 때만 0xce316을 저장한다. 같은 함수의 k는 정상 피벗에서 이미 증가한 상태다(`28c93` C:820–834).
- 첫 회귀에서는 희소 L/R의 수치 출력을 그대로 사용해 maros 반복 수 1802 → 1744, perold 1191 → 1211이 됐다. 첫 차이 피벗 475·165는 기준선과 같았지만 이번 범위를 넘는 변화라 실패로 보존했다(`before_dense_preservation_pathcmp.txt`). 최종 구현은 기존 `ftran_dense` 수치 결과를 유지한다. 희소 진입인 경우만 별도 L/R 계산으로 반환 표현을 판별하며, 이 중복 계산은 M1-02t에서 검증한 커널·반환 표현을 함께 사용하는 것으로 정리한다. 이번 작업은 시간 성능을 주장하지 않는다.
- 최종 회귀: `cargo test -p rsolver-lp` 42 통과·1 무시, `cargo test --release --workspace` 125 통과·11 무시, `cargo clippy --workspace --all-targets -- -D warnings` 종료 0. `PYTHONPATH=$PWD/python ../.venv/bin/python -m pytest` 657 통과.
- 기존 M1 검사기 12종(새 factor work 포함) `--self-test` 통과. `lsolve`·`update`·`usolve`는 `--rust`도 실행했고, BFRT 3,112호출 코퍼스 재생 `tools/m1_bfrt_rust.py --self-test`도 통과했다. 각 출력은 `bench/oracle/m1_02v/`에 저장했다.
- Netlib: `tools/proto/pathcmp.py all`의 64개 모델 출력이 기준 `27bbda9` 독립 빌드와 바이트 동일하다(완전 일치 27/64, 중앙 첫 차이 비율 0.752). `tools/lp_check.py --methods 1,0,-1 --timeout 120 --csv bench/oracle/m1_02v/lp_check.csv`의 273행 상태·목적값·반복 수·오류도 기준 빌드와 정확히 같다. 두 빌드 모두 정답 90/91 ×3, IterCount 일치 31/0/31이다. 시간은 비교 대상이 아니다. 기준 출력은 `base_pathcmp.txt`·`base_lp_check.csv`에 보존했다.
- 최종 회귀의 입력과 기준선은 변경하지 않았다. Rust 변이 검사는 21종을 실행한 뒤 22번째 기존 `c2761 k + s` 치환 대상이 새 CHUZR 식에도 들어가 중복 assert로 중단됐다. 통과로 세지 않고 `mutate_first21.txt`에 보존했다. 치환에 `self.work.add(0xc2761, …)`를 포함해 대상을 한정하고, `PYTHONPATH=$PWD/python ../.venv/bin/python -u bench/oracle/m1_02v/resume_mutations.py`로 나머지 17종을 실행한다. 기본 `--mutate`는 여전히 전체 38종을 실행한다.
- 새 0xc9436 변이의 사전 정의를 실행 전에 정정했다. 원래 안은 Kp 항을 없애는 것이었으나 저장된 kb2 유일 사건에서 Kp = 0이라 판별력이 없었다. 원본 0xc92bd·0xc9436의 양이 모두 4 단위인 것을 확인하고, 관측되는 prefix 계수 4 → 3 변이로 교체했다. 검사기의 합격 조건과 원본 자료·비트 비교 규칙은 유지했다.

### 지점별 비교 횟수

| 지점 | kb2 | degen2 k < 356 |
|---|---:|---:|
| 0x135e92 | 38 | 356 |
| 0xc7bcd | 38 | 356 |
| 0xc7c4f | 38 | 356 |
| 0xc82ee | 33 | 333 |
| 0xc8b67 | 33 | 333 |
| 0xc92bd | 18 | 181 |
| 0xc9436 | 1 | 0 |
| 0xcbcae | 3 | 46 |
| 0xc561a | 1 | 97 |
| 0xcce72 | 29 | 78 |
| 0xcd052 | 4 | 0 |
| 0xcd191 | 4 | 0 |
| 0xbe039 | 2 | 2 |
| 0xbe0a1 | 2 | 2 |
| 0xbe17e | 2 | 2 |
| 0xbe34d | 2 | 2 |
| 0xcdbb5 | 1 | 97 |
| 0xce1ef | 1 | 97 |
| 0xce316 | 1 | 3 |

분기 근거: `a4109` C:443–556·792·996·1353–1394, `1c35c5` C:438·488–752·788–873·1471·1564·1618, `2896a3` C:145–263, `15d2d` C:98–166·485, `1c8477` C:95–256·470. `2207de`(`../re/decomp/c/00120740_PRIVATE00000000002207de.c`) C:149–160·688–817에서 L/R 표현과 U 선택을 확인했다. quickselect carry는 asm 0xcbc89–0xcbcae, 0xc7c25–0xc7c4f를 직접 대조했다. 나머지 곱 결합은 02p 식 부록의 asm 근거와 이번 저장 비트 비교다.

## 7. 판정과 인수인계

- 구현 commit `5008420`(계약 `4785ce0`), branch `work/m1-02v`, worktree `../rsolver-m1-02v`.
- 다른 세션의 M2-06g-2 완료 뒤 shared 점유를 얻었고, main `04bb80e`를 작업 branch에 통합했다. 선행 미커밋 변경·원격 작업 없음.
- 다음 M1 작업: M1-02k(BTRAN Uᵀ 희소 판정의 dsum) → M1-02t(LU 풀이 work). M1-02u(48d5f·8b473 fold)도 남았다.
- 미검증: 0행, 빈 후보 오류·특이 basis·quad·Threads > 1·다른 호출자·degen2 k ≥ 356. 희소 U 수치 커널 전체의 동등성은 이 작업이 주장하지 않는다. row-copy multiplier가 −1이 아닌 big-M 문맥, BFRT 반복 재시도·원본 RNG cost shift, 인공 bound 목록 carry의 미관측 분기도 포함하지 않는다.
- 교훈: `docs/lessons/original.md`(후보 1개 저장 건너뛰기·FTRAN 반환 표현), `docs/lessons/tooling.md`(work 복원 중 수치 커널 변경의 회귀).
- 작업 완료·동등성 최종 판정과 상세 검증 결과는 실행 종료 뒤 기록한다.
