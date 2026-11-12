# M3-04c-19 보류·지원·라운드 오라클 검증 계약과 결과

## 1. 범위와 주장

- 종류: 원본 오라클 구축. 선행 main `6266711`의 04c-18까지 병합됨. 선행 미커밋 변경 없음. main 미추적 `.claude/settings.local.json`은 보존한다.
- double·Threads=1·batch 용량/미소비=0에서 단계 보류와 실패 증가, 승인 flip/pivot 지역 누계, 공급 고갈의 종료/support 해제, BTRAN 희소 표시와 dense fallback, 후보 압축 순서, 요청10·factor 반복/실패 게이트·재계산 진입/요청 지우기를 확인한다.
- 자연 arms·retry·interior와 Crossover=0 대조군을 각각 독립3회 읽기 전용 관측한다. 자연 미관측 경계는 원본 명령 구간의 합성으로 명시하며 자연 도달성으로 주장하지 않는다. 원본 명령을 재작성하지 않고 기존 15/17의 격리 실행 방식을 재사용한다.
- 제외: 실제 BTRAN/factor/FTRAN 수치 동등성(21), Rust 구현(20), 루프 연결(22), batch(23), 초기 후보 구성·정렬·x87·외부 해 설치·전체work/P3. BTRAN raw는 자연 실행에서 실제 수치를 기록하되 합성에서는 외부 backend다.

## 2. 원본과 상태 경계

- Gurobi13.0.3, SHA256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C: `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c`:1143–1176(패스 준비),1594/2089(승인수),1999–2062(BTRAN),2226–2418(보류·고갈·라운드). 공급자 `001c5310_PRIVATE000000000025615c.c`:573–686의 failure>=3 skip은 증가하지 않는다.
- ELF file offset 기준. `tools/dc.py 134a87`와 선행17 전체 asm에서 명령·진입 jump·레지스터를 확인한다. 자연 훅은 선행17의 복사본에 합치고 주소당 attach 하나를 유지한다.
- P=rsp+8, 후보/+8, 실패/+0x10, 지원표시/+0x18, cursor/+0x20, deferred/+0x24, count_sum/+0x30, successes/+0x38. 승인수 rsp+0x1ea0는 각 패스 초기화 후 flip/pivot만 증가한다. support rsp+0x209c, m의 double rsp+0x1e98. S=r14, F=*(S+0x470), retry=*(rsp+0x20e8).
- 시작/종료: 보류0x1c3b7b→다음 공급, 패스준비0x1c1ff8→첫 공급, 고갈0x1c3bdf→압축/요청10 또는0x1c4021 종료, BTRAN0x1c32b5→0x1c336d 또는 dense0x1c45c7→round, factor→0x1c3db4 재계산 진입/오류 cleanup, 재계산0x1c400a→다음 패스.
- 미초기화 support 표시를 읽지 않는다. support 양수 또는 실제 clear 이후에만 의미 있는 배열로 캡처하고 null을 비교 성공값으로 쓰지 않는다.

## 3. 비교 계약

| 대상 | 규칙 |
|---|---|
| 후보·실패·cursor/deferred·batch·support·marks·승인수·요청/status/retry | 순서와 정수 정확 비교, 길이/필수 상태/단계 연결도 검사 |
| BTRAN 입력 e_leave·raw count/목록/값·인자/별칭 | e_leave 및 실제 호출 인자 검증, 출력은 raw count와 순서·f64 비트. dense는 m개 값 |
| sqrt(m) 비교·승인수 | 원본 sqrt(m)<=accepted, 경계 m=완전제곱/비완전제곱·accepted=0/문턱 전/같음/후 |
| 후보 압축 | 기존 deferred prefix→미소비 batch(계약0)→읽지 않은 suffix. 정렬/중복제거/실패 reset 금지 |
| 보존 | 실패·통계·원래 후보 배열의 사용 구간, support 표시의 clear 유무, 반환 경계 요청 보존 |
| 출처/재현/비간섭 | 바이너리·입력·도구·원본 C/asm·캡처 manifest 해시, 독립 pid/run_id 3개, 새 사건 제거 시 선행17 전체 사건/최종결과 일치 |

일반 유한 f64와 내부 sentinel/±0을 비트 비교한다. NaN/IEEE무한대는 합성 계약 밖. 타이머/병렬/work 산술은 주장하지 않는다. 상호 null만으로 필수 출력 비교를 대신하지 않는다.

## 4. 사례와 원본 자료

- 자연: 기존 arms(지원·보류 반복), retry(재시도·고갈), interior(지원0)의 LP/extra를 그대로 사용한다. 대조군은 arms의 Crossover=0이다. 첫 탐색1회로 훅 실행/필드 유효성을 확인한 뒤 정식12회를 별도 저장했다.
- 합성106사례(첫 실행 전 입력표 고정): 보류 실패0/1/2/3, 진행0·전체보류 support0/1/-1·빈후보, sqrt 경계, prefix/suffix 순서·중복·실패 보존, BTRAN 희소0/1/다수·dense, factor 반환0/1/12/오류·status -999/-1000·attempt0/1·failure/numeric 문턱. ABI 확인 후 입력표를 첫 합성 실행 전에 커밋한다.
- 증거: `bench/oracle/m3_push_round/`에 탐색 history, 정식 자연/대조/합성 각3회, manifest·검증·review를 보관한다. 환경은 캡처별 OS/arch/runtime과 해시에 고정한다.

## 5. 재실행과 완료

도구: `tools/push_round_{agent.js,capture.js,runs.py,synthetic.py,check.py}`. `python3 tools/push_round_check.py --controls`와 `--acceptance` 분리, `--self-test`는 해시0·빈사건·목표0·필수누락/null·실행간 차이·대상 동작 변조를 모두 거부해야 종료0이다. 첫 차이 경로/기대/실제를 출력한다.

회귀: 선행 recompute/factor/supplier controls·acceptance·self-test, chain check·self-test 및 `cargo test -p rsolver-lp --test push_chain`→`python3 tools/push_step_chain_rust.py --compare`. Rust/API 변경이 없으므로 workspace 전체·clippy·확장재설치·pytest·Netlib·P3는 제외한다. 원본 캡처3회·비간섭·대상 제어 재생·오류 검출·독립검토를 통과한 범위만 오라클 완료로 병합한다.

## 6. 실행 결과

- 기준 main `6266711`, 계약 `d7d867a`, 훅첫동작 `318c68d`, 합성입력/ABI `bd0ca6b`, 첫 캡처·실패 보존 `ad4991c`, 정식 검사 `58368f8`, 독립검토 보강 `e585a4c`. x86-64 Linux/WSL2·Gurobi13.0.3·Threads1·Seed0, Python/Node/OS 버전은 각 캡처의 environment에 기록했다.
- 자연 arms410·retry161·interior180, 총751사건의 독립3회 재현과 Crossover0 대조0사건3회가 통과했다. 새 `roundctl` 사건은 **207건=127+49+31**이며 이전17 전체사건544건으로 투영했을 때 사건·최종 출력·basis가 그대로다.
- 자연 승인36(arms22/retry5/interior9), 보류6(arms3/retry3), 고갈9, round factor7·재계산7·종료3을 확인했다. 승인수는 패스마다0에서 시작하고 승인 flip/pivot만 증가한다. arms/retry의 동일 후보가 실패1→2→3으로 누적되며, 마지막 전체보류·support0에서 종료한다. failure>=3 공급 skip의 비증가와 충돌 보류는 선행13 자료·비간섭 및 회귀와 연결한다.
- 자연 BTRAN22회(arms17/retry4/interior1)는 **모두 count=1**이다. e_leave·인자 별칭·raw 순서/값·기존 표시 보존 및 제약 행 표시 추가를 검사했다. count0·다수·dense는 자연 미관측이며 아래 합성 제어 증거만 있다.

| 합성 종류 | 사례 수 | 원본 구간에서 확인한 동작 |
|---|---:|---|
| 단계 보류 | 4 | 실패0/1/2/3에서 prefix 쓰기·deferred 증가·실패+1 |
| 패스 준비 | 3 | support0/1/-1에 따른 표시 clear·prefix→suffix 압축 |
| 고갈/round | 69 | m0/1/2/4/9/10·승인수 sqrt 문턱, 전체보류·빈목록·support 해제·기존표시 보존/clear·중복과 순서·요청10 |
| BTRAN caller | 12 | sparse0/1/2/3/4·기존 표시 union·count의 값0도 표시, dense fallback·빈목록 종료·support0 호출생략/-1 |
| round factor wrapper | 18 | 반환1 반복·12/오류 우선순위·status -999/-1000·attempt0/1·failure/numeric 문턱·요청 보존·재계산 직전 |

- 합성 총106사례가 독립3회·원본 명령 구간과 Python 제어 재생에서 정수/순서/f64 비트 비교를 통과했다. `synthetic-inputs.json`과 `specs_sha256`로 첫 실행 전 고정한 입력을 연결한다. 초기 commit 제목의125는 오기이며 실제 입력은 처음부터106이다.
- `python3 tools/push_round_check.py --controls --acceptance --self-test --report bench/oracle/m3_push_round/verification.json` 종료0. 최종 변조 **157종**을 모두 거부한다. 해시/빈목록/목표0/필수누락·단일 필드·모든 합성의 출력 변화 외에, 독립검토에서 생존한 BTRAN caller 연결3종을 새 회귀로 추가했다. 검사 실패는 첫 경로·기대/실제를 출력한다.
- 선행 재계산 controls/acceptance/self-test82, factor41, supplier31, chain8 변조검사와 각 acceptance/controls가 종료0. `cargo test -p rsolver-lp --test push_chain`1통과, `python3 tools/push_step_chain_rust.py --compare`236단계·5,144필드 비트 일치, 42구간·재분해 비교 제외6을 유지한다. [재계산 회귀](../../bench/oracle/m3_push_round/regression-recompute.json), [factor/supplier 회귀](../../bench/oracle/m3_push_round/regression-factor-supplier.json), [chain 회귀](../../bench/oracle/m3_push_round/regression-chain.json).
- py_compile 3파일·Node --check 2파일·전체 diff 검사 통과. Rust/API·기존 solver 경로 변경이 없어 workspace 전체/clippy/확장설치/pytest/Netlib/P3는 제외했다.
- 새 원본 실행은 **36회=정식15(자연/대조12+합성3)+탐색1+초기 교체18(자연12+합성6)+독립검토2**다. 병렬 Gurobi·원격/업로드·공유 venv 변경은 하지 않았다.

### 실패 보존과 보정

1. 탐색 retry161사건에서 첫 패스 진입의 support 표시와 승인수 스택이 아직 초기화되지 않음을 확인했다. 표시가 실제 clear된 뒤에만 기록하고, 첫 공급 성공 전 승인수 슬롯은 직렬화하지 않는다. 고갈 시 live ebp도 승인수와 대조한다. `history/explore-retry.json`을 보존했다.
2. 초기 자연12회는 정상 종료 후 stale round flag 때문에 interior cleanup을 round error로 잘못 기록했다. 재계산 정상 종료에서 flag를 해제했고, 초기 agent/캡처를 `history/first-natural/`에 보존한 뒤 최종 agent로12회를 재캡처했다. 기본 선행사건은 모두 동일하다.
3. 초기 합성3회에 반환 오류 코드도 출력하도록 보강한 뒤 다음3회의 비교에서 pass-support0 total이2/2/4로 갈렸다. **원본1c1ff8의 live edx=total**을 trampoline이 준비하지 않아 entry주소 하위비트로 남긴 ABI 누락이다. 첫6회/당시 도구를 `history/first-synthetic/`, `history/pass-live-edx-failure/`에 보존했다. entry를r11에 두고 edx=P.total을 명시한 뒤 정식3회가 일치했다. 원본 명령/입력/합격 규칙은 바꾸지 않았다.
4. 독립검토는 BTRAN prepare/enter의 leave·unit row를 함께 변경, prepare/applied의 request를 함께6으로 변경, 두 번째 BTRAN의 기존 표시를 지우고 raw 행만 유지하는3종이 초기검사에서 생존함을 확인했다. 직전 applied의 step/selected/전체상태→BTRAN→다음 supplier P와 factorctl 입력을 연결해 모두 거부했다. 초기 checker/154검사와 검토자의 초기 변조 결과를 history/review에 보존한다.

## 7. 판정과 인수인계

- 작업 수행: **오라클 구축 완료(계약 범위 통과)**. 독립검토도 통과했다. Rust 구현 동등성은 **미검증**이며 신규 Rust 구현은 없다.
- 독립검토 `/root/m3_04c19_review`는 원본C/asm·합성ABI·검사기와 초기 생존3종의 수정을 검토했다. retry161사건·합성106사례를 각1회 독립 재캡처해 pid/run_id 외 전체 일치, 최종157검사와 직접9변조 거부를 확인했다. Rust/Cargo·소스변이는 재실행하지 않았다. [검토 기록](../../bench/oracle/m3_push_round/review/review.md).
- 정확한 범위: 자연3모델의 새207사건과 원본 합성106경계의 double·batch0 제어. 자연 sparse BTRAN count1 외의 도달성, 실제 factor/BTRAN 수치 계산, retry/오류의 자연 도달성·batch/x87·전체work/P3·전체 push는 미검증이다.
- 합성 pass 출구 stub은 공급 직전 계산된 eax(total)와 r12=0(cursor/deferred)을 저장한다. 이 stub 쓰기는 원본 본문 검증으로 세지 않는다. 승인수0 초기화는 첫 공급 성공 뒤이며 자연 각 패스의 승인계수와 live ebp로 확인한다. BTRAN/factor 합성 callback은 명시적 외부 backend다.
- 다음 작업은 **M3-04c-20** 보류·실패·지원·round Rust L1이다. BTRAN은 외부 backend로 받고, 후보 압축 순서/중복·실패 보존·승인수·support·요청10·종료를 이 자료로 비교한다. 실제 수치 backend21·루프22·batch23과 구분한다.
- 다음 명령: `python3 tools/session.py status` → `python3 tools/session.py claim M3-04c-20 --owner <새-owner>` → 이 계약·`primal_push_loop.md` §2.3/2.5/4.3 → 검증 계약 작성.
- branch `work/m3-04c-19`, worktree `../rsolver-m3-04c-19`, owner `codex-m3-04c19-20261112`가19/shared 점유. 최종 결과·병합 commit은 main의 `Merge M3-04c-19` 이력을 따른다. 미커밋 선행 변경 없음. 병합과 같은 단계에서 worktree/branch/점유를 정리한다.
- 문서: `11_progress.md`, `12_roadmap_diagram.md`, `14_sprint_plan.md`, `m3/sprint_design.md`, `handoff/M3.md`를 branch에서 갱신해 병합한다. 교훈은 `docs/lessons/tooling.md`의 live register·초기화 시점·caller 연결 검사 항목에 기록한다.
