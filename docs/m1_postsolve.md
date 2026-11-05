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
| 기본값 | fit1p 1/1 (scsd1, scsd6은 presolve가 아무것도 안 해 uncrush가 안 돈다) | 집계 레코드(type 2 리스트판, 6, 11, 21)는 아직 |

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
- 아직 안 한 것: type 6, 11, 12, 0x15, 0x16, 0x18, 0xd(`3a141c`), 0x10(`3a484d`)의 포팅. 기저 없는 경로(`param_7 == 0`), spx 없는 경로, 스케일된 원본 모델(`+0x3e0`) 분기.
