# M3-04b primal push 분기 조사

## 1. 착수 검증 계약

- 종류는 조사다. `134a87`·`256781`의 호출 조건, 단계별 읽기·쓰기 상태, 후보 순서, ratio 선택, bound 이동·basis 교환, 재분해와 반환을 분할한다.
- 기준선은 main `51cd515`다. M3-04a·M3-04a-1이 병합돼 있으며 미커밋 선행 변경은 없다. 독립 branch는 `work/m3-04b`다.
- 기준 원본은 Gurobi 13.0.3의 C·asm과 로컬 ELF다. 파일·주소·해시는 조사 결과에 기록한다. 함수명 중복은 파일 오프셋으로 구분한다.
- 시작 경계는 `13f716`의 primal push 호출 직전이다. 종료 경계는 push 반환 직후이며 C 반환, workspace 상태, 재시도 출력, 남은 후보 수를 별도로 다룬다.
- 입력은 workspace·factor·행렬·basis·내부 상태·bound·primal 시작 벡터·후보·파라미터·카운터·work 문맥이다. 수명과 정확한 필드·순서는 아래 표로 확정한다.
- 각 분기에 원본 파일·줄 또는 ELF 주소를 연결한다. 불명확한 ABI·SIMD·확장 정밀도는 asm과 대조하며 확인하지 못한 항목은 후속 작업으로 남긴다.
- 이번 합격은 분기별 함수·상태·재현 경계·합성 사례를 정의하고, 근거·해시·문서 링크·`git diff --check`를 확인하는 것이다. 합성 실행 가능한 작은 경계가 확인되면 별도 명시한 범위에서 원본을 실행한다.
- 전체 primal push 자연 실행 오라클, Rust 구현·동등성, dual push, cleanup, 시간·work 패리티는 제외한다. 오라클 구축이 추가로 필요하면 M3-04b-1로 분리한다. M3-04c는 검증 가능한 한 분기만 구현한다.
- 후속 비교는 상태·인덱스·순서를 정확히, 분기에 쓰는 double은 ±0를 포함해 비트 단위로 비교한다. NaN과 내부 무한대 sentinel을 일반 오차식으로 합격시키지 않는다. x87 값은 80비트 유효 부분과 저장 순서를 별도 계약으로 고정해야 한다.
- 관련 기준은 [스프린트 검증](../15_sprint_verification.md), [원본 포팅](../10_porting.md), [M3 설계](sprint_design.md), [선행 진입 오라클](crossover_entry_oracle.md)이다.

## 2. 조사 결과

primal push를 후보 구성, 선형계 재계산, 후보 FTRAN, 방향·ratio 선택, bound 이동 또는 basis 교환, 재분해·재처리, 종료 정리로 나눴다. `055f68`의 작은 합성 경계는 원본 독립 3회로 확인했다. 전체 push의 자연 실행과 Rust 동등성은 미검증이다.

### 2.1 원본 식별과 ABI

주소는 ELF file offset이다. Ghidra 주소는 이 값에 `0x100000`을 더한다. 실행 ELF는 `../.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so`이며 SHA-256은 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`다. C·도구·캡처 해시는 [manifest](../../bench/oracle/m3_primal_push/manifest.json)에 고정한다.

| 역할 | 함수와 ELF 위치 | C 근거 |
|---|---|---|
| 호출·재시도 제어 | `13f716@0x1bc5d0` | [C](../../../re/decomp/c/001bc5d0_PRIVATE000000000013f716.c) 851–878, 1130–1224 |
| double primal push | `134a87@0x1c0070` | [C](../../../re/decomp/c/001c0070_PRIVATE0000000000134a87.c) 전체 2,662줄 |
| x87 primal push | `256781@0x2937b0` | [C](../../../re/decomp/c/002937b0_PRIVATE0000000000256781.c) 전체 2,777줄 |
| 후보·변환 열 공급 | 같은 이름 `25615c@0x1c5310`, `25615c@0x298ea0` | [double](../../../re/decomp/c/001c5310_PRIVATE000000000025615c.c), [x87](../../../re/decomp/c/00298ea0_PRIVATE000000000025615c.c) |
| feasible basis ratio | `055f68@0xef960`, `1ef3c1@0x2428f0` | [double](../../../re/decomp/c/000ef960_PRIVATE0000000000055f68.c), [x87](../../../re/decomp/c/002428f0_PRIVATE00000000001ef3c1.c) |
| infeasible basis ratio | `054eef@0xeefa0`, `1ee42f@0x241da0` | [double](../../../re/decomp/c/000eefa0_PRIVATE0000000000054eef.c), [x87](../../../re/decomp/c/00241da0_PRIVATE00000000001ee42f.c) |
| ratio 재시도 | `056973@0xf0120`, `1efda1@0x243150` | [double](../../../re/decomp/c/000f0120_PRIVATE0000000000056973.c), [x87](../../../re/decomp/c/00243150_PRIVATE00000000001efda1.c) |
| factor 갱신·재분해 | `05b257@0xf15c0`, `1f4140@0x244cd0` | [double](../../../re/decomp/c/000f15c0_PRIVATE000000000005b257.c), [x87](../../../re/decomp/c/00244cd0_PRIVATE00000000001f4140.c) |

`S = *(model+0xe0)`를 workspace로 둔다. 호출자는 `S+0x28==0`이면 double, 아니면 x87 함수를 선택한다. x87은 다른 basis 형식이라는 기존 노트의 추론을 넘어, 16바이트 간격의 80비트 값과 `fldt/fstpt`를 사용하는 정밀도 변형임을 C와 ELF로 확인했다. 이 ELF 구간에는 `fldt/fstpt`가 합계 396개 있다. double의 수치 연산을 그대로 복사한 뒤 f64로 저장하면 이 경로를 복원할 수 없다.

두 push 함수는 `xmm0=FeasibilityTol`, `rdi=S`, `esi=진입 모델 상태`, `edx=시도 번호`, `rcx=primal 벡터 앞`, `r8=행 slack 앞`, `r9d=먼저 실행되는 push인지`를 받는다. 이어지는 stack 인자는 `double *condition`, `int *retry`, `int *remaining`, 계측 배열, work 문맥이다. C의 x87 첫 인자 `undefined8`을 정수 레지스터 인자로 해석하면 안 된다. ELF `0x293968`에서 `xmm0`를 저장한다.

`13f716`의 primal-first 호출은 마지막 정수 flag가 1이고 dual-first 뒤 호출은 0이다. 이 flag는 simplex phase 번호가 아니다. 시도 번호도 독립적이며 숨은 설정에 따라 첫 값이 0 또는 1이다. `Crossover=1·2`는 dual-first, `3·4`는 primal-first다. 자동 -1은 `local_60>1`에서 primal-first로 시작하고 `local_98>=2`의 재시작 문맥에서는 순서가 달라질 수 있다. 값 0은 정상 진입을 막는 상위 게이트와 구분한다. 내부 본체에 이미 들어온 값 0의 분기는 dual-first다.

push의 C 반환, `S+0xf0` 상태, `*retry`, `*remaining`은 서로 다르다. 호출자는 C 반환이 0이어도 `S+0xf0!=1`이면 다음 push를 생략한다. 첫 시도의 retry 요청은 두 번째 시도로 이어질 수 있고, 두 번째 시도에서는 뒤 push를 건너뛰는 분기가 있다. 따라서 `return 0`을 push 완료로 바꾸면 안 된다.

### 2.2 읽기·쓰기 상태

다음 표의 배열은 현재 모델·축약 모델의 열 n개와 행 slack m개 순서다. 공개 VBasis·CBasis 번호와 내부 상태 번호는 다르다. double은 `134a87` 162–220줄, x87은 `256781` 155–213줄에서 읽는다.

| 상태 | double offset | x87 offset | 의미와 변경 |
|---|---|---|---|
| 차원 | `S+0x68=n`, `S+0x64=m` | 동일 | 행렬·basis 길이 |
| 내부 상태 | `S+0x168` | 동일 | n+m 정수. 비음수는 basis 위치, -1 하한, -2 상한, -3 push 대상, -4 고정. 초기 처리에는 -5의 상한 대입도 존재 |
| CSC | `S+0x70/78/80/88` | 동일 | 열 시작·길이·행 인덱스·double 계수. slack 열은 단위 벡터로 구성 |
| 목적·rhs·bound | `S+0x98/a0/a8/b0` | 동일 | 목적·rhs·하한·상한. 행 slack은 n 이후 인덱스 |
| 후보 scratch | `(S+0x208)+8` | `(S+0x258)+8` | 후보 인덱스. 같은 구조의 `+0x10`은 basic width 배열 |
| 후보 거리 scratch | `S+0x1e0` | `S+0x230` | double 또는 x87 수치 배열. C에서 double 값으로 읽는 표현은 포인터 재해석 |
| basic 값·잔차 | `S+0x1e8/1f8/200` | `S+0x238/248/250` | lower shift된 basic 값, 잔차 검사 scratch, 갱신 rhs. 재계산과 이동에서 변경 |
| 희소 벡터 | `S+0x210/218/220/228` | `S+0x260/268/270/278` | FTRAN 후보·BTRAN 등 scratch. count<0은 dense 표현 |
| basic 표시 | `S+0x280` | 동일 | m byte. `B/A/N/U/L/X/F` 분류 |
| factor와 header | `S+0x470`, 그 `+0x18` | `S+0x498`, 그 `+0x18` | LU 상태·basis header. pivot·재분해 뒤 다시 읽어야 함 |
| 무한대·허용오차 | `S+0x370/380/390/398/3b0` | 동일 | 내부 무한대 I, feasibility τ, Markowitz, ratio pivot 임계값, 고정 폭 임계값 |
| 반복·수치·재분해 | `S+0x2a0/178/184/294/46c` | 동일 | 반복, 수치 문제, 실패 flag, factor 요청, 갱신 한도 |
| 한 단계 선택 | `S+0x2fc/2f8/328/32c` | 동일 | 진입 열, 이탈 위치, 방향, 이탈 열 |
| 출력 | `S+0x120`, 외부 포인터 3개 | 동일 | PInf, condition·retry·signed remaining |

호출자가 건네는 primal 벡터와 slack은 앞·뒤가 이어진 같은 할당이다(`13f716` 272–283줄). push는 할당 중 `param_5`와 `param_6`의 이 관계를 이용한다. 자연 실행 오라클은 주소 대신 이 별칭 관계와 차원을 저장해야 한다. 후보 scratch·basic 값·FTRAN 벡터·factor 내부 객체는 쓰기 가능하며 재분해 중 header와 벡터 포인터가 바뀔 수 있다.

### 2.3 단계별 분기

아래 줄 번호는 double C다. x87은 같은 단계에 별도 커널과 16바이트 수치 배열을 사용한다. x87의 조건별 비트 일치는 아직 검증하지 않았다.

| 단계 | 진입 조건·처리 | 다음 경계 | 근거 줄 |
|---|---|---|---|
| 준비 | 후보·재시도·지원 집합 scratch 할당. 스레드 수는 최대 64, batch 수는 `min(threads, trunc(0.5*log(10*m)))`, 2 미만이면 0 | 실패는 10001, 아니면 초기화 | 221–400 |
| 초기 factor | primal-first flag가 1이면 재분해 요청 10, `05b257` 반환 1을 반복. flag 0은 바로 rhs 구성 | 상태·수치 실패는 retry 또는 반환 | 1018–1051 |
| 후보·rhs 구성 | rhs 복사. 비기저 열의 -1/-4는 lb, -2/-5는 ub, -3은 시작값을 빼고 -3을 후보에 넣음. slack도 내부 상태로 분리 | 후보 거리·정렬 | 401–603 |
| 후보 정렬 | 열 key `-(nnz_j+min(distance,0.99))`, slack key `-1-min(distance,0.99)` | 원본 `aabbe3` 내림차순. 먼저 소비하는 것은 짧은 열·작은 거리 쪽 | 614–640, sorter 129–155 |
| basic 재계산 | `09cf9b`로 rhs 변환, `089d8f`로 잔차 검사. 잔차와 해 크기로 Markowitz를 0.0001·0.25·0.5 중 강화 | lower shift·표시·PInf | 649–867 |
| 후보 기준 조정 | PInf>1, 이전 PInf가 τ 미만, 후보 수가 큰 거리 계수보다 크고 아직 조정 안 했으면 입력 FeasibilityTol을 0.001·0.01·0.1배 | -1/-2 일부를 -3으로 바꾸고 후보·rhs를 한 번 재구성 | 895–1016, 1063–1141 |
| 후보 공급 | `25615c`로 후보 한 개와 FTRAN 열 획득 | 방향·ratio 선택 또는 재처리 라운드 종료 | 1143–1176, 2232 |
| 방향·ratio | basic 위반 수가 0이면 `055f68`, 아니면 `054eef`. 후자는 후보<0 또는 abs(pivot)<0.001이면 `056973` 재시도 | 무한대면 반대 방향, 이후 bound 이동·pivot·보류 | 1209–1591 |
| bound 이동 | theta가 진입 열의 bound 거리 -1e-10 이상이면 이탈 후보를 지움. 이탈 후보<0도 이동 경로 | 진입 상태 -1/-2 또는 free -3. basis header 보존 | 1589–1613, 2063–2083 |
| basis 교환 | 이탈 후보가 있고 abs(pivot)>=1e-6, theta*abs(pivot)>=-1e-5 | 희소 후보 위치를 행 위치로 변환, header·상태·basic 값·rhs 갱신 | 2084–2222 |
| 후보 보류 | pivot 크기·이동량 조건 실패 | 후보를 보류 앞부분에 쓰고 실패 횟수 증가. 3회 이상 후보는 공급자가 건너뜀 | 2226–2232, 공급자 587–592 |
| factor 갱신 | pivot 뒤 factor 요청 2·10 또는 기존 요청, batch 상태를 반영해 `05b257` | 반환 1 반복, 재분해면 basic·width·PInf 재계산 | 1725–1997 |
| 재처리 | 보류 후보·미소비 batch·후속 후보를 원래 구간 순서로 모음. 진행이 적으면 지원 집합 모드 해제 | 요청 10으로 재분해 후 재시도 또는 종료 | 2235–2418 |
| 마무리 | primal-first일 때 basic 해를 외부 x/slack에 기록. PInf·condition 검사 후 남은 후보 수의 부호 결정 | 성공·retry·잔여 후보 출력 후 scratch 해제 | 2419–2660 |

이 표는 실행 순서를 나타낸다. C의 주소순 label 배치를 위에서 아래로 한 번 읽는 방식으로는 초기 factor·기준 조정·반대 방향 재시도를 복원할 수 없다.

### 2.4 후보·방향·ratio의 계약

초기 basic 분류는 lower shift된 값 v와 폭 w를 사용한다. free는 `F`, `v < -τ`는 `B`, `-τ <= v < τ`에서는 w=0이면 `X`, 아니면 `L`이다. 나머지는 `v < w-τ` 또는 w>=I이면 `N`, `w+τ <= v`이면 `A`, 중간은 `U`다. 위반 수와 PInf 합에는 `B`의 -v와 `A`의 v-w만 들어간다(816–866줄). 이 등호 위치를 바꾸면 ratio 입력이 달라진다.

후보 공급자 `25615c@0x1c5310`는 단순 인덱스 pop이 아니다. batch가 있으면 변환 열을 미리 계산하고 앞선 pivot을 적용한다. 단일 경로는 후보 열을 CSC 또는 slack 단위 열로 만든 뒤 `09d008`로 FTRAN한다(577–686줄). 지원 집합 모드에서는 표시된 행과 겹친 후보를 보류한다. 사용 횟수·누적 nnz와 dense→sparse 정리도 갱신한다. 같은 이름의 x87 공급자에는 batch record 간격도 `0x60`에서 `0x70`으로 바뀐다.

feasible basis에서는 `r = c_j - Σ(alpha_i*c_basis_i)`, `dL=x_j-lb_j`, `dU=ub_j-x_j`, `e=0.01*τ`로 방향을 고른다. 아래 식이 참이면 `S+0x328=1`을 쓰고 FTRAN 열의 부호 비트를 뒤집는다(1210–1323줄).

```text
(10*dL < dU && -(r*dL) < e)
|| ((dL <= 10*dU || e <= dU*r) && 0 < r)
```

위반이 있으면 표시 `B/A`에 따른 `Σalpha`를 추가로 계산하고, reduced cost·100τ·10배 bound 거리 조건을 함께 사용한다(1333–1454줄). 이 분기는 feasible 식으로 대체할 수 없다. ratio가 I이면 free 여부와 현재 방향의 유한 bound 여부를 보고 방향을 뒤집어 한 번 더 호출한다(1464–1591줄).

push가 부르는 `055f68`은 theta 초기 상한을 **I**로 받는다. ELF `0x1c2884–0x1c28b8`과 `0x1c4ca8–0x1c4ce6`에서 `xmm2`를 `xmm3`에 복사한다. 진입 열의 bound 거리와 비교해 flip을 결정하는 것은 이 ratio 호출 다음의 push 코드다. 아래 합성 오라클의 유한 상한 사례는 helper 자체의 경계 검사이며 자연 push의 유한 상한 호출을 관측한 것으로 세지 않는다.

### 2.5 basis 변경·재분해·반환

pivot이 승인되면 먼저 sparse 위치 p에서 실제 행 r을 얻고, sparse 마지막 원소로 p를 채워 count를 하나 줄인다. `state[enter]=r`, `leave=header[r]`, `header[r]=enter`를 기록한다. leave의 폭·basic 표시·방향·이동량에 따라 상태를 -4, -1, -2 중 하나로 고른다. 이탈 변수를 항상 이동 방향의 bound로 보내는 단순 규칙은 원본과 다르다. 이어 basic 폭·값·위반 수를 갱신하고 열별 rhs를 원본 순서로 보정한다(2084–2222, 1614–1718줄).

factor 업데이트 중에는 `S+0x210`과 `LU+0x1c8`을 임시 교체한다. 반환 1이면 갱신을 반복하고 각 포인터를 복원한다(1758–1883줄). factor 요청이 남으면 header를 다시 읽고 basic 값·폭·표시·PInf를 재계산한다. LU 업데이트 횟수·채움·수치 카운터와 batch 상태가 다음 분기에 쓰이므로 basis 배열만으로 다음 pivot을 재생할 수 없다.

| 반환 경로 | 상태와 출력 | double C 근거 |
|---|---|---|
| 메모리 부족 | C 반환 10001. 조기 할당 실패는 remaining=0을 쓰고 해당 scratch 해제 | 327–332, 1054–1061, 2582–2660 |
| 중단·한도 | `a83b2f` 또는 `08b129` 결과를 `S+0xf0`에 기록. C 반환은 0일 수 있음 | 1186–1207 |
| factor 실패 | 코드 12는 retry 요청으로 바꾸는 경로. 1 초과의 다른 코드는 C 반환으로 전달 | 1021–1051, 1886–1897, 2294–2329 |
| 내부 특수 상태 | `S+0xf0 < -999`이면 1로 돌리고 retry=1 | 1043–1047, 2311–2322 |
| 품질 부족 | 첫 시도에서 condition>1e14 또는 PInf>1e10 또는 둘의 곱>1e16. PInf 항은 진입 상태 2에서 사용. condition>1e30은 별도 검사 | 877–896, 2483–2494 |
| 과도한 PInf | 초기 PInf>1e15이면 수치 카운터를 올리고 retry=1 | 891–896 |
| 잔여 후보 | remaining은 signed다. free 후보가 남거나 자동 설정의 잔여 수 조건에 따라 `-count`를 쓸 수 있음 | 2498–2524 |
| 양수 잔여 | 후보 상태를 시작값과 양 bound의 중점 비교로 -1/-2에 둠 | 2526–2541 |
| 음수 잔여 | n+m 범위에서 이미 -1/-2인 외부 값을 해당 bound로 대입 | 2544–2580 |

`remaining<0`은 오류 코드가 아니다. 호출자는 이후 cleanup 선택에서도 이 부호를 읽는다(`13f716` 1525–1526, 1660–1678줄). `retry`와 별도로 저장하고 비교해야 한다.

## 3. 합성 ratio 오라클 계약과 결과

이 조사의 실행 경계는 `055f68@0xef960` 직접 호출 전후다. factor의 실제 상태가 필요한 전체 push 대신, factor 조회에 필요한 갱신 횟수 0의 합성 객체와 workspace를 사용한다. 정상 LP·자연 push 상태라는 주장은 하지 않는다. x87 ratio·infeasible ratio·FTRAN·basis 교환·할당 실패·중단은 실행하지 않았다.

ABI는 double 4개와 정수·포인터 14개다. `xmm0=1e-6`, `xmm1=1e-9`, `xmm2=1e30`, `xmm3=초기 이동 상한`이며 `rdi=workspace`, `esi=m`, `edx=0`으로 열길이 가중 보정을 끈다. `rcx=열길이`, `r8=header`, `r9=희소 벡터`, stack에는 basic 값·폭·표시·후보 scratch·ratio scratch·theta 출력·abs(pivot) 출력·work 출력이 들어간다. 입력 객체는 호출이 끝날 때까지 보존한다. ELF prologue `0xef966–0xef9c3`에서 레지스터와 stack을 확인했다.

| 필드 | 비교 규칙 |
|---|---|
| 사례명·선택 위치·실제 행 | 순서와 정수 정확 비교. 희소 반환 위치와 실제 행을 별도 저장 |
| theta·abs(pivot) | little-endian f64 8바이트를 hex로 저장. ±0까지 정확 비교 |
| work | helper 입력 계수 `workspace+0x148=0.25`, 시작값 0. 원본 3회 비트 비교만 수행. 전체 솔버 work 패리티와 구분 |
| 입력 보존 | alpha·indices·basic 값·폭·표시·header·열길이의 전체 바이트 보존 여부 |
| factor 요청·이탈 상태 | `workspace+0x294/0x2f8` 정확 비교. 이번 사례에서는 모두 초기값 0 유지 |
| 출처 | 원본 합성임을 고정. ELF·소스·도구·입력 hash를 실제 파일·정의와 대조 |

[도구](../../tools/primal_push_ratio_oracle.py)의 `cases()`가 전체 입력과 사전 기대값이다. seed·RNG·스레드·타이머를 소비하지 않는 이 helper 경계에서는 별도 설정이 없다. 입력은 유한값과 내부 I, ±0만 사용하며 NaN은 포함하지 않는다.

| 사례군 | 확인한 조건 | 수 |
|---|---|---:|
| lower·upper | 양수 alpha의 하한, 음수 alpha의 상한 ratio | 4 |
| tie·tie-reversed·larger-pivot | 같은 크기 첫 후보 유지, sparse 순서 반전, 더 큰 pivot 선택 | 5 |
| free·upper-infinite | free 양수 alpha 또는 무한 상한 음수 alpha에서 선택 없음 | 4 |
| pivot-at-tol·pivot-above-tol | strict 피벗 임계값과 작은 pivot의 최대 계수 검사 | 4 |
| negative-zero | theta=-0 비트 보존 | 2 |
| empty·zero-alpha | 빈 벡터 또는 alpha=0에서 theta=I·pivot=0 | 4 |
| finite-flip·flip-tie·flip-relaxed-tie | 유한 초기 상한·정확 동률·Harris 허용폭 안에서 flip | 6 |
| 합계 | sparse 15개, dense 14개 | 29 |

원본 독립 프로세스 3회에서 29사례씩, 총 87호출을 실행했다. 사건 필드 9개가 모두 일치했으며 theta·pivot은 사전 기대 비트와도 일치했다. 저장 파일은 [ratio-1](../../bench/oracle/m3_primal_push/ratio-1.json), [ratio-2](../../bench/oracle/m3_primal_push/ratio-2.json), [ratio-3](../../bench/oracle/m3_primal_push/ratio-3.json)이다. 비교기는 최소 3개 캡처·모든 사건·필드를 요구한다.

변조 10개를 거부했다. ELF·입력·도구·소스 hash 0, 사건 비움, 자연 실행 출처로 변조, theta·선택 행·반환 위치 변조, pivot 누락이다. CLI에서도 별도 변조 검사를 수행하고 [검증 결과](../../bench/oracle/m3_primal_push/verification.json)에 실제 종료 코드를 기록한다. 원본 실행 첫 시도는 다른 Gurobi PID의 guard에 거부됐고 캡처를 만들지 않았다. 기존 `gurun.sh`로 기다린 뒤 3회를 순차 실행했다.

## 4. 후속 분할과 미확인 범위

| 번호 | 다음 산출물·시작 경계 | 필수 사례와 완료 조건 |
|---|---|---|
| M3-04b-1 | 정상 crossover에서 `134a87` 진입·공급자·ratio·교환·반환을 읽기 전용 훅으로 캡처 | primal-first와 dual-first, 목표 pivot·flip 각 1회 이상, 독립 3회. 목표 0회·핵심 상태 누락·합성만 존재하면 합격 거부 |
| M3-04c | double feasible 방향·ratio·bound 이동 또는 단일 pivot 중 **하나만** 구현 | 04b-1의 같은 입력 상태와 사건 순서 비교 후 구현 완료 판정. 전체 primal push로 승격하지 않음 |
| M3-04c-1 후보 | 초기 후보 구성·내림차순 정렬·기준 조정 | -1/-2/-3/-4/-5, bound 동률, τ 경계, 재구성 전후 rhs·후보·표시 비트 비교 |
| M3-04c-2 후보 | 위반 basis 방향·두 ratio·반대 방향 | B/A 표시와 0.001 경계, free의 두 방향, 1e-10 flip 경계 |
| M3-04c-3 후보(후속 분할 확정) | batch·지원 집합·재분해·재처리 | [04c-12 조사](primal_push_loop.md#43-확정한-후속-작업)에서 13–23으로 구체화. 포인터 복원, 후보 순서·거부 횟수, 요청 2·10, factor 반환 1·12·오류, 중단 상태 |
| M3-04c-4 후보 | x87 primal push 분기 | 80비트 입력·출력·중간 저장 시점·정밀도 전환 문맥을 새 계약으로 고정 |

`04c-1` 이후는 이번 조사에서 찾은 분할 후보이며 구현 완료 번호가 아니다. M3-04d의 dual push와 M3-04f의 cleanup은 기존 범위로 유지한다. 재시작 전체·고급 매핑·실패 상위 처리도 04a-2·04g와 연결해야 한다.

전체 push의 자연 입력에는 S·factor·후보 batch와 지원 집합 문맥이 필요하다. 입력 LP 파일만 고정해서는 충분하지 않다. 원본 allocator·factor를 합성해 전체 push를 직접 호출하는 방법은 이번에 검증하지 않았다. helper의 작은 pivot 사례도 강화 임계값으로 다시 ratio를 계산하는 루프에는 들어가지 않는다. 그 재시도와 LU 갱신 횟수>16의 요청 2는 미검증이다. 우선 자연 훅에서 상태를 수집한 뒤 독립 재생 가능한 경계를 정한다. 이 조사에서 만든 29사례를 전체 push 캡처 29건으로 세면 안 된다.

## 5. 재실행과 판정

작업 저장소 루트에서 저장 기준을 검사한다. 저장 검사에는 Python 표준 라이브러리와 원본 C 식별 자료가 필요하다. 원본 캡처에는 기존 Gurobi 환경·guard를 사용한다.

```bash
python3 tools/primal_push_ratio_oracle.py --check bench/oracle/m3_primal_push/ratio-{1,2,3}.json --selftest
for i in 1 2 3; do
  tools/gurun.sh ../.venv/bin/python tools/primal_push_ratio_oracle.py --capture "bench/oracle/m3_primal_push/review-$i.json"
done
python3 tools/primal_push_ratio_oracle.py --check bench/oracle/m3_primal_push/review-{1,2,3}.json
python3 -m py_compile tools/primal_push_ratio_oracle.py
git diff --check
```

캡처는 기존 파일을 덮어쓰지 않는다. 새 검토 파일명으로 실행한다. 위 `review-*`는 재실행 명령이며 이번에 실행한 이름은 `ratio-{1,2,3}.json`이다. ELF 재확인은 다음 경계를 사용한다.

```bash
sha256sum ../.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so
objdump -d --start-address=0xef960 --stop-address=0xef9d0 ../.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so
objdump -d --start-address=0x1c2841 --stop-address=0x1c28bd ../.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so
objdump -d --start-address=0x2937b0 --stop-address=0x293980 ../.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so
```

조사 수행은 완료했다. 합성 helper의 지정 범위에서 원본 재현성과 비교기 검사는 통과했다. Rust 구현·동등성, 전체 push·x87·infeasible ratio·factor·중단 경로는 미검증이다. 솔버·Python API를 바꾸지 않아 Cargo·Clippy·확장 통합 회귀는 실행하지 않았다.

작업 commit과 병합 상태는 [M3 인수인계](../handoff/M3.md)와 git 이력으로 추적한다. 상세 기록·검증 자료·스프린트 목록·진행·로드맵은 작업 branch에서 갱신해 `--no-ff`로 main에 병합한다. 병합 직후 작업 worktree·branch를 정리하고 M3-04b·shared 점유를 해제한다.
