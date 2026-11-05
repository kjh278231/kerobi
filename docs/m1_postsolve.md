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

## 5. 포팅 계획

1. `PostRec`에 type 11(집계)을 더한다. presolve 쪽에서 `519a52`/`37f484`를 옮길 때 같이 한다.
2. arena 바이트 위치를 흉내 낸다. push마다 크기를 누적하고, 시작 정렬은 오라클 `val_al`로 맞춘다.
3. `363daa`를 옮긴다(입력: presolved x/dj/상태). 비교는 오라클 출력 배열과 비트 단위로 한다.
4. presolved 입력은 오라클의 `pre_after`를 그대로 넣어 검증한다. 그래야 simplex 포팅과 무관하게 postsolve만 확인할 수 있다.
