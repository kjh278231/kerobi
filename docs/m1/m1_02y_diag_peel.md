# M1-02y BTRAN e_r 출력 기존 차이: FT 새 대각의 η 버퍼 정렬 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 기준 main `f45157e`(M1-02x 병합), 미커밋 선행 변경 없음(main의 미추적 `.env`는 사용·변경하지 않음). 작업자는 이 세션(Claude Code, "다음스프린트 작업해 너가 작업자야" 지시. M2-06h-5는 다른 세션이 먼저 점유해 사용자 결정으로 양보했다). branch `work/m1-02y`, worktree `../rsolver-m1-02y`.

## 1. 범위와 주장

- 작업 종류: 조사(M1-02x가 고정한 e_r 출력 기존 차이의 원인) + 구현(stair 원인).
- 조사 질문과 답(착수 조사, §4 근거):
  - stair e_r 31부터 104호출 값 차이: 첫 차이는 Uᵀ 단계 출력 idx 252 하나(1 ulp)이고, U 행·순서는 원본과 전부 같고 대각 LU+0x50[252]만 1 ulp 다르다. 그 대각은 FT 갱신(`05b257` C:321–372)의 새 대각 η·S다. 원본은 η 값 배열(LU+0x1d0 → +0x10)이 8 mod 16이면 첫 항을 lane 0에 먼저 더하고(9항 미만이면 전부 순차) Rust `dot8`은 16 정렬을 가정했다(M1-02l `shortcut:`).
  - bnl1 e_r 130·134·144·153·159·162·173 순서 차이: e_r 126 직전 재분해 직후부터 U 행·대각의 위치 번호가 원본과 다르다(위치 588–607, 대각 20·행 32). 값 계산이 아니라 재분해 번호 문제라 이번 구현 범위 밖이다 → **M1-02z**(M1-02w와 같은 재분해 영역).
- 완료할 동작(구현): `Lu::update`의 새 대각 내적에서 원본 η 버퍼 정렬을 따른다. 그 버퍼는 LU 할당 `045a76` C:274–296이 작업 공간 W(LU+0x178 → +0x10, `ac645d` = malloc) 안에 `W + 24·I + 8 + 8·(m + n) + 4·m`(idx)와 그 뒤 `+ 4·m`(값)으로 잘라 만든다. I = spx+0x3c8(할당 때 LUINTERVAL), n = spx+0x68. W가 16 정렬이면 값 배열 ≡ 8·(3I + 1 + n) mod 16, 즉 **I + n이 짝수이면 peel**이다. Rust는 `Engine::new`의 첫 `lu_interval`(= max(20, ⌊m^0.7⌋), 뒤에 Full pricing에서 낮아지기 전)과 `spx.n`으로 한 번 정한다.
- 포함 분기: peel 있음(I + n 짝수)·없음(홀수), 9항 미만 순차, 8항 이상 lane 합.
- 제외와 후속: bnl1 재분해 번호(M1-02z), LU+0x108 재분해 차이(M1-02w), LU 풀이 work(M1-02t), Method 0·quad·Threads > 1·MIP 호출자, malloc이 16 정렬을 주지 않는 환경(glibc x86-64 전제), 풀 기준 LU+0x150 정렬(이번 캡처에서 관측만, 기존 `reta_odd` 가정 유지).
- 선행: M1-02x(`tools/m1_lt_check.py`, `bench/oracle/m1_02x`), M1-02l(`dot8_peel`, `reta_odd`).

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- 새 대각: `../re/decomp/c/000f15c0_PRIVATE000000000005b257.c` C:321–372(값 포인터 `local_1a0` & 0xf ≠ 0이면 & 7 = 0일 때 1항 선처리, 아니면 순차 경로; `7 < n`·`peel + 8 ≤ n`일 때만 8레인). 결과는 LU+0x50[r], r = spx+0x2f8(진입 때).
- η 버퍼 할당: `../re/decomp/c/000e3130_PRIVATE0000000000045a76.c` C:36–56(m = spx+100, n = spx+0x68, I = spx+0x3c8, LU+0x188 = I), C:269–296(LU+0x178 작업 공간 크기 max(16m + 8 + lVar6, 48m + 1), lVar6 = 24I + 8 + 8(m+n); LU+0x1d0 idx = W + lVar6 + 4m, 값 = W + ((lVar6 + 8m + 7) & ~7)). `ac645d`(0x8d1880)는 사용자 할당 콜백이 없으면 malloc(size + 8)을 그대로 돌려준다.
- 시작·종료 경계: `05b257` 진입(0xf15c0, rdi = spx, LU = spx+0x470)·반환. 진입 때 LU+0x1d0 {nnz, idx, 값} 포인터, W, spx+0x64/0x68/0x3c8, LU+0x188, LU+0x150, r. 반환 때 반환값과 LU+0x50[r] 비트.
- 설정: Method=1, Presolve=0, Threads=1(M1-02x와 같음).

## 3. 비교 계약

| 필드·사건 | 규칙 | 방법 |
|---|---|---|
| 원본 3회 | 사건 전체 JSON 동일 | 독립 프로세스 3회 |
| 할당식 | 모든 갱신에서 idx − W = 24I + 8 + 8(m+n) + 4m, 값 − idx = 4m, W ≡ 0, LU+0x150 ≡ 0 (mod 16), 포인터 불변 | 캡처 |
| 정렬 규칙 | 값 mod 16 = 8 ⇔ (I + n) 짝수, 두 쪽 모두 1모델 이상 | 캡처 |
| Rust 입력 | Rust (I, n)가 원본 (첫 LU+0x188 = spx+0x3c8, spx+0x68)와 같고 peel 판정이 원본 정렬과 같음 | 탐침 |
| Rust 새 대각 | 같은 상태 범위(갱신 순서의 r이 같은 구간)에서 갱신마다 (r, LU+0x50[r] 비트)가 원본과 같음. 범위 안에 peel 여부로 값이 갈리는 갱신(대상) 1회 이상 | 탐침 |
| e_r 출력 | M1-02x `diffs.py`에서 stair 값 차이 목록이 사라지거나 줄고, 남는 첫 차이는 새로 국소화(다시 고정하지 않음) | `m1_lt_check.py --rust` |

- 비트 일치, 오차 없음. 시간·work 비교 없음.
- 같은 상태 범위는 각 모델에서 갱신 r이 처음 달라지는 곳까지이며 모델별로 고정한다(짧아지면 실패).
- 변이: peel 안 함, 항상 peel, peel인데 9항 미만 순차 경로 제거. 각각 원본 비교가 거부해야 한다.
- 회귀 규칙(착수 때 고정): I + n이 홀수인 Netlib 모델은 `pathcmp`·`lp_check`의 시간 밖 출력이 base와 같아야 한다. 짝수 모델은 바뀐 목록을 남기고 `pathcmp` 첫 차이가 앞당겨지지 않아야 한다. `lp_check` 정답 수가 줄지 않는다.

## 4. 사례와 원본 자료

| 사례 | 실제/합성 | 분기 | 근거 |
|---|---|---|---|
| stair(m 356, n 467, I 61) | 실제 | peel, 대상 갱신 다수 | 탐색: 갱신 #13(e_r 28 뒤) r 252 원본 `bfeeb36746e614e2` = peel 값, Rust `…14e3` |
| sc50a·scsd1·afiro | 실제 | peel(작은 모델) | 탐색 정렬 8 |
| kb2·degen2 | 실제 | peel 없음 | 탐색 정렬 0(kb2는 M1-02l 기준) |

착수 조사(탐색 자료 `bench/oracle/m1_02y/explore/`, 각 1회, 기준 자료 아님):

- `tools/m1_er_stage.js`(M1-02x 훅 + 0x12ec77에서 param_3 = Uᵀ 출력, param_2 = Lᵀ 입력 목록): stair e_r 31 Uᵀ 출력 13개 중 idx 252만 원본 `…0daf`, Rust `…0dae`. 행 eta·Lᵀ는 그 값을 그대로 옮긴다.
- `tools/m1_urow_dump.js`(e_r k번째 호출의 `21a941` 진입에서 행 방향 U 전체와 대각): stair e_r 31에서 Rust와 U 행 356개(순서 포함)가 같고 대각은 252 하나만 다르다. 원본 계산은 나눗셈이다(`21a941` C:80·159·247·340·382).
- Rust 갱신별 계산(임시 디버그, 커밋 안 함): 그 대각은 갱신 #13의 η·S(η 11항)이고 `dot8_peel(…, true)`가 원본 값을 낸다. e_r 60까지 peel 여부로 갈리는 갱신 6개(#13·#16·#6·#10·#16·#23, 리팩터 단위 번호).
- `tools/m1_eta_align.js` 14모델: 할당식 14/14, 정렬 규칙 14/14(8: 25fv47·afiro·bandm·maros·sc50a·scsd1·stair, 0: adlittle·bnl1·degen2·fit1p·israel·kb2·share1b). 포인터는 실행 내내 하나.
- `tools/m1_eta_alloc.js`: η 배열은 `045a76`(LU 할당)의 `ac645d` 블록(48m + 9바이트 = 48m + 1 + 8) 안에 있다.
- bnl1: e_r 130 Uᵀ 출력 집합부터 다르다. e_r 126(재분해 직후)·129·130 U 덤프가 모두 위치 588–607에서 번호가 어긋난다(예: 행 527 열 602 ↔ Rust 600).

기준 자료: `bench/oracle/m1_02y/<모델>_{1,2,3}.json.gz`(§5 캡처), manifest `bench/oracle/m1_02y/manifest.json`.

## 5. 재실행 명령

- 캡처: `tools/hrun.sh --timeout 580000 -- 'tools/m1_eta_align_runs.sh bench/oracle/m1_02y'`
- 비교: `tools/build_local.sh && PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_diag_peel_check.py --self-test`
- e_r 출력: `PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_lt_check.py --rust`, `PYTHONPATH=$PWD/python ../.venv/bin/python bench/oracle/m1_02x/diffs.py`
- 회귀: `bench/oracle/m1_02x/regress.sh <base|final>` 방식(cargo lp·workspace·clippy·pytest·M1 검사기·pathcmp·lp_check).

## 6. 실행 결과

- commit: 착수·탐색 `3f6be46`, 구현·검사기·기준 자료 `7e1b029`, 최종 회귀·clippy `063e424`, main `369a35a`(M2-06h-5, `crates/mip`·문서만) 병합 `b830231`. 자료 해시 `bench/oracle/m1_02y/manifest.json`. x86-64 Linux(WSL2), glibc malloc.
- 원본(Gurobi 18회, 6모델 × 3): 사건이 3회 모두 같다(두 힙 블록 사이 거리 `val-struct`만 기록용으로 비교에서 뺐다). 할당식·정렬 규칙이 모든 갱신에서 성립하고 LU+0x150도 16 정렬이다. peel: stair·sc50a·scsd1·afiro, 없음: kb2·degen2. 탐색 14모델(§4)과 합치면 할당식 14/14. 첫 기준 캡처는 r < 0 호출(풀이 끝 `05b257`)에서 LU+0x50 밖을 읽어 kb2 run 2가 달랐다. r ≥ 0일 때만 읽도록 고치고 18회를 다시 캡처했다.
- Rust(`tools/m1_diag_peel_check.py --self-test`, [check.txt](../../bench/oracle/m1_02y/after_main_merge/diag_peel.txt)): 6모델 모두 Rust peel·n이 원본과 같다. 갱신 정렬(원본의 첫 분해 전 호출, r < 0 호출, 재분해 직전 원본만 하는 갱신 제외)에서 (r, 새 대각 비트)가 stair 179/347, sc50a 30/30, scsd1 37/80, afiro 9/9, kb2 35/35, degen2 348/621 갱신까지 같다. 끝나는 곳: stair는 e_r 186(M1-02x/M1-02w의 고정 LU+0x108 재분해 e_r 185 바로 뒤)에서 r 45는 같고 대각이 원본 `bfe1aa84d82ff92a`·Rust `bfe1aa84d82ff93f`로 다르다(M1-02w에 넘기는 증상), degen2는 e_r 356(M1-02x 범위 끝), scsd1은 e_r 43에서 r이 다르고 `pathcmp`도 피벗 41에서 진입 열이 갈린다(497 ↔ 427, 변경 전과 같음). 변조 13종 거부. 계약과 다른 점: "범위 안 대상 갱신"은 검사기가 세지 않고 변이(stair 갱신 27)로만 보였다. I는 값이 아니라 peel 플래그의 패리티로 비교한다.
- e_r 출력(`bench/oracle/m1_02x/diffs.py`): stair 값 차이 104호출이 **모두 사라졌다**(같은 상태 범위 185호출 전부 형태·idx·값 비트 일치). bnl1 7호출(순서)은 그대로다(I + n 홀수라 이번 변경과 무관, §1). `tools/m1_lt_check.py`의 stair `KNOWN_VAL`을 비우고 `--self-test` 통과([lt.txt](../../bench/oracle/m1_02y/after_main_merge/lt.txt)). 변조 'pinned value call agrees'는 bnl1 고정 호출로 옮겼다.
- 변이(`bench/oracle/m1_02y/mutate.py`, [mutate.txt](../../bench/oracle/m1_02y/mutate.txt)) 5/5 거부: peel 안 함·항상 peel·패리티 반전·Full pricing 뒤 interval 사용(degen2에서 갈림), 그리고 플래그는 맞게 보고하면서 대각에서 무시(stair 갱신 27, r 252 `…14e2` ≠ `…14e3`에서 값 비교로 거부). 착수 때 적은 "9항 미만 순차 경로 제거" 변이는 넣지 않았다. 1항을 선처리하고 8항 이하이면 8레인 블록이 없어 순차 합과 같으므로 판별할 수 없는 등가 변이다.
- 회귀(`bench/oracle/m1_02y/final`, base = `bench/oracle/m1_02x/final`: 그 commit `2a4cd71`과 main `f45157e`의 LP 코드가 같다): cargo `-p rsolver-lp` 통과, `--release --workspace` 통과, clippy는 첫 실행에서 `manual_is_multiple_of` 1건 → 고쳐 0, pytest 657 통과, M1 검사기 16종 통과. main 병합 뒤(`after_main_merge/`) workspace 129 통과·0 실패·12 무시, clippy 0, `m1_diag_peel`·`m1_lt`·`m1_btran` self-test 통과.
  - `pathcmp.py all`: 바뀐 모델 8개(25fv47·fffff800·maros·perold·scagr25·scfxm2·scfxm3·stair), 첫 차이가 앞당겨진 모델 없음. fffff800은 첫 차이 181 → 291로 늦어지고 피벗 수가 Gurobi와 같아졌다(490). full match 27/64·중앙 0.752 그대로.
  - `lp_check` 273행 중 54행 변화(시간 열 제외), 바뀐 모델 23개는 **모두 I + n 짝수**(홀수 모델 변화 0, 규칙 충족). **정답 수: Method 1·−1이 91 → 90으로 줄었다(규칙 실패).** pilot이 −557.489561 → −557.484224(Hager 값 −557.489729, 1e-6 상대 기준 밖)로 바뀌었다. IterCount 일치는 Method 1 31 → 32/64.
- pilot 조사(사용자 지시로 병합 전 실행, 임시 디버그는 커밋하지 않음): pilot(1441 × 3652, I = 162, I + n 짝수 → peel)은 라이선스 크기 밖이라 원본 실행·경로 오라클이 없다. Rust 최종 해는 원래 공간에서 행 위반 5e-10, bound 위반 5.6e-7이다. 스케일 공간 최적(반복 4507)을 비스케일 공간에서 다시 보면 primal 위반 1.5e-6 > FeasTol이라 dual 1회 뒤 최적으로 끝난다. **반환한 basis**를 재면 primal 위반 0, dual infeasibility 최대 7.18e-7(< OptTol 1e-6), 1e-9 초과 8열이고 그중 7열은 위 bound가 무한(1e30)이다. Rust pilot 네 실행(Method 1·0 × 변경 전·후)은 −557.482047 ~ −557.489561에 퍼져 있고 넷 중 셋이 1e-6 상대 기준에서 실패한다. 이번 변화는 peel 연산이 원본과 달라서 생긴 것이 아니다(peel은 원본 대조 통과). 다만 원본이 pilot에서 어디서 멈추는지 확인할 수 없으므로 실패의 메커니즘은 미검증이다 → **M1-02y-2**.
- Method 0(탐색, `explore/{stair,kb2}_align_m0.json.gz` 각 1회): 원본 primal도 같은 I(61·20)·n으로 할당식과 정렬(stair 8, kb2 0)이 맞는다.
- 실행하지 못한 것: pilot 원본 실행(라이선스), bnl1 재분해 번호(M1-02z), 같은 상태 범위 밖.

## 7. 판정과 인수인계

- 작업 수행: 완료(조사 + 구현). 동등성: **새 대각 peel 판정·값은 계약 범위 통과**(6모델 갱신별 비트 일치, stair e_r 값 차이 104호출 해소, 변이 5/5 거부). bnl1 순서 차이는 원인(재분해 위치 번호)만 특정하고 **미해결 → M1-02z**. 회귀: 홀수 패리티 모델 무변화·pathcmp 규칙은 통과, **lp_check 정답 수 규칙은 pilot 한 모델에서 실패**(원본 비교 불가, 메커니즘 미검증, §6). 고친 연산 자체는 원본과 같다. pilot 조사 결과를 보고 사용자가 병합을 결정했다.
- 남은 한계: malloc 16 정렬 전제(glibc x86-64, 사용자 할당 콜백 없음). 비스케일 재풀이(`resolve_unscaled`의 새 `Engine`)도 같은 I·n으로 peel을 정하는데, 원본이 그 재풀이에서 LU를 새로 할당하는지는 확인하지 않았다. Method 0은 정렬 규칙만 2모델 탐색으로 확인했다. quad·Threads > 1·MIP 호출자·라이선스 밖 모델은 원본 미확인.
- 후속: **M1-02z**(bnl1 e_r 126 직전 재분해의 U 위치 번호 588–607, 첫 명령 `tools/gurun.sh node tools/m1_urow_dump.js ../re/bench/data/netlib/bnl1.mps.gz <out> 1 126`와 Rust U 덤프 대조), M1-02w(stair 갱신 179 대각 증상 포함), **M1-02y-2**(pilot 정확도: 원본 비교 수단부터), M1-02t, M1-02u.
- branch `work/m1-02y`, worktree `../rsolver-m1-02y`.
- 교훈: `docs/lessons/original.md`(η 버퍼 할당식과 패리티 규칙, 재분해 번호 차이), `docs/lessons/tooling.md`(r < 0 호출에서 배열 밖 읽기, 등가 변이).
