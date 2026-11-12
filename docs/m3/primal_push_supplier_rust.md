# M3-04c-14 단일 공급자 제어 Rust L1

## 1. 범위와 주장

- 종류: 구현 L1. 선행 main `03bdca3`(04c-13 병합), branch `work/m3-04c-14`, worktree `../rsolver-m3-04c-14`. 선행 미커밋 없음. main의 별도 `.claude/settings.local.json` 보존.
- 목표: batch 용량·채움·소비가 모두 0인 단일 공급자의 후보 소비, failure>=3·지원 충돌 보류, 원본 CSC/slack 구성, FTRAN 호출·spike 선택, raw count 통계와 dense 압축을 Rust로 복원한다.
- 외부 backend에 주는 것은 원본 열·출력 버퍼·선택적 spike·work다. backend는 캡처의 raw FTRAN 출력과 spike 후상태를 제공한다. 후보 반환·후상태·압축 alpha는 Rust가 계산하며 기대 출력을 입력하지 않는다.
- 제외: FTRAN/LU 수치(21), 공급자 사이 caller 상태 연쇄·push 루프 연결(22), batch(23), x87·NaN·무한 수치·work 패리티·전체 crossover. solver 연결은 하지 않는다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- `../re/decomp/c/001c5310_PRIVATE000000000025615c.c` 71–80·526–688, `00120680_PRIVATE000000000009d008.c`.
- ELF `25615c@0x1c5310`; SysV x86-64 5포인터 ABI는 [04c-13](primal_push_supplier_oracle.md)에서 검증했다. asm `0x1c5da6–0x1c5e48`에서 cursor 선증가·failure 우선·prefix 보류·원래 제약 행 support를 확인했다. `0x1c60d8–0x1c60f4`는 raw count 변환→성공 수 증가→통계 더하기→dense 분기다.
- 경계: supplier 진입부터 반환. 입력 m/n·CSC·drop, 후보·cursor·deferred·failures·support·marks·성공 수·count 합. 실패 횟수·marks·모델 보존. 고갈 시 alpha/scratch/spike는 기존 내용을 보존하며 alpha를 성공 출력으로 취급하지 않는다.
- 소유 버퍼로 S+0x220 입력·S+0x210 출력·F+0x1c8 spike 의미를 표현한다. support<=0에서는 marks를 읽지 않고 support를 0으로 정규화한다.

## 3. 비교 계약

| 필드·사건 | 규칙 | 재구성 |
|---|---|---|
| 반환 j·후보 전체·position·deferred·support·successes | 순서·정수 정확 | before만 Rust 입력, after와 비교 |
| failures·marks·m/total·batch | 원본 보존 | 문맥 전체 비교, 단 batch는 API의 0 전제 |
| count_sum·alpha 값 | LE f64 hex 비트, ±0 포함 | raw FTRAN만 주입, Rust가 합산·압축 |
| FTRAN 호출 수·input·spike 유무 | 정확, CSC 저장 순서/slack +1 | backend가 실제 호출 인자 trace 기록 |
| spike 전후 | backend 경계의 원본 값 | 자연 자료의 spike 전을 초기화하고 backend가 후를 제공; 수치 계산 주장 제외 |
| 고갈 | 호출0·기존 버퍼/work 보존 | canary 입력과 종료 버퍼를 비교 |

유한 raw 값·유한 비음수 drop가 전제다. 희소 순서를 유지하고 dense는 strict abs(v)>drop로 원래 행 오름차순 압축한다. work는 외부 backend로 전달하되 원본 work 비트 비교를 주장하지 않는다. 시간·병렬 분기는 범위 밖이다.

## 4. 사례와 원본 자료

- `bench/oracle/m3_push_supplier/manifest.json`과 자연 arms/retry/interior·Crossover=0 대조군·합성 각 독립3회 자료를 재사용한다. 신규 원본 실행은 필요하지 않다. 원본 C 파일 식별·도구·입력·캡처 해시는 선행 검사기로 다시 확인한다.
- 첫 회 대표 자연51(성공42·고갈9) + 합성18(성공14·고갈4), 총69입력. 자연 raw sparse33·dense9, 합성 sparse10·dense4. 구조/slack·지원0/1/-1·실패2/3·충돌·빈 목록/열·보류 prefix·홀짝 dense·±tol 바로 안팎·±0·전체 제거·count=-7 포함.
- 입력 투영에는 before·모델·drop·backend raw/spike만 포함한다. 원본 after·ret·압축 alpha·예상 FTRAN input/alias는 비교기에만 둔다.

## 5. 재실행 명령

- 선행: `python3 tools/push_supplier_check.py --controls`, `--acceptance`, `--self-test`.
- 투영·Rust 재생·비교: `python3 tools/push_supplier_rust.py --write` → `cargo test -p rsolver-lp --test push_supplier` → `python3 tools/push_supplier_rust.py --compare`.
- 증거 보존: `python3 tools/push_supplier_rust.py --write-manifest`는 현재 비교를 통과한 Rust trace와 코드·입력·선행 원본 manifest·결과 해시를 저장한다.
- 오류 검출: `--self-test`로 trace의 반환·문맥·호출·입력·alpha·순서를 변조하고 실패를 요구한다. `--mutate-rust <새 결과 경로>`로 failure 임계·지원 검사·spike 선택·raw 통계·dense strict/순서 등 Rust 동작 변이를 실행해 거부를 요구한다.
- 회귀: `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p rsolver-lp --test push_chain` → `python3 tools/push_step_chain_rust.py --compare`, `push_chain_oracle.py --check`·`--self-test`, `git diff --check`.
- Python API·기존 solver 실행 경로를 바꾸지 않으므로 확장 재설치·pytest·Netlib·P3는 제외한다.

## 6. 실행 결과

- 계약 `0e2c2e9`, 구현 첫 동작 `9d972bd`. WSL2 x86-64 Linux, 저장 재생 Python3.10, 현재 설치 Rust/Cargo. 환경 상세는 `bench/oracle/m3_push_supplier_rust/validation.json`과 선행 캡처 environment를 따른다.
- 자연51 + 합성18 = **69입력 모두 통과**. 후보·후상태·alpha·실제 backend 입력·호출 수·spike 유무/동일 버퍼·work 전달을 19필드 묶음씩, 총 **1,311묶음** 비교했다. 정수·순서 정확, f64는 ±0 포함 LE hex 비트 일치다. 자연 성공42/고갈9, 합성 성공14/고갈4. 고갈13회는 FTRAN 미호출과 기존 input/alpha/spike/work 보존도 검사한다.
- 선행 `push_supplier_check.py --controls`·`--acceptance`·`--self-test` 종료0, 기존 **31종** 오류 주입 거부. 바이너리·입력·도구/캡처 해시와 독립3회·Crossover=0 대조군·이전 agent 투영을 재확인했다. 신규 원본 실행 **0회**. 기존 독립3회 원본을 저장 재생한 결과이며 이번에 재캡처했다고 주장하지 않는다.
- 새 비교기 `--self-test`는 **29종 거부**(빈/누락/순서 사건, 19필드 변조, 중복/누락 필드, sparse/dense·prefix·통계·고갈 변조). 첫 차이의 사례·필드·토큰 인덱스·기준/실제값을 출력한다.
- Rust `--mutate-rust` **22종 모두 거부**. failure2/4, 지원 무시·marks·slack 충돌, 음수 support 정규화 누락, prefix 오기입·실패 증가, CSC 역순·slack 부호, spike 없음/항상, 성공 수·raw count 부호/압축 후 통계, dense 경계/부호/꼬리/순서, sparse 정렬, 후보/고갈 반환 오류를 포함한다. 모든 변이의 cargo 재생은 성공했고 출력 비교에서 거부됐다. 소스를 복구한 뒤69입력 재생·비교를 다시 통과했다. 결과는 `rust-mutations.json`이다.
- `cargo test --release --workspace` **131통과·실패0·기존 ignored12**, `cargo clippy --workspace --all-targets -- -D warnings` 종료0. 첫 Clippy는 원본처럼 raw count를 왼쪽 피연산자로 쓴 합산을 assign-op 규칙으로 거부했다. 중간 지역값을 두어 원본 연산 순서를 유지하고 수정 후 통과했다.
- `cargo test -p rsolver-lp --test push_chain` 1통과 → `push_step_chain_rust.py --compare` **236단계·5,144필드 비트 일치**,42구간·재분해 비교 제외6 유지. `push_chain_oracle.py --check`·`--self-test`(8종 거부)도 종료0. `py_compile`·`git diff --check` 종료0.
- 원본 after/ret/압축 alpha는 Rust 입력에 들어가지 않는다. 실제 backend 호출에서 input·alpha/spike 버퍼 동일성과 work 전달을 확인한다. 자연 spike 후값도 외부 backend 입력이므로 FTRAN/LU 수치 정확성의 새 증거가 아니다. batch 세 필드는 단일 API의 0 전제이며 m/total은 모델 차원과 후보 길이로 표현한다. 독립 검토자는 사용하지 않았고 이 작업자가 원본·전체 diff와 검사 결과를 검토했다.
- Python 경계·기존 solver 실행 경로를 변경하지 않아 확장 재설치·pytest·Netlib·P3는 제외했다. 계약 축소·원본 기준 변경 없음. 수치 LU, 공급자 사이 caller 상태 연쇄, batch·x87·전체 push/work는 미검증이다.

## 7. 판정과 인수인계

- 작업 **구현 L1 완료**, 명시한69입력·구간의 동등성 **통과**. 실제 solver에는 연결하지 않았다.
- 다음은 **M3-04c-15** factor 요청 선택·호출·복원·품질 검사 오라클. `python3 tools/session.py status` → `python3 tools/session.py claim M3-04c-15 --owner <새-owner>` → `docs/m3/primal_push_loop.md` §2.4·4.3과 `.agents/skills/oracle-capture/SKILL.md`를 읽는다. 실제 LU 연결21·루프 연결22·batch23은 별도다.
- branch `work/m3-04c-14`, worktree `../rsolver-m3-04c-14`, 선행 미커밋 없음. 최종 commit·병합은 main의 `Merge M3-04c-14` 이력에서 확인한다. 계약 범위 검증 후 --no-ff 병합과 같은 단계에서 worktree·branch·M3-04c-14/shared 점유를 정리한다.
- 갱신: `docs/11_progress.md`, `12_roadmap_diagram.md`, `14_sprint_plan.md`, `m3/sprint_design.md`, `handoff/M3.md`. 교훈: `docs/lessons/tooling.md` 검사기 항목(외부 backend 실제 호출 인자·별칭 검증). 원격 동작·외부 업로드 없음.
