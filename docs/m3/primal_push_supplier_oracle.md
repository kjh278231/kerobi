# M3-04c-13 단일 공급자 오라클

## 1. 범위와 주장

- 종류: 오라클 구축. 선행 main `cb0c701`, M3-04c-12. branch `work/m3-04c-13`, worktree `../rsolver-m3-04c-13`, 선행 미커밋 없음(main의 `.claude/settings.local.json`은 별개로 보존).
- 목표: batch 용량·채움·소비가 모두 0인 `25615c@0x1c5310`의 후보 소비·skip·보류·통계·dense 압축을 같은 입력에서 재생할 수 있도록 고정한다.
- 자연 사례: arms·retry·interior의 primal-first, Crossover=0 대조군. 각 독립 3회. 기존 agent 사건 전체에 투영해 비간섭을 확인한다. 구조 열·slack·support 0/1·실패 2/3·충돌·고갈은 자연 캡처에서 집계한다.
- 정확 경계: 별도 프로세스의 합성 직접 호출. 5포인터 ABI와 C/asm의 읽기 필드를 확인한 후 최소 S/P/F와 벡터를 구성한다. 수치 factor를 위조하지 않고 `09d008`만 명시적 합성 backend로 대체해 정한 raw 출력(sparse 역순, dense ±tol/그 안팎/±0)을 공급한다. support -1 정규화·failure 2/3·충돌과 미충돌·고갈·빈 열도 검사한다. 이 실험은 자연 FTRAN 수치 증거와 구분한다.
- 제외: Rust(14), FTRAN 수치 구현·실제 LU 연결(21), batch(23), x87·전체 crossover·work 패리티. backend 호출 전후 work는 수집하되 합격은 통계·제어와 alpha 비트에 한정한다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C: `../re/decomp/c/001c5310_PRIVATE000000000025615c.c` 71–80·526–688; `00120680_PRIVATE000000000009d008.c`.
- ELF offset=파일 이름 주소(Ghidra +0x100000). SysV x86-64 `int supplier(P*,S*,alpha**,support*,work*)`; `09d008(F*,input*,output*,spike*,0,0,0,work*)`.
- 시작/종료: 공급자 함수 진입/반환. 중첩 `09d008@0x120680` 진입/반환에서 원본 CSC/slack, raw alpha와 spike를 수집한다. 포인터 대신 S+0x210·S+0x220·F+0x1c8과의 별칭 참거짓을 기록한다.
- P: m,total,candidates,failures,marks,cursor,deferred,count 합·성공 수,batch 세 필드. S: m,n,CSC,drop tolerance. 사용 중인 prefix가 덮어써져도 순서를 보존한다. 고갈에서는 기존 alpha 내용을 유효 출력으로 해석하지 않는다.

## 3. 비교 계약

| 필드 | 규칙 | 방법 |
|---|---|---|
| 후보·cursor·deferred·failures·지원 표시·support·반환 j | 정수·순서 정확 일치; 실패 횟수와 지원 표시는 공급자 동안 보존 | 전 상태에서 제어 재생 후 후 상태와 비교 |
| 성공 수·count 합 | 압축 전 raw count 누적; f64 비트 일치 | raw FTRAN 출력으로 예측 |
| 원본 열 | CSC 저장 순서 또는 slack +1의 비트 일치 | push 진입 모델과 FTRAN 입력 비교 |
| raw/output alpha | sparse 순서 유지; dense는 abs(v)>tol만 행 오름차순, ±0 포함 비트 비교 | FTRAN 반환과 공급 반환 비교 |
| 별칭·호출 순서 | 지원0일 때 spike=F+0x1c8, 양수일 때 null; output=S+0x210,input=S+0x220 | 진입/반환 훅 |
| 출처·재현·비간섭 | 바이너리/입력/도구 해시·설정 고정, 독립3회 사건·출력 일치, 이전 사건 전체 투영 일치 | manifest + controls |

유한한 허용오차와 유한 raw 값이 전제다. NaN·무한대 수치 factor는 제외한다. 후속 분기에 쓰는 f64는 오차 허용 없이 LE hex 비트 비교한다. 시간 분기·병렬은 제외(Threads=1), work 수치 동등성은 제외다. 필수 상태 null·사건0회·합성만 존재하면 목표 합격 거부. 단 support<=0의 `support_marks`는 원본이 읽지 않는 초기화 전 메모리일 수 있어 의미적으로 null로 표기하고, support>0에서는 길이 m의 배열을 요구한다.

## 4. 사례와 원본 자료

`bench/oracle/m3_push_supplier/`에 자연3사례×3회·대조군3회·합성3회·manifest·검증 결과를 둔다. 자연 자료에는 실제 FTRAN 결과와 spike를, 합성에는 backend 대체를 명시한다. 최종 입력·기준 경로/해시는 [manifest](../../bench/oracle/m3_push_supplier/manifest.json)로 고정한다.

| 사례 | 입력과 설정 | 원본 근거 |
|---|---|---|
| arms | `bench/oracle/m3_push_arms/arms.lp`, 같은 디렉터리 `arms.extra.json`의 PStart/DStart·ScaleFlag=0 | `arms-{1,2,3}.json`, 공급29·성공25·고갈4 |
| retry | `bench/oracle/m3_push_retry/retry.lp`·`retry.extra.json` | `retry-{1,2,3}.json`, 공급11·성공8·고갈3 |
| interior | `bench/oracle/m3_primal_push_step/interior.lp`, 시작벡터 없음·ScaleFlag 기본값 | `interior-{1,2,3}.json`, 공급11·성공9·고갈2 |
| control | arms와 같은 모델·시작벡터, Crossover=0 | `control-{1,2,3}.json`, 사건0·최적 상태2 |
| 합성 제어 경계 | `push_supplier_synthetic.py:specs()`, 원본 supplier 호출·FTRAN만 명시적 대체 | `synthetic-{1,2,3}.json`,18사례·성공14·고갈4 |

합성18사례는 구조/slack 희소 출력, support -1/1, 구조/slack 충돌, failure2/3, 전체 실패·전체 충돌·소비완료·빈 목록·빈 열, dense 홀수/짝수·±tol·바로 안/밖·±0·전체 제거·음수 count -7, 기존 보류 prefix 보존이다. 반환 sparse의 순서는 `[8,0,3]`로 고정해 정렬 오류를 검출한다. NaN·무한 raw 값·음수 후보는 원본 전제 밖이므로 넣지 않았다.

탐색 `history/arms-primal-probe.json`·`synthetic-probe.json`과 오류 이력 `history/synthetic-m9-error-{1,2,3}.json`은 최종 합격 자료가 아니다. 초기 합성 생성기의 `m=8`이 문맥 안으로만 들어가 실제 호출은 m=9였고, 검사기의 raw 길이 검사가 이를 거부했다. m 전달·CSC 행 범위를 맞춘 뒤 최종 합성3회를 새로 캡처했다. 실패 자료를 기준 자료로 승격하거나 삭제하지 않았다.

## 5. 재실행 명령

- 전체 순차 캡처: `python3 tools/push_supplier_runs.py <새 출력 디렉터리>`(기존 파일 덮어쓰기 거부).
- 자연 캡처: `tools/gurun.sh node tools/push_supplier_capture.js primal <model> <output> '<extra>'`.
- 합성: `tools/gurun.sh ../.venv/bin/python tools/push_supplier_synthetic.py --capture <output>`.
- 검사: `python3 tools/push_supplier_check.py --controls`, `--acceptance`, `--self-test`, `--write-manifest`.
- 회귀: `python3 tools/push_chain_oracle.py --check`, `--self-test`; `cargo test -p rsolver-lp --test push_chain` → `python3 tools/push_step_chain_rust.py --compare`. Rust 변경이 없으므로 전체 workspace·clippy·확장 재설치는 제외.
- 오류 주입: 바이너리/입력 해시0·빈 사건·목표0회·필드 null·3회 차이·후보/보류/통계/별칭/raw/압축 순서 변조를 거부해야 한다. 동일 변조를3회 모두 적용해 재현성 검사만으로 거부되는 것을 방지한다.

## 6. 실행 결과

- 기준 main `cb0c701`, 착수 계약 `8a2db1d`, 훅 첫 동작 `581f507`, 캡처 `595ccb6`, 검사기·짝수 사례 수정 `ed0843c`. WSL2 x86-64 Linux, Node Frida17.26.2·Node v22.22.1, 캡처 Python3.13·저장 검사 Python3.10. 실제 세부 환경은 캡처의 environment와 manifest를 따른다.
- 자연 정식12회(3사례×3 + 대조군3), 합성 정식3회 모두 종료0. 자연 사례별 사건237/96/105, 대조군0; 각3회 사건·출력·결과·파라미터가 동일하다. pid/run_id 독립성을 확인했다(합성은 run_id와 순차 실행 명령). 새로운 필드를 뺀 전체 사건과 출력·결과·설정은 이전 agent 캡처와 일치한다. 투영은 `supplier:*` 사건과 supply의 `supplier` 필드, interior의 추가 `frame`, arms/retry의 추가 `frame.ninf`만 제거한다. 이전 스키마의 모든 필드·사건·순서는 그대로 비교한다.
- 대표 기록(반복3회 중 첫 회만 집계): **자연 공급51회**, 성공42·고갈9, 구조열40·slack2, support 양수39·0경로12, 실패2인 후보 조회2·실패>=3 skip1·지원 충돌skip13. raw FTRAN **sparse33·dense9**를 확인했다. dense9는 interior의9성공 전부이며 반환에서 sparse가 되어 보이지 않던 내부 표현이다. 압축 전 count 통계와 압축 후 순서·비트를 모두 대조했다.
- 합성 대표18사례: 성공14·고갈4, raw sparse10·dense4, support -1정규화1·실패2조회1·실패skip4·충돌skip4. FTRAN을 대체한 수치 입력을 사용하므로 원본 LU 수치 검증으로 세지 않는다.
- `push_supplier_check.py --write-manifest`·`--controls`·`--acceptance` 종료0. 자연51공급과 합성18사례의 반환·전후 문맥·호출·alias·원본 열·압축 결과가 일치했다. 필수 필드와 순서 검사를 포함하며 양쪽 null을 성공으로 세지 않는다.
- `--self-test` 종료0, **31종 거부**. 재현성에만 기대지 않도록 동작 변조는3회에 똑같이 주입하고 acceptance를 직접 호출했다. 첫 차이는 [self-test.json](../../bench/oracle/m3_push_supplier/self-test.json)에 있다. 조작 종류는 hash·빈 사건·목표0·회차 차이·투영·null·후보·cursor·deferred·failures·marks·통계·별칭·입력 열·raw·dense 경계/순서·sparse 순서·support 정규화·보류 prefix·합성 전용 출처다.
- 기존 `push_chain_oracle.py --check`·`--self-test` 종료0(선행8종 거부), `cargo test -p rsolver-lp --test push_chain` 1테스트 통과, `push_step_chain_rust.py --compare` **236단계·5,144필드 비트 일치**,42구간·재분해 뒤 비교제외6(기존 한계 유지). 처음 새 worktree에서 trace 생성 전에 --compare를 실행해 파일없음으로 실패했으며 cargo 테스트로 trace를 생성한 뒤 통과했다.
- 독립 검토: 자연 arms1회·합성1회 재캡처가 기준 사건·출력과 같고, 검사3종과 별도 변조7종도 통과했다. [검토 기록](../../bench/oracle/m3_push_supplier/review.md)에 최종 결론을 남긴다.
- Rust·Python API를 바꾸지 않았으므로 workspace 전체·clippy·확장 재빌드·전체 Netlib·P3는 미실행. 실제 LU 재구성·batch·x87·work·전체 push 동등성은 이 계약 밖이다. 계약 범위 축소 없음. m 전달 오류는 합격 조건을 바꾸지 않고 생성기를 고쳐 재캡처했다.

## 7. 판정과 인수인계

- 작업: **오라클 구축 완료(범위 한정)**. 원본 자연 캡처의 재현·비간섭과 supplier 제어 재생, 명시적 합성 FTRAN backend 경계까지 통과했다. Rust 구현 동등성은 **미검증**이며 이번 새 Rust 구현은 없다.
- 다음 **M3-04c-14**: `python3 tools/session.py status` → `python3 tools/session.py claim M3-04c-14 --owner <새-owner>`. 이 문서와 `primal_push_loop.md` §2.2를 읽고 `push_supplier_check.py:units/replay`와 자연·합성69입력으로 제어 L1을 구현한다. FTRAN 수치 계산은 외부 backend이고21로 분리한다. 나중에 여러 round를 연결할 때 공급자 사이 caller의 변경을 원본 스냅숏으로 재주입하지 않는 검증은22에서 한다.
- branch `work/m3-04c-13`, worktree `../rsolver-m3-04c-13`, 선행 미커밋 없음. 최종 commit·병합은 main의 `Merge M3-04c-13` 이력으로 확인한다. 병합과 같은 단계에서 branch·worktree 정리와 M3-04c-13·shared 점유 해제를 수행한다.
- 갱신: `docs/11_progress.md`, `12_roadmap_diagram.md`, `14_sprint_plan.md`, `m3/sprint_design.md`, `handoff/M3.md`. 교훈은 `docs/lessons/original.md`(내부 dense 관측)와 `tooling.md`(합성 차원·trace 생성 순서)에 기록한다. 원격 동작·외부 업로드 없음.
