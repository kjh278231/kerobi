# M1-01e M1 재분해 2×2 블록 단계 켜기 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 기준 main `1b91bd5`, 미커밋 선행 변경 없음. 작업자는 이 세션이다("다음 스프린트 진행해 너가 작업자야" 지시. M2-06h-6은 다른 세션이 점유 중이라 M1 인수인계의 "다음"을 맡았다). branch `work/m1-01e`, worktree `../rsolver-m1-01e`.

## 1. 범위와 주장

- 작업 종류: 구현(원본 상태 캡처 오라클 포함).
- 완료할 동작: M1 simplex(`simplex.rs` `refactor`)가 `1da0b2` 2×2 블록 단계를 원본처럼 켠다. S(simplex 구조체)의 S+0x2c0(재분해 수)·S+0x4e0(블록 단계 실행 수)·S+0x4e4(LU2X2BLOCK 기본 → 0, 20단계 이상 피벗한 실행 수, S+0x2c0 > 20에서 −1 또는 2e9로 고정)의 수명을 원본과 같게 둔다. 블록 단계 본체는 M3-04c-21h `Lu::factor_block`을 그대로 쓴다.
- 둘째 단계(같은 작업, 별도 커밋·별도 회귀): M1 t(LU+0x100) = s3 정정. M1 `Lu::finish`는 t를 "앞쪽 빈 U 열 수"로 세는데 원본은 `1da0b2`가 둔 열 singleton 단계 3의 끝(s3)이다([교훈](../lessons/original.md), M3-04c-21g). 이 단계의 회귀 규칙(§5)을 통과하지 못하면 블록 단계만 병합하고 t = s3는 **M1-01f**로 분리한다.
- 포함 분기: 블록 단계 실행(활성 ≥ 50)과 건너뜀, S+0x4e0 증가, 자연 사례에 있으면 S+0x4e4 증가(블록 피벗 ≥ 20)·S+0x2c0 > 20의 −1/2e9 고정·2e9 뒤 활성 < 50 실행, 특이 기저 재시도(`05b257` `LAB_001f3392` 루프는 S+0x2c0을 다시 올리지 않음).
- 제외: 블록 단계 내부의 원본 비교(M3-04c-21h에서 push 19재분해로 검증, 이번에는 factor 결과를 기존 M1 검사기로 간접 확인), 블록 단계 풀 부족 −1, 반복 수 차이의 다른 원인(M1-02w·02y-2·02t·02u).
- 선행: M3-04c-21h(`Lu::factor_block`, `BlockState`), M1-02z(원인 특정, `tools/m1_urow_cmp.py`, `RSOLVER_UROW_PROBE`).

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- `036ade@0xd4190`의 호출자는 `05b257@0xf15c0` 하나다(`tools/dc.py 036ade`). 호출 인자: param_1 ZERO·param_2 Markowitz tol은 xmm, S = args[0](= `05b257` param_1), nb = args[2], n = args[3], 기저 head = args[8]. LU = S+0x470(`05b257` C:113), `1da0b2`의 param_21 = LU+0x20이라 C:1514–1534가 LU+0xd8·0xdc·0x100(+0xe0)·0xe4(+0xc4)를 쓴다.
- 상태 갱신 위치: `02f7e1` C:270–281(LU2X2BLOCK → S+0x4e4: 0→−1, 1→2e9, 기본 0), `05b257` C:1399(S+0x2c0 +1, `LAB_001f3392` 재시도 루프 밖), `1da0b2` C:681–684(실행 조건)·C:1494–1510(갱신). 다른 쓰기 위치는 캡처의 연쇄 검사로 확인한다(§3).
- 시작 경계: `036ade` 진입(S+0x2c0/0x4e0/0x4e4, head). 종료 경계: `036ade` 반환(S+0x4e0/0x4e4, 반환값, LU+0xd8/0xdc/0xe0/0xe4/0x100).
- 설정: Method=1, Presolve=0, Threads=1, 면허 크기 Netlib. 캡처 도구 `tools/m1_block_state.js`(읽기 전용, 피벗 번호 k는 M1-02q와 같은 `8b473`@0xc58d5).

## 3. 비교 계약

| 필드 | 규칙 | 방법 |
|---|---|---|
| 원본 재현성 | 정식 사례 × 독립 3회의 사건 목록 JSON 정확 일치 | `tools/m1_block_check.py --repro` |
| 원본 연쇄(다른 writer 없음) | 같은 S의 연속 호출에서 enter[i+1] = (enter[i].x2c0 + 1, leave[i]) | 같은 검사기 |
| Rust 블록 상태 | 원본과 Rust의 재분해를 순서대로 맞추되 **head가 같은 재분해까지** 진입 (x2c0, x4e0, x4e4)·반환 (x4e0, x4e4) 정확 일치. 첫 head 차이 위치를 보고한다 | Rust 탐침 `RSOLVER_BLOCK_PROBE=<file>`, 같은 검사기 |
| LU 경계(둘째 단계) | 같은 범위에서 LU+0xd8·0xe4·0x100(t) 정확 일치 | 같음 |
| factor 결과 | bnl1 e_r 126·129·130 행 U·대각 비트 일치(M1-02y 원본 덤프) | `tools/m1_urow_cmp.py` |

- 판별력: Rust 변이(임시 사본) "재분해마다 상태 `{0,0,0}` 재설정", "x2c0 미증가", "x2c0 시작값 1", "블록 끔(OFF)"을 반드시 거부해야 한다. 검사기 변조(바이너리 해시 0, 사건 비우기, 연쇄 필드 변경)도 거부해야 한다.

## 4. 사례와 원본 자료

- 탐색: 면허 크기 Netlib 64개를 1회씩 캡처해 분기(블록 실행, x4e4 증가, x2c0 > 20 결정 −1/2e9, 2e9 뒤 실행, 특이 재시도 rc ≠ 0)를 덮는 모델을 고른다(`bench/oracle/m1_01e/explore/`).
- 정식: 고른 모델 × 3회(`bench/oracle/m1_01e/formal/`). 결과는 §6에 적는다.

## 5. 회귀 규칙(착수 때 고정)

- 코드 수정 전 기준선 `bench/oracle/m1_01e/regress.sh base`를 먼저 돌린다(cargo lp·workspace·clippy, pytest, M1 검사기, pathcmp, lp_check, urow 3건).
- `lp_check` Method 1·0·−1 정답 수가 줄지 않는다. 줄면 병합하지 않고 사용자에게 올린다.
- `pathcmp` 완전 일치 수(현재 27/64)가 줄지 않는다. 바뀐 모델 목록과 첫 차이 위치 전후를 남기고, 첫 차이가 앞당겨진 모델은 원인을 적는다.
- `m1_lt_check.py`의 bnl1 고정 7호출(`KNOWN_IDX`)은 블록 단계로 설명되므로 이 작업에서 고정 목록을 고친다. 다른 M1 검사기는 base와 같은 결과여야 하며 바뀌면 원인을 적는다.
- 블록 켜기와 t = s3는 커밋과 회귀를 나눈다.

## 6. 실행 결과

(실행 후 작성)

## 7. 판정과 인수인계

(실행 후 작성)
