# M1-02n 검증 계약과 결과: FT 갱신의 접두 경계·LU+0xf0 순서와 재분해 판정

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. [M1-02m](m1_02m_usolve.md) §8의 후속이다. 사용자의 "스프린트 이어서 작업해줘 너가 작업자야" 지시로 착수했다(2026-11-11). M2는 다른 세션이 M2-06c-3을 점유해 M1을 골랐다([시작 절차](../handoff/README.md) §1-3).

## 1. 범위와 주장

- 작업 종류: 조사(새 읽기 전용 캡처) + 조건부 구현. 구현은 (Q1·Q2) 원본 규칙을 캡처로 확정하고 Rust의 첫 차이가 그 규칙의 입력·연산임을 보인 뒤에만 같은 branch 둘째 단계로 넣는다. Q3가 simplex work를 요구하면 구현하지 않고 후속 번호로 넘긴다(근사 금지).
- 조사 질문(degen2 Method=1 Presolve=0 Threads=1):
  - **Q1**: 피벗 313 갱신 뒤 접두 t(LU+0x100)가 Rust 357, 원본 356이고 LU+0xec가 176 대 175다. `05b257` C:463–512(새 경계 local_40)의 어느 입력·분기가 다른가.
  - **Q2**: 피벗 316 재분해 뒤 t·ec는 같고 LU+0xf0 순서가 다르다(02m 첫 329). 재분해 자체인가, 317–328의 어느 갱신인가.
  - **Q3**: 원본은 갱신 39에서 재분해하고(02m 검사기: 피벗 356 원본 갱신 0, Rust 40) Rust는 계속한다. `06268b` LAB_001f6eb5(C:1273–1303)의 세 조건(주기 2, 채움 10, 39 % 10 = 9의 work 10) 중 어느 것이 참인가. C:1263의 다른 이유 10 경로와 `028c93` 안의 이유도 구분한다.
- 포함: degen2·kb2의 dual 실행 전체에서 `06268b`가 부른 `05b257` 모든 호출(반환 1로 다시 부르는 호출 포함)과 LAB_001f6eb5 판정 전부.
- 제외와 후속: simplex work 누적(Rust 없음, G1 P2)은 Q3가 work 조건이면 새 후속 번호로 넘긴다. τ 값·반복 275 mode 0 목록은 M1-02o. primal `0b2cdf`(같은 꼴, 상수 8)는 범위 밖.
- 선행: main `b5c118a`. 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca2…0e98f9c`, 주소는 file offset(ghidra − 0x100000). 모델 `../re/bench/data/netlib/degen2.mps.gz`, `kb2.mps.gz`.
- `05b257(spx, &work)` `0xf15c0`, 호출 `06268b` C:183·188(반환 1이면 다시 부름). 진입 때 spx+0x294 ≠ 0이면 재분해, 0이고 spx+0x290 = 0이면 FT 갱신(m1_notes §8.10).
- 판정 지점 `06268b` LAB_001f6eb5 = `0xf6eb5`(`mov 0xe8(%rsp),%r11`, 8바이트 명령 경계, 함수형 probe). 이때 r14 = spx, [rsp+0xe8] = LU, r12 = work 포인터(`0xf8aad` 사용 전 r12 쓰기 없음), [rsp+0x78] = m, [rsp+0x80] = 1.5m, [rsp+0x88] = 4m².

```text
LAB_001f6eb5 (C:1273–1303, asm 0xf6eb5–0xf8afd), k = LU+0x18c
  k >= LU+0x188                                  -> 이유 2
  else: f = (k < spx+0x46c) ? 0
             : (1.5·LU+0x190 + 2·LU+0x184)·m − min(((LU+0x1e8 + LU+0x1e0) − 1.5m)², spx+0x46c·4m²)
        f > 0                                    -> 이유 10 (채움)
        k % 10 == 9 and spx+0x158 > 0 and 14·spx+0x158 + 100·spx+0x3d0 < *work − spx+0x150 -> 이유 10 (work)
spx+0x150 = 분해 시작의 work, spx+0x158 = 분해 work의 평균((새 + 옛)/2, 첫 값은 그대로)
  (036ade C:531–539, C:5760–5772, env+0x2454 SIMPLEXENDFAC ≠ 0이고 env+0x2728 < 2일 때만)
```

- C:463–512(새 경계 local_40, 새 t = local_40 + 1)와 입력: r = spx+0x2f8, q = pos[r](LU+0x40), p = max(q, t − 1), l = max(p, spike 목록(LU+0x1c8)의 r 아닌 원소의 pos) — 값 필터 없이 목록 원소 전부(C:268–302), n′ = 목록 수 − 1(r이 있을 때), LU+0xe8에서 내려가는 빈 열 탐색(열 시작 LU+0x88 = 끝 LU+0x90). 식은 m1_notes §8.10 5.
- 읽기 전용. 주소는 저장하지 않는다.

## 3. 비교 계약

새 캡처 도구 `tools/m1_update.js`(`tools/m1_usolve.js`의 `8b473` 피벗 번호 그대로)와 검사기 `tools/m1_update_check.py`.

| 검사 | 규칙 |
|---|---|
| M 입력 | degen2·kb2 각 3회 gz SHA-256 고정·바이트 동일, 정상 종료, 바이너리·모델·도구 해시 |
| N 비간섭 | 02m 캡처의 커널 피벗 k마다 (t, ec, f0) = 이 캡처에서 피벗 k 직전 `05b257` 반환 상태. 반환 상태가 없는 피벗(그 사이 재분해 등)은 수를 따로 센다 |
| R 재분해 판정 | 모든 LAB_001f6eb5 판정에서 위 세 술어를 캡처 입력으로 계산한 이유(0·2·10)가 다음 `05b257` 진입의 spx+0x294와 같다. 그 사이 다른 쓰기가 있으면(진입 이유 ≠ 판정 이유) 수와 첫 사례를 따로 보고하고 실패로 센다. 판정마다 어느 술어가 참인지(채움·work·둘 다), k % 10 = 9인 판정 중 work 술어가 거짓인 수를 출력한다 |
| B 접두 경계 | 모든 FT 갱신 호출(진입 이유 0, 0x290 = 0, 반환 ≠ 1): §2 식으로 계산한 local_40 + 1 = 반환 t, 빈 열 탐색 결과 = 반환 LU+0xe8(탐색한 호출만). 분기별 실행 수 출력 |
| Q 답 | degen2 피벗 313 갱신의 입력(q, t, p, l, n′, e8, 빈 열)과 분기, 피벗 356 앞 판정의 이유와 참인 술어, 피벗 316–329의 f0 첫 차이 위치(Rust 대조) |
| Rust 대조 | (`--rust`) 이 worktree 빌드의 `RSOLVER_SNAP`: 같은 갱신 수인 피벗에서 FTRAN 시점 (t, f0) = 이 캡처의 피벗 진입 상태. 첫 차이 피벗과, 그 피벗에서 Rust 입력(q, l, n′, 빈 열) 중 처음 다른 것 |
| T 변조 | (`--self-test`) 바이너리 해시 0, 빈 사건, 반환 t 1 변경, 재분해 이유 변경, work 값 한 비트, 모델 변이 3종(work 상수 14 → 8, `% 10` 생략, n′을 값 > ZERO 원소 수로). 모두 거부 |

- 정수·순서는 정확 비교, work·평균 등 double은 IEEE 비트(hex)로 저장해 원본과 같은 연산 순서로 계산한 값과 비트 비교한다.
- 원본 반복: 같은 설정 독립 프로세스 3회, gz 바이트 동일.
- 합격(조사): M·N·R·B 전부 통과, 그리고 Q1·Q2·Q3 각각 원인을 특정(입력·분기·피벗)하거나 특정하지 못한 이유를 기록.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 | 원본 출력 |
|---|---|---|---|---|
| degen2 Method=1 Presolve=0 Threads=1 | 원본 실제 실행 | Q1·Q2·Q3, 재분해 9회 이상 | `degen2.mps.gz` | `bench/oracle/m1_02n/degen2_up_{1,2,3}.json.gz` |
| kb2 같은 설정 | 원본 실제 실행 | 비회귀(02h·02m 범위) | `kb2.mps.gz` | `bench/oracle/m1_02n/kb2_up_{1,2,3}.json.gz` |

## 5. 재실행 명령

- 원본 캡처(작업 worktree 루트, Gurobi 한 프로세스씩): `for r in 1 2 3; do for m in degen2 kb2; do tools/gurun.sh node tools/m1_update.js ../re/bench/data/netlib/$m.mps.gz bench/oracle/m1_02n/${m}_up_$r.json.gz; done; done`.
- 판정: `python3 tools/m1_update_check.py --self-test`. Rust: `tools/build_local.sh && PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_update_check.py --rust --self-test`.

## 6. 실행 결과

(실행 후 작성)

## 7. 판정과 인수인계

(실행 후 작성)
