# M3-04c-18 재계산 Rust L1 검증 계약과 결과

## 1. 범위와 주장

- 종류: 구현 L1. raw FTRAN 반환 뒤 basic 복사(별도 버퍼) 또는 제자리 사용(별칭), 유한 하한 이동·폭·F/B/X/L/N/A/U 분류·위반수·PInf·retry를 구현한다. pivot/round의 같은 계산을 공유하며 요청을 지우기 전에서 끝낸다.
- 선행: main `484243d`(M3-04c-17 병합), branch `work/m3-04c-18`, worktree `../rsolver-m3-04c-18`. 선행 미커밋 없음. main의 미추적 `.claude/settings.local.json`은 보존한다.
- 전제: double, Threads=1, batch=0, 유한 입력 및 유한 중간값, 유효한 새 header와 bound, I>h>0. 입력 raw는 외부 FTRAN의 결과다.
- 제외: rhs 준비·FTRAN 호출/수치/입력 버퍼 변형(21), 로그·전체 work/P3, 품질 helper(15a), 보류·지원·라운드(19→20), solver 루프 연결(22), batch(23), x87. rhs/input 별칭은 외부 경계이며 Rust가 구현했다고 주장하지 않는다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, 라이브러리 SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c` 1897–1980, 2329–2418. 원본 확인은 main 위치에서 수행했다. asm은 선행 `bench/oracle/m3_push_recompute/source.asm` 전체4036줄 및 SHA를 사용한다.
- 시작: FTRAN 복귀 `0x1c38e3`(pivot)/`0x1c3e25`(round). 종료: 분류·retry 뒤 `0x1c3aed`/`0x1c400a`, retry 쓰기 뒤 cleanup `0x1c08d5`. round의 요청0 쓰기는 `0x1c400e`이므로 포함하지 않는다.
- 새 header는 F+0x18, bound는 열 좌표, raw/basic/width/marks는 basis 위치 좌표다. basic은 별칭이면 제자리, 별도면 raw를 복사한다. m=0은 빈 분류와 +0 PInf이며 FTRAN 호출은 외부 경계다.
- 읽기: header, lb/ub, raw, I/h, attempt/model_status. 쓰기: basic, width, marks, 위반수, PInf, retry(조건 충족 때만1). request와 입력 배열은 보존한다. PInf는 basis 위치 오름차순으로 순차 가감한다.

## 3. 비교 계약

| 필드 | 규칙 | 재구성 |
|---|---|---|
| basic·width·PInf | ±0 포함 little-endian f64 비트, 오차 없음 | 원본 raw·bound·header를 Rust에 공급하고 원본 leave와 직접 비교 |
| marks·위반수·retry·exit·request | 정확 | 원본 leave에서 추출, retry 초기0 |
| raw/basic 별칭 | 정확, 안전한 Rust enum으로 제자리/별도 버퍼 선택 | 원본 aliases.raw_basic과 실제 Rust 버퍼 주소 관계 |
| header·lb/ub 보존 | 배열 순서·비트 정확 | 원본 preserved에서 추출 |

NaN/IEEE 무한대는 제외하고 내부 sentinel I는 포함한다. 시간·전체 work·RNG 소비는 해당 계산에 없다. 17의 독립3회 자료는 manifest·controls·acceptance를 다시 확인해 재사용하고 §6.1의 추가10사례는 새 원본 독립3회로 보강했다.

## 4. 사례와 원본 자료

- `bench/oracle/m3_push_recompute/manifest.json`과 자연 arms/retry/interior 각3회·대조3회, 합성3회를 사용한다. 첫 실행의 자연8회·119행 및 합성120사례·152행을 Rust에 투영한다(반복을 사례 수로 중복 집계하지 않음).
- 추가10사례·18행은 본 작업의 `extra-{1,2,3}.json`·`extra-manifest.json`에 있다. 총138입력·289행이다. 기존 자료와 새로운 자료의 입력/도구/바이너리 해시를 따로 검사한다.
- 자연 두 경계(pivot1/round7), 합성 두 경계 각각의 free·고정·한쪽 무한·비영 하한·±h/w±h의 인접/동률·±0·basis 역순 누적·PInf=1e10 전후와 attempt/status 교차·별칭4조합·m=0을 포함한다.
- 자연 raw/basic 별칭true·F/X·retry·정확 경계의 도달성은 미검증이며 합성 제어 증거로만 보고한다. FTRAN 수치는 외부 입력이므로 해당 구현 동등성을 주장하지 않는다.
- 새 입력·Rust trace·비교·오류 검출·회귀·환경·해시는 `bench/oracle/m3_push_recompute_rust/`에 보존한다. Rust 입력에 원본 정답은 넣지 않는다.

## 5. 재실행 명령

```bash
python3 tools/push_recompute_extra.py --check
python3 tools/push_recompute_rust.py --write
cargo test -p rsolver-lp --test push_recompute
python3 tools/push_recompute_rust.py --compare
python3 tools/push_recompute_rust.py --self-test
python3 tools/push_recompute_rust.py --mutate-rust bench/oracle/m3_push_recompute_rust/mutations.json
```

비교는 138입력의 모든 계약 필드·순서를 확인하고 첫 차이에서 실패한다. 빈 목표·누락·중복키·변경된 비트·분류·요청·retry 등의 trace 변조84종과 Rust 경계/누적/요청 변이33종을 모두 거부해야 한다. 소스 변이는 반드시 원복 후 정상 trace를 재생한다.

처음128입력의 사전 분석은 33종 중29거부·4생존을 예상했다. sentinel 동률은 작은 bound가 I로 반올림되고, 기존 무한폭 입력은 이미 v<w-h이며, A행은 합산 재결합에 비트 차이가 없었다. 실제로는 역순 합산까지 생존해 §6의 실패를 기록했고 §6.1의 추가 계약으로5개 미검증 경계를 보강했다. 최종138입력에서는 **33종 전부 거부**를 요구한다.

회귀: `push_recompute_check.py --controls --acceptance --self-test`, `push_factor_check.py --controls --acceptance --self-test`, supplier의 `--controls`·`--acceptance`·`--self-test` 각각, chain의 `--check`·`--self-test` 각각, factor/supplier Rust 재생, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `push_step_chain_rust.py --compare`, py_compile·diff 검사. 기존 solver/API 경로 미변경으로 확장 설치·pytest·Netlib·P3는 제외한다.

## 6. 실행 결과

- 첫 동작: `--write` → `cargo test -p rsolver-lp --test push_recompute` → `--compare` 종료0. 자연8·합성120, 총128입력·271행·2,790필드 비트/정확 일치. 최초 `--self-test`는62종 trace 변조를 모두 거부했다. 최종 건수는 아래 §6.2다.
- 최초 Rust 변이33종은28거부·5생존이었고, 예상하지 않은 `reverse-basis` 생존 때문에 검사가 종료1이었다. 기존 누적은 큰 값2^54와 작은 값1 두 개라 역순 합의 +2가 반 ULP로 반올림되어 같은 값이었다. 이 실패를 `history/initial-mutations.json`에 보존한다. 선행 회귀 CLI의 배타적 플래그 조합 오류2건도 `history/initial-oracle-regression.json`에 보존하고 개별 명령으로 고친다.

### 6.1 추가 원본 캡처 계약(캡처 전 고정)

- 기존 실패를 없애기 위해 합격 조건을 완화하지 않는다. 17의 도구·입력·캡처는 수정하지 않고, 같은 원본 명령 구간 실행기를 재사용하는 `push_recompute_extra.py`로 두 진입점 각각5사례(총10사례·18행)를 추가한다. 원래128사례는 유지한다.
- I=1e30·h=0.125를 유지한다. lower=-I/upper=I/2, lower=I/2/upper=I, width=I에서 v=2I, A행 앞 PInf=1인 재결합 구분, 2^54 뒤 작은 값1 세 개를 합산하는 역순 구분을 고정한다. 이들은 자연 도달성을 주장하지 않는 합성 기계 상태다. model_status=1, attempt=0, request=10, 별칭false다.
- 원본 두 경계의 ABI·입출구·FTRAN/로그 대체는17과 동일하다. 새 원본 독립3회는 `tools/gurun.sh ../.venv/bin/python tools/push_recompute_extra.py --capture bench/oracle/m3_push_recompute_rust/extra-N.json`으로 순차 실행한다. 기존 파일 덮어쓰기는 거부한다. source/도구/입력/바이너리/캡처 해시를 별도 manifest에 고정하고 사건·환경·입력을 대조한다.
- 기준 캡처 전체 필드를 기존 `recompute` 계산 모델과 대조하되 Rust 기대값은 모델 출력이 아닌 원본 result에서 읽는다. 빈 사건·해시0·입력·회차·null·수치·순서 변조를 거부해야 한다. 추가 후138입력·289행에서 Rust 변이33종 모두를 거부해야 하며, 초기 실패 이력은 보존한다.
- `oracle-capture` 스킬의 완료 전 독립 검토 지침에 따라 다른 에이전트에게 읽기 전용 검토를 요청한다. 원본 추가 캡처는 작업자 실행이 끝난 뒤 순차로 수행한다.

### 6.2 최종 결과

- `push_recompute`가 raw/basic의 별도 복사와 제자리 분류를 처리한다. `RecomputeInput`은 새 header·열 bound·I/h·attempt/status, `RecomputeState`는 PInf·위반수·retry·보존 request다. 기존 solver 경로에는 연결하지 않았다.
- 자연8·기존합성120·추가합성10, 총 **138입력·289행·2,958필드**가 원본 결과와 비트/정확 일치했다. pivot66/round72입력이다. trace 변조84종·Rust 동작 변이33종 모두 출력 차이로 거부했다. 최종 소스를 원복한 뒤 정상 재생도 통과했다. 원본 정답은 Rust 입력에 없다.
- 추가 원본10사례·18행은 독립3회 사건·환경·입력·바이너리 식별값이 동일하며 추가 캡처 검사기의 변조21종을 거부했다. 17의 자연/합성 독립3회 및 대조군은 원래 manifest 그대로 재사용했다. 새 원본 실행은 작업자3회+독립검토1회=총4회다.
- 독립 검토 **통과**: `/root/m3_04c18_review`가 C/asm·Rust·비교기·33변이 결과를 검토하고 추가10사례를 원본에서1회 재캡처해 pid/run_id 외 전체 필드 일치를 확인했다. 원본82·추가21·trace84 자기검사와 직접16종 변조를 거부했다. [검토 기록](../../bench/oracle/m3_push_recompute_rust/review/review.md)·[검토 결과/해시](../../bench/oracle/m3_push_recompute_rust/review/review.json). 검토자는 Cargo 재빌드·Rust 소스변이는 새로 실행하지 않았다.
- `cargo test --release --workspace` **133통과·실패0·기존 ignored12**, `cargo clippy --workspace --all-targets -- -D warnings` 경고0. factor89입력·978필드, supplier69입력·1,311묶음, chain236단계·5,144필드(42구간·재분해 비교제외6 유지) 회귀가 일치했다.
- 선행 재계산/factor/supplier의 controls·acceptance·self-test(82/41/31종), chain check·self-test8종도 통과했다. py_compile·전체 diff 검사 통과. 명령·종료코드·로그·환경은 `oracle-regression.json`, `rust-regression.json`, `environment.json`에 있다.
- 초기 Clippy `if_same_then_else`는 두 sentinel 조건의 같은 width=I 반환을 하나의 OR 조건으로 합쳐 해결했다. 동일 분기 변이 앵커를 갱신하고 Rust33변이·workspace·clippy·회귀를 다시 통과했다. 초기 실패 로그는 history에 보존했다.
- Python 확장 설치·pytest·Netlib·P3는 API/기존 solver 실행 경로가 바뀌지 않아 제외했다. 새로운 수치 LU나 전체 crossover 동등성을 검증한 것으로 보고하지 않는다. NaN/IEEE 무한대·retry 초기비영도 이번 원본 입력 범위 밖이다.
- [최종 manifest](../../bench/oracle/m3_push_recompute_rust/manifest.json)는 구현·도구·입력·trace·원본17/추가 manifest·실패 이력·검토·회귀 파일 SHA를 연결한다. 문서와 자기 manifest는 순환 해시를 피하려고 이 목록에서 제외한다.

## 7. 판정과 인수인계

- 작업 수행: **구현 L1 완료, 계약 범위 통과**. basic 복사/제자리·shift·width·분류·위반수·PInf·retry·request 보존, 두 경계138입력에서 비트/정확 일치다.
- 한계: 자연 별칭true·F/X·retry·정확 경계·큰 sentinel 산술의 도달성은 미검증(합성 증거). rhs/FTRAN 호출·수치 backend, 실제LU21·품질수치15a·루프22·batch23·x87·전체work/P3는 제외다.
- 다음은 **M3-04c-19** 보류·지원 BTRAN·고갈·재처리 라운드 오라클이다. `python3 tools/session.py status` → `python3 tools/session.py claim M3-04c-19 --owner <새-owner>` 후 `primal_push_loop.md` §2.3·2.5·4.3과 `.agents/skills/oracle-capture/SKILL.md`를 읽는다. 실패한도·승인수·support 해제·보류→미소비후보 압축순서·요청10·종료가 필수다. shared 변경 전 별도 점유한다.
- branch `work/m3-04c-18`, 기준 main `484243d`, 계약 `74cfec6`, Rust첫동작 `ed46b96`, 추가계약 `751bb9d`, 첫추가캡처 `de2cba7`, 확장비교 `1eff0fa`. 최종 commit/병합은 main의 `Merge M3-04c-18` 이력으로 확인한다. 선행 미커밋 없음. 검증 뒤 --no-ff 병합과 같은 단계에서 worktree·branch·18/shared 점유를 정리한다.
- `docs/handoff/M3.md`, `docs/11_progress.md`, `docs/12_roadmap_diagram.md`, `docs/14_sprint_plan.md`, `docs/m3/sprint_design.md`를 같은 branch에서 갱신해 병합한다. main의 별도 `.claude/settings.local.json`은 보존한다. 원격 동작·외부 업로드 없음.
- 교훈은 `docs/lessons/tooling.md`의 누적 순서·반 ULP 및 A행 재결합 구분 입력에 기록했다.
