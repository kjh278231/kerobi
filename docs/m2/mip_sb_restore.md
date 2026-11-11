# M2-06g-2 strong branching 결과 소비·복원 계약

기준: [검증 기준](../15_sprint_verification.md), main `a6b7702`, M2-06g. 사용자 지시로 작업자 Codex가 2026-11-11 착수. 미커밋 선행 변경 없음(main의 미추적 `.env`는 건드리지 않음).

## 1. 범위와 주장

작업 종류는 조사·읽기 전용 오라클이다. `691453` 부모 LP 확인, `691b56` 디스패치와 `266b6a` 후보 평가, `62ec1d` 후보 분할, `62f909` SBMAXCANDS·목적 배열·배율·비용 누적의 원본 계약을 확정한다. Rust 구현은 변경하지 않는다.

Threads=1 선형 MIP 자연 실행을 대상으로 한다. 병렬·QP·할당 실패·외부 중단은 정적 제어 흐름을 기록하고 동적 통과로 주장하지 않는다. 전체 factor 내부·LP 수치 커널 재현은 후속 M2-06h/M1이다. 복구는 단순히 함수 진입/반환 메모리 전체가 같다는 가정을 두지 않고, 저장/복사/복원 함수와 bound·basis의 관측 경계를 구분한다.

## 2. 원본과 상태 경계

Gurobi 13.0.3 lib SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. C는 `../re/decomp/c/0056f470_*691453.c`, `0056faf0_*691b56.c`, `0029d980_*266b6a.c`, `00543740_*62ec1d.c`, `00541280_*62f909.c`와 이들의 저장·복원 helper다. file offset 기준 ELF asm으로 ABI·명령 경계를 확인한다.

시작/종료: 각 함수 진입/반환 및 helper 호출. 원시 주소 대신 호출 번호·상대 오프셋·배열 내용으로 식별한다. 입력은 후보 순서·방향별 배열·신뢰성 질의 결과·work·LP 파라미터·bound·LP 상태이며, 쓰기는 후보와 두 배열의 동시 교환, 평가 결과, 반복 비용, 임시 파라미터 복원이다. basis/factor 복사는 별도 helper의 C·asm 지도와 관측 호출 사슬로 기록한다.

### 2.1 호출과 ABI

디컴파일의 인자 목록에는 xmm0 인자가 빠진 곳이 있다. 실제 SysV ABI는 다음과 같다. 스택 오프셋은 피호출자 진입의 rsp 기준이다.

| 함수 | GPR 인자 | 스택 인자 | xmm0 |
|---|---|---|---|
| `691453` @ `0x56f470` | N, LP, &classification, flag, work | 없음 | 없음 |
| `691b56` @ `0x56faf0` | LP, N, symmetry, n, J, workers | +8 types, +16 stall, +24 down, +32 up, +40 &iters, +48 work | 시간 예산 |
| `266b6a` @ `0x29d980`와 `695bd7` @ `0x571770` | LP, N, n, stride, J, types | +8 stall, +16 down, +24 up, +32 &iters, +40 work | 시간 예산 |
| `62ec1d` @ `0x543740` | N, W, J, n, extra, &두 목적 배열 | +8 &start, +16 &count, +24 work | 없음 |

근거는 각 함수 프롤로그와 `0x570510`–`0x570573` 호출 인자 구성이다. 기존 `re/notes/15_mip_branch_and_bound.md`의 `691453 = 순차 SB`, `691b56 = 병렬 SB` 설명은 부정확하다. `691453`에는 후보 인자가 없다. 부모 LP의 준비·최적 상태·수치 상태를 확인하고 필요하면 재풀이한다. 후보 평가는 `691b56`이 맡는다. workers < 2 또는 n < 6이면 `266b6a`로 직접 들어가고, workspace+0x28 == 0이면 `695bd7` double 커널로 넘어간다. 이번 149회는 모두 이 double 경로다.

`691453`의 classification은 0(변화 없음), 1(다시 읽어 후보 재분할), 2(후보 정보를 다시 읽되 이번 SB 생략), 3(수치/기타 결과로 이번 SB 생략), −1(부모 infeasible)이다. 분기 근거는 C:90–173와 `62f909` C:266–365, 1064–1164다. 캡처는 204회 중 관측한 반환만 검사하며 이 분류표 전체를 동적으로 검증하지 않는다.

### 2.2 후보 분할과 SBMAXCANDS

`62ec1d`는 `6eed2e`가 불신으로 판정한 열을 모은다. extra == 0이면 앞부터 훑고, 불신 후보를 앞쪽 빈 칸과 교환한다. J와 목적 배열 두 개를 같은 순서로 교환한다. 반환은 start=0, count=불신 수다. extra > 0이면 첫 n개를 뒤부터 훑어 불신 후보를 뒤에 모으고, 뒤의 extra개는 앞부터 훑어 불신 후보를 이어 붙인다. work는 `(n+extra)*3 + count*6 + work0`이다(C:33–128). extra > 0은 이번 사례에서 0회다.

`62f909` C:381–425, asm `0x541bb9`–`0x541bfb`는 다음 값을 정한다.

```text
SBMAXCANDS == −1: cap=100, stall=2, workers=1
그 외: cap=SBMAXCANDS, stall=2000000000, workers는 트리 입력 값
평가 후보 수 = min(불신 후보 수, cap)
```

cap=0이면 평가하지 않고 정리한다. 후보가 cap보다 많거나 자동 정체 한도 2보다 많으면 예상 이득과 probing으로 순서를 정한다(C:433–618). 평가 뒤 결과 배열을 원래 순서로 돌려놓는다(`aa0345`, C:870). 예상 이득·probing의 점수 자체는 M2-06i 범위다. 평가 커널은 양방향 cutoff 또는 stall 조건으로 조기 종료할 수 있어 전달한 후보 수가 실제 풀이 수와 같지는 않다. 이번 자료는 자동 cap만 실행했다.

### 2.3 방향별 결과와 소비

LP 레코드 `L+0xc0`은 down 목적 배열, `L+0xc8`은 up 목적 배열이다. 초기값 −1e100은 미평가, 1e100은 infeasible을 나타낸다. 같은 후보의 평가 순서는 up, down이다. 정수 열의 up bound는 `floor(x + env+0x2468) + 1`, down bound는 `floor(x + env+0x2468)`이다(C `266b6a`:440–449, `695bd7`:433–442). env+0x2468의 덧셈을 생략하거나 up을 무조건 ceil(x)로 바꾸면 허용오차 경계가 달라진다. propagation(`6953ff`)이 모순을 찾으면 LP 풀이 없이 1e100을 쓴다. 연속 타입 C의 별도 경로는 정적 확인만 했다.

실제 LP 풀이 뒤에는 다음 규칙을 쓴다(`266b6a` C:494–576, 880–961와 double 대응 블록).

| LP 상태 | 해당 방향 목적 |
|---|---|
| 2 | max(ObjVal, 부모 LP ObjVal) |
| 3 또는 6 | 1e100 |
| 7, 11, −1000 | max(부모 LP ObjVal, workspace+0x138 dual bound) |
| 그 외 | 진단 출력 후 기존 값 유지 |

부모 LP ObjVal은 평가 커널이 읽은 값이다. 뒤의 pseudocost 소비가 쓰는 노드 base와 같은 필드라고 가정하지 않는다. 다른 후보의 해를 이용한 사전 결과 채움과 propagation 모순의 수치 유도는 후속 범위다. 이번에는 LP 풀이를 실제로 거친 742방향의 결과 저장과 상태 2·6·7을 비교했다.

`62f909`의 입력과 배율은 다음과 같다(C:403–411, 934–1029, asm `0x541b17`–`0x541bb0`). pow의 둘째 인자는 asm과 rodata로 확인했다.

```text
base = I+0x10
cut = min(5e7204(M), 5e99)
weight = pow(1.0 / (depth + 1.0), cut < 5e99 ? 0.1 : 0.2)
목적 = max(방향 목적, base)
방향 목적이 평가됐고 목적 < cut: Δ=(목적−base)*weight
목적 >= cut이고 cut < 5e99: Δ=(cut−base)*weight
```

둘 다 평가됐고 cutoff보다 작으면 `6ee5d1` 캐시를 쓴다. 각 방향은 `6edfcf`에도 Δ를 보낸다. cutoff에 닿은 정수 방향은 SBFIX 설정이면 반대 bound 조임을 기록하고, 두 방향 모두 cutoff이면 local_120을 세워 노드를 버리는 경로로 간다. bound 조임의 실제 적용·재평가는 `603734`가 맡는다. 이번 probe는 base·weight 비트와 후보 예산을 대조한다. cache/pseudocost 인자·조임 결과의 전체 소비 사슬은 기존 M2-06g 범위와 후속 M2-06i를 구분한다.

### 2.4 저장·복원표

동일 이름의 helper 두 계열은 구조체와 벡터 폭이 다르다. 함수 이름만으로 훅 주소를 선택하면 안 된다.

| 단계 | x87 계열(`266b6a`) | double 계열(`695bd7`) | 의미 |
|---|---|---|---|
| 저장 버퍼 할당 | `0x2a0430` | `0x576220` | workspace와 factor용 별도 버퍼 |
| 저장 `68fa4c..0` | `0x29ff60` | `0x576a30` | 현재 basis·수치 상태 복사 |
| 초기화 `68f0c1` | `0x2a51a0` | `0x575ce0` | LP 상태 2로 설정, 임시 상태 정리 |
| 복원 `68fa4c..1` | `0x2a49a0` | `0x5754e0` | 저장 basis·수치 상태와 factor 버퍼 복사, 커서 초기화 |
| LU/작업 객체 | +0x498 / +0x4a0 | +0x470 / +0x478 | 메모리 배치 차이 |
| 수치 벡터 | +0x230의 16B[m+n] | +0x1e0의 8B[m+n] | 이번 동적 비교는 double만 |

공통으로 workspace+0x168의 int[m+n] 상태, +0x418의 byte[m+n] flags, LU+0x18의 int[m] basic 인덱스를 복사한다. 이번 캡처는 여기에 위 수치 벡터와 workspace+0x294·+0x3ec·+0x2f0을 더해, 저장 source → 저장 destination, 고정된 저장 source → 매 방향 destination을 비교한다. factor 내부 전체는 검사하지 않는다. 정적 C에서는 `68fa4c..1`이 별도 버퍼의 factor 배열도 복원하고 permutation·cursor를 초기화한다(double C:143–294).

성공 경로의 순서는 저장 → bound 변경·전파 → up LP → 상태/목적/IterCount 소비 → reset → 저장 basis 복사 → bound 복구 → down 방향 반복 → cleanup → 부모 LP 재풀이이다. propagation만으로 방향이 infeasible이면 그 방향 LP는 생략한다. 원래 bound 복구는 전파가 없으면 해당 열, 전파가 있으면 변경 목록의 LB/UB를 사용한다. `266b6a` C:583–740, 963–1043과 double 대응 블록에 있다.

임시 env+0x2454(SIMPLEXENDFAC), +0x244c(PRIMDUALSWITCH), +0x241c는 cleanup에서 복구한다. `691b56`은 +0x2510(Sifting)을 0으로 두었다가 복구한다. `62f909`는 SB 반복 한도가 1보다 크면 +0x2960(IterationLimit)을 그 값으로 두고 `691b56` 뒤 원래 값으로 복구한다. `691453`은 +0x28bc(RCFIX)를 양수일 때 1로 제한했다가 반환 때 복구한다. 이 RCFIX 값은 이번 캡처에 포함하지 않았으므로 C/asm 근거다.

오류 경로는 완전 복원을 보장하지 않는다. bound 변경 후 solver/API가 비영 rc를 내면 방향별 복원을 건너뛰어 cleanup으로 갈 수 있다. 최종 부모 LP 재풀이도 rc==0일 때만 실행한다(`266b6a` C:1098–1175, `695bd7` C:1100–1174). `691453`의 solver 오류는 SIMPLEXENDFAC 복구를 건너뛴다(C:89 → 52, asm `0x56f60d`). 오류를 넣어 실행한 결과는 없으므로 이 부분은 정적 계약이다.

### 2.5 SB 비용

`691b56`의 비용 출력은 방향별 `GRBgetdblattr("IterCount")`를 실행 순서대로 더한 값이다. 마지막 부모 LP 재풀이의 반복 수는 이 합에 더하지 않는다. `62f909` C:858–864와 asm `0x542623`–`0x542660`이 `T+0x68 += 반환 비용`을 한다. 따라서 이 필드는 벽시계 시간이 아니라 SB 방향 풀이의 반복 누계다. 병렬 경로는 worker별 합을 더하고 실패하면 순차 재시도를 할 수 있다(`691b56` C:249–292). 이 병렬 실패 경로는 실행하지 않았다.

## 3. 비교 계약

| 대상 | 규칙 | 방법 |
|---|---|---|
| 원본 식별·재현 | 바이너리·모델·도구 해시, 설정, 독립 3회 사건 전체 동일 | manifest·읽기 전용 캡처 |
| 후보 분할 `62ec1d` | 신뢰성 질의 순서대로 후보·두 배열을 C의 swap 순서로 재생; 반환 시작/개수와 work 비트 일치 | 전후 배열·질의 결과 |
| 후보 평가 경계 | bound와 임시 파라미터 복구를 실제 관측 필드별 비트/정확 비교; 저장·복원 호출 필수 | `266b6a` 및 helper 훅 |
| SB 비용 | `691b56` 출력 반복 비용과 `T+0x68` 누적 비트 일치 | 전후 경계 |
| 목적·배율 | 목적 배열 의미, base·cutoff·depth 지수와 소비식을 C/asm으로 명시 | 정적 근거 + 읽기 전용 probe |
| 비간섭 | 훅 없는 같은 모델·설정의 최종 출력과 일치 | 대조 실행 |

부동소수는 u64 hex 비트 비교(오차 없음). 주소·padding은 비교하지 않는다. Threads=1, Seed=0, NodeLimit=300, 벽시계 종료 한도 없음. 탐색 1회 뒤 실행 분기를 기록하되, 자연 실행 0회를 통과로 바꾸지 않는다. 필수 사건 0회·필수 필드 null이면 검사 실패.

## 4. 사례와 원본 자료

자연 입력 `pk1`, `neos5`, `tr12-30`(`../re/bench/data/miplib/*.mps.gz`), 각 독립 3회. 모델 해시는 캡처 manifest에 기록한다. 탐색 결과로 실행 분기와 0회 분기를 나누어 기록한다. 실패 캡처는 삭제하지 않는다. 합성 호출로 자연 실행을 대체하지 않는다.

## 5. 재실행 명령

작업 worktree에서 `tools/hrun.sh --timeout 50000 -- 'tools/mip_sb_restore_runs.sh bench/oracle/m2_06g2/final'`(124면 --wait). Gurobi는 `tools/gurun.sh`로 직렬 실행한다. 판정 `python3 tools/mip_sb_restore_check.py --self-test`, 회귀 `python3 tools/mip_sb_read_check.py`. 비교기에는 바이너리/모델 해시 변조·빈 사건·0회 목표·후보/배열/work/복원/비용 값 변조를 넣어 모두 거부시킨다. Rust·Python 패키지 불변이므로 cargo/pytest는 해당 없음.

### 탐색 뒤 사례 보완(판정 완화 아님)

기본 설정 3개 모델은 NodeLimit=300에서 `691b56`이 0회였다. `T+0x64 == 1`일 때 `129879 → 12ec93` tableau penalty만 쓰기 때문이다. 탐색 파일 `explore_*.json.gz`를 보존한다. 후보 LP 풀이 필수 사건은 그대로 유지하고, 같은 세 모델에 원본 파라미터 `GURO_PAR_SBMAXITERS=10`을 명시한 사례를 추가한다. Frida로 원본 상태를 쓰지 않는다. pk1 탐색에서 batch 24·double kernel 24·restore 110회를 확인했다. 기본 설정 사례는 후보 분할·예산만, iters10 사례는 평가·복원·비용을 판정한다. helper에서 실제 복사하는 basis 상태·flags·수치 벡터·basic index를 비교하고, factor 내부 전부의 내용은 후속이다.

## 6. 실행 결과

기준 캡처 도구 commit `8ab7565`, 기준 main `a6b7702`. 원본은 Threads=1, Seed=0, NodeLimit=300이며, iters10 사례만 GURO_PAR_SBMAXITERS=10이다. [manifest](../../bench/oracle/m2_06g2/final/manifest.json)가 모델·바이너리·도구·외부 C·캡처 해시를 고정한다. OS·Node·아키텍처는 캡처 runtime에 있다.

| 사례 | 후보 분할 | 부모 확인 | 예산 | 실제 SB 묶음 |
|---|---:|---:|---:|---:|
| pk1 기본 | 286 | 5 | 23 | 0 |
| neos5 기본 | 283 | 6 | 19 | 0 |
| tr12-30 기본 | 310 | 44 | 111 | 0 |
| pk1 iters10 | 284 | 24 | 24 | 24 |
| neos5 iters10 | 280 | 14 | 14 | 14 |
| tr12-30 iters10 | 311 | 111 | 113 | 111 |

각 행은 독립 원본 3회가 캡처 전체·gz SHA까지 동일하다. 별도 무훅 대조 1회씩 총 6회와 Status·NodeCount·IterCount·ObjBound·SolCount·ObjVal이 동일하다. 내부 궤적 전체의 비간섭 증명은 아니다.

후보 분할 1,754회는 질의 순서·후보 순서·두 목적 배열·start/count·work 비트가 재생과 같다. SB 149묶음은 double 커널 결과 → 디스패처 결과 → 비용 누적을 비교했다. neos5의 4묶음은 binary 대칭 orbit 압축과 결과 재확장도 비교했다.

double 커널의 저장 149회와 방향 복사 742회에서 §2.4 필드가 source/destination 비트까지 같다. 평가 함수 전후 bound와 임시 파라미터, 최종 부모 LP 재풀이 진입 bound도 같다. 풀이 891회는 상태 2 373회·7 516회·6 2회다. 여기에는 마지막 부모 LP 재풀이 149회가 포함된다. 나머지 742방향에서 상태별 목적 저장과 IterCount를 비교했다. 비용 출력과 T+0x68 누적, IterationLimit 복구도 비트 일치다.

`python3 tools/mip_sb_restore_check.py --self-test`는 변조 15종을 거부했다. 바이너리/모델 해시, 빈 사건·0회 목표, 목적 벡터, work, basis 복사, 소비 결과, 복원 누락, bound, 비용, IterationLimit, weight, 질의 누락과 분기 bound 미적용을 각각 바꿨다. 변이 6종 중 swap work 생략·지수 0.1/0.2 교환은 6사례, 비용 누적 생략은 iters10 3사례에서 거부됐다. **동반 목적 벡터 교환 생략**은 입력 벡터 값이 같아 6사례 모두 판별 불가다. **상태 7의 base clamp 생략**도 관측 dual bound가 base 아래로 내려가지 않아 판별 불가다. 두 생존 변이는 검사기 기대표에 고정했다. 관측 상태 2·7의 base clamp 제거는 neos5 iters10의 최적 상태 한 건에서 거부된다. 기대 지수 변이의 초기 사례 목록에서 neos5 기본을 빠뜨려 self-test가 실패했고, 실제 여섯 사례 모두 거부되는 결과로 고쳤다. 판별 규칙은 바꾸지 않았다.

실패 자료 `explore_pk1_final.json.gz`는 GRBgetdblattr의 PLT `0xbcc70`에 훅을 붙여 DONE 전 종료된 탐색이다. 실제 export `0x7a3820`로 옮겨 해결했다. 성공 탐색과 최종 기준은 구분한다. hrun의 현재 pane 식별자가 유효하지 않아 첫 실행이 `pane_not_found`로 실패했다. 이후 `HERDR_ENV=0 tools/hrun.sh`의 직접 실행 경로를 사용했고, 각 명령의 실제 종료 코드를 확인했다.

Rust·Python 패키지는 변경하지 않아 cargo/pytest는 실행 대상이 아니다. 기존 M2-06g 저장 캡처 회귀·독립 검토 결과는 아래 최종 인계에 기록한다.

## 7. 판정과 인수인계

작업 수행 **완료(조사·오라클)**, 동등성 **이번 관측 필드·사례 범위 통과**. Rust 구현 동등성은 미검증이다.

검사기 `--self-test`는 종료 0(`ACCEPTANCE PASS (+ self-test)`), 기존 `python3 tools/mip_sb_read_check.py`도 종료 0(`ACCEPTANCE PASS`)이다. [판정 로그](../../bench/oracle/m2_06g2/check.log)와 [회귀 로그](../../bench/oracle/m2_06g2/regression.log)를 보존했다.

독립 검토는 정적 C/asm 재유도와 검사기 실행, 원본 neos5 iters10 재캡처 1회를 포함한다. 검토에서 분기 bound를 적용하지 않은 기록이 통과하는 결함을 찾았다. solve의 입력 LB/UB가 부모보다 느슨해지지 않고 선택 열의 방향 bound가 엄격히 조여졌는지 검사해 수정했고, 변조 15번째로 추가해 거부했다. 상태 2 clamp까지 제거하는 변이도 추가해 neos5에서 거부했다. 검토자가 만든 [재캡처](../../bench/oracle/m2_06g2/review_neos5_iters10.json.gz)는 final/neos5_iters10_1과 JSON·gz 바이트가 같다(SHA-256 `d1fa4f3703670267b363ad95a272a073ed262da9d7f47b76f2bc869bcef4eae2`). 

작업 branch는 `work/m2-06g-2`, worktree는 `../rsolver-m2-06g-2`다. 계약 `60622bd`, 첫 훅 `93ddf79`, 캡처 `8ab7565`, 검사기·최종 결과 `8eeeb72`로 단계를 보존했다. 미커밋 선행 변경은 없다. 병합은 main의 `Merge M2-06g-2`로 확인한다. 병합 뒤 작업 worktree와 branch를 정리하고 점유를 해제한다.

`docs/handoff/M2.md`, `11_progress.md`, `12_roadmap_diagram.md`, `14_sprint_plan.md`, `m2/sprint_design.md`를 갱신했다. 교훈은 `docs/lessons/original.md`와 `tooling.md`에 기록했다.

 다음 M2-06h-1은 LP 저장·복구 캡처를 먼저 준비하고, M2-06h-2는 그 입력으로 후보 하나의 제어를 Rust에서 재생한다. 전체 factor·방향별 원상 복구 순간·QP·x87 본문·Threads>1·오류·extra>0·cap 명시값·0회 LP 상태·허용오차 경계는 미검증이다. 현재 자료는 factor 전체 재생에 충분하지 않으므로 M2-06h를 06h-1(저장·복구 오라클)과 06h-2(L1 제어 구현)로 분리했다. 이 분할은 전체 factor 입력 없이 06h 구현 완료를 선언하지 않기 위한 것이다.
