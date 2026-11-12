# M3-04c-21 직렬 수치 backend 연결 조사

## 1. 범위와 주장

- 종류: 조사. 선행 main `6b371c0`, branch `work/m3-04c-21`, worktree `../rsolver-m3-04c-21`. 선행 미커밋 변경 없음. main의 미추적 `.claude/settings.local.json`은 작업과 무관하며 보존한다.
- 질문: double·Threads=1·batch=0에서 공급 FTRAN의 목록/값/spike/clear, 지원 BTRAN의 좌표/표현, 지연 LU와 재분해 번호 대응의 읽기·쓰기·수명은 무엇인가? M1 Rust Lu와 기존 원본 자료로 무엇을 재사용하고 무엇을 추가 캡처해야 하는가?
- 완료: C/asm 근거와 기존 자료를 재실행 가능한 집계로 고정하고, 수치 오라클 뒤 구현 순서의 새 하위 번호·필수 비교 필드·실패 조건을 등록한다.
- 제외: 새 원본 실행·신규 솔버 구현·수치 동등성 판정. 품질 수치15a·루프22·batch23·초기화·x87·전체 work/P3는 별도다.

## 2. 원본과 상태 경계

- 원본 Gurobi 13.0.3, 라이브러리 SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`(선행 manifest와 로컬 파일을 대조한다).
- ELF offset 기준으로 `09d008@0x120680`, `2207de@0x120740`, `09cf9b@0x1205d0`, 지원 BTRAN wrapper/커널, `05b257@0xf15c0`, `134a87@0x1c0070`, `25615c@0x1c5310`을 좁혀 읽는다. 동명 x87은 제외한다.
- 시작: 초기화된 factor에서 공급자 수치 호출 전. 종료: raw 출력·spike 저장 뒤, factor 갱신/재분해 전후 및 재계산 FTRAN 종료. 주소는 의미 있는 배열·인덱스·별칭으로 표현한다.
- 세부 ABI·필드·원본 위치는 조사 결과에 추가한다. 아직 모르는 상태를 추정해 실행하지 않는다.

### 2.1 FTRAN: wrapper·좌표·spike의 계약

`F=*(S+0x470)`. C는 wrapper 인자를 생략하므로 [wrapper asm](../../bench/validation/m3_push_lu/ftran-wrappers.asm)을 함께 읽었다. 기본 벡터는 `{i32 count; idx*@8; val*@16}`다. **일반 sparse의 val은 목록 위치별 packed 값이지만 spike의 val은 인덱스로 접근하는 dense 저장소**다. 두 직렬화를 혼용하지 않는다.

| 경계 | 확인한 읽기·쓰기·순서 | 근거 |
|---|---|---|
| `09d008(F,in,out,spike,tmpval,tmpmark,opaque,work)` | spike 비null이면 이전 목록의 값들을 `+0`으로 지우고 count=0. `2207de`에 save=1·spike를 넘김. null이면 save=0. 나머지 세 인자는 현재 caller에서 0 | wrapper C:13–36, asm `0x120688–0x12073d`; supplier C:629·679 |
| `09cf9b(F,in,out,save,work)` | save 비영일 때만 F+0x1c8을 지운다. save=0은 저장 spike를 지우지 않는다. 본체에 spike/tmp/opaque=0을 넘긴다 | wrapper C:14–37, asm `0x1205d8–0x12066e` |
| 입력 행 변환 | sparse 입력 idx를 F+0x30으로 **제자리 치환**. dense는 F+0x20으로 gather하고 입력/출력의 idx·val 포인터를 교환한다 | `2207de` C:115–164, asm `0x12083b` 이후 |
| L→R | L은 sparse이고 `nnz·F104<=m && F128<0.1m`이면 `21b9de`, 아니면 `214344`. R은 생성 순 eta, F+0x180개·레코드 F+0x198(24 byte), sparse 유지와 dense 전환이 다름 | C:165–179·180–685; asm `0x1208b6/0x120922` |
| spike 저장 | **U 풀이 전** L/R 결과를 저장. dense 결과는 strict `F10<abs(v)`인 위치를 오름차순으로, sparse는 목록을 그대로 복사하고 dense 값 저장소의 idx 위치에 쓴다. sparse 저장에 동일한 추가 drop을 넣지 않음 | C:686–778, asm `0x121184–0x121216` 및 `0x121cdb` 이후 |
| U | 이번 wrapper는 두 번째 RHS=0이므로 sparse는 `21686f`, dense는 `2126cd`. 판정은 `m-t+ec+nnz<m || (F108+F110)^2·nnz<2m`, nnz<0이면 dense. `21574d`는 두 번째 RHS가 있는 별도 경로 | C:687–700·780–787, asm `0x121c61–0x121ccc` |
| 종료 | 출력 count·목록/값, 변경된 입력 표현과 두 벡터 버퍼의 의미상 교환, scratch clear, 저장 spike를 함께 관측 | C:872–885·1045–1049, 본체 [asm](../../bench/validation/m3_push_lu/ftran-body.asm) |

spike·L/R의 내부 좌표와 외부 제약 행 좌표를 이름만 같다고 취급하지 않는다. F+0x20/0x30은 행 순열/역순열이고, F+0x28/0x38은 기저 위치 순열/역순열이다. U 출력은 ratio가 소비하는 기저 위치다. 후속 캡처에서는 이 네 배열과 U 갱신 순열(+0x40/0x48)을 함께 저장해 인덱스 공간을 연결한다. Rust는 L/R 계산을 원래 행으로 표현하므로 입력/출력/지원 표시는 외부 의미 좌표로, 원본 내부 spike는 순열을 통해 Rust 행으로 대응한다. 배열 순서는 원본 순서 그대로 유지한다.

R의 dense 내적은 eta 값 주소 mod16에 따라 선처리 1개를 하고 8레인을 합친다. asm `0x120c5f–0x120cb3`, sparse 유지 쪽 `0x120f89–0x121019`는 `((0+2)+(4+6))+((1+3)+(5+7))`, 잔여는 순차다. C의 왼쪽 결합 표현으로 바꾸지 않는다. M1-02l의 **두 RHS** 캡처가 같은 규칙의 재사용 근거지만 이번 **한 RHS** 실행 검증은 아니다. raw 주소 대신 값 배열 mod16·풀 offset/길이·기준 정렬을 캡처한다.

### 2.2 BTRAN: 지원 행과 FT용 Uᵀ scratch

`134a87` C:1999–2023은 이탈 **기저 위치**에 값1을 둬 `09e376(F,in,out,work)`를 호출하고 반환 sparse idx의 **제약 행**을 지원 표시로 쓴다. caller는 추가 수치 drop을 하지 않는다. count<0이면 지원=-1·표시 해제·압축으로 간다(C:2024–2062). 따라서 `Lu::btran_dense(...).0`만 꺼내 모두 sparse로 취급하면 제어가 달라진다.

`09e376`은 네 번째 인자 work를 보존한 뒤, **커널의 네 번째 인자에는 F+0x1d0을 넣는다**(asm `0x131716–0x13172b`). 이 포인터가 있는 `221173` 경로는 입력 r의 U 행이 비지 않았으면 F+0x1d8=r을 기록하고 희소 Uᵀ `21a941`를 강제하며, Uᵀ 출력을 F+0x1d0에 복사한다(C:1197–1227). 이는 FT 갱신이 재사용하는 상태다. U 행이 비면 cache count=0 뒤 일반 경로로 돌아간다. wrapper를 일반 `09e288`과 같은 값 계산으로만 치환하면 이 부수 상태를 놓친다. [BTRAN wrapper asm](../../bench/validation/m3_push_lu/btran-wrapper.asm).

나머지는 Uᵀ→Rᵀ→Lᵀ다. 일반 Uᵀ sparse 판정의 정수 cast는 `(int)(F108+F110)·nnz<=m`이고 FTRAN의 제곱식과 다르다(C:86–90). eta가 있고 Uᵀ 결과가 dense이거나 `m<F104·nnz`이면 dense eta, 그 외 sparse eta다(C:1237–1572). dense eta는 `abs(v)<=F10`이면 v를0으로 지우고 전파하지 않으며, sparse eta는 v≠0을 보고 끝에 `abs(v)>=F10`으로 압축한다. sparse 출력은 C:1174–1185의 F+0x20 치환, dense는 C:1736–1764의 F+0x30 gather와 버퍼 교환을 거친다. [본체 asm](../../bench/validation/m3_push_lu/btran-body.asm), M1-02i 계약 참조.

### 2.3 지연 LU와 재분해 번호

하나의 header로 모든 상태를 나타낼 수 없다. 다음 세 상태를 구분한다.

1. 현재 basis header/vstat: `push_apply`가 이미 교환한 상태.
2. 마지막 수치 분해·FT 갱신의 L/U/eta·행/열 순열: support!=0·요청0이면 이번 pivot을 반영하지 않은 상태.
3. 현재 공급자의 spike와 BTRAN Uᵀ scratch: 다음 갱신에 소비될 일시 상태. support>0 공급은 spike 포인터 자체가 null이다.

`05b257` 요청0 경로는 FT 갱신 후 F+0x18c를 증가시키고 spike count=0으로 만든다(C:1327–1329). 요청비영 경로는 `036ade`로 **현재 header**를 분해하고(C:1454 근처) 이전 spike 목록 값을0으로 지운다(C:2006–2033). factor 반환1 반복·retry/status는 선행16의 wrapper 계약이고 수치 갱신 성공/실패와 별도다.

pricing/QP 객체가 없는 경로(S+0x478/0x480/0x488=0을 후속 캡처에서 확인해야 함)의 재분해 후 C:2036–2090은 다음을 수행한다. `old_of_new=F28`, `new_of_old=F38`이고 `head_new[p]=head_old[old_of_new[p]]`, `vstat[head_new[p]]=p`, 이탈위치가 음수가 아니면 `leave_new=new_of_old[leave_old]`다. header와 spike의 **idx 버퍼**를 교환하므로 spike count0만 보고 alias가 고정됐다고 판단하지 않는다. C:2570–2595는 F+0x28/+0x38을 identity로 쓰고 +0x40/+0x48이 그 배열을 가리키게 한다. [재번호 asm](../../bench/validation/m3_push_lu/factor-renumber.asm).

basic/width/marks는 임의로 같은 순열을 적용해 종료하지 않는다. push는 새 header·rhs로 dense `09cf9b(save=0)`를 실행하고 그 raw 결과를 분류한다(선행17/18). rhs는 제약 행 좌표이므로 header 순열로 섞지 않는다. 지원 표시는 별도의 제약 행 배열이다. 오류·singular slack 교체·pricing/QP가 있는 경로는 이 단순 재번호 식에 포함되지 않는다.

### 2.4 M1 Rust Lu 재사용과 빠진 경계

| API·M1 근거 | 재사용 | 이번에 필요한 보강 |
|---|---|---|
| `Lu::ftran_lr`·`ftran_upper`·`ftran_dense`, M1-02j/l/m | L/R/U 계산, eta 정렬·U 저장 순서 | `ftran_lr`는 두 RHS `09d112` 기준. 원본 한 RHS의 표현·spike·clear·버퍼 교환·희소 U 출력과 대조 필요. L path B는 shortcut |
| `ftran_flip` | 표현 판정 조건 참고 | clone에 L/R 목록 계산 뒤 실제 값은 dense로 재계산하는 shortcut. sparse 목록·spike를 반환하지 않으므로 numeric supplier로 바로 연결 불가 |
| `btran_dense`, M1-02i | 출력 목록·dense bool, eta drop | factor가 대각이고 갱신0인 분기는 Uᵀ 판정3 고정(M1-02k 미완). cache F1d0/F1d8·입력 버퍼 변경·지원 좌표를 새 경계로 확인 |
| `Lu::update`, M1-02l/m/n | spike 목록 순서, 행/열 저장·prefix/f0·pool | 반환bool은 원본 반환코드/retry와 다름. 현재 header와 factor가 다른 기간·빈 spike·cache 수명·재분해를 backend가 관리해야 함 |
| `renumbered`·`relabel`, `Engine::refactor` | old_of_new, 기저 위치 재번호·vstat 역대응 | Engine은 simplex pricing과 해 재계산을 같이 함. push의 rhs/분류·요청 정책을 대체하지 않음 |
| `work.rs`, M1-02p/q/r, 미완02t | factor work의 일부·커널별 위치 지도 | LU 풀이 work 누적 순서는 미완. numeric 결과 통과를 전체 work 통과로 세지 않음 |

현재 `Lu`는 ZERO=1e-13 상수다. 후속 오라클에서 F10이 같은지 고정·검사하고 다른 값은 별도 계약을 요구한다. 초기 L/U를 원본에서 받아 같은 입력 커널을 재생하는 검사와, Rust가 현재 header에서 직접 factor를 생성하는 검사는 다른 층이다. 전자의 통과로 factor 생성까지 통과시킬 수 없다.

## 3. 비교 계약

| 대상 | 규칙 | 방법 |
|---|---|---|
| 원본 위치·자료 식별 | 정확 offset·SHA-256·줄 범위 | C/asm·기존 manifest |
| 기존 자연 실행 | 대표 기록만 집계, 사건·목록·count·호출 순서 정확 | 선행 controls/acceptance 뒤 읽기 전용 분석 |
| 미래 수치 계약 | f64는 ±0 포함 비트, count/목록/번호/alias는 정확 | 후속 원본 오라클에서 실행 |
| 누락과 제한 | 저장되지 않은 factor 내부 상태·경로는 명시 | 기존 agent와 Rust API 대조 |

시간·work·병렬은 이번 조사 판정 밖이다. 기존 독립3회 자료는 재사용이고 새 원본 재현성 검증으로 세지 않는다. 신규 구현 동등성은 미검증으로 남긴다.

## 4. 사례와 원본 자료

기존 supplier13·factor15·recompute17·round19의 자연 arms/retry/interior와 M1 LU 계약·자료를 읽는다. 합성 backend 출력은 실제 수치 계산 증거로 세지 않는다. 조사 집계에는 사용 파일과 해시·사건 수·필드 누락을 기록한다. 동률·오류·dense 다수/빈 출력은 후속 사례로 특정하며 미관측을 통과 처리하지 않는다.

### 4.1 저장 자료의 실제 도달 범위

[집계](../../bench/validation/m3_push_lu/survey.json)는 선행19의 manifest·독립3회·대조군·선행17 비간섭·자연/합성 acceptance를 통과시킨 뒤 **각 자연 모델의 첫 기록만** 센다. 다른 세대의 같은 사건을 중복 합산하지 않는다. 합성106경계는 선행 검사 회귀에만 포함하고 아래 수치 실행 수에서 제외했다.

| 대표 모델 | 전체 사건 | 공급 raw FTRAN | spike null / 저장 | 지원 BTRAN | factor 요청0 / 10 | pivot factor 호출 / 생략 | 재계산 FTRAN |
|---|---:|---|---|---:|---|---|---:|
| arms | 410 | 25 sparse(count1=16·2=9) | 25 / 0 | 17(count1) | 0 / 4 | 0 / 17 | 3(round) |
| retry | 161 | 8 sparse(count1=5·2=3) | 7 / 1(count1) | 4(count1) | 0 / 3 | 0 / 4 | 2(round) |
| interior | 180 | 9 dense(count=-1) | 1 / 8(count8) | 1(count1) | 5 / 4 | 6 / 1 | 3(pivot1·round2) |
| Crossover=0 | 0 | 0 | 0 / 0 | 0 | 0 / 0 | 0 / 0 | 0 |

합계는751사건·공급 FTRAN42·지원 BTRAN22·factor16·재계산8이다. pivot 호출6은 요청0 갱신5와 요청10 재분해1이며, 나머지 factor는 초기화/라운드다. factor 반환16회는 모두0, header 순서 변경7회다. interior의 factor 진입 갱신 수 최대3은 실제 FT 실행을 보여주지만 **행 eta 수>0이나 희소 U 실행 증거는 아니다**. 현재 캡처에는 그 상태·커널 사건이 없다.

빠진 필드: 공급 wrapper 반환 뒤 입력의 count/idx/val·버퍼 교환, L/R 중간 상태와 선택 커널, L/U·eta·pool 정렬과 순열, BTRAN F1d0/F1d8, factor의 재번호 전후 순열과 alias. 기존 raw 출력은 수치 backend가 낸 결과로서 유효하지만 이 자료만으로 **동일한 LU 입력 상태를 재구성할 수 없다**. header만으로 새 factor를 만들어 같은 입력이라고 간주하지 않는다.

### 4.2 후속 수치 오라클의 필수 스냅숏

- 호출 식별: 모델/push/call/step·factor epoch·직전 지원 pivot 목록·현재 header와 분해 기준 header·요청·갱신수·batch=0. factor epoch는 성공 재분해마다만 바뀌며 FT 갱신 순번은 따로 센다.
- factor: F8/F10·행/열 순열(F20/28/30/38)·U 순열/역순열(F40/48), L 열(F58/60/68)·행(F70/78/80)의 저장순서·값, U 열(F88/90/98/a0/a8)·행(Fb0/b8/c8/d0)·대각(F50), 경계/목록(Fdc/e4/e8/ec/f0/f8/100), sparse 계수/level(F104/108/110/118/128/130/140). eta는 F180/184/198의 생성순서·길이·값/idx·mod16, F1a0의 마지막 접촉, pool F148/150/160/168과 용량 상태를 포함한다. scratch F1a8/1b0/1b8도 역할·초기화/clear를 구분한다. **Rust API가 현재 읽는 필드만** 저장해 원본 누락을 숨기지 않는다.
- 벡터: 일반 sparse packed와 spike dense-indexed를 구분해 count·목록·활성 값, dense m값·±0, input/output/spike의 의미상 alias를 전후 모두 저장한다. clear 검사는 직전 목록의 값0과 새 비활성 값 보존을 선택해 읽으며 미초기화 공간은 비교하지 않는다.
- BTRAN: 호출 전후 F1d0 벡터와 F1d8, Uᵀ/Rᵀ/Lᵀ 경계·출력 표현. support에 쓰는 목록과 연결한다.
- 재분해: old_of_new/new_of_old를 **identity로 덮기 전** 저장하고 header/vstat/leave의 후상태, spike idx 버퍼와 header alias 교환, 요청0/비영·ret/retry/status·후속 rhs FTRAN을 같은 사건열로 연결한다.

첫 오라클은 유효한 자연 실행의 읽기 전용 훅으로 이 상태를 모은다. 원본 allocator·F를 추정해 직접 생성하지 않는다. 저장 factor의 의미상 상태를 Rust에 초기 입력으로 넣는 커널 검증은 허용하지만, 원본 **중간 출력**을 Rust의 다음 연산 입력으로 재주입해 연쇄 통과를 만들지 않는다. 수치 경로가 갈리면 처음 다른 L/R/U 필드와 상태를 보고하고 범위를 새로 기록한다.

### 4.3 확정한 후속 번호

공통으로 원본 독립3회·대조군·선행 전체 사건/최종 출력 비간섭, 필수 사건0회/필드누락·해시변조 거부를 요구한다. f64는 ±0 포함 비트, 목록/분기/count/alias/번호는 정확 비교다. 새 비교기는 순서·drop 경계·spike 위치·dense flag·재번호·cache 변조 중 담당 동작 오류를 거부해야 한다. 미관측 분기는 합성 **원본 수치 명령을 실행**할 ABI를 별도 확정하거나 후속으로 남긴다. 수치를 stub한 제어 합성으로 대체하지 않는다.

| 번호·종류 | 선행·범위 | 필수 사례·완료 기준 |
|---|---|---|
| **M3-04c-21a 오라클** | 21·13·17. `09d008/09cf9b→2207de` 한 RHS, 초기 factor 입력·L/R→spike→U·raw/반환 입력/alias | 자연 arms/retry/interior부터 sparse/dense·구조/slack·spike null/저장·save0·갱신0/비영. eta>0·비identity 순열·희소 U는 새 관측 필요. ±ZERO·count0·동률/순서 경계는 원본 수치 합성 여부 구분. 현재42/8회 재현과 신규 상태·커널 증거·오류 검출 |
| M3-04c-21b 구현 | 21a. Rust 한 RHS FTRAN 결과 표현·spike·clear adapter, 수치 커널 | 같은 초기 factor 상태에서 raw count/목록/값·입력 후상태·alias·spike 전후 비트 비교. `ftran_flip` shortcut 재사용으로 통과 처리하지 않음. M1 공유 수치 변경은 M1 점유 조율·관련 M1 회귀 필요 |
| M3-04c-21c 오라클 | 21·19. `09e376→221173`, 지원 caller와 F1d0/F1d8 | U 행 빈/비영·강제 sparse Uᵀ·sparse/dense eta·Lᵀ 표현·계산 후 지원 표시. 기존 자연22회는 count1만; 다수/빈/dense의 **실제 계산**은 별도 모델 또는 검증된 원본 수치 경계로 확보. 미도달이면 분할·미검증 명시 |
| M3-04c-21d 구현 | 21c. Rust BTRAN 결과+표현+cache adapter | 같은 factor에서 raw/지원 좌표·cache·입력 변화·clear 비교. M1-02k 일반 밀도 분기의 근사 여부를 적용 범위에서 해결하거나 제외 명시. sparse/dense를 결과 nnz만으로 추정 금지 |
| M3-04c-21e 오라클 | 21a·21c. 지연 LU→FT/재분해·번호→재계산 FTRAN의 실제 수치 연쇄 | 생략22회(arms17·retry4·interior1)·interior 요청0 갱신5회/요청10·header 변경7회를 시작점으로 §4.2 상태 수집. 동일 초기 factor에서 여러 호출 사이 상태 보존, old/new 번호와 현재/factor header 구별. 반환1/실패/singular는 실제 계산 자료 확보 전 제외·별도 등록 |
| M3-04c-21f 구현 | 21b·21d·21e. 직렬 numeric backend의 factor 수명·spike 소비·FT·재분해·재번호·rhs 계산 | 같은 초기 상태 뒤 원본 중간 factor 재주입 없이 여러 수치 호출 연쇄의 순서·배열/값·alias·status 비교. 현재 header로 매번 분해하는 변이와 무조건 갱신 변이 거부. 초기 factor 생성이 원본 입력이면 생성 동등성은 별도 판정 |

품질15a, **외부 수치 backend를 쓰는 L1 루프22**, batch23은 독립 범위로 유지한다. 21f는 수치 backend 연쇄이고 전체 solver 루프 설치가 아니다. 루프22에 실제 backend를 설치하는 연결 검증과 초기화·최종 해 설치는 그 선행 결과가 준비된 뒤 별도 계약으로 등록한다.

## 5. 재실행 명령

저장소 루트에서 실행한다. 외부 C·동일 바이너리가 있어야 조사 해시를 재검사할 수 있다. 새 원본 프로세스는 시작하지 않는다.

```bash
python3 tools/push_lu_survey.py --check
python3 tools/push_lu_survey.py --check --self-test
python3 tools/push_supplier_check.py --controls
python3 tools/push_supplier_check.py --acceptance
python3 tools/push_supplier_check.py --self-test
python3 tools/push_factor_check.py --controls --acceptance --self-test
python3 tools/push_recompute_check.py --controls --acceptance --self-test
python3 tools/push_round_check.py --controls --acceptance --self-test
python3 tools/push_chain_oracle.py --check
python3 tools/push_chain_oracle.py --self-test
python3 -m py_compile tools/push_lu_survey.py
git diff --check
```

기본 출력 `python3 tools/push_lu_survey.py > <새 경로>`는 저장 survey와 바이트 비교할 수 있다. `--self-test`는 입력 무결성8종의 거부이며 **신규 수치 오라클의 판별력 증거가 아니다**. Rust 코드를 바꾸지 않는 조사라 workspace/clippy·확장설치·pytest·Netlib·P3는 제외한다. 명령·실행 결과는 §6과 `bench/validation/m3_push_lu/`에 고정한다.

## 6. 실행 결과

- 기준 main `6b371c0`, 착수 계약 `befcbe9`, 원본 asm/집계 첫 동작 `d471ab3` 뒤 조사 트리에서 실행했다. WSL2 x86-64 Linux·Python3.10.12, 새 원본 프로세스 **0회**, Rust/확장 코드 변경0이다. C11파일·Rust API2파일·M1 계약4파일·asm5파일·선행19 manifest/대표4캡처의 SHA-256과 동일 로컬 바이너리 해시를 survey에 고정했다.
- `push_lu_survey.py --check` 통과. 기본 출력 두 독립 실행은 저장 survey와 바이트 동일(최종 검증 기록에 해시). 자연3모델751사건, FTRAN42·spike null33/저장9·BTRAN22·factor16·pivot 호출6/생략22·header 변경7·재계산8이다. Crossover=0 사건0도 확인했다. 새 수치 비교 필드 수는 **0**이며 기존 실제 수치 출력의 도달성 조사다.
- `--check --self-test` 통과: 바이너리0·모델0·빈 사건·FTRAN/BTRAN/factor/recompute 경계 제거·spike 필드 누락의 입력 무결성8종을 거부했다. 신규 numeric 구현 변이나 수치 오라클의 판별력으로 세지 않는다.
- [선행 회귀8명령](../../bench/validation/m3_push_lu/regression.json) 모두 종료0. supplier controls/acceptance/self-test(31변조), factor controls/acceptance/self-test(41), recompute(82), round(157), chain check/self-test(8)를 저장 재생했다. 이 숫자는 선행 검사기의 기존 오류 검출 범위다. 로그를 같은 디렉터리에 보존했다.
- 초기 supplier CLI는 세 flag가 상호배타여서 종료2였고 `initial-cli-error.log`에 남겼다. flag별로 분리한 정식3명령은 모두0이다. factor asm 첫 시작주소0xf44d0은 명령 중간이라 거짓 디코드가 섞였고 `history/factor-renumber-mid-instruction.asm`에 보존했다. 함수 시작부터 디스어셈블한 실제 경계0xf44cf로 고쳐 캡처와 해시를 재생성했다. 잘못된 첫 asm은 원본 근거로 사용하지 않는다.
- Python 구문·문서 상대링크·분할번호·`git diff --check`를 최종 트리에서 확인했다. 원본 C/asm 검토와 재현성 검사에 독립 검토자를 사용하지 않았다. 저장된 원본3회는 선행 작업의 자료이며 이번 재캡처가 아니다.
- workspace 전체·clippy·Rust 테스트·확장설치·pytest·Netlib·P3는 솔버/확장 코드가 바뀌지 않은 조사여서 제외했다. 새 LU 동일 입력 비교·모든 새 경로·자연 BTRAN count다수/dense·x87·전체 work/P3는 미검증이다. 착수 계약 범위는 그대로다.

## 7. 판정과 인수인계

- 작업 수행: **조사 완료**. FTRAN/BTRAN의 표현·상태·좌표·spike/cache 수명, 지연 factor와 현재 header 구별, 재분해 순열과 Rust API의 차이, 저장 자료의 도달/누락을 근거로 답하고 후속21a–f를 등록했다. 신규 수치 동등성: **미검증**.
- 다음은 **M3-04c-21a FTRAN 수치 오라클**. `python3 tools/session.py status` → `python3 tools/session.py claim M3-04c-21a --owner <새-owner>` → `.agents/skills/oracle-capture/SKILL.md`와 이 문서 §2.1·4.2·4.3 → `python3 tools/push_lu_survey.py --check`. 자연 시작자료는 `bench/oracle/m3_push_round/{arms,retry,interior}-1.json`이며 새 훅은 선행 agent의 전체 사건/최종 출력 비간섭을 검사해야 한다.
- branch `work/m3-04c-21`, worktree `../rsolver-m3-04c-21`, 선행 미커밋 없음. 최종 commit·병합은 main `Merge M3-04c-21` 이력을 따른다. 계약 검증 뒤 `--no-ff` 병합한 같은 단계에서 worktree/branch 정리·21/shared 점유 해제한다. main 미추적 `.claude/settings.local.json` 보존. 원격/업로드 없음.
- 진행/로드맵·스프린트·설계·인계는 `docs/11_progress.md`, `12_roadmap_diagram.md`, `14_sprint_plan.md`, `m3/sprint_design.md`, `handoff/M3.md`에 반영한다. 교훈은 `docs/lessons/original.md`의 crossover FTRAN spike/clear·BTRAN Uᵀ cache와 `tooling.md`의 asm 명령 경계 항목이다.
