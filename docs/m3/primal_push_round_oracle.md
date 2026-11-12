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

- 자연: 기존 arms(지원·보류 반복), retry(재시도·고갈), interior(지원0)의 LP/extra를 그대로 사용한다. 대조군은 arms의 Crossover=0이다. 첫 탐색1회로 훅 실행/필드 유효성을 확인하고 정식12회를 별도 저장한다.
- 합성: 보류 실패0/1/2/3, 진행0·전체보류 support0/1/-1·빈후보, sqrt 경계, prefix/suffix 순서·중복·실패 보존, BTRAN 희소0/1/다수·dense, factor 반환0/1/12/오류·status -999/-1000·attempt0/1·failure/numeric 문턱. ABI 확인 후 입력표를 첫 합성 실행 전에 커밋한다.
- 증거: `bench/oracle/m3_push_round/`에 탐색 history, 정식 자연/대조/합성 각3회, manifest·검증·review를 보관한다. 환경은 캡처별 OS/arch/runtime과 해시에 고정한다.

## 5. 재실행과 완료

예정 도구: `tools/push_round_{agent.js,capture.js,runs.py,synthetic.py,check.py}`. `python3 tools/push_round_check.py --controls`와 `--acceptance` 분리, `--self-test`는 해시0·빈사건·목표0·필수누락/null·실행간 차이·대상 동작 변조를 모두 거부해야 종료0이다. 첫 차이 경로/기대/실제를 출력한다.

회귀: 선행 recompute/factor/supplier controls·acceptance·self-test, chain check·self-test 및 `cargo test -p rsolver-lp --test push_chain`→`python3 tools/push_step_chain_rust.py --compare`. Rust/API 변경이 없으므로 workspace 전체·clippy·확장재설치·pytest·Netlib·P3는 제외한다. 원본 캡처3회·비간섭·대상 제어 재생·오류 검출·독립검토를 통과한 범위만 오라클 완료로 병합한다.

## 6. 실행 결과

착수 상태. 원본 신규 실행0회, 신규 동등성 미검증.

## 7. 판정과 인수인계

작업 미완료. branch `work/m3-04c-19`, worktree `../rsolver-m3-04c-19`, owner `codex-m3-04c19-20261112`가19를 점유했다. 다음은 훅/명령 구간 ABI 확인과 탐색 캡처다. 완료 전 shared 점유 후 progress·roadmap·스프린트·handoff·교훈을 갱신한다.
