# M3-04c-15 직렬 factor 제어 오라클

## 1. 범위와 주장

- 종류: 오라클 구축. 승인 pivot 뒤 요청 선택 → 직렬 factor 호출·alpha 복원 → 오류/retry 게이트 → 재계산 진입 또는 품질 주기 제어를 검증한다.
- 기준 main `79a0893`, branch `work/m3-04c-15`, worktree `../rsolver-m3-04c-15`. 선행 미커밋 없음. 담당 `codex-m3-04c15-20261112`, M3-04c-15·shared 점유.
- 자연 arms·retry·interior와 Crossover=0 대조군은 기존 13 오라클과 사건 전체 투영·최종 출력이 같아야 한다. factor 판단 필드·호출 인자·복원·품질 주기와 support 생략을 추가 관측한다.
- 합성 경계는 실제 원본 명령을 격리 프로세스에서 호출한다. 요청 선택은 원본 `0x1c336d`부터 호출/생략 분기까지, wrapper는 `0x1c3465`부터 정상 종료·retry/오류 또는 **재계산 직전**까지다. 합성 wrapper의 factor·품질 계산·두 문맥 전환 helper는 명시적 backend 대체다. 합성 helper 결과를 자연 LU 증거로 세지 않는다.
- 제외: 재계산 본문17·실제 LU21·batch23·전체 push22·x87·전체 work/P3. 품질 helper `089f61`의 수치 오차 계산은 이번 caller 계약 밖이며 별도 후속15a로 등록한다. 16 구현의 품질 계산은 외부 backend다.

## 2. 원본과 상태 경계

- Gurobi13.0.3, SHA256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`, x86-64 Linux, ELF file offset 기준.
- C: `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c`1725–1997; `00115ed0_PRIVATE0000000000089f61.c`238–272(수치 helper는 정적 참고).
- asm `0x1c336d–0x1c345a`: r14=S, rsp+0x2048=F, +0x1e78=1.5m, +0x1e80=5m², +0x1e98=m, +0x209c=support. 원본 branch target 3465(call),3b6e(skip)을 합성 전용 반환 stub으로 교체한다. 내부 산술·비교 명령은 바꾸지 않는다.
- wrapper는 위 frame 및 +0x2090=alpha, +0x1fa0=attempt, +0x1fe0=초기 numeric+10, +0x1fd8=품질 주기, +0x20e8=retry pointer를 사용한다. batch capacity/count/position=0. 별도 fake stack에 충분한 하위 호출 공간을 두고 SysV callee-saved 레지스터를 보존하는 trampoline을 쓴다. 원본 주소를 절대값으로 기록하지 않는다.
- 자연 훅은 함수 진입 훅 외에는 함수형 probe를 쓴다. 명령 경계·incoming branch를 asm에서 확인한다. `S+0x210` 원래 포인터와 현재 alpha는 주소 대신 별칭 boolean·벡터 내용으로 기록한다. batch=0에서 F+0x1c8 보존도 확인한다.

## 3. 비교 계약

| 필드/사건 | 규칙 | 확보 방법 |
|---|---|---|
| 요청 우선·k==limit·k<q/==q/>q·N·support | 정수·호출 유무 정확 일치 | 자연 판단 전후 + 원본 선택 slice 합성 |
| m·L/U·C/E·준비 상수 | 정수 정확, f64 ±0 포함 LE hex 비트 | 자연 frame 및 합성 입력 |
| 비용식 | ((f64(C)+f64(E))*2)*m-min(q*((m*5)*m),(f64(L+U)-m*1.5)^2), >0만 요청10 | asm 결합 순서, 유한 입력만 |
| factor 호출·반환1 반복·12·오류 | 순서·인자·요청·반환 정확 | 자연 + 명시적 합성 backend |
| alpha 교체/복원·spike 보존 | 자연은 비영 포인터의 별칭/복원 boolean, 합성은 별칭 및 sentinel 3버퍼 내용 보존. null-null 거부 | 자연 전후·합성 sentinel 세 버퍼 |
| retry·status·attempt·failure·numeric 게이트 | 상태·쓰기·요청 보존 정확 | 원본 wrapper slice, cleanup 전에 종료 |
| quality 호출·increment·request6·주기 | 호출 순서·local_100·최종 request 정확 | wrapper 원본, 품질 수치는 외부 입력 |

- 입력은 유한 수·유효 int32/int64 합(overflow 없음). `sum-convert`의 L/U는 큰 signed 수를 상쇄시키는 **합성 기계 입력**이며 비음수 nnz인 실제 factor 상태로 도달 가능하다고 주장하지 않는다. 자연 벡터 내용의 수치 계산/불변성은 검증하지 않는다. NaN/Inf 비용식은 제외. helper의 수치 오차는 외부 backend라 오차식을 적용하지 않는다.
- Threads1·Seed0, 타이머 분기와 work 누적 비교 제외. 원본 캡처는 독립3회 사건/결과 동일 및 기존13 사건 투영 동일을 요구한다.

## 4. 사례와 원본 자료

- 자연: 13의 arms·retry·interior 입력/starts/params를 그대로 사용. Crossover0은 사건0이어야 하고 목표 자연 자료는 판단·factor 사건이 비어 있으면 실패한다.
- 선택 합성: 기존 요청2/6/10, k==limit 및 양옆·limit 초과, k=q 전후, 비용0/양/음, q*5m²와 a²의 min 양쪽, support0/1/-1, N0/비영. 모델 가능한 자연 상태로 주장하지 않는다.
- wrapper 합성: 반환0/1→0/1→1→0/12/>1, status -999/-1000, attempt0/1, failure 및 numeric 임계값 양옆, request0/비영 재계산 진입, 품질 주기 k==주기·k0·주기>50 clamp, increment5/10/20/50, request6 재호출·status12.
- 출력 `bench/oracle/m3_push_factor/`, manifest에 바이너리·모델·도구·입력 명세·캡처·출처 C 해시를 연결한다. 동일 ABI/실행 설정을 새 독립3회로 검증한다.

## 5. 재실행과 완료 기준

도구: `push_factor_capture.js`/`agent.js`, `push_factor_synthetic.py`, `push_factor_runs.py`, `push_factor_check.py`.

```bash
python3 tools/push_factor_runs.py bench/oracle/m3_push_factor-recapture
python3 tools/push_factor_check.py --controls
python3 tools/push_factor_check.py --acceptance
python3 tools/push_factor_check.py --self-test
```

캡처 도구는 기존 출력 덮어쓰기를 거부한다. 새 디렉터리를 인자로 주거나 별도 review 파일을 사용한다. `--write-manifest`는 고정 도구·캡처 파일을 확인한 뒤 기준 manifest를 명시적으로 만드는 명령이다.

1. 탐색1회 → 자연/대조/합성 독립3회.
2. `python3 tools/push_factor_check.py --controls`(출처·재현·비간섭)와 `--acceptance`(자연 사건 존재·제어 전이 재생)를 분리한다.
3. `--self-test`는 해시0·빈 사건·목표0회·누락/null·회차 차이·요청/호출/별칭/게이트/주기 변조를 전부 거부해야 한다.
4. 선행 supplier 검사3종, push chain 검사·self-test, Rust `push_chain` 및236단계/5,144필드 비교를 실행한다. Rust/확장 미변경이므로 workspace 전체/clippy/확장 설치/Netlib/P3는 제외한다.
5. oracle-capture 스킬이 요구하는 독립 검토를 받고, 결과·한계·다음 명령·교훈·공유 문서를 갱신해 --no-ff 병합한다.

## 6. 실행 결과

- 계약 `4a0ba43`, 도구 첫 동작 `4501da9`, 초기 캡처 `755d74a`. 초기 51사례와 당시 도구·manifest·검토 원자료는 `bench/oracle/m3_push_factor/history/initial-capture.tar.gz`에 바이트 확인 후 보존했다. 최종 합성은 아래 55사례다. 실패 사례나 기준값을 Rust 출력으로 덮어쓰지 않았다.
- 환경: WSL2 x86-64 Linux, 원본13.0.3(§2 SHA), Node Frida17.26.2, Python3.13. 원본 주소는 ELF offset을 사용하며 [source.asm](../../bench/oracle/m3_push_factor/source.asm)에 호출·복원·품질·오류 경계를 저장했다. 원본/입력/도구/기준 자료 해시는 [manifest](../../bench/oracle/m3_push_factor/manifest.json)에 연결했다.
- 자연 arms/retry/interior 각각 독립3회 사건271/104/137이 동일하다. 대조군 Crossover0도 독립3회 사건0이다. 새 사건을 제거하고 `factor_ctl` 확장 필드를 빼면 **기존13의 JSON 사건 전체·최종 출력·basis와 동일**하다.
- 대표 자연3기록에서 판단28회(arms17·retry4·interior7), support 생략22회·호출6회다. 전체 factor16호출 중 승인 pivot wrapper6회가 alpha 교체·복원·spike 보존을 통과했고, 나머지10호출은 초기화/라운드의 관측이며 wrapper 검증 수에 더하지 않는다. 실제 quality helper 호출은0회다.
- 합성 **55사례=선택24+wrapper31**의 원본 독립3회가 동일하다. 선택은 기존 요청 우선·k==limit/초과·k=q 전후·비용 음/0/양·min 두 항·support0/1/-1을 포함한다. q항이 min인 C19/20/21은 5m² 계수·>0 경계를 구분하고, 상쇄 L/U 기계 입력은 정수합 뒤 변환을 구분한다. 후자는 자연 도달성 증거가 아니다.
- wrapper31사례는 반환1 반복(1회/2회),12·오류7·오류의 status보다 우선, -999/-1000, attempt0/1, failure·numeric 임계(입력과 backend 변경), 비영 요청 재계산 진입, k0의 주기 보존·비영k의50 clamp, 품질 increment5/10/20/50, k=주기/초과, 요청6 재호출·반복, 품질 status12를 포함한다. 실제 원본 caller 명령이 세 sentinel 버퍼의 포인터·내용을 보존하고 quality enter=7을 전달했다. factor/quality/문맥 helper는 대체했으므로 수치 동등성 증거가 아니다.
- `push_factor_check.py --controls`·`--acceptance`·`--self-test`: 종료0, 변조 **41종 모두 거부**. 자연 상태/frame의 전후 연결·attempt/numeric_limit·quality enter·별칭/내용·출처·빈 사건·목표0회까지 검사한다. 각 정수/순서/준비 상수 bit/상태 필드는 계약 규칙과 일치했다. 비용 중간값 자체의 비트와 수치 LU는 비교한 것으로 보고하지 않는다.
- 독립 최초 검토에서 자연 restored.status·call.frame.attempt 변조가 생존해 상태/frame 연결을 추가했고, quality enter·sentinel 내용·비용식 경계도 보강했다. 보완 전 자료와 검토는 history에 남겼다. 최종 독립 재검토는 **통과**(자연137사건·합성55사례 직접 재캡처 일치·추가변조9종 거부)이며 [review.md](../../bench/oracle/m3_push_factor/review.md)에 기록한다.
- 최종 전체 diff 검사에서 agent 말미 공백·asm 잘린 끝 명령의 공백을 정리했다. agent 해시 변경에 따라 자연/대조12회를 다시 캡처했고 이전 `aa85d38`의 사건·최종 결과와 전부 동일하다. 합성55 기준·도구의 동작은 불변이다.
- 회귀: supplier controls/acceptance/self-test31종, push chain check/self-test8종 모두 종료0. `cargo test -p rsolver-lp --test push_chain`1통과, `push_step_chain_rust.py --compare` **236단계·5,144필드 비트 일치**(42구간·재분해 뒤 제외6 유지). py_compile·node --check·git diff --check 통과. 명령별 결과는 [regression.json](../../bench/oracle/m3_push_factor/regression.json), [rust-regression.log](../../bench/oracle/m3_push_factor/rust-regression.log)에 보존한다.
- 초기 회귀 명령을 한 번에 여러 mutually-exclusive 플래그로 실행해 argparse가 거부했다. 각 플래그를 독립 명령으로 재실행한 종료코드를 위 기록에 보존했으며 초기 실패를 통과에 세지 않았다.
- Rust/solver/API 변경이 없어 workspace 전체·clippy·확장 재설치·pytest·Netlib/P3는 제외했다. 이 작업으로 Rust 구현 일치·실제 LU·품질 helper의 수치·재계산 본문·batch·전체 push가 검증됐다고 주장하지 않는다.

## 7. 판정과 인수인계

- 작업 수행: 오라클 구축 완료(최종 독립 검토 통과). 동등성: **명시한 자연·합성 제어 경계 통과**, Rust 구현은 미검증이다.
- 다음: **M3-04c-16**. `python3 tools/session.py status` → `python3 tools/session.py claim M3-04c-16 --owner <새-owner>` 후 이 계약과 `push_factor_check.py`의 select/wrapper 비교를 읽고 외부 backend 기반 L1을 작성한다. 재계산 진입을 중단 경계로 보존한다. 실제 재계산은17→18이며 그 뒤 연결은22다.
- 품질 helper의 FTRAN/오차/Markowitz 수치 자체는 **M3-04c-15a**로 별도 등록했다. `089f61` caller의 반환 인자/상태 소비와 helper 수치 동등성을 구분한다.
- branch `work/m3-04c-15`, 선행 미커밋 없음. 최종 커밋/병합은 main `Merge M3-04c-15` 이력으로 확인한다. 병합과 같은 단계에서 worktree·branch 정리와 M3-04c-15·shared 점유 해제를 한다. main의 별도 `.claude/settings.local.json`은 보존한다.
- 공유 기록: `docs/11_progress.md`, `docs/12_roadmap_diagram.md`, `docs/14_sprint_plan.md`, `docs/m3/sprint_design.md`, `docs/handoff/M3.md`를 같은 branch에서 갱신해 병합한다. 교훈은 `docs/lessons/tooling.md`와 `original.md`에 기록한다. 외부 업로드·원격 동작 없음.
