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

진행 중. 계약 작성 뒤 원본 세부 분기와 Rust 계산 위치를 대조한다.

## 7. 판정과 인수인계

진행 중. branch `work/m1-02v`, worktree `../rsolver-m1-02v`. shared 문서는 다른 세션이 점유 중이므로 점유를 얻은 뒤 갱신한다. 원격 작업 없음.
