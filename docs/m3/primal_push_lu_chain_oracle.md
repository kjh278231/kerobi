# M3-04c-21e 지연 LU 연쇄 수치 오라클 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 선행 조사는 [M3-04c-21](primal_push_lu_survey.md) §2.3·4.2·4.3, 같은 틀의 오라클은 [21a FTRAN](primal_push_ftran_oracle.md)·[21c BTRAN](primal_push_btran_oracle.md), 구현 adapter는 [21b](primal_push_ftran_rust.md)·[21d](primal_push_btran_rust.md)다.

## 1. 범위와 주장

- 작업 종류: **오라클 구축**. 기준 main `2ce74e1`(M3-04c-21d 병합), branch `work/m3-04c-21e`, worktree `../rsolver-m3-04c-21e`. 선행 미커밋 변경 없음. main의 미추적 `.claude/settings.local.json`은 작업과 무관하며 보존한다. 사용자의 "다음 M3 스프린트 진행해 너가 작업자야" 지시로 이 세션이 작업자로 수행한다(작업자 선택 질문 생략).
- 완료할 동작: crossover push 안에서 factor 처리 `05b257`이 불릴 때 **진입·반환의 factor 전체(L 열·행 방향, U 열·행, eta, 순열, 접두, 스칼라)·spike·Uᵀ cache F1d0/F1d8·현재 header·vstat·이탈 위치**를 저장하고, FT 갱신 경로의 행 eta 호출 `09e47c`와 재분해 경로의 `036ade` 경계(identity로 덮기 전 F28/F38)를 함께 읽기 전용 훅으로 수집한다. 선행21a/21c의 FTRAN·BTRAN 스냅숏과 한 사건열로 이어, 같은 push 호출 안에서 **factor가 FTRAN·BTRAN·factor 처리 사이에 보존되는지**(지원 생략 pivot에서 header만 바뀌는지), FT 갱신과 재분해·재번호가 C 규칙대로 상태를 바꾸는지를 원본 독립3회·선행21c 비간섭·판정 규칙·변조 거부로 고정한다.
- 포함 분기: 자연 arms/retry/interior와 21c의 Netlib adlittle·scsd1·capri에서 나오는 factor 처리 전부(선행 집계: 3모델 factor16 = 요청0 5·요청10 11, 지원 생략 pivot 22, header 변경7). 요청0 FT 갱신(행 eta 호출 유무·cache 재사용), 요청≠0 재분해(재번호·spike idx/헤더 버퍼 교환·identity 덮기), 그 뒤 재계산 FTRAN과 다음 공급 FTRAN·지원 BTRAN의 factor 입력을 자연 실행에서 나오는 만큼 센다. 미관측 분기는 미관측으로 기록하고 통과로 세지 않는다.
- 제외 범위: Rust 구현과 수치 비트 재생(21f), `036ade` 분해 자체의 내부 수치(Markowitz 피벗·L/U 값 생성은 M1-02 계열과 21f의 생성 동등성 판정), FT 갱신 내부 중간 상태(M1-02l/m/n/r가 simplex caller로 다룸), pricing/QP 객체가 있는 재번호 경로(S+0x478/0x480≠0), factor 반환1·실패·singular slack 교체(자연 발생 시 사건만 기록하고 별도 등록), 품질15a·루프22·batch23·x87·work 산술 동등성·전체 P3. 원본 수치 명령을 실행하는 합성 경계는 만들지 않는다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 ELF file offset.
- C: `../re/decomp/c/000f15c0_PRIVATE000000000005b257.c`(2,772줄), `000d4190_PRIVATE0000000000036ade.c`, `00131760_PRIVATE000000000009e47c.c`. 선행 FTRAN/BTRAN 본체와 커널은 21a·21c 문서 §2.
- 호출 지점(asm, `tools/dc.py 05b257 --asm`): `09e47c` 호출 `0xf1967`(반환 `0xf196c`; rdi=F, rsi=S+0x220, edx=이탈 위치, rcx=work), `036ade` 호출 `0xf34ed`(반환 `0xf34f2`; xmm0/xmm1 double 2개, rdi=S, 정수/포인터 인자 뒤 stack `[rsp+0x20]`=&local_120, `[rsp+0x28]`=&local_120+4 → Frida 인자 a[10]·a[11]. 반환 뒤 `0x8c(%rsp)`≠0이면 오류 종료 `0xf3501`). `05b257` 진입 `0xf15c0`에는 선행15 훅이 이미 있으므로 같은 attach 안에서 새 사건을 덧붙인다(주소당 attach 하나).
- 경로(원본 C, `05b257`):
  - 진입: `F=S+0x470`, 이탈 위치 `r=S+0x2f8`, 요청 `S+0x294`, `S+0x290`, pricing 객체 `S+0x478/0x480/0x488`.
  - **요청0·S+0x290=0 (FT 갱신, C:144–1335)**: spike `F+0x1c8`의 목록을 읽어 `i52=max(F40[r], F100−1)`, `i35=max(i52, max_{s∈spike.idx, s≠r} F40[s])`를 구하고(C:263–309), `i35≠i52`이고 `r≠F1d8`이면 `09e47c(F, S+0x220, r, work)`로 Uᵀ 행을 다시 구한다(C:311–320). `r=F1d8`이면 21c가 남긴 cache F1d0를 그대로 쓴다. 성공하면 `F18c+1`, spike count=0(C:1327–1329).
  - **요청≠0 (재분해, C:1336–)**: `036ade`가 F180·F184·F18c를 0으로 두고(`036ade` C:281–283) 분해한다. 성공 뒤 spike 목록 값을 0으로 지우고 count=0(C:2006–2033). S+0x478=0·S+0x480=0이면 `r≥0`일 때 `r'=F38[r]`, `head_new[p]=head_old[F28[p]]`를 spike idx 버퍼에 쓰고 **spike idx 버퍼와 header 버퍼를 교환**하며 `vstat[head_new[p]]=p`(C:2036–2090). 이어 `F40←F28`, `F48←F38` 포인터, `F28[i]=F38[i]=i`, `F1a0[i]=2e9`, U 열·행 끝을 다음 시작으로(C:2570–2595).
  - 공통 종료(C:2765–2770, 유일한 return): cache count=0, `F1d8=−1`.
- 시작 경계: push `134a87` 진입(선행 사건). 각 factor 처리의 시작은 `05b257` 진입, 종료는 반환 직후. 연쇄의 앞뒤 스냅숏은 21a `ftran:enter/leave`, 21c `btran:enter/leave`가 담는다.
- 벡터·포인터: 21a/21c와 같다. 포인터는 저장하지 않고 의미상 별칭(버퍼 동일성 불리언)만 저장한다. 재분해의 버퍼 교환은 `lu:leave`에서 "spike idx 버퍼 = 진입 header 버퍼", "header 버퍼 = 진입 spike idx 버퍼", "F40 버퍼 = F28 버퍼", "F48 버퍼 = F38 버퍼"로 기록한다.
- 모델·설정: 21c와 같은 7사례(arms/retry/interior/control/adlittle/scsd1/capri), Threads=1·Seed=0·Presolve=0·Method=2·Crossover=3·LPWarmStart=2, 대조군 arms Crossover=0. RNG·타이머 분기는 주장하지 않는다.

## 3. 비교 계약

| 필드·사건 | 의미·좌표 | 규칙 | 캡처 |
|---|---|---|---|
| `lu:enter` | 요청·S+0x290·이탈 위치·진입 위치(S+0x2fc)·pricing 객체 null 여부, factor 전체(21c `btFactor`: L 행 방향·level·cache·F1d8 포함), vstat 전체, F190·F1f0, work | 정수·목록 정확, f64 ±0 포함 비트 | `05b257` onEnter(선행15 `factor:enter` 뒤) |
| `lu:roweta` | `09e47c` 진입/반환: 행 인자·cache·F1d8·F180/F184 | 호출 여부·인자 정확 | 반환 주소 `0xf196c` 필터 |
| `lu:refactor` | `036ade` 진입/반환: header·순열 F20/28/30/38/40/48·spike count, 반환값·오류 출력 두 칸·S+0xf0, F180/184/18c | 정수 정확 | 반환 주소 `0xf34f2` 필터 |
| `lu:leave` | 반환값, `lu:enter` 필드 전체, 버퍼 교환 불리언 | 위와 같음 | `05b257` onLeave(선행15 `factor:leave` 뒤) |
| 선행 사건 | 21c agent의 전체 사건·최종 결과 | 새 `lu:*` 사건을 빼면 저장된 21c 캡처(`bench/oracle/m3_push_btran/*-1.json(.gz)`)와 JSON 전체 일치(pid/run_id·schema·agent/tool 해시 제외) | 같은 agent에 훅 추가 |

판정 규칙(결과를 보기 전 고정, 원본 C의 위 식에서 재계산). "factor"는 `ftran:*`·`btran:*`·`lu:*` 사건의 factor 스냅숏, 공통 키만 비교한다.

1. **식별**: push 안 `factor:enter/leave` 쌍마다 `lu:enter/leave` 1쌍, `lu:leave` 반환 = `factor:leave` 반환. `lu:roweta`는 요청0 호출 안에만, `lu:refactor`는 요청≠0 호출 안에만, 각각 enter/leave 쌍으로 있다. `lu:enter` 요청 = `factor:enter` 요청.
2. **factor 무결성**: 21a 규칙2(F8=m·ZERO 비트·역순열·시작/끝 단조·eta 수/길이 합)와 21c의 L 행=L 열 전치·level 범위를 `lu:enter/leave`에도 적용한다. scratch F1b0/F1b8 0.
3. **연산 사이 보존(연쇄)**: 같은 push 호출 안에서 factor 스냅숏을 사건 순서로 늘어놓을 때, 한 연산의 반환(또는 push의 첫 스냅숏) 뒤 다음 연산 진입까지 factor는 **header를 빼고 전부 같다**. 단 다음 연산이 spike가 있는 공급 FTRAN이면 spike는 wrapper가 지우므로 제외한다(21a 규칙5가 따로 판정). 이 규칙으로 지원 생략 pivot 사이 factor 보존(header만 바뀜)을 판정하고, 그런 구간 수와 header가 바뀐 구간 수를 센다.
4. **FT 갱신(요청0·S+0x290=0, `lu:refactor` 없음)**: (a) `lu:enter` spike = 직전 factor 스냅숏의 spike(규칙3 포함)이고 count≥1. (b) `09e47c` 호출 ⇔ `i35≠i52 ∧ r≠F1d8`(진입 상태로 §2 식 재계산), 호출 행 인자 = r. (c) 반환 0이면 `F18c` 반환 = 진입+1, spike count=0, header 불변. (d) `F180` 증가 ∈ {0,1}이고 1이면 `i35≠i52`.
5. **재분해(`lu:refactor` ≥1회, 마지막 `036ade` 반환0·오류 칸0, S+0x478=S+0x480=0)**: 마지막 `036ade` 반환의 F28을 `o`, F38을 `n`이라 하면 (a) `lu:leave` header[p] = `lu:enter` header[o[p]], (b) `lu:enter` r≥0이면 `lu:leave` S+0x2f8 = n[r], (c) 모든 p에서 vstat[header[p]] = p, (d) F28·F38은 identity, F40 버퍼=F28 버퍼·F48 버퍼=F38 버퍼, F1a0 전부 2,000,000,000, (e) spike count=0, spike idx 버퍼 = 진입 header 버퍼·header 버퍼 = 진입 spike idx 버퍼, (f) F180=F184=F18c=0(`036ade` C:281–283 이후 갱신 없음).
6. **공통 종료**: `lu:leave` cache count=0, F1d8=−1(C:2765–2770).
7. **재현·비간섭**: 바이너리·모델·도구 해시, 독립 pid/run_id 3회 사건 동일, 대조군 사건0, 21c 투영 일치.

- 수치: 모든 f64는 ±0 포함 비트 비교. NaN/IEEE무한대가 나오면 실패. FT 갱신·분해 수치의 독립 재계산은 21f다.
- work·시간: work 누적값·F190·F1f0은 기록만 하고 판정하지 않는다.
- 재현성: 자연 6사례·대조군 각 독립3회. 첫 탐색 1회는 `history/`에 보존하고 정식 판정에 쓰지 않는다. 탐색 뒤 필드표나 규칙을 바꾸면 정식 캡처 전에 이 문서에 이유를 적는다.

## 4. 사례와 원본 자료

| 사례 | 실제/합성 | 노리는 분기 | 입력 | 증거 |
|---|---|---|---|---|
| arms | 실제 | 지원 생략 17, 재분해(요청10) | `bench/oracle/m3_push_arms/arms.lp`+extra | 정식3회 |
| retry | 실제 | 지원 생략 4, 재분해 | `bench/oracle/m3_push_retry/retry.lp`+extra | 정식3회 |
| interior | 실제 | 요청0 FT 갱신 5, 재분해, 재계산 FTRAN | `bench/oracle/m3_primal_push_step/interior.lp` | 정식3회 |
| control | 실제 | Crossover=0, push 사건0 | arms | 정식3회 |
| adlittle·scsd1·capri | 실제(Netlib) | cache 경로 BTRAN 뒤 factor 처리, level L | `../re/bench/data/netlib/*.mps.gz`(저장소 밖, 해시 manifest) | 정식3회 |

- 도구: `tools/push_lu_{agent.js,capture.js,runs.py,check.py}`(21c agent 복사 + 훅 추가). 자료 `bench/oracle/m3_push_lu_chain/`. Netlib은 결정적 gzip.
- 미관측 분기는 사건 수와 함께 표로 남기고 통과로 세지 않는다.
- **탐색 뒤 계약 변경(정식 캡처 전).** (1) 첫 탐색(interior 1회, `history/explore-fail1/`)은 push 초기화의 첫 `05b257` 호출(caller `0x1c07c6`, 요청10)에서 factor 배열이 아직 없어(L 시작 배열 미초기화, 순열 null) 읽기가 실패했다. 재분해는 옛 factor 수치를 읽지 않으므로(C:1336–1455, `036ade`가 F+0x20 배열을 새로 씀) **`lu:enter`의 factor 전체는 요청0일 때만** 저장하고, 요청≠0이면 header·spike count·vstat·스칼라만 저장한다. `lu:refactor`의 순열은 null이면 null로 기록한다. 규칙3은 요청≠0 진입을 연결 비교에서 건너뛰고 반환부터 다시 잇는다. (2) 규칙3의 비교에서 L 행 방향의 level bucket(`F+0x1a8[m..]`)은 factor가 아니라 FTRAN 레벨 L·BTRAN 레벨 Lᵀ가 같이 쓰는 scratch라(21a §2, 21c C:620–624) 제외하고 변화 수만 센다. FTRAN 스냅숏(21a `factorState`)에 없는 스칼라 x140·x1e0은 공통 키만 비교한다. (3) 기대 호출 수를 탐색에서 고정했다: push 안 `05b257` arms4·retry3·interior9·adlittle14·scsd1 2·capri23(선행 집계 arms4·retry3·interior9와 일치). 판정 규칙의 기존 문구는 바꾸지 않았다. 탐색 7사례 모두 새 사건을 빼면 21c 정식 캡처와 JSON 전체가 같았다. (4) **정식 1차(21캡처, `history/formal-fail1/`)는 `--controls`에서 재현성으로 거부됐다.** 7사례 모두 다른 필드는 같고, push 초기화 첫 호출의 `lu:refactor` 진입 순열 F20/28/30/38만 회차마다 달랐다(미초기화 메모리). 이 칸은 `036ade`가 새로 쓰는 출력이라(진입 값을 읽지 않음) 진입 사건에서 순열을 빼고 header·spike count·F180/184/18c만 남겼다. 반환 쪽 순열과 판정 규칙은 그대로다. 같은 agent로 정식 캡처를 다시 했다.

## 5. 재실행 명령

- 캡처(작업 worktree 루트, Gurobi 한 프로세스): `tools/hrun.sh --timeout 580000 -- 'python3 tools/push_lu_runs.py <새-출력디렉터리>'`. 기존 출력 덮어쓰기 거부.
- 판정: `python3 tools/push_lu_check.py --controls`, `--acceptance`, `--self-test`.
- 회귀: `push_btran_check.py --controls --acceptance --self-test`, `push_ftran_check.py --controls --acceptance --self-test`, `push_btran_rust.py --compare`·`push_ftran_rust.py --compare`(cargo test 선행), py_compile·`node --check`. Rust 변경이 없으면 workspace 전체·clippy·확장·pytest·Netlib·P3는 제외한다.

## 6. 실행 결과

- 환경: WSL2 x86-64 Linux 6.6.87.2, Node v22.22.1, Python 3.10(검사기)·3.13(gurobipy venv). 바이너리 해시는 매 캡처에서 위 값과 일치했다. 정식 도구 해시 agent `159da25a…`, capture `06eae883…`(전체 값은 [manifest](../../bench/oracle/m3_push_lu_chain/manifest.json)). 계약 `30e1d03`, 도구·탐색·계약 보강 `09202ef`, 정식1차 실패 보존·agent 수정 `d0c8bbf` 이후 정식 캡처.
- 원본 실행: Gurobi 프로세스 총 50회. 탐색 실패 1회(`history/explore-fail1/`), 탐색 7회(`history/explore/`), 정식1차 21회(재현성 거부, `history/formal-fail1/`), 정식 21회. 탐색·정식1차 자료는 판정에 쓰지 않았다.
- **재현성·비간섭**(`--controls` 종료0): arms 662사건(새 16)·retry 249(12)·interior 288(32)·adlittle 547(52)·scsd1 237(8)·capri 832(80)·대조군 0사건이 각각 독립 pid/run_id 3회에서 동일하다. 새 `lu:*` 사건을 빼면 21c 정식 캡처(`m3_push_btran/*-1`)와 사건·최종 결과·출력 전체가 같다.
- **판정 규칙**(`--acceptance` 종료0, [보고](../../bench/oracle/m3_push_lu_chain/verification.json)): push 안 `05b257` **55호출**(arms4·retry3·interior9·adlittle14·scsd1 2·capri23) 전부 규칙 1–6 통과, 연산 사이 연결 188개 전부 규칙3 통과.

| 분기 | 관측 수 | 비고 |
|---|---:|---|
| FT 갱신(요청0) / 재분해(요청10) | 36 / 19 | FT는 interior5·adlittle11·capri20. 재분해 19 = push 초기화(`0x1c07c6`) 6 + 라운드(`0x1c3d31`)·pivot 13 |
| FT: 행 eta `09e47c` 호출 / cache 재사용(F1d8=r) / 구간 자명(i35=i52) | 26 / **0** / 10 | cache 재사용 0: 지원 BTRAN 뒤에 FT 갱신이 오는 사례가 없다 |
| FT: F180 +1 / +0 | 23 / 13 | +1은 모두 i35≠i52. 행 eta를 불렀지만 eta가 늘지 않은 3회 포함 |
| 재분해: `036ade` 호출 / 반환0·오류 칸0 | 19 / 19 | 반환≠0·반복 호출·singular 0 |
| 재분해: header 순서 변경 / 옛 순열 비identity / 이탈 위치 재번호 | 15 / 15 / 18 | 모든 재분해에서 pricing 객체 null, header 재번호식·vstat·identity·F1a0·버퍼 교환 성립 |
| 연산 사이 연결 / header만 바뀐 연결 | 188 / 68 | header만 바뀐 연결이 지원 생략 등 factor 갱신 없는 pivot이다. factor는 header 외 전부 보존 |
| 연결 중 bucket(F1a8) 변화 | 19 | 모두 capri(레벨 L/Lᵀ scratch), 판정 제외 |
| 지원 BTRAN 직전 factor 처리 | 재분해 32/32 | 그래서 BTRAN eta>0은 여전히 0회 |
| 공급 FTRAN eta>0 | 41 | interior5·adlittle14·capri22(21a는 interior5만) |

- **변조 거부**(`--self-test` 종료0): 54종. 출처·재현·비간섭9(바이너리·manifest·모델 해시, 빈 사건, 회차 차이, 21c 투영 차이 2종, 기준 결과, 새 사건0), 규칙1 6(lu enter/leave 누락, null-null, 반환, 요청, factor 밖 사건, 재분해 안 행 eta), 규칙2 4, 규칙3 8(지원 생략 연결 factor 변경, **pivot마다 재분해**·**pivot마다 FT 갱신** 변이, FT·재분해 뒤 factor 변경, spike 연결, F1d8 연결), 규칙4 11(행 eta 누락/추가/인자, cache 재사용 판정, 구간 판정, F18c, spike 잔류, header 변경, eta 증가 2, 구간 자명인데 eta 증가, 버퍼), 규칙5 14(header 재번호·옛 순열·이탈 위치·new_of_old·vstat·identity·F40/F48·버퍼 교환·F1a0·F18c 초기화·`036ade` 반환/오류·pricing·spike 잔류), 규칙6 2. 처음 세 변조(구간 판정·eta 증가2·자명 구간 eta)가 목표 규칙보다 앞선 무결성 검사(순열·eta 수)에서 거부돼, 협조 변조로 고쳐 목표 규칙에서 거부됨을 확인했다.
- **회귀**([요약](../../bench/oracle/m3_push_lu_chain/regression/summary.txt), 14명령 모두 종료0): 21c `push_btran_check.py --controls`·`--acceptance`, 21a `push_ftran_check.py --controls`·`--acceptance`, `cargo test -p rsolver-lp --test push_btran`·`push_btran_rust.py --compare`(21d), `--test push_ftran`·`push_ftran_rust.py --compare`(21b), `push_lu_survey.py --check`, py_compile·`node --check` 3종, 이번 검사기 전체 실행, `git diff --check`.
- 실행하지 않은 검사: Rust·확장 코드 변경이 없어 workspace 전체·clippy·확장 설치·pytest·Netlib 풀이 비교·P3는 제외했다. 21a/21c의 self-test는 그 검사기가 바뀌지 않아 다시 돌리지 않았다. 독립 검토자는 쓰지 않았다(사용자 지시로 이 세션이 작업자·마스터를 겸함).
- 계약 변경: §4 "탐색 뒤 계약 변경" (1)–(4). 판정 규칙 문구는 그대로이고, 필드표에서 미초기화 칸(요청≠0 진입 factor, `036ade` 진입 순열)을 뺐다.

## 7. 판정과 인수인계

- 작업 수행: **오라클 구축 완료**(계약 범위). 원본 식별·독립3회·21c 비간섭·C 규칙 판정·변조 54종 거부를 확인했다. 독립 검토는 미실시다.
- 확인한 원본 동작(캡처 근거): 같은 push 안에서 factor는 FTRAN·BTRAN·factor 처리 사이에 header 외 전부 보존된다(188연결). 요청0 FT 갱신은 C:263–320의 구간 식대로 `09e47c`를 부르고 F18c+1·spike 비움·header 불변이다(36). 재분해는 `036ade` 반환 순열로 header·이탈 위치·vstat를 재번호하고 spike idx/header 버퍼를 교환한 뒤 순열을 identity로 덮는다(19). 모든 종료에서 cache count=0·F1d8=−1이다.
- 동등성: Rust 구현을 포함하지 않는다. FT 갱신 수치(U/eta/접두 갱신)·`036ade` 분해 수치·재번호의 Rust 비트 비교는 **미검증**(21f).
- 기록만 하고 판정하지 않는 필드: work·F190·F1f0, `lu:enter` vstat의 비기저 부분, 재분해 뒤 L/U 값 자체(무결성만).
- 남은 한계(자연 미관측): FT 갱신의 cache 재사용(F1d8=r), 요청0·S+0x290≠0 경로, factor 반환≠0·`036ade` 반복/실패·singular, pricing/QP 객체 경로, 지원 BTRAN eta>0(32회 모두 직전이 재분해). 지원 BTRAN이 항상 재분해 뒤에 오는지는 push C로 확인하지 않았다.
- 다음: **M3-04c-21f**(직렬 numeric backend의 factor 수명·FT·재분해·재번호·rhs). `python3 tools/session.py claim M3-04c-21f --owner <새-owner>` → 이 문서 §2·§6, `primal_push_ftran_rust.md`·`primal_push_btran_rust.md`, M1-02l/m/n(FT 갱신)·M1 분해 기록을 읽고, `bench/oracle/m3_push_lu_chain/*-1.json(.gz)`의 push 첫 `lu:leave` factor를 초기 상태로 같은 push의 FTRAN·BTRAN·FT 갱신·재분해 연쇄를 원본 중간 factor 재주입 없이 비교한다. 재분해 수치(`036ade`)를 원본 입력으로 둘지 Rust 생성으로 둘지 계약에서 먼저 정한다. 기준 재검사 `python3 tools/push_lu_check.py --controls --acceptance`.
- 재실행: `python3 tools/push_lu_check.py --controls --acceptance --self-test`. 새 캡처 `tools/hrun.sh --timeout 580000 -- 'python3 tools/push_lu_runs.py <새-출력디렉터리>'`.
- branch `work/m3-04c-21e`, worktree `../rsolver-m3-04c-21e`, 선행 미커밋 없음. 최종 commit·병합은 main `Merge M3-04c-21e` 이력을 따른다. main 미추적 `.claude/settings.local.json` 보존. 원격/업로드 없음.
- 교훈: `docs/lessons/original.md`(push 안 factor 보존·FT 행 eta 판정·재분해 재번호, 지원 BTRAN은 재분해 뒤), `tooling.md`(push 초기화 첫 factor 호출과 `036ade` 출력 칸의 미초기화 메모리).
