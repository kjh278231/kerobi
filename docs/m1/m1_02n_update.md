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

### 6.1 원본 캡처와 원본 규칙(조사)

- 캡처 commit `f70558b` 뒤 도구 `tools/m1_update.js`(SHA는 캡처 안 `tool_sha256`), Gurobi 6회(한 프로세스씩, `tools/gurun.sh`). degen2 `DONE 2 634`(05b257 637·판정 635), kb2 `DONE 2 38`(40·39). 3회 gz 바이트 동일: degen2 `10646348…`, kb2 `61305231…`(검사기에 고정).
- 탐색 1회(scratch, 같은 도구)를 기준 캡처 전에 돌렸다. 그 gz는 기준 degen2 캡처와 SHA-256이 같다(`10646348…`, 바이트 동일). 그래서 §6.2의 입력 대조(탐색 캡처 사용)는 기준 캡처와 같은 자료다.
- `python3 tools/m1_update_check.py --self-test` 종료 0:

```
degen2: B l=p 111, q<p empty column 66, q<p no empty column 307, q>=p p=t-1<l 2, q>=p other 135 (e8 searched = returned 360),
        spike entries |v| <= ZERO 0, N 267, R decisions 635 (reason 0 622, reason 10 13: fill 7, work 6), k%10=9 with work false 53
kb2:    B l=p 10, q<p empty 1, q<p no empty 18, q>=p other 6, N 7, R decisions 39 (reason 10 3: fill 3, work 0)
Q degen2 pivot 313 update 35: r 365 q 140 t 357 LU+0xe8 357 n' 40, U row r 5 entries, q<p no empty column, searched to 356, new t 356
Q degen2 pivot 326 update 10: r 256 q 256 t 364 LU+0xe8 388 n' 20, U row r 1 entries, q<p empty column, searched to 388, new t 364
Q degen2 decision before pivot 356: update count 39, reason 10 (fill False, work True): 1066922.27 > 14*32951.87 + 100*3978
```

  - B: FT 갱신 656회(진입 이유 0인 모든 호출, degen2 621·kb2 35) 전부 §2 식의 새 t가 반환 t와 같다. 다섯 분기가 모두 실행됐다. 반환 LU+0xe8은 탐색 결과 이상이다(탐색 392회 중 같음 379, 나머지는 판정 뒤 C:979–1019 또는 m1_notes §8.10 12단계(n′ = 0)가 올림).
  - R: 판정 674회 중 뒤에 `05b257`이 오는 672회 전부(각 실행 마지막 판정 1회씩은 다음 호출이 없어 비교하지 않음) 세 술어로 계산한 이유가 다음 `05b257` 진입의 spx+0x294와 같다. 이유 4·1(시작·`01ca57`)은 판정을 거치지 않는다(probe 이유 0 판정에서 다른 쓰기 0회). degen2 이유 10 재분해 13번은 피벗 16·37·82·167·228·278·316이 채움, **356·396·436·486·536·586이 work**다. work 술어는 갱신 33부터 참이었지만 39에서만 발동한다(k % 10 = 9이고 work 거짓인 판정 53회 포함).
  - N: 02m 커널 피벗 274회 모두 (t, ec, LU+0xf0, 갱신 수) = 직전 `05b257` 반환 상태(계측 영향 없음).
  - 변조 6종(바이너리 해시, 빈 사건, 반환 t + 1, 이유 10 → 0, work 한 비트, N만 읽는 `05b257` 반환 LU+0xf0 두 원소 맞바꿈(검토 뒤 추가))과 모델 변이 2종(work 상수 14 → 8: 피벗 346, `% 10` 생략: 피벗 315)이 거부됐다. 변이 "n′·l을 값 > ZERO 원소로"는 **생존**했다. 캡처의 spike 목록에 ≤ ZERO 원소가 0개라 목록 수 = 값 수이기 때문이다(판별 불가, 아래 계약 변경).
- **답.**
  - Q3: 피벗 356 앞 재분해는 **work 조건**이다(갱신 39, 분해 시작부터 work 1,066,922 > 14·32,951.87 + 100·3,978 = 859,126; 채움 −196,183). spx+0x150은 분해 시작 work, spx+0x158은 분해 work의 이동 평균이다(§2). Rust에는 simplex work 누적이 없어 이 판정을 옮길 수 없다(근사하지 않음 → 후속 M1-02p).
  - Q1: 피벗 313 갱신의 원본 입력은 q 140 < p 356 < l 441, LU+0xe8 357이고 357 열이 비어 있지 않아 빈 열 없음 → t − 2(새 t 356)이다. 원본은 이 판정을 행 r(U 행 5원소)을 열들에서 지우기(C:979–1019) **전에** 한다.
  - Q2: 316 재분해 뒤 상태는 피벗 326 갱신 입력까지 원본과 같았고, 그 갱신(Q1과 같은 꼴: 원본은 LU+0xe8 388의 빈 열을 찾음)에서 Rust t가 1 컸다. 329의 LU+0xf0 차이는 그 결과다. Q1과 원인이 같다.

### 6.2 Rust 첫 차이와 구현

- 임시 덤프(갱신마다 q·t·p·l·n′·LU+0xe8·LU+0xec·빈 열·LU+0xf0, 커밋하지 않음)를 탐색 캡처와 순서대로 맞췄다. 수정 전: 갱신 0–306은 입력·출력이 같고, 갱신 307(피벗 313)은 **입력이 모두 같은데** 반환 t가 Rust 357·원본 356이었다. spike 목록과 값 필터 집합의 차이는 degen2 614갱신 중 0이었다(먼저 세운 가설 반증).
- 원인: Rust `Lu::update`는 행 eta 분기에서 행 r을 U 열들에서 지우고 빈 열로 LU+0xe8을 올린 **뒤** 접두 판정을 했다. 원본은 판정이 먼저라 막 빈 열을 보지 못하고, 판정이 쓴 LU+0xe8을 그 뒤 C:979–1019가 올린다. 수정: 판정 블록을 행 eta 분기 앞으로 옮겼다(코드 이동만, 식은 그대로). 스냅숏에 `e8`·`emp`(빈 U 열 위치)를 더했다(`Lu::empty_cols`).
- `PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_update_check.py --rust --self-test`(worktree 빌드) 종료 0: degen2 같은 갱신 수인 피벗 0–355 **356/356**, kb2 0–37 **38/38**에서 FTRAN 시점 (t, LU+0xec, LU+0xf0, LU+0xe8, 빈 열) = 원본. degen2는 피벗 356(Rust 갱신 40, 원본 재분해)에서 멈춘다(Q3). Rust 변조 2종(LU+0xf0 맞바꿈, LU+0xe8 + 1) 거부. 수정 후 덤프로도 갱신 0–347의 q·l·n′가 원본과 같았다.
- 판별력: 옛 순서만 되돌린 임시 빌드(되돌림 확인 `git diff --quiet crates`)는 degen2 피벗 314 `t Rust 357 original 356`에서 거부(실패 32곳).
- 02m 검사기 `m1_usolve_check --rust --self-test`: 제외 7 → **0**, degen2 K·C·O·I 238/238(인라인 R 79), kb2 7/7. `m1_lsolve_check --rust`: K 356·U 238·S 210·τ 다른 피벗 66 그대로. M1 검사기 11종(기존 10종 + `m1_update_check`) `--self-test` PASS.
- 회귀: `cargo test --release --workspace` 125 통과·11 무시, `cargo test -p rsolver-lp`(디버그 단언 포함) 42 통과(degen2는 release 확장으로만 실행했으므로 `update`의 debug_assert는 degen2에서 돌지 않았다), `cargo clippy --workspace --all-targets -- -D warnings` 0, worktree 빌드 pytest 657 passed.
- Netlib(기준선 main `b5c118a`를 `../rsolver-m1-02n-base`에 따로 빌드, `tools/lp_check.py --methods 1,0,-1 --timeout 120`, `tools/proto/pathcmp.py all`): 정답 90/91 ×3, IterCount 일치 Method 1·−1 31/64·Method 0 0/64, pathcmp 완전 일치 27/64·중앙 0.752로 기준선과 같다. **64모델의 pathcmp 첫 차이 피벗이 모두 기준선과 같다.** 첫 차이 뒤 반복 수만 9모델에서 바뀌었다(80bau3b M1 5057 → 4829, bnl1 1032 → 1025, bnl2 M0 10211 → 9814, degen3 2152 → 2169, greenbea M0 24382 → 23870, greenbeb 6450 → 6775, maros-r7 7214 → 8143, perold 1307 → 1191, pilot87 11947 → 11573. 상태·정답 변화 없음). degen2 첫 차이는 357 그대로다(Q3).
- 실행하지 못한 검사: 없음. main worktree 공유 `.venv` 확장은 갱신하지 않았다(병합 뒤 별도).
- 독립 검토(advisor, 전체 기록 열람): 수정·세 답에서 관측을 바꾸는 오류 없음. 지적 7건 반영: 디버그 단언 실행(`cargo test -p rsolver-lp`), 계약 변경 (1)이 결과를 본 뒤라는 서술, 탐색 캡처 = 기준 캡처 해시 확인, R 672(마지막 판정 제외), 교훈의 근거 표시(C), N 변조 추가, M1-02p 함수 목록은 후보로.
- 계약 변경: (1) B의 "탐색 결과 = 반환 LU+0xe8"을 "반환 LU+0xe8 ≥ 탐색 결과(같은 수 출력)"로 바꿨다. 판정 뒤 C:979–1019와 §8.10 12단계(n′ = 0)가 LU+0xe8을 올리는 것이 C에 있어 첫 판정 규칙이 틀렸다. 검사기를 쓰기 전에 탐색 캡처에서 피벗 313의 탐색 356·반환 358을 이미 보았으므로 결과를 본 뒤의 변경이다(근거는 C 줄 순서). (2) 변이 "n′·l을 값 > ZERO로"는 캡처의 ≤ ZERO 원소가 0개일 때만 "판별 불가 생존"으로 보고하고 실패로 세지 않는다(그런 원소가 생기면 변조로 판정). (3) Rust 대조는 q·l·n′를 스냅숏에 넣지 않고 FTRAN 시점 (t, LU+0xec, LU+0xf0, LU+0xe8, 빈 열)만 고정 검사로 비교한다. q·l·n′는 임시 덤프로 확인했다(위).

## 7. 판정과 인수인계

- 작업 수행 상태: **완료(조사·구현)**.
- 동등성:
  - 조사: Q1·Q2·Q3에 답했다. 접두 경계 규칙(B 656갱신)과 재분해 판정(R 672판정)이 원본 캡처와 **통과**.
  - 구현: **통과**(계약 범위: degen2 피벗 0–355, kb2 0–37의 FTRAN 시점 t·LU+0xec·LU+0xf0·LU+0xe8·빈 U 열). 02m 범위도 제외 없이 통과.
- 남은 한계: 재분해 work 조건(degen2 피벗 356부터 재분해 6번)은 Rust에 없다. 그래서 degen2 경로는 357에서 갈린다. spike 목록의 ≤ ZERO 원소 처리(n′·l)는 판별되지 않았다. primal `0b2cdf`의 같은 꼴 판정(상수 8)은 범위 밖이다. C:1040–1069 맞바꿈(02m 미검증)은 이번 검사로도 판별 여부를 따로 세지 않았다.
- 후속:
  - **M1-02p**(조사): dual simplex work 누적(`*work`에 더하는 곳, 후보: `28c93`·`8b473`·FTRAN/BTRAN 커널·`05b257`·`036ade` 분해 work)과 spx+0x150/0x158/0x3d0. 이 캡처의 판정마다 work가 있으니 분해 사이 증분을 구간별로 대조할 수 있다. 첫 명령 `python3 tools/dc.py 028c93 --grep 'param_3\|\*param'`. degen2 피벗 356 재분해가 목표다.
  - **M1-02o**(조사): τ 값(첫 피벗 242)과 반복 275 mode 0 목록.
- branch `work/m1-02n`(기준 main `b5c118a`), worktree `../rsolver-m1-02n`, 기준선 빌드 `../rsolver-m1-02n-base`(병합 뒤 정리). 미커밋 선행 변경 없음.
- 인수인계·진행: `docs/handoff/M1.md`, `docs/11_progress.md`·`docs/12_roadmap_diagram.md` 맨 위, `docs/14_sprint_plan.md` M1-02n·02p 행.
- 교훈: `docs/lessons/original.md`(FT 접두 판정은 행 r 제거 전, 재분해 work 조건), `docs/lessons/tooling.md`(입력이 같은데 출력이 다르면 함수 안 부수효과 순서).
