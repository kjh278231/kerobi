# M1 LP postsolve 작업 노트

[m1_presolve.md](m1_presolve.md)의 짝. 2026-11-05 서브에이전트 조사 결과(인용 줄 번호는 `re/decomp/c/`의 각 함수 파일). 포팅하면서 확인되는 대로 고친다.

## 1. 기본 LP 경로에서 도는 함수

- 기본값(simplex)으로 푼 LP는 `0c3d0d`에서 다음 순서로 처리된다.
  1. presolved 모델을 simplex로 푼다(`cda5b` @0x14c18d).
  2. `c379f`를 부른다. SolutionTarget<1이면 바로 돌아온다.
  3. uncrush **`363daa`**를 돌린다(asm 0x14b2f5).
  4. presolved 모델과 postsolve 객체를 해제한다.
  5. **원래 모델**에서 simplex를 다시 돈다(`cda5b` @0x14fe68). 기저와 x는 넘기고 쌍대값은 넘기지 않는다.
  6. 반복이 있었으면 "Extra simplex iterations after uncrush: %d"를 찍는다.
- `363164`(원시)와 `365159`(쌍대)는 `36578f` 경로(SolutionTarget≥1 등)에서만 돈다. "big uncrush violation" 검사도 `36578f`에만 있다.
- afiro, adlittle, share2b, sc50a, blend에서 확인했다.
  - 5단계의 추가 반복은 0이었다.
  - VBasis/CBasis는 `363daa` 기저와 정확히 같았다.
  - X/RC/Slack/Pi는 마지막 비트가 다르다(상대 1e-12 이내). 5단계가 다시 계산하기 때문이다.
- **그래서 postsolve의 비트 비교 대상은 `363daa`의 출력 배열이다.** 최종 속성까지 맞추는 것은 simplex 포팅(5단계)의 몫이다.

## 2. `363daa` 규칙

**준비 (L200–262)**
- 남은 열·행은 presolved simplex 배열에서 x/dj/상태를 복사한다(`spx+0x200`, `+0x1f8`, `+0x168`; `spx+0x28`이면 `+0x250/+0x248`).
- `+0x408` 표시가 있는 presolved 열은 x와 dj의 부호를 바꾸고, 상태 −1과 −2를 맞바꾼다.
- 지운 열: 상태는 `−3−min(colmap,−2)`, dj=0이다.
- 지운 행:
  - rowmap < −2이면 상태 −1, slack 0이다.
  - rowmap이 −1이나 −2이면 기저이고, y=0이다.

**레코드 처리**: 목록 머리(`post+0x80`)부터 `rec+0x10`을 따라간다. LIFO이므로 `PostRec`는 역순으로 돈다.

**type별 처리** (`365159` 줄 번호. `363daa`는 같은 경우에 기저 처리를 더한다)
- **1** (:123)
  - x=v.
  - d = obj + Σ y·a. 아래의 SIMD 내적이다.
  - 상태는 저장된 상태이다. −4는 d≥0이면 −1, 아니면 −2가 된다.
- **2** (:197): x = c0 + Σ val·x, d=0.
  - mode<0 (행을 남긴 경우):
    - 행 상태가 기저이면 열이 기저를 넘겨받고 행은 −1이 된다.
    - 아니면 상태 = mode이다. (mode −1, a<0)이나 (mode −2, a>0)이면 y[r]의 부호를 바꾼다. d = a·y[r].
  - mode≥0: y[r] = −(obj + Σ y·colval)/a.
- **3** (:354)
  - x는 행 bound 값들의 min(dir 0) 또는 max이다.
  - 걸리는 행이 있으면 열이 그 행의 상태를 받고, 행은 −1이 된다.
  - 없으면 상태는 dir≠0일 때 −1, 아니면 −2이다.
- **5** (:463, `50b4e5`): y[r0] < 0이면 y[r1] = y[r0]/ratio, y[r0]=0으로 두고 행 상태를 맞바꾼다. 평행 **행** 레코드이다(노트 12 §4.2의 "열"은 틀렸다).
- **6** (:472): 1e-6 허용치 논리. 미해독.
- **7** (:511)
  - `363daa`는 상태를 기저에서 정한다(`365159`는 d의 ±1e-6로 정한다).
  - code bit 8이면 |x−x_rec| ≤ 1e-6이어야 한다. bit 1·2는 bound 쪽이 맞아야 한다.
  - 그다음 y[r] = −d/a, d=0으로 두고, 열이 행의 상태를 받으며 행은 −1이 된다.
- **8** (:539)
  - d/(val·dir)가 가장 음수인 열을 고른다(비율 t).
  - y[r] = −(dir·t)로 두고, 모든 k에 대해 d_k −= val_k·(t·dir)를 한다. asm으로 보면 곱은 한 번이다.
  - `363daa`는 모든 열을 −2나 −1로 두고, 고른 열이 행의 상태를 받는다.
- **11** (:608, 집계 `519a52`→`37f484`): 행 상태는 열 상태를 받고, 열은 기저(0)가 된다. afiro에도 6개 있다. **`PostRec`에 아직 없다.**
- **12** (:613): d += Σ.
- **14, 0x15(`35fd68`), 0x16(:697, y_i += a·y_k), 0x18(`361ab1`), 4(`35d235`)**: 미해독.

**마무리**
- `a8ff5e`(L1020)가 모든 slack을 rhs − Σ_j a_ij x_j로 덮어쓴다. 열 순서로 누적하며, 곱한 뒤 더한다. 레코드가 쓴 slack 값은 쓰이지 않는다.
- 원래 `>` 행은 y 부호를 바꾼다(L1029–1046).
- 관례(5개 모델로 확인): **Pi = −y_out**, **RC = +d**.
- 기저 출력: 0 이상이면 기저, −1은 −1, 내부 −4는 −1로 보고된다.
- 목적 상수(`post+0x120`)는 모든 것이 지워졌을 때만 쓴다. 이때 ObjVal = sense·const이다.

## 3. 비트 일치 함정

- **내적**: 항이 8개 이상이면 SIMD이다.
  - `val_ptr & 0xf == 8`이면 한 항을 먼저 뗀다.
  - 합 순서는 ((s0+s2)+(s4+s6)) + ((s1+s3)+(s5+s7))이고, 그다음 남은 항을 스칼라로 더한다(`365159.s` 373825–373838).
- **정렬**: 레코드 arena(`ac68d7`)는 정렬 없는 bump allocator이다. 그래서 내적의 정렬은 레코드의 바이트 위치에 달려 있다.
  - 각 push의 크기 식(예: type 1은 `((cnt*4−1)&~7) + cnt*8 + 0x50`)으로 위치를 흉내 내야 한다.
  - 또는 오라클이 잡는 정렬 `val_al`로 검증한다. afiro에서도 0과 8이 섞여 있다.

**arena `ac68d7(env, arena, size)`**
- arena 필드: +0 최소 청크, +8 다음 청크 크기, +0x10 현재 청크 사용량, +0x20 현재 청크.
- 청크 필드: +0 base, +8 크기, +0x10 다음 청크.
- 할당 규칙:
  - `크기 − 사용량 ≥ size`이면 `base + 사용량`을 돌려주고 사용량을 size만큼 늘린다. 정렬은 하지 않는다.
  - 아니면 새 청크를 잡는다. 크기 `c = max(다음 크기, size)`를 `ac645d`(malloc 래퍼)로 할당한다. 다음 크기는 `max(최소, min(0x10000, 2c))`가 되고, 사용량은 size가 된다.
- 그래서 레코드 val 배열의 16바이트 정렬은 다음 세 가지로 정해진다:
  - 청크 base의 정렬(`ac645d`의 헤더에 따라 다름, 오라클로 확인);
  - 같은 청크 안에서 앞선 할당 크기들의 합;
  - 레코드 안의 val 오프셋.
- arena는 `3a484d`가 만든다(state+0x368). 최소·첫 청크 크기는 아직 확인하지 않았다.

## 4. 오라클 `tools/uncrush_oracle.py`

```
tools/gurun.sh ../.venv/bin/python tools/uncrush_oracle.py <model.mps.gz> <out.json> [Param=Value ...]
```

- `0c3d0d` 안의 `363daa` 호출(`call rel32`)을 라이브러리 근처 페이지의 jump stub을 거쳐 ctypes thunk로 돌린다.
- 잡는 것:
  - 레코드 목록(type, data 포인터, val 정렬);
  - presolved 입력: `pre_after` = 363daa가 쓰는 x/dj/상태, n_pre+m_pre;
  - 출력: basis int[n+m], x[n+m](열, 그다음 slack), dual[n+m](d, 그다음 y);
  - 최종 속성.
- `cda5b` 호출과 `36578f` 경로 호출 횟수도 센다.
- 13.0.3 빌드, Threads=1 기준이다.
- **SolutionTarget=1은 오라클로 쓰면 안 된다.** state+0x100=1이 되어 presolve가 달라진다.

## 5. 상태 (2026-11-05, 1단계 달성)

| 설정 | 일치 | 비고 |
|---|---|---|
| PRELIMIT=17 | **58/58** (uncrush가 도는 모델. 나머지 6개는 reduction이 없어 `363daa`가 안 돈다: degen3, grow7, grow15, grow22, scsd1, scsd6) | 입력은 오라클 `pre_after`. basis·x·dual 비트 일치 |
| PRELIMIT=1..16 | 전부 일치 | 같은 방법 |
| 기본값 | §8 참고 (2단계: 53/59) | |

코드: `crates/lp/src/presolve/postsolve.rs`(`Post`, `uncrush`, `simd_sum`, 타입 4 `parallel_col`). 비교: `tools/postsolve_check.py`.

```
S=<scratch>; tools/gurun.sh ... uncrush_oracle.py (64개 반복은 S/cap.sh 꼴의 루프)  -> $S/or17/<name>.json
PYTHONPATH=$PWD/python python tools/postsolve_check.py $S/or17 --param GURO_PAR_PRELIMIT=17
python tools/postsolve_check.py ... --oracle-align   # 아레나 시뮬레이션 대신 오라클의 실제 주소 정렬
python tools/postsolve_check.py ... --arena          # 레코드 크기로 설명 안 되는 주소 간격
```

## 6. 레코드 배치와 아레나 (확정)

레코드 하나 = 아레나 할당 하나. `rec`(0x18 바이트: type, data 포인터, next) 바로 뒤가 data 블록이다. 값 배열의 정렬은 `할당 시작 + 오프셋`의 16 나머지이다.

| type | 할당 크기 | 값 배열 오프셋 | data 블록 |
|---|---|---|---|
| 1 | `0x48 + r8(4n) + 8n` | `0x48 + r8(4n)` | j, v@8, obj@0x10, status@0x18, n@0x1c, ind@0x20, val@0x28 |
| 2 | `0x68 + r8(4nr) + 8nr` (+ `r8(4nc) + 8nc` if nc ≥ 1) | 행 `0x68 + r8(4nr)`, 열 위 크기 합 | j, r, c0@8, a@0x10, obj@0x18, nr@0x20, ind@0x28, val@0x30, mode@0x38(<0 행 유지, 아니면 nc), ind2@0x40, val2@0x48 |
| 3 | `u20+u22+8c+u27+8K+0x70` (u20=(4c+3)&~7, u22=(4c−1)&~7, u27=(4K−1)&~7, c 행 수, K 항 수) | `0x70+u20+u22+u27`(행 t는 거기에 8·start_t) | j, dir@4, **bound@8(dir 1이면 lb, 0이면 ub)**, c@0x10, start@0x18, rows@0x20, ind@0x28, val@0x30, rhs/a@0x38 |
| 4 | 0x50 | | x, y, lx@8, ly@0x10, ux@0x18, uy@0x20, r@0x28, 정수 표시@0x30 |
| 5 | 0x28 | | a, b, ratio@8 |
| 7 | 0x38 | | j, r, code@8, a@0x10, x@0x18 |
| 8 | `r8(4n) + 8n + 0x38` | | dir(double)@0, r@8, n@0xc, ind@0x10, val@0x18 |
| 14 | `r8(4n) + 8n + 0x50` | | a, b, k, ratio@0x20 |

- `r8(x) = (x+7)&~7`. 크기는 모두 8의 배수라서 청크 안의 위치 정렬은 누적 크기의 `& 8`이다.
- **아레나 청크** (`ac66be`로 만들고 `ac68d7`이 쓴다): 첫 청크 크기 = `nnz/5 + 10`바이트(`3a484d` asm 0x396c39 앞, nnz = 원래 nnz). 요청이 남은 크기를 넘으면 새 청크를 잡는다. 크기는 `max(next, 요청)`이고 `next = max(첫 크기, min(0x10000, 2·그 크기))`이다. 청크 base는 16 정렬이다.
- 레코드만 이 아레나를 쓴다. 외부 할당이 없다는 것은 58개 모델의 레코드 주소로 확인했다. 청크가 힙에서 바로 붙으면 "간격"처럼 보이니 주의한다(블렌드에서 16, 24바이트).
- 런마다 절대 주소는 바뀌지만(ASLR) 청크 안 상대 위치는 같다.

## 7. `363daa` 포팅 메모

- 입력 `pre_after`는 `12edc8`/`12ef7f`(스케일 해제), +0x408 부호 뒤집기, +0x54 블록까지 끝난 값이다. 이 앞부분은 포팅하지 않았다(기본값에서 simplex를 포팅할 때 필요).
- 맵: `row_map`/`col_map`은 살아 있으면 새 번호, 지웠으면 state의 `row_cnt`/`col_cnt` 값(−1, −2, −3−r)이다(`3b0917` L380–470). 범위 열(분할 행)은 살아 있는 열 뒤에 붙는다. `n_base` = 살아 있는 열 수(`post+0x3c`).
- 기저 값: 양수는 presolved 기저 위치이다. 그것을 그 위치의 원래 행 번호로 바꿔 쓴다(`local_80`). type 11/14는 기저로 리터럴 1을 쓴다.
- **정렬 의존 내적**: type 1, 2(행·열), 3(행마다)이 쓴다. 곱은 교환이라 `y*a`든 `a*y`든 같고, 합 순서만 중요하다(`simd_sum`, 단위 테스트가 순서를 고정한다). 최종 합은 트리 `((s0+s2)+(s4+s6))+((s1+s3)+(s5+s7))`로 맞았다(선형 합이 아니다).
- type 8의 끝은 4개씩 묶은 루프에서 `val*(t*dir)`, 꼬리에서 `(val*t)*dir`이다. dir=±1이라 값은 같다.
- **마무리 slack(`a8ff5e`)**: 열 순서로 `x_j*a + s`를 누적하고 `rhs − s`를 한다. 원본은 `>` 행을 부호 뒤집어 들고 있다가 마지막에 되돌려 놓는다. 그래서 `>` 행은 `−((−rhs) − s')`(s' = s가 0이면 +0, 아니면 −s)로 써야 0의 부호까지 맞는다(rhs=0이면 +0, 합이 rhs와 같으면 −0).
- 타입 4 (`35d235`): LP 열(정수 표시 0)만 포팅했다. 정수 표시 1, 2 분기는 안 했다. 기저 인자는 디컴파일 호출에서 빠져 있어 넘겨진다고 가정했다(fit1d 일치로 확인).
- 아직 안 한 것: type 12, 0x16, 0xd(`3a141c`), 0x10(`3a484d`)의 포팅(Netlib 기본값 오라클에는 없다). 기저 없는 경로(`param_7 == 0`), spx 없는 경로, 스케일된 원본 모델(`+0x3e0`) 분기.

## 8. 기본값과 집계 레코드 (2단계, 2026-11-05)

오라클을 파라미터 없이 64개 떴다. `363daa`가 도는 모델은 59개, 레코드 종류 수는 type 2: 7498, 1: 5715, 7: 1848, 6: 458, 11: 353, 8: 236, 21: 109, 4: 105, 3: 21, 14: 18, **24: 14**이다.

**결과: 53/59 일치**(5개는 uncrush 없음). 틀린 6개 중 degen2·degen3은 `47c445` 미포팅 때문이고(레코드가 모자라 인덱스 초과), 나머지 4개(beaconfd, finnis, forplan, ganges)는 아래 type 24 때문이다. 일치하지 않는 모델 중 정렬이 원인인 것은 없다(`--oracle-align`과 시뮬레이션 결과가 같다).

### type 6 (`AggBound`, 집계 단계 D) — `363daa` case 6, 기저 경로
데이터: code@0, e@4, k@8, r@0xc, b@0x10, a_e@0x18, a_k@0x20 (code bit 8 = col_flag 0x80은 PWL이라 안 했다).
- k의 상태가 −1이면 code&1, −2이면 code&2가 있을 때만 동작한다.
- 동작: `y[r] −= d[k]/a_k`, `d[e] = ((−d[k])·a_e)/a_k`, `d[k] = 0`, `bs[k] = bs[e]`.
- `bs[e]`는 a_e·a_k ≤ 0이면 (k의 옛 상태 ≠ −1 ? −2 : −1), > 0이면 (옛 상태 == −1 ? −2 : −1)이다.

### type 11 (`SlackCol`) — `x[slack r] = a·x[j]`, `bs[행 r] = bs[j]`, `bs[j] = 0`

### type 21 (`PairAgg`, `35fd68`)
데이터: r@0, n1@4, n2@8, e1@0xc, e2@0x10, k1@0x14, k2@0x18, f1@0x20, f2@0x28, a1@0x30, a2@0x38, obj1@0x40, obj2@0x48, ub1@0x50, ub2@0x58, ind@0x60, val@0x68.
- `x[e1] = f1·x[k1]`, `x[e2] = f2·x[k2]`.
- `s1 = obj1 + Σ_{첫 n1항} y·a`, `s2 = obj2 + Σ_{나머지} y·a`. 둘 다 정렬 의존 내적이고 두 번째 val 포인터는 `val + 8·n1`이다.
- 경우 가르기: x[e1] ≥ 1e−6 또는 x[e2] ≥ 1e−6이면 (|a2|·x2 ≤ |a1|·x1 ? 경우 1 : 경우 −1). 아니면 d[k1] ≥ 1e−6일 때 (d[k2] < 1e−6 ? −1 : 0), 아니면 1.
- 경우 1: `y[r] = −s1/a1`, `d[e1] = 0`, `d[e2] = s2 + a2·y[r]`, `d[k2] −= f2·d[e2]`, `bs[e2]` = x2 < 1e−6 ? −1 : (ub2−1e−6 < x2 ? −2 : −3). 경우 −1은 e1, e2를 바꾼 것이다.
- 경우 0: 후보 e1, e2, k1, k2 중 비율(s/|a|)이 가장 작은 것이 기저에 들어가고(`bs = r`), 나머지 둘 중 e1, e2는 −1이 된다. y[r], d 네 개를 식대로 고친다.

### type 2 (집계판)
`4f4d0a`판과 배치가 같다. mode = Lc ≥ 0이면 기존 처리(`y[r] = −(obj + Σ y·a)/a`)로 맞다. 단계 B의 pivot 열 자신이 row에 들어 있는 경우도 그대로 맞았다.

### 아레나
- 첫 청크 크기는 **생성자 시점의 nnz**/5 + 10이다. 집계가 `st.nnz`(+0xd8)를 바꾸므로 `run_post`가 `State::new` 직후의 값을 저장해 `Post::new`에 넘긴다.
- 청크 전환 위치는 59개 모델 모두 시뮬레이션과 일치했다.

### type 24 (`361ab1`) — LP presolve에서는 `5b488d`가 쌓는다
`3a008f`(MIP 경로, 호출자 `5a4d8b`/`5a5eff`)는 쌍둥이일 뿐이다. LP는 `5b488d`(L1483 `ac68d7` 호출, L1495 `*puVar21 = 0x18`)다. 기본값 오라클의 4개 모델에 14개가 있다.

**기록 내용** (`PostRec::ColRows { j, lb, ub, vtype, y_idx, y_add, rows: Vec<ColRow{row, sense, rhs, terms}> }`)
- j: 열. lb/ub: push 시점 `state+0xa0/0xa8`. vtype: 'C'(0x43) 고정.
- rows: 열 j의 **열 CSR**(`col_beg[j]`, 길이 `col_len[j]`)을 순서대로 걸으며 `row_ind ≥ 0 && row_cnt ≥ 0`인 행마다 하나.
  - sense = `state+0xc0`[행], rhs = `state+0xb8`[행].
  - terms = 그 행의 **행 CSR** 항목 중 `col_ind ≥ 0 && col_cnt[col] ≥ 0`인 것을 CSR 순서로(열 j 자신 포함).
- c = rows 수, K = Σ `row_cnt[행]`(push 시점 값. terms 수와 같아야 한다).
- y_idx / y_add: 열 j에 대해 `|x| ≤ 1e−10`(x는 `5b488d`의 `local_90[j]`. 어느 배열인지 확인하지 못했고, 아마 obj)이면 **−1 / 0.0**. 아니면 y_idx = 기준 행(`local_160`), y_add = `−obj[j] / a_ij`(a_ij는 그 행의 열 j 항목. 행이 비었거나 못 찾으면 1.0이 들어간다. 그리고 아래 obj, objcon 갱신에 같은 값이 쓰인다). forplan에서 y_idx = 159, y_add = −obj(a = 1)로 확인했다.

**바이트 배치** (rec = 할당 시작, data = rec+0x18. 필드를 쓰는 순서는 L1486–1514 순)
| 순서 | 위치 | 내용 |
|---|---|---|
| 1 | rec+0x08 | data 포인터 = rec+0x18 |
| 2 | data+0x30 | int* start = rec+0x78 |
| 3 | rec+0x00, data+0x00, data+0x18 | type 0x18, int j, byte vtype 'C' |
| 4 | data+0x38 | int* rowid = rec+0x80+u41 |
| 5 | data+0x50 | int* ind = rec+0x88+u41+u20 |
| 6 | data+0x58 | double* val = ind + 8 + u39 |
| 7 | data+0x40 | char* sense = val + 8 + (8K−8) |
| 8 | data+0x48 | double* rhs = sense + u38 + 8 |
| 9 | rec+0x10 | 다음 레코드 = 옛 `st+0x370`, 그다음 `st+0x370` = 새 레코드 |
| 10 | data+0x08 / 0x10 | lb / ub |
| 11 | data+0x1c / 0x20 | int y_idx / double y_add |
| 12 | data+0x28 | int c |
| 13 | 배열 | rows를 걸으며 start[t], rowid[t], sense[t], rhs[t]와 ind/val |

- 크기 = `(8K−8) + u39 + u41 + u20 + u38 + 8c + 0xa0` (u39 = (4K−1)&~7, u41 = (4c+3)&~7, u20 = (4c−1)&~7, u38 = (c−1)&~7). **`3a008f`와 다르다**: `5b488d`에는 행 번호 배열(u20 항)이 더 있어서 u20 항이 추가되고 상수가 0x98 → 0xa0이다. `layout()`을 이 식으로 고쳤다.
- 배열 순서(작은 주소부터): start(c+1개 int, rec+0x78), rowid(c개), ind(K개), val(K개 double), sense(c바이트), rhs(c개 double). 이 중 정렬 의존 내적에 쓰이는 것은 없다.
- 레코드를 쌓은 뒤 `5b488d`가 하는 일(presolve 쪽): 열 j의 live 행마다 `row_cnt = −1`, 그 행의 항목들을 열에서 지움(`col_cnt −1`, `row_ind = −1`), `+0x404`++. 마지막에 `col_len[j] = −1`, **`col_cnt[j] = −2`**, `+0x408`++. 지운 행의 `row_cnt`가 −1이라서 uncrush 준비에서 행이 "자기 번호 기저"로 남는다(위 forplan 불일치의 원인이 `−3`이었다).

- uncrush: 각 행에서 `v = (rhs − Σ_{k≠j} a·x_k)/a_ij`. 상한 후보(a_ij > 0 또는 `=`)는 최솟값, 하한 후보(a_ij < 0 또는 `=`)는 최댓값을 잡는다(초기값은 ub, lb). 하한이 그대로 lb이면서 유한하거나, (상한이 무한이거나 ub−1e−10 이하이고 |L| < |U|)이면 L, 아니면 U를 x_j로 둔다. 상태: lb와 같으면 −1, ub와 같으면 −2, 아니면 그 한계를 낸 행의 기저를 받고 그 행은 −1이다. 마지막에 `y[y_idx] += y_add`.
- `PostRec::ColRows`, `layout`, `kind`, 위 uncrush를 넣었다. 쌓는 쪽이 없어 단위로는 검증하지 못했다. 임시로 forplan의 Subst(빈 행, 빈 열)를 ColRows로 바꿔 보니 x, d, y가 모두 맞고 기저만 한 곳 달랐다(행 159의 `row_cnt`가 −3이라 −1이 된다. 위 `5b488d`처럼 −1이면 맞을 것이다).
- 기본값 오라클과의 차이(rsolver는 type 2, Gurobi는 type 24):
  - forplan: Subst(j 72, r 159, 빈 행·열) ↔ type 24(j 72, lb 0, ub ∞, 행 1개).
  - finnis: Subst(j 55, r 19, mode −1) ↔ type 24(j 155, lb −0.001, ub ∞).
  - beaconfd: Subst(빈 행) 6개 ↔ type 24 5개(위치도 앞쪽에 몰림).
  - ganges: type 24 7개(열 1039, 1050 등은 행 4–6개), type 1 몇 개가 사라진다.
