# M1-02y-2 pilot 정확도: 원본 비교 수단과 원인 조사 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다.

## 1. 범위와 주장

- 작업 종류: **조사.** 14의 M1-02y-2는 "pilot Method 1·−1이 Hager 값 대비 1e-6 상대 밖, 라이선스 밖이라 원본 비교 수단부터"이고 종료 조건은 "원본 비교 수단 또는 원인 특정"이다.
- 조사 질문: (1) Rust pilot 목적값 오차(−557.4842295, Hager −557.489729284 대비 9.9e-6)는 어느 단계에서 생기는가. (2) 원본은 그 끝점을 최적으로 받아들이는가. (3) 원본 자신은 pilot류 모델에서 1e-6 상대 기준을 지키는가. (4) 같은 pilot류 입력에서 Rust와 원본은 어디서 멈추는가.
- 답:
  1. 오차는 **스케일 공간 dual이 끝난 시점에 이미 있다.** Method 1은 비스케일 재검사(`2e9ae`, Rust `resolve_unscaled`)에서 primal 위반 0·dual 위반 최대 8.52e-7(< OptTol 1e-6)이라 재풀이 없이 끝난다. 끝 basis는 하한 0에 있는 열 7개(j 1162·1216·1470·1471·1601은 상한 무한, 3596은 상한 61, 3645는 상한 20.96)의 작은 음의 reduced cost(9.8e-8–8.5e-7)를 가진 허용오차 최적점이다. Method 0은 스케일 최적 −557.4833259에서 비스케일 primal 위반 1.57e-6으로 dual 2회 뒤 −557.4833257로 끝난다(오차 1.15e-5, 위반 열 10개, 최대 7.18e-7). 비스케일 재풀이는 오차 원인이 아니다.
  2. **원본의 수용은 축소 모델(스케일링)에 따라 갈린다: 8개 크기 중 4개에서 받아들인다.** pilot의 비기저 열을 Rust 값에 고정해 면허 안(열 ≤ 2000)으로 줄인 모델에서 원본을 Rust basis로 warm start하면 keep 1900–2000(4개 크기)에서 0회 반복, 목적 −557.4842294602(Rust와 같은 끝점)로 끝난다. keep 1600–1850(4개)에서는 2–3회 반복 뒤 다른 허용오차 최적점(−557.4881459, 오차 2.8e-6)에서 멈춘다.
  3. **원본 자신도 pilot류 모델에서 1e-6 기준을 대부분 넘는다.** 같은 축소 모델 8개(keep 1600–2000, 실제 최적 −557.4897146 ~ −557.4897253)를 원본이 처음부터 풀면 7개가 상대 오차 2.0e-6–7.3e-6에서 멈추고 keep 2000 하나만 3.5e-8이다(크기마다 3회 같은 값). 면허 안 Netlib 64개에서도 원본은 etamacro를 비스케일 dual 위반 6.30e-7인 끝점으로 마친다(Rust 같은 값).
  4. **같은 입력(축소 MPS 8개)에서 Rust와 원본은 IterCount가 모두 다르고 둘 다 허용오차 최적점에서 멈춘다.** 원본 오차 3.5e-8–7.3e-6(중앙 2.5e-6), Rust 1.9e-6–1.1e-5(중앙 4.2e-6). Rust가 더 부정확한 크기가 6/8이다. 원본은 파일에서 읽어도 메모리 축소 모델과 같은 반복·목적이다.
- 판정: pilot의 lp_check 실패(1e-6 상대, Hager 값)만으로는 **Rust 종료 판정이 원본과 다르다고 볼 수 없다.** Rust 끝점은 원본의 허용오차 안이고(원본이 4/8 크기에서 0회 반복으로 수용), 원본 자신도 pilot류 입력에서 1e-6을 넘는 끝점에 멈춘다. 같은 입력에서 Rust와 원본은 궤적(IterCount)이 갈리고 끝점 정확도가 다르다. 그러나 정확도 차이가 궤적에서만 오는지는 이번에 검증하지 않았다. 축소 MPS가 면허 안이라 경로 오라클로 첫 분기점을 찾을 수 있다(02t·02u의 pilot류 사례). 8개 중 6개에서 Rust가 덜 정확한 경향은 표본이 작아 판단하지 않는다. lp_check의 Hager 1e-6 기준은 수학적 정확도 기준이며 원본 동작 기준이 아니다.
- 제외·후속: 원본의 실제 pilot 끝점(면허 필요), 축소 모델과 pilot의 같은 입력 상태(원본 스케일링이 다른 행렬을 본다), 부수 발견 두 가지(§6, M1-06e).
- 선행: main `552d2a7`, 미커밋 선행 변경 없음. 솔버 코드 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 공개 API(gurobipy)만 사용, 훅 없음.
- 설정: Method 1, Presolve 0, Threads 1. 축소 모델 "tight"만 OptimalityTol = FeasibilityTol = 1e-9.
- 경계: 풀이 끝(반환된 X·RC·Slack·Pi, IterCount, ObjVal)과 warm start 입력(VBasis·CBasis).
- 원본 위치(정적 근거만): `2e9ae` 원 공간 재검사와 재풀이(Rust `simplex.rs` `solve`의 `resolve_unscaled`). 이번 작업은 이 함수의 제어 흐름을 다시 대조하지 않았다.
- 면허: size-limited(열·행 각 2000 한도). gurobipy `read`는 pilot도 읽고 `optimize`만 거부한다. 면허 검사를 우회하는 훅·패치는 쓰지 않았다.

## 3. 비교 계약

| 필드·사건 | 비교 규칙 | 방법 |
|---|---|---|
| 끝 비스케일 dual 위반(열·행 최대, 1e-9 초과 개수) | 원본·Rust 각각 값 기록. IterCount가 같은 모델은 같은 값이어야 한다(정보) | `tools/m1_end_dualvio.py` |
| 원본의 Rust 끝 basis 수용 | warm start 반복 0이면 수용 | `bench/oracle/m1_02y2/reduced_pilot.py` |
| 원본 자체 끝점 정확도 | 축소 모델 tight 최적 대비 `(obj − z)/|z|` | 같은 도구 |
| 같은 입력의 Rust·원본 끝점 | IterCount 정확 비교, 오차는 기록 | `bench/oracle/m1_02y2/reduced_cmp.py` |

- 끝 dual 위반 정의: 열이 경계에 있으면(|x − b| ≤ 1e-9·max(1,|b|)) RC의 틀린 부호 부분, 경계 사이면 |RC|. 행은 |slack| ≤ 1e-6이면 Pi의 틀린 부호 부분, 아니면 |Pi|. 최소화 기준(ModelSense 곱).
- 계약 순서: 질문은 착수 때 정했으나 이 문서의 비교 규칙은 탐색 실행 뒤에 적었다(§6 도구 수정 두 번 포함).

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 것 | 자료 |
|---|---|---|---|
| Netlib 면허 안 64개 | 실제 | 원본 끝점의 dual 위반 분포 | `bench/oracle/m1_02y2/gurobi_m1.jsonl`(SHA-256 `0ac6e8e4…`), `rust_m1.jsonl`(`2a4493ab…`, pilot 포함 65) |
| pilot 축소 9개(keep 1500–2000) | 합성(pilot에서 비기저 열 고정) | 원본의 Rust 끝 basis 수용, 원본 자체 정확도 | `reduced_sweep.jsonl`(크기별 실행 수·동일 여부), 입력 `basis.json`(`ac8e182c…`)·`rust_x_rc.json`(`e8d52b40…`) |
| 축소 MPS 8개(keep 1600–2000) | 합성 | 같은 입력의 Rust·원본 | `reduced_cmp.jsonl`. MPS는 `reduced_pilot.py --write`로 다시 만든다(커밋하지 않음) |

- keep 1500은 고정한 열이 최적에 영향을 줘(tight 최적 −557.4842293) 정확도 비교에서 뺀다.
- 축소 방법: Rust 끝 basis의 기저 열은 모두 두고, 비기저 열 중 |RC|가 작은 쪽부터 남긴다. 버린 열은 Rust 값으로 고정해 rhs·목적 상수로 옮긴다. Rust basis는 축소 모델의 basis이고 primal 가능하다. 남긴 비기저 열에 Rust dual 위반 열 7개가 모두 들어간다(도구의 `rust_dual_violating_kept`는 basis 상태와 RC만 보므로 고정 열 j 223을 셀 수 있으나 그 열은 |RC| = 0.018이라 버려진다).

## 5. 재실행 명령

```
# Rust 끝 basis(임시 탐침, 커밋하지 않음): 패치를 적용해 격리 빌드
git apply bench/oracle/m1_02y2/tmp_probe.patch && tools/build_local.sh
RSOLVER_TMP02Y2_BASIS=bench/oracle/m1_02y2/basis.json PYTHONPATH=$PWD/python tools/gurun.sh ../.venv/bin/python bench/oracle/m1_02y2/rust_pilot_end.py
git apply -R bench/oracle/m1_02y2/tmp_probe.patch && tools/build_local.sh
# 원본: 축소 pilot(cold·warm·tight), keep 크기별
for k in 2000 1990 1950 1900 1850 1800 1750 1600 1500; do tools/gurun.sh ../.venv/bin/python bench/oracle/m1_02y2/reduced_pilot.py --keep $k; done
# 같은 입력의 Rust·원본(z는 각 크기의 tight 최적)
for k in 2000 1990 1950 1900 1850 1800 1750 1600; do tools/gurun.sh ../.venv/bin/python bench/oracle/m1_02y2/reduced_pilot.py --keep $k --write /tmp/pilot_k$k.mps; done
PYTHONPATH=$PWD/python tools/gurun.sh ../.venv/bin/python bench/oracle/m1_02y2/reduced_cmp.py /tmp/pilot_k2000.mps=-557.4897252834571 ...
# 끝 dual 위반 64모델
tools/gurun.sh ../.venv/bin/python tools/m1_end_dualvio.py gurobi --out bench/oracle/m1_02y2/gurobi_m1.jsonl <64개 이름>
PYTHONPATH=$PWD/python tools/gurun.sh ../.venv/bin/python tools/m1_end_dualvio.py rust --out bench/oracle/m1_02y2/rust_m1.jsonl <64개 이름> pilot
../.venv/bin/python bench/oracle/m1_02y2/summary.py
../.venv/bin/python tools/m1_end_dualvio.py --self-test
```

탐침 패치는 `RSOLVER_TMP02Y2`(스케일 최적·재검사 진입·재풀이 뒤 목적과 dual 위반 열 출력)와 `RSOLVER_TMP02Y2_BASIS`(끝 basis 파일)를 더한다.

## 6. 실행 결과

- commit: 작업 branch `work/m1-02y-2`(기준 main `552d2a7`). Rust는 worktree 격리 빌드.
- Rust 국소화(탐침): Method 1 `scaled-opt obj −557.484229460191 iter 4084`, 재검사 진입 `pinf false pmax 0 dinf false dmax 8.52e-7` → 재풀이 없음. Method 0 `scaled-opt −557.483325858178 iter 32809`, `pinf true pmax 1.57e-6 dinf false dmax 7.18e-7` → `resolved −557.483325684898 iter 32811`, dual 위반 열 10개(최대 7.18e-7, j 1601).
- 64모델 끝 dual 위반(`summary.py`): 원본 1e-9 초과 3개(etamacro 6.30e-7, scsd1 3.27e-8, scsd6 2.50e-8), Rust 4개(pilot 8.52e-7과 같은 셋). IterCount 같은 36모델에서 목적·위반이 비트 같음 28, 1e-12 안 36/36(비트가 다른 8개는 위반 1e-13 이하이거나 목적 마지막 자리 차이).
- 축소 pilot(원본, 크기마다 3회 같은 값, keep 1500은 2회):

| keep | tight 최적 | cold 반복·오차 | warm(Rust basis) 반복·오차 |
|---|---|---|---|
| 2000 | −557.489725283 | 2292 · 3.5e-8 | 0 · 9.9e-6 |
| 1990 | −557.489725283 | 2369 · 2.0e-6 | 0 · 9.9e-6 |
| 1950 | −557.489725283 | 2225 · 7.3e-6 | 0 · 9.9e-6 |
| 1900 | −557.489725283 | 2145 · 2.0e-6 | 0 · 9.9e-6 |
| 1850 | −557.489725283 | 2107 · 2.5e-6 | 2 · 2.8e-6 |
| 1800 | −557.489725283 | 2040 · 4.0e-6 | 2 · 2.8e-6 |
| 1750 | −557.489725284 | 1981 · 2.4e-6 | 3 · 2.8e-6 |
| 1600 | −557.489714637 | 1607 · 6.3e-6 | 2 · 2.8e-6 |

- 같은 입력(축소 MPS, `reduced_cmp.jsonl`):

| keep | 원본 반복 · 오차 | Rust 반복 · 오차 |
|---|---|---|
| 2000 | 2292 · 3.5e-8 | 2549 · 2.4e-6 |
| 1990 | 2369 · 2.0e-6 | 2356 · 4.7e-6 |
| 1950 | 2225 · 7.3e-6 | 2185 · 1.1e-5 |
| 1900 | 2145 · 2.0e-6 | 2341 · 4.3e-6 |
| 1850 | 2107 · 2.5e-6 | 2195 · 1.9e-6 |
| 1800 | 2040 · 4.0e-6 | 1958 · 4.2e-6 |
| 1750 | 1981 · 2.4e-6 | 2058 · 4.1e-6 |
| 1600 | 1607 · 6.3e-6 | 1630 · 4.2e-6 |

- 부수 발견(범위 밖, 고치지 않음):
  - **Rust API의 `>` 행 Pi 부호가 Gurobi와 반대다.** 기본 경로(`optimize`, Presolve 기본)에서 pilot4 `>` 행 93/93, adlittle 1/1이 부호만 반대이고 `<`·`=` 행은 같다. `driver` `attrs`가 "`>` 행이 이미 부호 처리된" 입력을 가정하는 경로로 보인다(미확인). → **M1-06e**. 도구는 Rust `>` 행 Pi를 뒤집어 읽는다(`shortcut:` 주석).
  - Rust Slack은 비기저 행에서 1e-12–1e-8 반올림이 남는다(원본 0). 도구 판정만 영향(처음 절대 1e-9 판정이 pilot4·perold·maros에서 오탐 → |slack| ≤ 1e-6으로 바꿈).
- 실행하지 못한 것: 원본 pilot 실행(면허), `2e9ae` 제어 흐름 재대조(이번 질문에는 불필요: 오차가 재검사 전에 있다).

## 7. 판정과 인수인계

- 작업 수행 상태: 완료(조사).
- 결론: pilot 오차 원인은 허용오차 최적 끝점(스케일 dual 종료 시 비스케일 dual 위반 8.5e-7, 열 7개)이다. 원본은 그 끝점을 축소 모델 4/8 크기에서 0회 반복으로 받아들이고, 원본 자신도 pilot류 모델 8개 중 7개에서 1e-6 상대 기준을 넘는다. 따라서 lp_check pilot 실패는 원본 종료 규칙과의 차이를 증명하지 않는다. 같은 입력에서 Rust와 원본의 궤적이 갈리며(8/8), 정확도 차이가 궤적에서만 오는지는 미검증이다. 원본의 pilot 실제 끝점도 미검증(면허)이다. 비교 수단: 축소 MPS(면허 안)로 원본·Rust를 같은 입력에서 돌릴 수 있다.
- 제안(사용자 결정): lp_check의 면허 밖 모델 판정을 "Hager 값 1e-6"에서 "원본처럼 허용오차 최적(비스케일 primal ≤ FeasTol, dual ≤ OptTol)이면 통과, Hager 값과의 차이는 기록"으로 바꾸는 것. 이번 작업에서는 바꾸지 않았다.
- 후속: 계획대로 M1-02t → M1-02u(축소 pilot MPS를 경로 비교 사례로 쓸 수 있음). 새 미착수 항목 M1-06e(Pi 부호).
- 교훈: `docs/lessons/original.md`(원본도 허용오차 최적점에서 멈춤), `docs/lessons/tooling.md`(면허 밖 모델의 축소 warm start 수단).
