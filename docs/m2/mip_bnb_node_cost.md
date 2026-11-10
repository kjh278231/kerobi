# M2-10a 노드별 복사·LP 준비·풀이 비용 계측 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다.

## 1. 범위와 주장

- 작업 종류: 조사.
- 조사 질문: 현재 Rust B&B(`crates/mip/src/bnb.rs` `solve_started`)에서 노드 하나를 처리하는 비용이 어느 구간에 쓰이는가. 구간은 노드 선택(`retain`·최소 경계 스캔·종료 검사), 분기(`branch`의 `Model` 복사), 노드 LP(스케일 작업 공간 생성·엔진 생성·dual 풀이·무스케일 재확인), incumbent 검사, 루프 밖의 presolve·루트·복원이다. 결과로 모델·구간별 비용표와 데이터로 고른 개선 대상 하나를 M2-10b에 넘긴다.
- 보조 질문(M2-10b 입력): 부모 최종 basis로 자식 LP를 dual warm start(`solve_model_warm`)하면 반복 수와 시간이 얼마나 되는가. 반사실 측정이며 탐색 궤적에는 쓰지 않는다.
- 포함 전제: Threads 기본, NodeLimit 500, TimeLimit 없음(벽시각으로 종료 지점을 만들지 않음), 나머지 파라미터 기본값.
- 제외: 최적화 구현(M2-10b 이후), Gurobi 노드 처리 속도와의 비율 판정(서로 다른 일을 하므로 P3 패리티가 아님), `crates/lp` 수정(M1 활성 영역).
- 선행: main `7f86824`, 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- 원본: 해당 없음. Rust 자체 비용 조사이며 원본 동작과의 동등성을 주장하지 않는다. Gurobi를 실행하지 않는다.
- 계측 경계: `solve_original` 진입부터 반환까지. 루프 구간은 `solve_started`의 `while` 한 바퀴를 위 구간으로 나눈다. 잔여(루프 벽시각 − 구간 합)를 따로 적는다.
- LP 내부 분할: `crates/lp`를 고치지 않고 `solve_model_run`(simplex.rs:1720)이 쓰는 pub 조각(`Spx::new`, `Engine::new`, `Engine::solve(1)`, `Spx::unscaled`, `Engine::resolve_unscaled`)을 같은 순서로 재생한다. 재생의 status·iter·obj 비트·x 비트가 매 노드 실제 호출(`solve_model_deadline`)과 같아야 분할을 인정한다. `deadline`은 private이지만 TimeLimit 없음이면 실제 호출도 `None`이다.
- 계측 코드: 임시 패치 `bench/oracle/m2_10a/profile.patch`(bnb.rs만). 측정 뒤 되돌린다. 하네스는 무시 테스트 `crates/mip/tests/bnb_profile.rs`.

## 3. 비교 계약

| 필드·사건 | 의미 | 규칙 | 방법 |
|---|---|---|---|
| 계측 무영향 | 상태·노드 수·반복 수·경계·목적·경로 | 패치 전후 정확히 같음 | 하네스 출력 비교 |
| LP 재생 일치 | 노드별 status·iter·obj 비트·x 비트 | 정확히 같음, 노드 1개라도 다르면 분할 무효 | 패치 안 `assert` |
| 구간 합 | 구간 합 / 루프 벽시각 | 차이를 기록(판정 아님) | 패치 출력 |
| 반복 분산 | 계측 실행 3회의 구간 시간 | 중앙값과 최소·최대, loadavg 기록 | 스크립트 |
| warm 반사실 | 부모 basis dual warm의 status·obj와 반복 수 | status 같고 obj `abs(a-b) <= 1e-6*(1+abs(a))`인 노드만 반복 비교, 불일치 수는 별도 | 패치 출력 |

- 시간은 벽시계(`Instant`)다. 같은 기계의 다른 세션 부하가 잡음원이므로 loadavg를 기록한다.

## 4. 사례

5초 기준표([mip_bnb_5s](mip_bnb_5s.md))에서 경로·크기·노드 속도가 다른 12개를 측정 전에 고정한다.

| 모델 | 5초 경로 | 5초 노드 | SHA-256 앞 16자 |
|---|---|---:|---|
| gen-ip002 | reduced | 8797 | cb37e723fa218794 |
| pk1 | reduced | 6301 | 6c64530538b254a8 |
| neos5 | reduced | 3413 | 2888e9aeb10a4cf1 |
| tr12-30 | reduced | 1237 | 94f8cf2fd2fc920e |
| neos-3754480-nidda | reduced | 1225 | 63f322dd9be49acf |
| gmu-35-40 | reduced | 532 | 0c91e71117164bec |
| assign1-5-8 | reduced | 459 | 305c26c9eee8da72 |
| supportcase26 | reduced | 2497 | e5769bd12cc2b5e7 |
| markshare_4_0 | unsupported_records | 67107 | 1a1feb04f5637db9 |
| mas74 | unsupported_records | 4720 | 019a9ca2dac80034 |
| neos-860300 | unsupported_records | 100 | d1b2a0a0500834ef |
| timtab1 | unsupported_records | 4539 | 5867d1a9c1067a84 |

입력은 `../re/bench/data/miplib/<모델>.mps.gz`다.

## 5. 재실행 명령

- `tools/mip_bnb_profile.sh <출력 디렉터리>`: 패치 없는 하네스 1회 → 패치 적용 → 계측 3회 → 패치 되돌림. 패치가 이미 적용됐거나 bnb.rs가 수정돼 있으면 시작하지 않는다.
- 요약: `python3 tools/mip_bnb_profile.py <출력 디렉터리>`가 무영향 비교와 표를 출력하고, 무영향 또는 LP 재생이 깨지면 0이 아닌 값으로 끝난다.

## 6. 실행 결과

(측정 뒤 작성)

## 7. 판정과 인수인계

(측정 뒤 작성)
