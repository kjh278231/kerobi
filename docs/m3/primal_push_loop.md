# M3-04c-12 push 루프 본체 분할 조사

## 1. 범위와 주장

- 종류: 조사. `134a87`의 공급자 `25615c`, factor 갱신 판단·호출, 재분해 뒤 basic 재계산, 보류 후보 재처리를 독립 검증 가능한 후속 번호로 나눈다.
- 질문: 각 구간은 무엇을 읽고 쓰며 어떤 순서로 연결되는가? 기존 캡처와 Rust API로 어디까지 재생할 수 있고, 추가로 필요한 원본 상태는 무엇인가?
- 선행: main `ec76fd6`, M3-04c-10·11. `work/m3-04c-12`는 같은 commit에서 분기했고 미커밋 선행 변경은 없다. 중단된 Claude 세션의 점유를 사용자 재개 지시에 따라 Codex로 인계했다.
- 포함: double·Threads=1을 우선 구현할 분할, batch와 오류 분기를 별도 계약으로 등록. 기존 자연 캡처를 분석하되 새 캡처나 구현 동등성 검증으로 세지 않는다.
- 제외: Rust 솔버 구현, 새 원본 실행, 전체 crossover·dual push(M3-04d), x87(M3-04c-4), 초기 후보 구성(M3-04c-1), 전체 work·시간 패리티.

## 2. 원본과 상태 경계

- 원본: Gurobi 13.0.3. 바이너리 식별값은 기존 `bench/oracle/m3_push_chain/manifest.json` 및 개별 캡처와 연결한다. 이번 조사는 저장 C·asm 및 기존 캡처를 읽는다.
- C: `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c`, `001c5310_PRIVATE000000000025615c.c`, `000f15c0_PRIVATE000000000005b257.c`. ELF file offset 기준이며 Ghidra 주소는 +0x100000이다.
- ABI: `25615c(local*, S*, alpha_out**, support*, work*)`. S+0x470은 factor, S+0x210은 후보 alpha scratch. 입력 상태·별칭·쓰기 순서와 세부 경계는 조사 결과에 기록한다.
- 시작: 초기화 이후 첫 공급자 호출. 종료: 다음 공급자 호출, 재처리, 또는 push 반환. 초기화·마무리는 선행 조사와 후속 연결의 경계로만 다룬다.

### 2.1 자료와 원본 위치

바이너리 SHA-256은 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`다. 로컬 라이브러리의 해시도 확인했다. 아래 C 파일의 전체 해시는 [조사 집계](../../bench/validation/m3_push_loop/survey.json)의 `sources`에 고정한다. asm은 `tools/dc.py <정확한 offset> --asm --range <시작>,<끝>`으로 해당 바이너리에서 읽었다. 저장 asm 디렉터리에 두 함수의 파일이 있다고 가정하지 않는다.

| 구간 | C 파일·줄 | 확인할 ELF 지점 |
|---|---|---|
| 단일 공급·건너뛰기·FTRAN | `001c5310…25615c.c` 573–686, 공통 압축 526–568 | `0x1c5d5e–0x1c5e55`: support를 양수 또는 0으로 정규화, failure>2, cursor 증가, 보류 목록 쓰기 |
| batch 계산·소비 | 같은 파일 71–525 | 공급자 진입 `0x1c5310`, record 간격 0x60 |
| 지원 집합 유지·factor 판단 | `001c0070…134a87.c` 1725–1752, 1999–2062 | `0x1c336d–0x1c3458`: 요청 우선·정수 비교·수식·support별 호출 생략 |
| factor 호출·복원·후처리 | 같은 파일 1753–1997 | `0x1c3465` 진입, `0x1c34a2/0x1c34a9` S+0x210 저장·교체, `0x1c3b23–0x1c3b79` 품질 검사·요청 지우기 |
| 재처리 라운드 | 같은 파일 1143–1176, 2226–2418 | `0x1c3b7b` 보류, `0x1c3b9f` 공급 재호출, `0x1c3bdf–0x1c3d16` 종료·압축·요청 10 |
| 재계산 | 같은 파일 1897–1980, 2331–2418 | `09cf9b@0x1205d0` 전후 + 다음 공급자 직전 |
| 품질 검사 helper | `00115ed0…089f61.c` 36–97, 238–272 | `089f61@0x115ed0`, 호출 `0x1c3b38` |
| LU 실제 갱신·재분해 | `000f15c0…05b257.c` 144–145, 1327–1341, 2435 이후 | `05b257@0xf15c0`: 요청 0은 갱신 경로, 비영 요청은 재분해 경로 |

### 2.2 공급자의 상태와 단일 경로

`P=&local_20d0`를 공급자 문맥으로 쓴다. 함수 이름 `25615c`는 x87에도 있으므로 항상 `0x1c5310`으로 특정한다. `S`는 simplex workspace, `F=*(S+0x470)`다.

| 위치 | 의미·수명·변경 |
|---|---|
| P+0 / +4 | 행 수 m / 현재 후보 총수 |
| P+8 / +0x10 / +0x18 | 후보 배열 / 열별 실패 횟수(n+m) / 지원 표시(m) 포인터 |
| P+0x20 / +0x24 | 다음 소비 위치 / 보류 앞부분 길이. 두 값은 같지 않다 |
| P+0x30 / +0x38 | 공급 성공 시 압축 **전** alpha count를 더하는 f64 합 / 성공 호출 수. dense의 음수 count도 그대로 합산한다 |
| P+0x40 + k·0x60 | batch record의 alpha 벡터. +0x58 원본 열, +0x70 spike 벡터, +0x88 후보, +0x8c pivot 행, +0x90 pivot, +0x98 방향 |
| P+0x1840 / +0x1844 / +0x1848 | batch 용량 / 채운 개수 / 소비 위치 |
| P+0x1850 / +0x1858 / +0x185c | 희소 위치 역색인 / batch 생성 시 F+0x180 값 / worker 한도 |
| `*support` | 양수만 유지, 음수는 0으로 정규화. m byte의 basic 분류 표시와 별도인 int 값 |
| alpha 입출력 | 원본 열 scratch S+0x220 → `09d008` → 단일 경로 S+0x210. 반환 포인터는 다음 단계가 열을 반전·pivot 원소 삭제하면서 수정함 |

용량 0이고 미소비 batch도 없는 입력을 첫 구현 전제로 한다. 후보 목록의 인덱스는 처음부터 `[0,n+m)`여야 한다. C는 음수 여부 검사보다 먼저 실패 배열을 읽으므로 음수 후보를 합성 목록에 넣어 안전한 skip 사례로 쓰면 안 된다.

단일 경로는 다음 순서다.

1. `*support=max(*support,0)`을 기록하고 출력 alpha 포인터를 S+0x210으로 정한다.
2. 후보를 읽고 **먼저** P+0x20을 증가시킨다. `failures[j]>=3`이면 그 후보를 `candidates[deferred]`에 쓰고 deferred를 증가시킨다. 이 경우 FTRAN하지 않는다.
3. support가 양수이면 구조 열의 CSC 행 또는 slack의 `j-n` 행을 P+0x18 표시와 대조한다. 하나라도 겹치면 같은 보류 경로다. 비교 좌표는 원래 제약 행이며 basic 위치가 아니다.
4. 구조 열은 CSC의 저장 순서로 복사하고 slack은 `(j-n,1.0)`으로 만든다. support=0이면 `09d008`에 F+0x1c8 spike 저장 벡터를 넘기고, support>0이면 null을 넘긴다(C 629·679).
5. 성공 수와 count 합을 갱신한다. alpha가 dense이면 `S+0x3a8 < abs(value)`인 원소만 행 번호 오름차순으로 in-place 압축한다. `<=`인 값은 버린다. 이미 sparse이면 그 목록 순서를 유지한다.
6. 끝까지 건너뛰면 -1을 반환한다. 이때 반환 alpha의 기존 내용은 성공 출력으로 취급하지 않는다.

공급자가 건너뛴 후보와 단계 판단에서 pivot을 보류한 후보는 같은 앞부분 배열을 공유하지만, **실패 횟수 증가는 단계 보류(C 2226–2231)만 한다.** 공급자의 failure>=3 skip에 다시 +1을 붙이면 원본과 다르다.

### 2.3 지원 집합은 LU를 매번 갱신하지 않는 경로다

시작 support는 `후보 수 >= sqrt(m)`(C 908–910)이다. 승인 pivot의 leave가 음수가 아니고 support가 비영이면 `BTRAN(e_leave)`를 계산한다(C 1999–2023). 희소 출력의 제약 행들을 지원 표시에 추가한다. 다음 공급자는 이 행과 겹치는 원본 열을 보류한다. dense BTRAN이면 support=-1, 표시 초기화·후보 압축 뒤 라운드 재분해로 간다(C 2024–2062). 다음 공급자는 -1을 0으로 바꾼다.

factor 판단에서 요청이 0으로 남고 support가 비영이면 **`05b257` 호출을 생략**한다(C 1750, asm `0x1c343d–0x1c344b`). basis header·vstat의 교환과 LU 갱신은 같은 사건이 아니다. 이 구간에서는 header를 바꾼 뒤에도 factor의 변환은 이전 상태에 남는다. 매 pivot마다 현재 header로 factor를 새로 만들거나, 무조건 FT 갱신하는 backend는 이 경로를 바꾼다. 기존 arms·shift 캡처에서 공급 단계 다수가 support=1이고 factor는 라운드 경계에서만 호출된다. 수치적 정당성 전체는 이 조사에서 새로 검증하지 않았다.

### 2.4 factor 요청 선택과 실행을 분리한다

factor 처리는 승인 pivot 뒤에 온다. flip·보류는 다음 공급자로 가므로, ratio가 기록한 요청이 있다는 이유만으로 flip 직후 factor를 호출하면 안 된다. 다음 표는 C 1725–1752와 asm `0x1c336d–0x1c3458`의 순서다. `k=F+0x18c`, `limit=F+0x188`, `q=S+0x46c`, `N=S+0x178`이다.

| 우선순위 | 판단 | 출력 |
|---|---|---|
| 1 | 기존 `S+0x294 != 0` | 요청을 보존하고 factor 호출. ratio 요청 2도 포함 |
| 2 | `k == limit` | 요청 2. `>=`로 넓히지 않음 |
| 3 | 아래 비용식 > 0 또는 `N!=0 && k>q` | 요청 10 |
| 4 | 요청 0, support=0 | FT 갱신 호출 |
| 5 | 요청 0, support!=0 | 호출 생략 후 요청 0 기록, 다음 공급자 |

비용식의 연산 순서는 다음과 같다. `L=F+0x1e0`, `U=F+0x1e8`은 int64, `C=F+0x190`은 int64, `E=F+0x184`는 int32다. `mf=f64(m)`이다.

```text
a = f64(L + U) - (mf * 1.5)       # 정수 합 후 한 번 변환
b = (mf * 5.0) * mf              # 루프 준비 때 저장
cost = 0.0                      # k < q일 때
cost = ((f64(C) + f64(E)) * 2.0) * mf - min(q*b, a*a)  # k >= q일 때
```

원본 `minsd`는 `q*b`를 목적 피연산자, `a*a`를 소스 피연산자로 쓴다. 일반 유한 상태의 위 식과 NaN 입력 계약은 구분해야 한다. `Engine::pivot`의 dual simplex 식은 `4m²`·다른 계수·`k>=interval`을 쓰므로 그대로 재사용할 수 없다. 이번에는 자연 요청 2·6·비정상 상태를 관측하지 못했다.

호출 wrapper의 계약은 요청 선택과 별도다.

- S+0x210을 현재 alpha로 임시 교체한다. batch 소비 위치>0이면 F+0x1c8도 해당 batch spike로 바꾼다. 원시 주소 대신 벡터의 의미·별칭과 복원 여부를 캡처해야 한다.
- 반환 1이면 반복한다. batch에는 저장 요청 P 문맥(`local_20a8`) 재적용·재호출과 갱신 수 0일 때 미소비 후보 회수 경로가 더 있다(C 1758–1883).
- S+0x210과 바꾼 F+0x1c8을 복원한 다음 종료 코드를 처리한다. 반환 12는 retry=1, 다른 반환>1은 오류 전달, S+0xf0<-999는 상태를 1로 복구하고 retry=1이다.
- attempt=0이며 실패 flag `S+0x184!=0` 또는 수치 카운터가 `진입 S+0x178+10` 이상이면 retry다. 두 번째 시도는 같은 게이트가 아니다.
- 남은 요청이 비영이면 §2.5 재계산을 수행한다. **요청 0의 FT 갱신 뒤에는 이 재계산을 하지 않는다.**
- 품질 검사 주기는 local_100(초기 50)이다. F 갱신 수가 이 값 이상이면 `089f61(S,enter,&increment)`를 호출한다. 요청이 생기면 local_100=20으로 두고 wrapper를 다시 돈다. 요청이 없으면 반환 increment를 더한다. 갱신 수가 0이 아니면서 주기>50이면 50으로 줄이는 경로도 있다(C 1982–1995).
- `089f61`은 진입 열을 다시 FTRAN해 단위벡터와 비교한다. 기본 increment=50, 오차 크기별 5·10·20, 허용값 S+0x3a0 초과이면 요청 6·수치 카운터·Markowitz 강화 또는 상태 12를 쓴다(C 238–272). 원본 C의 부호·max 표현과 NaN까지 구현하려면 별도 asm·실행 계약이 필요하다.
- 정상 끝에서 S+0x294=0(C 1996). helper가 만든 비영 요청을 지운 다음 검사를 시작하는 순서로 바꾸면 안 된다.

### 2.5 재분해 뒤 재계산과 라운드

재계산은 두 진입점(C 1897–1980, 2331–2418)에서 같은 분류를 수행한다. F+0x18의 **새 header**를 읽고, 갱신 rhs S+0x200을 S+0x228 벡터로 복사(별칭이면 생략)한 뒤 count=-1로 `09cf9b`를 부른다. 결과를 S+0x1e8 basic 값으로 복사한다. 이 결과가 입력이 되는 작은 분류 함수와 FTRAN 자체를 분리해 검증할 수 있다.

각 새 basis 위치 p에 대해 j=header[p], l=lb[j], u=ub[j]라 할 때, l<=-I이면 width=I이고 값은 shift하지 않는다. 그렇지 않으면 width는 u>=I일 때 I, 아니면 u-l이며 basic 값에서 l을 뺀다. free(l<=-I && u>=I)는 F다. 나머지 값 v·폭 w는 아래 순서로 분류한다.

```text
v < -h              : B, violations += 1, PInf -= v
v < h               : w == 0이면 X, 아니면 L
v < w-h 또는 w >= I : N
w+h <= v            : A, violations += 1, PInf += v-w
그 외               : U
```

순서는 basis 위치 오름차순이며 PInf 합산 순서를 바꾸지 않는다. `local_1b0` 위반 수와 S+0x120 PInf를 갱신한다. model_status=2·attempt=0이고 PInf>1e10이면 retry다. 단계별 `push_apply`는 위반 수를 갱신하지만 이 PInf 전체 재합산은 수행하지 않으므로, 두 경계를 혼동하면 안 된다.

보류 목록에는 공급자의 지원 충돌·실패 한도 skip와 단계 보류가 함께 들어간다. 공급자 -1 뒤에는 다음 순서다(C 2235–2418).

1. deferred==total이고 (deferred==0 또는 support==0)이면 라운드를 끝낸다. 모두 보류됐어도 support!=0이면 support=0으로 한 번 더 시도한다.
2. 이번 패스의 승인 flip+pivot 수가 sqrt(m) 미만이면 support=0. 이상이고 support!=0이면 지원 표시를 초기화한다. 승인 수는 S+0x2a0 누계가 아니라 별도 지역값 local_238이며 flip(C 1594)·pivot(C 2089)만 증가시킨다.
3. **이미 앞부분에 있는 보류 후보 → 미소비 batch 후보 → 아직 읽지 않은 후보** 순서로 한 배열에 모은다. 실패 횟수는 유지한다. 총수·cursor·deferred와 batch cursor를 갱신한다. 정렬·중복 제거·실패 횟수 초기화는 하지 않는다.
4. 요청 10을 기록하고 `05b257`의 반환 1을 반복한다. 실패·retry 규칙을 적용한 뒤 §2.5 재계산을 수행하고 요청을 0으로 지운다. 후보가 남으면 다음 라운드로 간다.

첫 라운드 시작(C 1143–1176), dense BTRAN 이후, batch 중 factor 갱신 수 0 이후에도 같은 압축 패턴이 있다. 빈 목록이면 factor가 항상 호출되지 않는다는 식의 일반화 대신 각 진입점별 분기를 보존한다.

### 2.6 batch의 별도 계약

초기 batch 용량은 `min(min(worker수,64), trunc(0.5*log(10*m)))`, 2 미만이면 0이다(C 221–231). 따라서 Threads=1로 검증한 자료를 batch 실행 증거로 쓸 수 없다. 공급자는 용량·누적 count/호출 수에 따라 작업 수를 조절하고 여러 열을 먼저 FTRAN한다(C 71–370). 미소비 열에는 앞서 소비한 record의 pivot·방향·alpha를 순서대로 적용하고 `09bae6(F,저장 F+0x180,spike,work)`로 뒤따른 row eta를 반영한다(C 371–525). `09c062@0x1205c0`은 F+0x180을 돌려주는 helper다.

이 record의 버퍼·역색인·worker별 work·mutex·미소비 후보 회수·재분해 무효화는 단일 경로 오라클에 없다. x87 record는 0x70이며 수치 저장 형식도 다르므로 별도 조사다. batch 제어만 보고 Threads>1의 결정적 사건 전순서나 work 비트 일치를 주장하지 않는다.

## 3. 비교 계약

이번 완료 조건은 실행 동등성이 아니라 원본 위치·필수 상태·분할·재실행 가능한 자료 분석을 확정하는 것이다.

| 대상 | 규칙 | 방법 |
|---|---|---|
| 원본 근거 | 함수의 정확한 ELF offset과 C 줄 범위, 필요한 asm 지점 | `tools/dc.py` 및 저장 C·asm |
| 기존 자료 집계 | 사건·후보·요청·지원 모드·반환값을 순서와 정수 그대로 사용 | 읽기 전용 저장 캡처 분석 |
| 미래 수치 비교 | alpha·basic·width·PInf·rhs는 ±0을 포함한 f64 비트, 후보·header·지원 표시·카운터는 순서와 정수 정확 비교 | 후속 오라클에서 새 경계를 캡처 |
| 누락 상태 | 미수집 필드는 추정 복원하지 않고 명시 | 기존 agent와 Rust API 대조 |

Threads>1·batch는 별도 경계로 나눈다. 타이머·work는 수치 경로와 별도 검증하며 기존 자료 집계를 새 원본 독립 3회로 보고하지 않는다. 미래 오라클은 동일 입력 원본 3회, 필수 사건 0회·필수 상태 누락 거부와 동작 변조 검출을 요구한다.

## 4. 사례와 원본 자료

기존 04b-1·04b-2·04b-3·04c-2·7·8·11의 자연 캡처를 조사 대상으로 삼는다. 각 작업의 manifest·검사기를 재사용하며, 본 조사에서 새 목표 사건이 실행됐다는 주장은 하지 않는다. 합성 factor 객체로 전체 push를 직접 호출하는 방법은 ABI·수명 계약이 검증되지 않아 사용하지 않는다.

### 4.1 기존 자료가 입증하는 범위와 빠진 필드

`push_loop_survey.py`는 선행 `load_all`과 04c-11 검사기를 먼저 통과한 **각 사례 첫 기록**을 집계한다. 반복 3회를 세 배로 더하지 않는다. 18기록의 사건 236단계는 기존 04c-11 연쇄 재생과 같은 입력 집합이다.

| 항목 | 저장 자료 집계 | 후속에서 필요한 보강 |
|---|---:|---|
| 성공 공급 / 고갈 / 단계 보류 | 236 / 38 / 18 | 공급 반환 때 후보 배열·deferred·failures·support의 **후상태**, 성공 count 통계 |
| factor 호출 | 73(요청 0=26, 10=47) | 호출 직전 S/F 전체 판단 필드와 caller 구간 tag |
| factor 반환 | 73개 모두 0, 요청값은 호출 전후 동일 | 반환 1 반복·12·오류, 요청 2·6, 상태<-999와 retry 게이트 |
| factor 전후 header 변경 | 33(전부 요청 10) | 새 header·vstat·factor 변환 상태의 의미 대응 |
| support | 0·1 관측, 실패 횟수 최대 3 | BTRAN 원시 출력·출력 dense 여부, support=-1·전후 지원 표시 |
| batch / 공급 반환 dense | 모두 0 | batch 별도 오라클. 공급자는 dense 출력을 sparse로 압축하므로 반환만으로 내부 dense FTRAN 0회라고 판단할 수 없음 |

`primal_push_recall_agent.js`의 factor 훅은 request·header·ret만 기록한다. 다른 사건에 F 갱신 수·한도·L/U nnz가 있더라도 정확한 요청 판단 직전 F+0x190·+0x184, S+0x46c·+0x3a0, local_100·실제 support 변화를 확보하지 못한다. 공급자 진입에는 deferred·count 통계가 없고 반환에는 전체 문맥 후상태가 없다. `support_marks`는 support>0일 때만 기록한다. 따라서 사건 집계로 요청 수식을 재검증하거나 보류 목록을 완전히 재생했다고 보고할 수 없다.

04c-10·11의 연쇄 검사는 요청 10 뒤 다음 공급자의 상태에서 새 구간을 시작하며 단계 뒤 비교 6개를 제외한다. 이번 조사에서도 그 재분해를 Rust가 계산한 것으로 승격하지 않는다.

### 4.2 Rust 재사용 경계

| 기존 API | 재사용할 부분 | 보강·주의 |
|---|---|---|
| `push_pick` / `push_apply` | 방향·ratio·선택·단계 적용 | 공급자·factor·round를 호출하는 루프는 없음. 희소 double 한정 |
| `Lu::ftran_lr` / `ftran_upper` / `ftran_dense` | 기존 L/R/U 수치 커널 | `09d008→2207de`의 count·목록 순서·spike 저장·clear까지 자동으로 검증된 것은 아님 |
| `Lu::ftran_flip` | FTRAN 표현 선택 조사에 참고 | dense 계산과 별도 경로 판정 shortcut이 있음. 실제 sparse 출력 순서·spike가 없는 API |
| `Lu::btran_dense` | 희소 목록과 dense 플래그 | 지원 표시 좌표와 dense fallback을 새 원본 경계로 대조해야 함 |
| `Lu::update`, `renumbered`, `relabel` | FT와 재분해 뒤 번호 대응 | `05b257`의 요청·오류·pointer swap wrapper와 동등하지 않음 |
| `Engine::pivot` / `reinvert` | 기존 simplex 구현의 자료 구조 참고 | private·simplex 문맥, 다른 요청 수식·갱신 정책. push의 support 경로를 대체하지 않음 |

P1 제어를 먼저 원본 수치 backend 결과로 재생하는 L1과, Rust LU를 연결해 실제 계산하는 검증을 분리한다. L1 후속 구현에 원본 숫자를 입력으로 넣었다면 전체 FTRAN·LU 일치로 보고하지 않는다.

### 4.3 확정한 후속 작업

이 표의 번호는 [스프린트 목록](../14_sprint_plan.md)에 등록한다. 오라클 작업을 통과하기 전 구현 완료를 선언하지 않는다. 모든 새 오라클은 원본 독립 3회·기존 agent 비간섭·목표 0회/필수 상태 누락 거부·출처 해시 변조와 해당 동작 변조 검출을 요구한다.

| 번호·종류 | 비교 경계·필수 상태 | 사례·완료 기준 |
|---|---|---|
| **M3-04c-13 오라클** | 단일 `25615c` 전후. P의 후보·cursor·deferred·failures·support 표시·통계, CSC/slack, FTRAN 입출력·spike 별칭 | 구조 열·slack, support 0/1/-1, 충돌 유무, failures 2/3, 고갈, sparse 순서와 dense 압축 경계. 처음은 자연 arms·retry·interior, 정확 경계는 ABI 확인 후 합성. 반환 j·전후 문맥·FTRAN 호출 인자·출력 필수 |
| M3-04c-14 구현 L1 | 13의 후보 소비·보류·압축, FTRAN은 명시적 외부 backend | 같은 입력으로 j·호출 순서·목록·통계 비트 비교. 전체 FTRAN 수치는 제외하고 21로 넘김 |
| M3-04c-15 오라클 | 승인 pivot 이후 factor 판단→호출·복원→품질 검사, batch=0 | §2.4 필드 전부. 기존 요청 보존, k=limit 전후·k=q 전후·비용식=0 경계, support 호출 생략, 반환 1/12/오류·요청 6·retry 게이트. 자연 미관측은 순수 선택 함수 합성 또는 ABI가 확인된 경계에서만 실행. 불가 범위는 분할·미검증 명시 |
| M3-04c-16 구현 L1 | 15의 순수 요청 선택 + 직렬 호출 제어. factor·품질 계산은 외부 backend | 요청값·호출 횟수·vector 별칭 복원·retry/status·local_100 순서 비교. M1의 factor 내부 동등성과 별개 |
| M3-04c-17 오라클 | factor 종료→FTRAN rhs→basic 폭·분류·PInf. 두 재계산 진입점 | 새 header·rhs·raw FTRAN 출력·lb/ub·I/h·attempt/model_status 필수. free·한쪽 무한·lb≠0·폭0·±h·w±h·위반0/비영, 별칭 유무. 자연 재분해 반복 자료를 새 훅으로 확보 |
| M3-04c-18 구현 L1 | 17의 FTRAN 결과를 받는 조건부 shift·분류·PInf·retry | basic·width·marks·위반 수·PInf 비트 일치. FTRAN은 21의 수치 연결 전까지 외부 입력 |
| M3-04c-19 오라클 | 보류 기록→공급 고갈→round 압축·support 결정·재분해 요청, BTRAN 전후 | P 전체·local_238 승인 수·sqrt(m)·BTRAN 원시 출력·support 상태·미소비 batch=0. 진행0·전체보류·세 번 보류·support해제·dense BTRAN 경계. 미관측 dense는 합성 결과를 주는 제어 경계와 자연 실행을 구분 |
| M3-04c-20 구현 L1 | 19의 보류·실패·지원 표시·후보 재처리 제어, BTRAN은 외부 backend | 후보 순서·횟수·종료·요청 10·표시 초기화 비교. 재시도에서 실패 횟수 보존 |
| M3-04c-21 조사 | 직렬 수치 backend: `09d008/2207de`, `09e376/221173`, `05b257`와 Rust Lu 연결 | M1 경계와 sparse 출력 순서·spike·지연 LU 상태·재분해 번호 대응을 확정하고 수치 오라클/구현을 추가 하위 번호로 분할. 기존 API 존재만으로 착수 완료 아님 |
| M3-04c-22 구현 L1 | **초기화된** 루프 상태→종료 판정. 14·16·18·20 + 기존 pick/apply 연결, 수치 backend 외부 | 여러 round의 호출·후보·배열·요청·status·retry가 연쇄 일치하고 중간 원본 상태 재주입 금지. 초기 후보 구성·최종 외부 해 설치는 별도라 전체 primal push로 승격 금지 |
| M3-04c-23 조사 | batch record·thread·factor 포인터와 재처리 | §2.6 상태·분할, natural Threads>1의 도달성·결정성부터 확인. worker별 work·미소비 후보·재분해 무효화 오라클과 구현을 별도 번호로 등록 |

04b의 포괄적인 `M3-04c-3 후보`는 이번 13–23으로 구체화했다. 초기화·정렬·기준 조정 후보 04c-1, x87 후보 04c-4, signed remaining·외부 해·재시작은 [04b 조사](primal_push_branches.md#4-후속-분할과-미확인-범위)와 04a-2·04g의 경계로 남긴다. 22를 통과해도 이 항목들이 끝나지 않으면 전체 push 완료가 아니다.

## 5. 재실행 명령

저장소 루트에서 실행한다. 새 원본 실행 없이 선행 검사기로 출처·재현성·대조군을 재검사한다.

```bash
python3 tools/push_loop_survey.py > /tmp/m3-push-loop-survey.json
cmp bench/validation/m3_push_loop/survey.json /tmp/m3-push-loop-survey.json
python3 tools/push_chain_oracle.py --check
python3 tools/push_chain_oracle.py --self-test
cargo test -p rsolver-lp --test push_chain
python3 tools/push_step_chain_rust.py --compare
git diff --check
```

Rust·Python 확장 코드는 바꾸지 않으므로 workspace 전체 빌드·확장 재설치는 이번 조사 회귀에 포함하지 않는다. 원본 C/바이너리 해시는 survey와 §2.1로 대조한다. 새 오라클 도구를 만드는 13부터는 `.agents/skills/oracle-capture/SKILL.md`를 먼저 읽는다.

## 6. 실행 결과

- 기준 main `ec76fd6`, 계약 commit `41b6476`, 저장 집계 도구·자료 commit `035856c` 이후 조사 트리에서 실행했다. WSL2 x86-64 Linux, Python 표준 라이브러리와 저장소 Rust 환경을 사용했다. 원본 C6파일·라이브러리 해시를 §2.1 및 survey와 대조했다. 새 원본 실행은 **0회**다.
- `python3 tools/push_loop_survey.py` 재실행과 `cmp` 종료0. 기존 검사기가 독립3회·대조군·비간섭을 재검사한 대표18기록에서 공급236·고갈38·보류18·factor73호출, header 변경33을 확인했다. 공급 반환 dense=0은 내부 FTRAN dense 미실행을 뜻하지 않는다.
- `python3 tools/push_chain_oracle.py --check` 및 `--self-test` 종료0. shift·resolve 목표5태그가 존재하고, 기존 오류 주입8종(바이너리0·사건 비움·목표0회·실행 간 차이·도구/모델 해시·x87)을 거부했다. 이번 새 오라클의 판별력으로 세지 않는다.
- `cargo test -p rsolver-lp --test push_chain` 1테스트 통과. `python3 tools/push_step_chain_rust.py --compare` 종료0: **236단계·5,144필드 비트 일치**, 42구간, flip70·pivot148·보류18·재호출20·위반80. 재분해 뒤 비교 제외6은 기존 계약 그대로이며 새로 검증한 구간이 아니다.
- `python3 -m py_compile tools/push_loop_survey.py`, `git diff --check` 통과. 문서 상대 링크·후속 번호 정합성도 확인한다. 실제 명령·집계는 [verification.json](../../bench/validation/m3_push_loop/verification.json)에 기록한다.
- 독립 검토자가 원본 C/asm·문서·집계 도구를 읽고 survey를 재실행해 저장 결과와 바이트 동일함을 확인했다. 바이너리/모델 해시0·빈 사건·목표 단계 제거를 각각 직접 주입해 거부를 확인했고, 기존 `--check`·`--self-test`와 Rust 연쇄 비교도 실행했다. 다중 push 기록의 초기 factor 문맥을 위해 `push:enter`에서 `last_supply`를 초기화하도록 보정했다. 현재18기록은 각각 push1회여서 집계값은 같으며, 도구 해시는 새 값으로 갱신했다. 내부 dense와 공급 반환 표현을 구분하고 C 줄 범위를 보정했다. [독립 검토 최종 기록](../../bench/validation/m3_push_loop/review.md): 통과, 병합을 막는 문제 없음.
- 실행하지 않은 검사: workspace 전체 테스트·clippy·Python 확장 재빌드는 Rust·확장 변경이 없는 조사이므로 제외했다. 신규 원본 캡처·factor 오류·batch·x87·자연 요청2/6·내부 FTRAN 표현은 후속 작업의 미검증 범위다. 계약 범위 변경은 없다.

## 7. 판정과 인수인계

- 작업 수행: **조사 완료**. 원본 구간·읽기/쓰기·별칭·수식·기존 오라클의 누락·후속 번호를 특정했다. 구현 동등성: **미검증**. 기존 Rust236단계 통과는 선행 구현의 회귀 결과다.
- 다음은 **M3-04c-13** 단일 공급자 원본 오라클이다. `python3 tools/session.py status` → `python3 tools/session.py claim M3-04c-13 --owner <새-owner>` 후 oracle-capture 스킬과 이 문서 §2.2·§4.3을 읽는다. 시작 자료는 `bench/oracle/m3_push_arms/arms-primal-1.json`, `bench/oracle/m3_push_retry/retry-primal-1.json`이며 기존 agent는 수정하지 않고 새 훅으로 보강한다.
- branch `work/m3-04c-12`, worktree `../rsolver-m3-04c-12`, 선행 미커밋 변경 없음. 최종 commit·`--no-ff` 병합은 main의 `Merge M3-04c-12` 이력으로 추적한다. 병합 후 같은 단계에서 worktree·branch 정리와 M3-04c-12·shared 점유 해제를 수행하고 main에서 최종 상태를 확인한다.
- 진행·로드맵·설계·스프린트·인계: `docs/11_progress.md`, `12_roadmap_diagram.md`, `m3/sprint_design.md`, `14_sprint_plan.md`, `handoff/M3.md`에 반영한다. 교훈은 `docs/lessons/original.md`의 crossover 항목(지연 LU·요청 수식·보류와 raw count)에 기록한다. 외부 업로드나 원격 동작은 없다.
