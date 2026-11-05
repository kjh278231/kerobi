# M1 작업 노트 (LP 엔진 포팅)

M1 진행 중에 알아낸 사실과 작업 방법. 확정된 내용은 해당 설계 문서(04, 10)로 옮긴다.

## 1. 비교 기준선

| 디렉터리 | 설정 | 용도 |
|---|---|---|
| `bench/gurobi/` | 기본값(Method=-1, presolve 켬), Threads=1, WorkLimit 60 | 최종 P1 |
| `bench/gurobi_dual_nopre/` | Method=1, Presolve=0, Threads=1 (LP 64개) | M1 첫 목표: dual simplex 단독 |

## 2. 상태 오라클 (`tools/spx_oracle.py`)

- SIMPLEX 콜백 안에서 `model+0xe0`(spx)을 읽어, 지정한 반복마다 스칼라와 배열을 JSON으로 남긴다. double은 `float.hex()`라 비트 단위로 비교한다.
- 반복 0에는 스케일(`colscale`, `rowscale`), 스케일된 행렬(`beg/len/ind/val`), `b`, `sense`도 남긴다.
- 반복 0까지 이미 끝난 단계: setup(`02f7e1`), 스케일링(`07d3ce`), dual 초기화(`0534f7`, crash 기저 포함), 첫 분해, 첫 `048d5f`. 배열별로 어느 단계가 틀렸는지 가른다: 스케일 → 07d3ce, `c/l/u` → 02f7e1, `vstat` → 0534f7, `d/xB` → 048d5f·1bae9c.
- Gurobi는 한 프로세스만 돌 수 있다(`gurobi_guard.pth`). 다른 세션이 쓰는 중이면 기다렸다 재시도한다.

## 3. 관찰한 사실

- 무한 bound는 내부에서 1e30이다. dual simplex는 이 값을 실제 bound처럼 써서, dual 위반 열을 1e30 쪽에 둔다(인공 bound 방식 phase 1). afiro 반복 0의 목적 −1.8e30 = Σ(음의 reduced cost)·1e30.
- dual 초기 기저는 slack 기저가 아니다(25fv47: 반복 0에 구조 변수 188개가 기저). `0534f7`의 crash 경로를 포팅해야 한다.
- 스케일 인자는 2의 거듭제곱이다. afiro는 열 1.0, 행 {0.25, 1}; 25fv47은 1/32..4.
- 스케일링 경로 선택(`07d3ce` L1490–1625): ScaleFlag=-1(자동)이고 LP·dual이면 모델 등급 D[0]이 4(network)가 아닌 한 **ScaleFlag 3 경로**로 간다. 결과를 D+0x3d0에 캐시한다.
- ScaleFlag 3 경로(L2826~): 최대 10패스. 표시된 열(음수 값)마다 `max_i |a_ij·r_i|`를 구해 역수를 취하고, 열 bound 크기(1..1e30 사이 유한값)와 `sqrt`로 섞은 뒤 [0.5, 2) 밖이면 2의 거듭제곱으로 반올림하고, 직전 값 대비 배율(`local_108`) 한도로 자른다. 음수 부호는 "이번 패스에 갱신 대상" 표시다.
- `07d3ce`는 스케일 전에 모델 데이터 D를 직접 바꾼다: 상한만 있는 열(하한 −inf, 상한 유한)은 x' = −x로 뒤집고(목적·계수·bound 부호, D+0x3c=1), 저장된 스케일이 있으면 다시 적용한다.
- 의사코드에서 `ilogb()`·`scalbn()`의 인자가 빠진 곳이 많다. 해당 지점은 objdump로 인자를 확인한다.

## 4. 진행

| 단계 | 대상 | 상태 |
|---|---|---|
| A | 스케일링 `07d3ce`(ScaleFlag 3 경로 우선) | 진행 중 |
| B | setup `02f7e1`, dual 초기화 `0534f7`(crash) | 대기 |
| C | LU(`036ade`, `1da0b2`, MKL dgetrf), FTRAN/BTRAN | 대기 |
| D | dual 반복(`028c93` 외) → 반복 0·1 궤적 일치 | 대기 |
