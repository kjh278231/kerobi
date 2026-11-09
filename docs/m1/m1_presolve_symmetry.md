# M1 LP presolve: 대칭 검출 `5577ba`/`551cc0` 해독 노트

[m1_presolve.md](m1_presolve.md) §7 패스 0의 대칭 검출을 해독한 노트다. 2026-11-05에 의사코드와 asm으로 해독하고, 오라클로 비트 단위 확인했다. 줄 번호는 `re/decomp/c/004a0900_PRIVATE0000000000551cc0.c`(`551cc0`)와 `0049fda0_…5577ba.c`(`5577ba`) 기준이다. 코드는 `crates/lp/src/presolve/symmetry.rs`, 호출부는 `mod.rs`.

## 0. 범위와 호출 (`390989` L962–990)

```
if k == 0 && local_38 == 0 && PRESYMMPASSES(env+0x2c28) < 0:
    nodes = 5, limit = state+0x1f0 / 5 + 1e7
    if n − (+0x408) < 500 || m − (+0x404) < 100 || +0x330 || +0x258 || +0x27c: 건너뜀
    else:
        if Symmetry(env+0x2864) > 1: nodes = 5000, limit *= 10
        r = 5577ba(1.1, limit, env, state, nodes, 0, &f4)
        if r == 0 && f4 > 0 && PreDepRow == −1: local_808 = 0     // 47c445 두 모드 모두 끔
        if r != 0: 상태 코드로 끝
```

- 호출 횟수(PRELIMIT)에 세지 않는다. `+0x330`, `+0x258`(Q 비대각), `+0x27c`(QC 수)는 LP에서 0이고, `local_38`(축약 모드)도 꺼져 있다.
- state+0x1f0은 presolve 작업 예산이다(`390989` L212–244). 값은 nnz(상태 생성 직후 +0xd8)·k + 1e7이다. k는 Presolve 1이면 200, Presolve 2이거나 PrePasses > 1이면 1e100(무한), 그 밖에는 1000이다. 오라클로 확인했다(예: 25fv47 2.04e7, 한도 1.408e7).
- `Model.presolve()`에서는 Symmetry=0이어도 `5577ba`가 불린다(오라클로 확인). PRESYMMPASSES를 0으로 바꾸는 "Symmetry==0 → 0" 해석은 최적화 드라이버(노트 11)에만 있다. 그래서 포트 조건은 PRESYMMPASSES < 0 하나다. `Params`에 `symmetry`(−1)와 `presym_passes`(−1)를 더했다.
- `5577ba`의 6번째 인자가 0이라, 대칭을 쓰는 블록(`3a0a17` type 19, `564ce0`, `397538`)은 돌지 않는다. 결과는 반환값과 local_f4뿐이다.

## 1. `5577ba` LP 경로 (state+0xe0 = 1): 열 CSR 압축

`551cc0`을 부르기 전에 열 CSR(+0x40 시작, +0x48 길이, +0x50 행 번호, +0x58 값)을 제자리에서 압축한다. 행 CSR은 건드리지 않는다. 이 상태 변화는 뒤에도 남는다. 예를 들어 ganges는 `5b488d` 직후 살아 있는 열의 슬롯 합이 3943인데, 47c445 진입 때는 3897(= col_cnt 합)이다.

```
p = 0
for j in 0..n:
    old, len, cnt = +0x40[j], +0x48[j], +0x68[j];  +0x40[j] = p
    if cnt > 0:
        if cnt == len: p > old일 리 없음; p < old면 memmove, work += (len + len)·mult;  p += len
        else: 슬롯마다 행 ≥ 0이고 row_cnt[행] ≥ 0이면 남김;  work += mult·4·len
    +0x48[j] = p − 시작                     // 제거·빈 열은 0 (포트에서 −1이던 값도 0이 된다)
work += mult·3·n;  +0x40[n] = p
```

mult는 +0x448(nnz ≤ 1e5면 1)이다. 그 뒤 `551cc0(1.1, limit, env, state, n, m, +0x40, 0, +0x50, +0x58, obj, +0x250(Q, NULL), lb, ub, rhs, sense, +0x68, +0x60, +0x150(타입, NULL), +0x170, 0, nodes, 0, &out1, NULL, NULL, &out2, NULL, NULL, &f4, +0x458)`을 부른다.

## 2. `551cc0` LP 경로

인자 해석: param_1 = 1.1(L그룹 비율), param_2 = 작업 한도, param_8 = 0(열 길이 배열 없음, 시작 n+1개로 읽음), param_12/19 = NULL(Q, 변수 타입. 모두 `C`), param_21 = 0(rhs·obj를 2^−10로 반올림하는 블록 꺼짐), param_23 = 0(라운드 중 `54d420` 끔).

한쪽(행 또는 열)의 분할은 셀 번호(`cell`), 셀 시작 위치(`start`), 셀 순으로 늘어선 원소(`perm`), 셀 수, 단일 셀 수, 큐(`ac3c8b`)로 이뤄진다. 큐는 id 목록과 id별 위치를 가진다. 위치는 −1이면 큐에 없음, −2이면 고정(단일 셀, 다시 넣지 않음)이다. 넣기는 `ac3f7f`(−1일 때만), 꺼내기는 `ac4062`(끝에서, 위치 −1로 되돌림), 고정은 `ac40d4`(−1일 때만 −2), 고정 확인은 `ac41fc`다. 큐를 만들 때 work += 크기다.

| 단계 | 내용 | 작업량 |
|---|---|---|
| 큐 | 열 큐(0..n), 행 큐(0..m) | n, m |
| 행 분할 | 1차 `=` 행, 2차 나머지 살아 있는 행. 각 차수의 행이 있으면 셀 하나(1개면 고정). 제거된 행은 2차에 `perm` 끝에서부터(역순) 넣고, 뒤에 하나씩 단일 셀(고정) | 차수마다 3m |
| `549df5`(행, rhs) | 살아 있는 행 위치에서 셀을 rhs로 쪼갬(§3) | §3 |
| 끝 판정 | (행 셀 수 − 제거 행) × 1.1 ≥ 살아 있는 행이면 끝(f4 0) | |
| 열 분할 | 살아 있는 열(+0x170 = 0)을 셀 0에. +0x170 ≠ 0 열과 제거된 열은 끝에서부터(역순) 단일 셀(고정). 살아 있는 열 ≤ 1이면 끝 | 4n |
| `549df5`(obj), lb, ub | 같은 함수(lb·ub는 인라인 사본). 각각 뒤에 열 쪽 1.1 판정 | §3 |
| `790e3` | 열 CSR → 행 CSR(행 안은 열 순서) | 열마다 2·len, +n, 열마다 5·len, +3n |
| `54ad3b` | 계수 정수 키(§3) | §3 |
| 난수 해시 정제 | 아래 | |

1e7 작업량마다(§4) 판정을 하는 자리는 행 1.1 판정 뒤, ub 판정 뒤, 전치 앞뒤, 키 뒤, 루프 A 해시 뒤, 루프 B의 열·행 정제 앞, 끝 판정 앞이다.

### 2.1 해시 정제

```
rng = 0x1328f02; rand = max(n, m)칸; round = 0; stable = 0; retries = 2
loop A:
    rand[i] = (a7b4d6() << 32) + a7b4d6()   (i < max(n, m));  work += max(n, m)
    if retries == 0 && (어느 쪽이든 현재 줄이 사본): 둘 다 원본으로 돌리고 54ad3b로 키를 다시 만듦
    행 해시[i] = 행 셀[i] + Σ rand[열 셀[j]]·행 키[e] (현재 행 줄, i64 wrap);  행마다 work += 4·len, 끝에 +m
    열 해시도 같은 식으로 계산;  +n
    모든 열 셀 id(0..), 모든 행 셀 id를 큐에 넣음
    제거 열 < 열 단일 셀이면: 고정된 셀의 열마다 local_178 += 현재 열 길이 (행도 같은 식으로 local_170)
    if round < nodes:                                   // 루프 B, r = round부터
        54e0c4(열 줄, 행 셀·큐, &local_170);  열 큐가 빌 때까지 꺼내 셀을 훑고(work += 3·크기) 해시가 다르면 54e5eb
        열 셀 1.1 판정에 걸리면: round = r, stable이 아니면 끝(f4 0), 맞으면 아래 검증으로
        54e0c4(행 줄, 열 셀·큐, &local_178);  행 큐도 같은 식으로 정제;  행 1.1 판정도 같음
        셀 수가 라운드 시작 때와 같으면 stable = 1, round = r, 검증으로
        adf2be(작업 한도)에 걸리면 round = nodes, 검증으로
        r += 1;  r ≥ nodes면 round = r, 검증으로
    검증: 열 셀 수 == n이면 끝(f4 0)
          f4 = 54d420(...)                              // 1, −1, 0
          f4 == 0: stable이 아니거나 round ≥ nodes − 1이거나 retries == 0이면 f4 = −2로 끝
                   아니면 retries −= 1, round += 1, loop A (새 난수, 셀은 그대로)
          f4 == 1이고 열 셀 < (n − 제거 열)/5, 행 셀 < 살아 있는 행/5이면 f4 = 2;  끝
```

- `stable`(cVar74, 스택 사본 local_c0)은 한 번 1이 되면 0으로 돌아가지 않는다. 첫 루프 A에서 1.1 판정에 걸리면 바로 f4 0으로 끝나고, 재시도 뒤에 걸리면 검증으로 간다.
- 정수 열이 있을 때만(local_a8) 정수 열 정제, `5507cb`(탐색 트리), `54d420`의 다른 호출이 돈다. LP에서는 돌지 않는다.
- 출력: f4가 1, 2, −1이면 out1 = 열 셀 수, out2 = 행 셀 수다. `5577ba`는 이 둘을 쓰지 않는다.

## 3. 콜리

- `549df5(N, 큐, &셀 수, &단일 수, start, perm, cell, key, buf, work)`: 위치 [0, N)의 셀마다 첫 원소와 1e-9 넘게 다른 값이 있는지 본다(work += 3·크기). 있으면 `ab9789`(perm 구간을 key로 간접 정렬. `aaedd9`와 같은 순서)로 정렬한다. 그다음 무리의 첫(가장 작은) 값과 1e-9 이내인 원소를 한 무리로 묶는다(무리마다 work += 2·(크기 − 1)). 첫 무리는 원래 id를 갖고, 나머지 무리는 새 id(셀 수 순서)를 받는다(work += 2·크기). 크기 1인 무리는 고정한다.
- `54ad3b(env, nnz, 열 값, 행 값, 열 키, 행 키, work)`: |x|의 최댓값(초깃값 1e-9)을 구하고, 모든 |x|가 정수인지 본다(|x| − floor(|x| + 1e-9) ≤ 1e-9). work += 2·nnz.
  - 정수가 아니면 `aaa891`로 |x|를 내림차순 정렬한다. 마지막으로 남긴 값보다 1e-9 넘게 작은 값만 남긴다(work += 2·nnz). 남은 값이 1e5개 미만이면 `a7bea8`의 결과 g로 scale = 1/g를 쓴다. g = 0이거나 1e5개 이상이면 scale = 2^48/최댓값이다. 정수면 scale = 1이다.
  - 키 = round(scale·x)를 i64로 바꾼다(범위 밖·NaN은 i64::MIN). work += 4·nnz. 열 키의 OR가 0이 아니고 끝 0비트가 있으면 두 키 배열 모두 그만큼 산술 시프트한다(work += 2·nnz).
- `a7bea8(n, p)`(실수 공약수):
  - 최대 > 2e9이거나 최소 < 1e-5이면 0이다. 최대 − 최소 ≤ 1e-10이면 최소를 돌려준다.
  - 모두 최소의 배수면(허용치 max(1/최소, 1)·1e-10) 최소를 돌려준다.
  - 최대·600 ≤ 2e9이고 모두 1/600의 배수면 round(600|p|)의 정수 gcd/600이다. 그렇지 않고 최소 ≥ 1.9999999999이며 모두 정수면 정수 gcd다.
  - 그 밖에는 실수 유클리드(나머지 > 1e-10까지, 결과 < 1e-9면 0)로 g를 구한다. 1/g가 정수에 가깝고 모두 g의 배수면 g다.
  - 아니면 g의 연분수 p/q(q ≤ 1000)로 근사해, 모두 q/p 배로 정수에 가까우면 p/q, 아니면 0이다.
- `a7b4d6`(공용 난수): s = s·0x5851f42d4c957f2d + 0x14057b7ef767814f, 상위 32비트를 템퍼링(>>11, &0x13a58ad <<7, &0x1df8c <<15, >>18)하고 & 0x7fffffff.
- `54e0c4(env, N, 원본 시작, 원본 번호, 키, &현재 시작, &현재 번호, &사본 시작, &사본 번호, 상대 셀, 상대 큐, &상대 고정 nnz, work)`:
  - 상대 고정 nnz ≤ 0.2·현재 nnz면 아무것도 안 한다.
  - 사본이 없으면 원본을 복사해 사본을 만들고, 현재 = 사본으로 둔다. 사본이 이미 있으면 현재 포인터는 건드리지 않는다.
  - 사본에서 고정된 상대 셀에 닿는 항목과 그 키를 빼서 압축하고(줄마다 work += 3·len, 끝에 +N), 상대 고정 nnz를 0으로 만든다.
  - 루프 A 3회차에 현재가 원본으로 돌아간 뒤 이 함수가 다시 압축하면, 사본은 계속 줄고 키도 사본 순서로 다시 섞인다. 원본 그대로 옮겼다(§6).
- `54e5eb(N, 현재 시작·번호, 키, 해시, &셀 수, cell, start, perm, 큐, &단일 수, &고정 nnz, 상대 해시, 상대 셀, 상대 큐, rand, NULL, NULL, buf, c, 끝, buf, work)`:
  - 셀 c 구간을 해시로 정렬한다(`ab848c`: **부호 없는** u64 키, `aaedd9`와 같은 알고리즘. 32개 미만은 Shell 정렬).
  - 해시가 같은 연속 무리를 나눈다(무리마다 work += 4·크기). 현재 줄 길이 합이 가장 큰 첫 무리(합이 모두 0이면 첫 무리)가 c를 갖고, 나머지는 순서대로 새 id u를 받는다.
  - 크기 1인 무리는 고정하고, 고정 nnz에 그 줄 길이를 더한다.
  - u로 옮기는 원소의 현재 줄 항목마다 상대 해시 += 키·(rand[u] − rand[c])(wrap)를 하고, 상대 셀을 큐에 넣는다. 원소마다 work += 4·len, 무리 끝에 work += 3·크기다.
- `54d420` → `54c63b`(행, 그다음 열)는 실제 값으로 검증한다.
  - 크기 2 이상인 셀마다, 첫 원소의 값(행: rhs와 sense, 열: obj, lb, ub. 허용치 1e-9)과 상대 셀별 계수 합을 다른 원소와 비교한다. 합은 비교 원소에서 처음 나온 상대 셀까지 포함해 1e-9 넘게 다르면 실패다.
  - 행이 실패하면 0, 행·열 모두 통과하면 1이다. 열만 실패하면 PRESYMMPASSES > 0이거나, 양쪽 셀 수(제거분 뺀 값)가 항목의 1/20 이하이고 101개 미만일 때 −1, 아니면 0이다.
  - 작업량 계산은 분기마다 다르다(셀마다 2·크기, 첫 원소 7·len, 원소마다 5·len, 비교한 칸 수 2배, 실패 지점까지, 셀 끝 5·위치와 칸 초기화 2배). `symmetry.rs::unequal`에 원본 덧셈 순서대로 옮겼다.

## 4. 타이머와 한도

- `ade736`: timer[0] = (0 − work) + 1이다. 경과 = timer[0] + work(`adf00e`)이고, 한도 = param_2(< 1e100이면 `adf1a6`)다.
- 1e7마다(경과 − 지난 값 > 1e7) `3ba5ac`(로그)와 `3ba9de`를 부르고, 그 뒤 state+0x1dc(PRELIMIT) < 0이면 0을 돌려주며 조용히 끝낸다. `3ba9de`는 presolve 예산을 넘으면 +0x1dc를 −1로 만든다.
- `adf2be`(라운드 끝마다): 한도 < 0이면 참, 0이면 거짓, 그 밖에는 경과/한도 ≥ 1이다.
- 노드 한도는 루프 B 라운드 수다(재시도도 1씩 올림). Symmetry > 1이면 5000이다.

## 5. 검증

도구:

- `tools/symmetry_oracle.py <out.json> [--param ...] [--relax DIR] [--presolved DIR] [모델…]`(`tools/gurun.sh`로 감쌈)는 `390989`의 `call 5577ba`(file+0x38f1f0)와 `5577ba`의 `call 551cc0`(+0x4a0555)를 thunk로 돌린다.
  - 기록: 직전 n, m, 제거 수, 예산, 한도, 노드 수, *(state+0x458) 전후, 반환값, local_f4, `551cc0`의 out1·out2·f4와 작업량, 그리고 `551cc0` 안 콜리 호출 지점별 횟수(레지스터를 보존하는 카운터 스텁).
  - `--relax`는 LP 완화를 쓰고 그 MPS를 남긴다. `--presolved`는 Gurobi presolved 모델을 남긴다.
- `tools/symmetry_check.py <oracle.json> [--src DIR] [--param ...]`는 포트와 비교한다. 비교 항목은 호출 여부, local_f4, 작업량이다.
  - 포트는 `RSOLVER_SYM_TRACE`가 있으면 호출마다 stderr에 한 줄(전후 work 비트, f4)을 쓴다.
  - 포트의 호출 직전 work는 다른 미포팅 항 때문에 Gurobi와 다르다. 그래서 `RSOLVER_SYM_BASE`로 오라클의 직전값을 넘기면, 포트가 상태 사본에서 그 값부터 한 번 더 돌린다. 그 직후값을 오라클 직후값과 비트로 비교한다.
- `tools/presolve_check.py <dir> --src DIR`은 MPS 파일 모델의 presolved 결과를 비교한다.

결과(기본값, Symmetry=2도 같은 수):

| 모음 | 모델 | `5577ba` 호출 | 호출 일치 | local_f4 | 작업량(비트) |
|---|---|---|---|---|---|
| Netlib | 64 | 21 | 64/64 | 21/21 (모두 0) | 21/21 |
| MIPLIB small 38개의 LP 완화 | 38 | 17 | 38/38 | 17/17 (1: neos-911970, neos-3627168-kasai, 2: cod105) | 17/17 |
| 합성(`tools/symmetry_models.py`) | 7 | 7 | 7/7 | 7/7 (1: k3, k5, k6, 2: k8, k12, −2: cube8, −1: cycles) | 7/7 |

- Netlib 호출 모델: 25fv47, bnl1, degen3, fffff800, fit1p, ganges, gfrd-pnc, grow15, grow22, maros, modszk1, perold, pilot4, scfxm2, scfxm3, scrs8, scsd6, sctap2, shell, ship04s, standmps.
  - 증가분 예: 25fv47 +337758, degen3 +727311, grow15 +244144.
  - 증가분은 압축 3n(+ 이동·걸러냄)과 `551cc0` 몫의 합이다. 예: ganges 12973 + 178510.
  - gfrd-pnc, modszk1, shell처럼 47c445가 돌지 않는 모델도 불린다.
- 경로: scrs8, gmu-35-*는 열 값 정제 뒤 1.1 판정으로 끝난다. 대부분은 루프 B 1–4라운드 안에 1.1 판정으로 끝난다.
  - 검증까지 간 것은 neos-911970, kasai, cod105, 합성 전부다.
  - 재시도(검증 3번, −2)를 탄 것은 sym_cube8이다. 노드 한도(5라운드)를 탄 것은 sym_cycles다.
  - neos-860300은 mult ≠ 1인 압축 작업량이 비트 일치했다.
- presolved 모델:
  - 기본값 Netlib 64/64, postsolve 64/64(`tools/postsolve_check.py`, 기존 ordef 캡처)다.
  - 합성 sym_k*는 기본값 5/5와 PRESYMMPASSES=0 5/5가 일치하고, 둘이 서로 다르다. 기본값은 종속 행 R*_{qe−1}을 남기고, PRESYMMPASSES=0이면 47c445가 지운다.
  - MIPLIB 완화는 36/38이다. cod105(상한 1.002)와 enlight_hard(`NAME Presolved`)는 PRESYMMPASSES=0에서도 같은 차이가 나고, enlight_hard는 `5577ba`를 부르지 않는다. 대칭과 무관한 기존 차이다.
- 회귀: pytest(`tests/test_symmetry.py`)를 두었다. sym_k5 presolved는 기본값과 PRESYMMPASSES=0 둘 다 Gurobi와 같은지 본다. sym_k5, sym_cube8, sym_cycles는 local_f4와 작업량 직후값이 비트로 같은지 본다. 기대값은 오라클 출력으로 고정했고, Gurobi 없이 돈다.

재현:

```
tools/gurun.sh ../.venv/bin/python tools/symmetry_oracle.py /tmp/o.json                          # Netlib
PYTHONPATH=$PWD/python ../.venv/bin/python tools/symmetry_check.py /tmp/o.json
tools/gurun.sh ../.venv/bin/python tools/symmetry_oracle.py /tmp/r.json --relax /tmp/rel --presolved /tmp/relpre <MIPLIB 이름…>
PYTHONPATH=$PWD/python ../.venv/bin/python tools/symmetry_check.py /tmp/r.json --src /tmp/rel
../.venv/bin/python tools/symmetry_models.py /tmp/syn                                               # 합성 모델
```

## 6. 양성 사례 만드는 법

- 같은 블록을 k번 나란히 복제한다(`block_lp`). 사본들이 등가 분할이 되어 검증을 통과한다.
  - 살아 있는 열 ≥ 500, 행 ≥ 100이어야 한다(presolve가 패스 0까지 줄인 뒤 기준).
  - 계수·비용·rhs는 블록 안에서 서로 달라야 한다. 그래야 셀이 사본 묶음으로 갈린다.
  - nnz > 2.5n이면 PreDepRow 자동값이 3이 되어 효과가 보인다.
  - 블록마다 종속 `=` 행(앞 두 행의 합)을 넣으면, 대칭을 찾았을 때만 그 행이 남는다.
- f4 = 2는 열 셀 수(제거된 열의 단일 셀 포함)가 (n − 제거 열)/5 미만이고, 행 셀 수가 살아 있는 행/5 미만일 때 나온다. 사본이 많을수록(k8, k12) 나오기 쉽다.
- MIPLIB 완화 중에는 neos-911970, neos-3627168-kasai, cod105가 양성이다. 다만 앞의 둘은 47c445가 원래 할 일이 없어 presolved 모델이 대칭 유무와 상관없이 같다.
- 재시도 경로: 키는 같고 실제 합은 1e-9 넘게 다르게 만든다. 계수 1 + 6e-10은 `54ad3b`에서 정수로 보인다(`cube_lp`).
- 노드 한도: 정제가 한 칸씩 번지는 긴 순환에 열 하나만 비용을 다르게 둔다(`cycles_lp`). 끝이 있는 경로는 포트 presolve가 끝 축약을 Gurobi와 다르게 해서(대칭 무관) 쓰지 않았다.

## 7. 지름길과 미확인

- `3ba9de`의 presolve 예산 정지는 옮기지 않았다(`shortcut:`, mod.rs의 예산 미모델과 같음). 1e7마다 하는 판정은 GURO_PAR_PRELIMIT < 0만 본다. 확인한 모든 사례에서 `551cc0` 작업량이 1e7 미만(최대 cod105 1.8e6, neos-860300 5.7e6)이라 이 판정은 한 번도 걸리지 않았다. 작업 한도(`adf2be`)도 걸린 사례가 없다.
- nnz = 0일 때 원본이 나가는 분기는 추적하지 않았다. 0을 돌려준다고 가정했다(`shortcut:`).
- 루프 A 3회차에 "현재 줄은 원본, 압축은 사본"이 되는 동작은 그대로 옮겼다. 하지만 압축(`54e0c4`)과 재시도 2번이 함께 나온 사례가 없어 확인하지 못했다. sym_cube8은 재시도는 탔지만 고정 셀이 없어 압축은 안 했다.
- LP 인자로 죽은 경로는 포팅하지 않았다: 정수 열(local_a8: 열 길이 키 정제, `5507cb` 탐색, `ab6a5f`), Q 대각(param_12), param_21 반올림, param_8(열 길이), param_25–29 출력.
- +0x170(열 플래그) ≠ 0 열은 LP에서 0이라 확인하지 못했다.
- Aggregate ≥ 2면 패스 1 이후 집계가 압축된 열 CSR 뒤에 새 구간을 붙인다(agg.rs). Gurobi가 압축으로 생긴 빈 공간을 다르게 쓰는지는 확인하지 않았다(기본값은 패스 0에서만 집계한다).
