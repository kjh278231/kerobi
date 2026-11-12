# M3-04c-21c 지원 BTRAN 수치 오라클 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 선행 조사는 [M3-04c-21](primal_push_lu_survey.md) §2.2·4.2·4.3, 같은 틀의 FTRAN 오라클은 [M3-04c-21a](primal_push_ftran_oracle.md)다.

## 1. 범위와 주장

- 작업 종류: **오라클 구축**. 기준 main `6d52021`(M3-04c-21b 병합), branch `work/m3-04c-21c`, worktree `../rsolver-m3-04c-21c`. 선행 미커밋 변경 없음. main의 미추적 `.claude/settings.local.json`은 작업과 무관하며 보존한다. 사용자의 "다음 M3 스프린트 진행해 너가 작업자야" 지시로 이 세션이 작업자로 수행한다(작업자 선택 질문 생략).
- 완료할 동작: crossover push 지원 라운드의 BTRAN wrapper `09e376`이 본체 `221173`을 cache 인자(F+0x1d0)와 함께 부를 때, **같은 호출의 초기 factor 상태(L 열·행 방향, U 열·행, eta, 순열)·입력 벡터·cache F1d0/F1d8·Uᵀ 커널 `21a941` 경계·raw 출력·입력 후상태·버퍼 별칭·scratch**를 읽기 전용 훅으로 수집해 원본 독립3회·선행21a 비간섭·판정 규칙·변조 거부로 고정한다.
- 포함 분기: 자연 arms/retry/interior의 지원 BTRAN 22회(선행19 집계: 17/4/1, 모두 입력 count1)와, 탐색 뒤 추가한 Netlib 자연 실행 adlittle·scsd1·capri의 10회(§4). U 행 빈/비영, 강제 희소 Uᵀ, Uᵀ 출력 count0 조기 종료, cache 복사, eta 없음/있음과 그 sparse/dense 경로, Lᵀ sparse/dense, 최종 sparse 출력의 F+0x20 치환·dense 출력의 F+0x30 gather와 버퍼 교환 중 **자연 실행에서 나오는 만큼** 센다. 미관측 분기는 미관측으로 기록하고 통과로 세지 않는다.
- 제외 범위: Rust 구현과 수치 비트 재생(21d), 지연 LU·FT 갱신·재분해 연쇄(21e·21f), cache를 소비하는 `05b257` FT 갱신의 수치, 일반 BTRAN `09e288`·simplex caller, L 레벨 경로(F+0x140≥0)의 수치, 품질15a·루프22·batch23·x87·work 산술의 동등성·전체 P3. 원본 수치 명령을 실행하는 합성 경계는 이번 범위에서 만들지 않는다. 자연 미관측 분기(입력 count≥2·dense 입력·sparse eta·level Lᵀ·±ZERO 정확 경계)는 21d 착수 전 보강 후보로 남긴다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 ELF file offset(로드 base + offset). Ghidra 레이블 `LAB_0022xxxx`는 file offset `0x12xxxx`다(+0x100000).
- C: `../re/decomp/c/00131710_PRIVATE000000000009e376.c`(wrapper), `0012de60_PRIVATE0000000000221173.c`(본체, 1,766줄), `00130b70_PRIVATE000000000021a941.c`(희소 Uᵀ 커널). 동명 x87(`0x26ce50`·`0x26fc10`)은 제외한다.
- wrapper asm(`0x131710–0x131750`): `rcx = *(F+0x1d0)`, F+0x200=0 뒤 본체 호출(`0x13172b`, 반환 `0x131730`). 본체 진입 asm `0x12de71–0x12de7a`: rbx=F, r13=in, r12=out, r15=cache. 본체가 부르는 함수는 `21a941`(`0x12e586`, 반환 `0x12e58b`)과 memset(`0x12f6b3`, level Lᵀ)뿐이다. 커널 인자(asm `0x12e567–0x12e582`, 커널 진입 `0x130b84–0x130c04`): `(xmm0, rdi=F, esi=m, rdx=F+0x20, rcx=in, r8=out, r9=F+0x1b0, [rsp]=F+0x1b8)`.
- 시작 경계: `221173` 진입(wrapper가 F+0x200=0을 쓴 뒤). 종료: `221173` 반환 직후. wrapper 반환 raw는 선행19 `roundctl:btran:leave`가 담는다.
- 벡터: `{i32 count; i32* idx @8; f64* val @16}`. sparse val은 목록 위치별 packed, count=−1이면 m개 dense. cache F+0x1d0은 같은 구조체 포인터이고 F+0x1d8은 i32 행이다.
- 읽는 factor 상태(C의 `param_1 + off`, 커널은 F+0x20 기준): m F+8, ZERO F+0x10, 순열 F+0x20/0x30(sparse 출력 치환·dense gather), U 순서 F+0x48, 대각 F+0x50, L 열 시작 F+0x58, **L 행 방향 시작 F+0x70(long m+1)·값 F+0x78·열 F+0x80**, U 행 시작/끝 F+0xb0/0xb8·값 F+0xc8·열 F+0xd0, F+0xd8·0xe4·0xf8·0x100·0x104·0x108·0x110, level F+0x130/0x138/0x140, eta F+0x180/0x184/0x198, nnzL F+0x1e0, scratch F+0x1a8(level)·0x1b0(i32 표시)·0x1b8(f64 값), work F+0x200. 21a `factorState` 전체(U 열·eta 풀 offset/mod16·spike 포함)에 L 행 방향·level·F1e0·cache·F1d8을 더해 저장한다.
- 경로 판정(원본 C):
  - cache 경로(C:1197–1227): cache≠null이면 `cache.count=0`, `r=in.idx[0]`. U 행 r이 비면(`F+0xb0[r]==F+0xb8[r]`) cache=null로 일반 경로(C:86). 아니면 `F+0x1d8=r`, 희소 Uᵀ `21a941` 강제, 출력 count0이면 즉시 반환, count>0이면 출력 목록·값을 cache에 복사하고 cache.count=count.
  - 일반 경로(C:86–90): `count≥0`이고 `(int)(F108+F110)·count ≤ m`(asm `0x12dee6–0x12deee` cvttsd2si)이면 희소 Uᵀ, 아니면 조밀 Uᵀ(본체 안).
  - eta(C:1231–1572): F+0x180=0이면 Lᵀ로. Uᵀ 출력 count≥0이고 `m < F104·count`이면 조밀 eta, count≥0이고 아니면 희소 eta, Uᵀ 조밀이면 조밀 eta. 조밀 eta 뒤는 조밀 Lᵀ(C:1573–1766).
  - Lᵀ(C:263–): count≥0이고 `F104·count ≤ m`이며 `F140 < 0.1m`이면 희소 Lᵀ(출력 idx를 F+0x20으로 치환, C:1174–1185), 아니면 조밀 Lᵀ(F+0x30 gather, 출력 count −1, C:1736–1764).
- 모델·설정: 선행19/21a와 같은 arms/retry/interior LP·extra, Threads=1·Seed=0·Presolve=0·Method=2·Crossover=3·LPWarmStart=2, 대조군 arms Crossover=0. RNG·타이머 분기는 주장하지 않는다.

## 3. 비교 계약

| 필드·사건 | 의미·좌표 | 규칙 | 캡처 |
|---|---|---|---|
| `btran:enter` | 호출 식별(id, 라운드 frame 인자 일치), 위 factor 상태 전체, 입력, out count, cache, F1d8, F+0x200 | 정수·목록 정확, f64 ±0 포함 비트. 포인터는 저장하지 않고 의미상 별칭만 | `221173` onEnter(반환 주소 0x131730, push 활성·지원 라운드 안) |
| `btran:kernel` | `21a941` 진입/반환의 in/out, 인자 일치, cache·F1d8·scratch | 사건 순서·인자·벡터 비트 | 커널 onEnter/onLeave(반환 주소 0x12e58b) |
| `btran:leave` | raw out, 입력 후상태, 별칭, factor 전체, cache·F1d8 | factor는 cache·F1d8·F+0x200 외 진입과 동일 | `221173` onLeave |
| 선행 사건 | 선행21a agent의 전체 사건·최종 결과 | 새 사건을 빼면 저장된 21a 캡처와 JSON 전체 일치(pid/run_id·schema·agent/tool 해시 제외) | 같은 agent에 훅 추가 |

판정 규칙(결과를 보기 전 고정, 원본 C의 위 식에서 재계산):

1. **식별**: 선행 `roundctl:btran:enter/leave` 한 쌍마다 본체 enter/leave 1쌍, 커널 사건은 enter/leave 쌍으로 본체 안에만 있다. 본체 반환 out은 선행 wrapper 반환 raw와 count·목록·값 비트 동일. 본체 인자 factor/in/out은 라운드 frame 위치, cache는 `*(F+0x1d0)`이며 null이 아니다.
2. **factor 무결성**: F+8=m, F+0x10=1e-13 비트, F20↔F30·F28↔F38·F40↔F48 역순열, L 열·L 행·U 열·U 행 시작/끝이 단조·범위 안, L 열 방향과 L 행 방향이 같은 비대각 원소 집합(값 비트 포함)의 전치, eta 레코드 수=F180·길이 합=F184. 진입·커널 경계·반환의 factor 필드는 cache·F1d8·F+0x200 외 동일. scratch F1b0/F1b8은 진입·커널 진입/반환·본체 반환에서 모두 0.
3. **cache 경로**: r=입력 idx[0]. U 행 r이 비면 커널은 일반 경로 판정을 따르고 반환 cache.count=0·F1d8 불변. 비어 있지 않으면 커널 진입 시 F1d8=r·cache.count=0이고 커널이 반드시 1회 불린다. 커널 출력 count=0이면 본체 반환 out count=0·cache.count=0, count>0이면 반환 cache = 커널 반환 out의 count·목록·값 비트, count<0이면 cache.count=0.
4. **Uᵀ 경계**: 커널 입력 = 본체 진입 입력(count·목록·값 비트, 치환 없음). 일반 경로에서 커널 호출 여부 = 위 희소 Uᵀ 판정식.
5. **eta·Lᵀ 경로와 출력 표현**: 위 판정식으로 Uᵀ 출력(커널 반환 count)에서 경로를 정한다. eta 없음이면 Lᵀ 판정을 Uᵀ 출력 count로 바로 계산해 본체 반환 count 부호(sparse ≥0 / dense −1)가 일치해야 한다. 조밀 eta로 가면 반환 count=−1. 희소 eta는 eta 뒤 count를 캡처하지 않으므로 반환 표현만 기록하고 경로 판정을 통과로 세지 않는다(관측 시 미판정으로 보고).
6. **출력·별칭**: sparse 반환은 목록이 모두 [0,m) 안이고 중복이 없다. dense 반환은 m개. 본체 반환 out/in 구조체의 idx/val 버퍼 별칭(in/out 교환 여부)을 기록하고, 입력 후상태를 기록한다.
7. **재현·비간섭**: 바이너리·모델·도구 해시, 독립 pid/run_id 3회의 사건 동일, 대조군 사건0, 선행21a 투영 일치.

- 수치: 모든 f64는 ±0 포함 비트 비교. NaN/IEEE무한대가 나오면 실패. Uᵀ·eta·Lᵀ 수치의 독립 재계산은 21d다.
- work·시간: F+0x200 누적값은 기록만 하고 판정하지 않는다.
- 재현성: 자연 3모델·대조군 각 독립3회. 첫 탐색 1회는 `history/`에 보존하고 정식 판정에 쓰지 않는다. 탐색 뒤 필드표를 바꾸면 정식 캡처 전에 이 문서에 이유를 적는다.

## 4. 사례와 원본 자료

| 사례 | 실제/합성 | 노리는 분기 | 입력 | 증거 |
|---|---|---|---|---|
| arms | 실제 | 지원 BTRAN 17(count1) | `bench/oracle/m3_push_arms/arms.lp`+extra | 정식3회 |
| retry | 실제 | 지원 BTRAN 4 | `bench/oracle/m3_push_retry/retry.lp`+extra | 정식3회 |
| interior | 실제 | 지원 BTRAN 1, 요청0 갱신 뒤 eta | `bench/oracle/m3_primal_push_step/interior.lp` | 정식3회 |
| control | 실제 | Crossover=0, push 사건0 | arms | 정식3회 |
| adlittle | 실제(Netlib) | U 행 빈 일반 경로 1·U 행 비영 cache 기록 뒤 조밀 Lᵀ 1 | `../re/bench/data/netlib/adlittle.mps.gz`(저장소 밖, 해시 manifest) | 정식3회 + 21a agent 기준선1회 |
| scsd1 | 실제(Netlib) | U 행 비영 cache 기록 뒤 희소 Lᵀ 1 | `../re/bench/data/netlib/scsd1.mps.gz` | 정식3회 + 기준선1회 |
| capri | 실제(Netlib) | L level 구조(F+0x140≥0)에서 U 행 빈 2·비영 5, 희소/조밀 Lᵀ | `../re/bench/data/netlib/capri.mps.gz` | 정식3회 + 기준선1회 |

- 도구: `tools/push_btran_{agent.js,capture.js,runs.py,check.py}`(21a agent 복사 + 훅 추가). 자료 `bench/oracle/m3_push_btran/`. Netlib 캡처·기준선은 크기 때문에 결정적 gzip(`mtime=0`)으로 저장한다.
- **탐색 뒤 계약 변경(정식 캡처 전).** (1) 첫 탐색(`history/explore/`)에서 선행 자연 22호출이 모두 한 경로(U 행 빈 → 일반 희소 Uᵀ 출력 count1 → eta0 → 희소 Lᵀ)였다. 계약 분기(cache 기록·강제 Uᵀ·조밀 Lᵀ)를 실제 실행으로 얻으려고 경량 조사 훅 `tools/push_btran_survey{,_agent}.js`로 Netlib 소형 45모델(`history/survey-netlib45.json.gz`)과 중형 9모델(`survey-netlib-mid32.json.gz`, ship04l에서 size-limited 라이선스 오류로 중단)을 풀었다. 54모델 401호출에서 U 행 비영·cache 기록·커널 출력 count≥2·조밀 Lᵀ·level 구조는 나왔고 **eta>0·조밀 Uᵀ(커널 미호출)·커널 출력0·입력 count≥2는 0회**였다. 조사 자료의 모델 귀속은 stdout 줄과 Frida 메시지의 도착 순서에 의존해 근사다(stocfor1 2회로 집계됐으나 단독 실행 0회). (2) 전체 agent로 6모델을 1회씩 탐색(`history/explore2/`)해 adlittle·scsd1·capri를 정식 사례로 골랐다(recipe·share2b는 같은 분기를 더 큰 자료로 반복). Netlib은 선행21a 저장 캡처가 없으므로 **21a agent로 기준선 1회를 따로 캡처**해 비간섭 투영 기준으로 쓴다. (3) level 배열을 C:620–624(`F+0x130[행]`·`F+0x138[레벨]`, bucket `F+0x1a8[m..m+F140]`)로 확인해 `F+0x138` 길이를 F140+1로 하고 bucket을 기록 필드로 더했다. bucket 변화는 level Lᵀ 실행의 기록 증거이고 판정 규칙이 아니다. (4) 규칙2에 L 행 방향=L 열 방향의 전치(원소·값 비트)와 level 범위, 규칙1에 wrapper 입력=본체 입력, 규칙3에 U 행 빈 경우 F1d8 불변을 명시했다. 커널 출력0은 cache 여부와 관계없이 즉시 반환한다(C:1209–1211)는 점을 규칙3·5에 반영했다. 판정 규칙의 기존 문구는 바꾸지 않았다.
- 미관측 분기는 사건 수와 함께 표로 남기고 통과로 세지 않는다.

## 5. 재실행 명령

- 캡처(작업 worktree 루트, Gurobi 한 프로세스): `tools/hrun.sh --timeout 580000 -- 'python3 tools/push_btran_runs.py <새-출력디렉터리>'`. 기존 출력 덮어쓰기 거부.
- 판정: `python3 tools/push_btran_check.py --controls`, `--acceptance`, `--self-test`.
- 회귀: `push_ftran_check.py --controls --acceptance --self-test`, `push_round_check.py --controls --acceptance --self-test`, `push_lu_survey.py --check`, `push_ftran_rust.py --compare`(21b), py_compile·`node --check`. Rust 변경이 없으면 workspace 전체·clippy·확장·pytest·Netlib·P3는 제외한다.

## 6. 실행 결과

- 환경: WSL2 x86-64 Linux 6.6.87.2, Node v22.22.1, Python 3.10(검사기)·3.13(gurobipy venv). 바이너리 해시는 매 캡처에서 위 값과 일치했다. 정식 도구 해시: agent `99092edf…2f50d3`, capture `653cb399…8b7a01`(전체 값은 [manifest](../../bench/oracle/m3_push_btran/manifest.json)). 계약 `0a594b2`, 도구·탐색·계약 보강 `ddc92c6`.
- 원본 실행: Gurobi 프로세스 총 36회. 탐색 1차 4회(`history/explore/`), 조사 2프로세스(Netlib 54모델), 탐색 2차 6회(`history/explore2/`), 정식 21회(7사례×3), Netlib 21a agent 기준선 3회(`baseline/`). 탐색·조사 자료는 판정에 쓰지 않았다.
- **재현성·비간섭**(`--controls` 종료0): arms 646사건(새 68)·retry 237(16)·interior 256(4)·adlittle 495(8)·scsd1 229(4)·capri 752(28)·대조군 0사건이 각각 독립 pid/run_id 3회에서 동일하다. 새 `btran:*` 사건을 빼면 arms/retry/interior/control은 선행21a [`m3_push_ftran`](../../bench/oracle/m3_push_ftran/) `*-1.json`, Netlib은 21a agent 기준선과 사건·최종 결과·출력 전체가 같다.
- **판정 규칙**(`--acceptance` 종료0, [보고](../../bench/oracle/m3_push_btran/verification.json)): 지원 BTRAN **32호출**(arms17·retry4·interior1·adlittle2·scsd1 1·capri7) 전부 규칙 1–6 통과.

| 분기 | 관측 수 | 비고 |
|---|---:|---|
| 입력 count1 / ≥2 | 32 / 0 | 지원 caller는 이탈 위치 단위 벡터(C:1999–2023)라 ≥2는 이 caller에서 구조적으로 없음 |
| U 행 r 빈(일반 경로) / 비영(cache 경로) | 25 / 7 | 비영 7은 adlittle1·scsd1 1·capri5. 선행 22회는 모두 빈 행 |
| 희소 Uᵀ 커널 호출 / 조밀 Uᵀ | 32 / 0 | 일반 경로 판정 `(int)(F108+F110)·1 ≤ m`이 모두 참 |
| 커널 출력 count1 / ≥2 / 0 / −1 | 25 / 7 / 0 / 0 | ≥2는 모두 cache 경로 |
| cache 기록(F1d0=Uᵀ 출력, F1d8=r) | 7 | 빈 행 25회는 cache.count=0·F1d8 불변 |
| eta 0 / 조밀 eta / 희소 eta | 32 / 0 / 0 | 54모델 조사에서도 eta>0은 0회 |
| Lᵀ 희소 / 조밀 | 30 / 2 | 조밀 2는 adlittle·capri의 cache 경로(출력 count −1, out 구조체가 자기 버퍼 유지) |
| L level 구조(F140≥0) / bucket 변화 | 7 / 6 | 모두 capri. 희소 Lᵀ 6회에서 bucket(F1a8[m..])이 바뀌어 level 경로 실행을 기록, 조밀 Lᵀ 1회는 불변 |
| 비identity 순열 | 32 | |
| 반환 out 버퍼 = 입력 쪽(희소) / 자기 쪽(조밀) | 30 / 2 | 기록 필드 |

- **변조 거부**(`--self-test` 종료0): 41종. 바이너리·manifest·모델 해시, 빈 사건, 회차 차이, 선행21a·Netlib 기준선 투영 차이, 기준 결과, 새 사건0, null-null, 본체 leave·커널 누락, wrapper 밖 사건, 규칙1(cache 인자·wrapper 입력), 규칙2(ZERO·역순열·L 행 값/열·level 범위·scratch·커널 scratch·factor 변경), 규칙3(cache 값·순서·누락, 반환/커널 진입 F1d8, 빈 행 cache·F1d8, U 행 협조 위조), 규칙4(커널 입력·인자, 일반 경로 판정 입력), 규칙5(x104·x140 협조 변조, 조밀 eta 위조, 조밀 출력을 희소로), 규칙6(출력 비트·중복·wrapper raw). 모든 거부 메시지가 해당 규칙의 첫 차이를 가리키는지 확인했다(처음 `R2-factor-changed`는 scsd1 대각 0번이 원래 1.0이라 같은 값으로 바꿔 거부되지 않았고, 비트 반전으로 고쳤다).
- **회귀**([요약](../../bench/oracle/m3_push_btran/regression/summary.txt), 16명령 모두 종료0): 21c controls/acceptance/self-test, 선행21a `push_ftran_check.py` 3종, `push_round_check.py --controls --acceptance --self-test`, `push_lu_survey.py --check`, `cargo test -p rsolver-lp --test push_ftran`·`push_ftran_rust.py --compare`(21b), py_compile·`node --check` 4종, `git diff --check`.
- 실행하지 않은 검사: Rust·확장 코드 변경이 없어 workspace 전체·clippy·확장 설치·pytest·Netlib 풀이 비교·P3는 제외했다. 독립 검토자는 쓰지 않았다(사용자 지시로 이 세션이 작업자·마스터를 겸했고 별도 검토 세션을 띄우지 않음).
- 계약 변경: §4의 "탐색 뒤 계약 변경"에 정식 캡처 전 변경 네 가지를 기록했다. 판정 규칙 문구는 바꾸지 않았고, Netlib 사례 추가로 범위를 넓혔다.

## 7. 판정과 인수인계

- 작업 수행: **오라클 구축 완료**(계약 범위). 원본 식별·독립3회·선행21a/기준선 비간섭·C 규칙 판정·변조 41종 거부를 확인했다. 독립 검토는 미실시다.
- 동등성: Rust 구현을 포함하지 않는다. Uᵀ·Lᵀ 값의 독립 재계산과 Rust 비트 비교는 **미검증**(21d).
- 기록만 하고 판정하지 않는 필드: 반환 out/in 별칭, 입력 후상태, level bucket 변화, F+0x200 work.
- 남은 한계(자연 미관측): **eta>0(희소·조밀 eta)**, 조밀 Uᵀ(일반 경로 판정 거짓), 커널 출력0 조기 종료, 커널 출력 −1, 입력 count≥2·dense 입력(이 caller에서는 구조적으로 없음). eta>0이 지원 라운드에서 구조적으로 불가능한지는 C로 확인하지 않았다. 지원 모드의 pivot은 factor 갱신을 생략하므로 eta는 그 epoch 앞선 요청0 FT 갱신에서만 생길 수 있고, 21e(지연 LU 연쇄) 조사에서 함께 확인한다. level Lᵀ의 내부 하위 분기(heap 경로와 level 경로의 선택식)는 bucket 변화로만 기록했고 판정식은 21d에서 C로 확정한다.
- 다음: **M3-04c-21d**. `python3 tools/session.py status` → `python3 tools/session.py claim M3-04c-21d --owner <새-owner>` → 이 문서 §2·§6과 `docs/m1/m1_02i_eta.md`, `primal_push_ftran_rust.md`를 읽고 `bench/oracle/m3_push_btran/*-1.json(.gz)`의 `btran:enter` factor를 초기 상태로 Rust 지원 BTRAN(희소 Uᵀ 커널·cache·Lᵀ 희소/level/조밀·출력 치환/gather·버퍼 교환)을 32호출에서 비트 비교한다. 기준 재검사는 `python3 tools/push_btran_check.py --controls --acceptance --self-test`. eta>0 사례가 필요하면 21e 캡처와 묶거나 별도 오라클로 등록한다.
- branch `work/m3-04c-21c`, worktree `../rsolver-m3-04c-21c`, 선행 미커밋 없음. 최종 commit·병합은 main `Merge M3-04c-21c` 이력을 따른다. main 미추적 `.claude/settings.local.json` 보존. 원격/업로드 없음.
- 교훈: `docs/lessons/original.md`(지원 BTRAN 자연 경로와 cache 조건), `tooling.md`(조사 훅의 모델 귀속 경합, size-limited 라이선스, 같은 값 변조).
