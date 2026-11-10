# 도구·환경 교훈과 실패한 접근

형식은 [README](README.md)를 따른다.

## Frida 훅

- **함수 주소 하나에는 리스너를 하나만 붙인다.** 같은 주소에 `Interceptor.attach`를 둘 붙였더니 onLeave가 전부 빠졌다(실패한 접근). 기존 사건과 새 사건을 한 리스너 안에서 같이 기록한다. (M2-03c-2)
- **원시 포인터를 키로 쓴 기록은 실행마다 달라진다.** heap 주소가 재사용되면 다른 노드의 오래된 기록이 걸린다. v1 캡처의 3회 비교가 이 때문에 실패했다(`bench/oracle/mip_retry_basis_v1_failure/`). 현재 메모리 내용의 해시가 기록과 같을 때만 기록을 인정한다. (M2-03c-2)
- **Frida 콜백 안에서 `NativeFunction`으로 부른 원본 함수는 다시 가로채지지 않는다.** 원본 함수를 합성으로 직접 호출해도 자연 사건 기록이 오염되지 않는다. (M2-03c-2)
- **새 훅이 실행을 바꾸지 않았음을 보이려면, 새 캡처를 이전 저장 캡처의 필드만 남기게 투영해 JSON 전체를 비교한다.** 출력 줄(`DONE status nodes`)도 비교한다. 예: `project()` in `tools/mip_retry_basis_check.py`. (M2-03c-2)
- **궤적 비교에는 TimeLimit 대신 NodeLimit을 쓴다.** TimeLimit은 훅 부하 때문에 궤적이 바뀐다. (M2-03c-1)
- **훅 오프셋은 objdump로 함수 진입점인지 확인한다.** 진입점은 보통 `push %r12`로 시작한다. 디컴파일 파일 이름의 앞 숫자가 그대로 훅 오프셋이다. (M2-03c-2)
- **Python venv에는 frida 모듈이 없다.** Node Frida(`tools/mip_retry_basis.js`의 require 경로)를 쓴다. (M2-03b-1)

- **함수 중간 지점 훅은 objdump로 명령 경계, 그 지점으로 가는 jmp 전부의 출처, 레지스터 쓰기를 확인한 뒤 붙인다.** `0x1c3004`는 jmp 9개가 모두 단계 구간 안에서 왔고 r14 쓰기가 없었다. 새 훅의 사건만 빼서 이전 캡처와 같으면 비간섭 증거가 된다. (M3-04b-1, [§3](../m3/primal_push_step_oracle.md#3-캡처-경계-보완))

## 디컴파일 조회

- **디컴파일 C의 `sed`·`cat`이 Bash 출력량의 47%였다.** 11-05~11-10 트랜스크립트 72개 기준이다. 508개 파일 중 192개를 여러 세션·에이전트가 다시 읽었다(`3cd406` 15회). `tools/dc.py`로 크기·호출관계·노트 참조를 먼저 보고 `--grep`·`--range`로 좁히거나 subagent에 맡긴다. 해석한 함수는 `../re/notes`에 요약을 남기면 다음 조회의 참조에 나온다. (도구 정비 2026-11-10, 트랜스크립트 집계)

## 원본 주소 인용

- **캡처에 10진수로 남은 반환 주소는 손으로 바꾸지 말고 `tools/dc.py --dec <값>`으로 함수와 오프셋을 구한다.** 손으로 변환하다 `0x722794`를 `0x722894`로 문서에 적었다(검토에서 발견). `dc.py`는 `../re/decomp/index.tsv`를 bisect한다. (M2-03c-2, 도구는 2026-11-10)
- **문장마다 근거가 C인지 캡처인지 구분한다.** C로 추정한 "별도 트리"를 캡처한 사실처럼 썼다가 고쳤다. (M2-03c-2)

## Herdr·실행

- **`pane wait-output --match`는 화면에 다시 찍힌 명령줄에도 맞는다.** 끝 표시는 `--regex "EXIT=[0-9]"`처럼 숫자까지 넣어 기다린다. (M2-03c-2)
- **`pane wait-output --regex`에 끝 앵커 `$`를 넣었더니 즉시 종료코드 1로 끝났다(실패한 접근).** 시작 앵커를 쓴 `--regex "^ALL_DONE_RC=[0-9]"`는 화면에 다시 찍힌 명령줄과 구분되고 정상으로 기다렸다. (M2-03d-2)
- **같은 pane을 재사용하면 `clear` 뒤에도 `wait-output`·`read`가 이전 실행의 끝 표시에 맞는다(실패한 접근).** 실행마다 다른 끝 표시를 쓰거나 표시 개수를 세며 기다린다. (M3-04b-2)
- **pane 끝 표시에 `cmd; echo; echo RC=$?`를 쓰면 RC는 `echo`의 0이다(실패한 접근).** `( cmd ); r=$?; echo RC=$r`로 받는다. 괄호가 없으면 명령 안의 `exit`가 pane 셸을 끝내 끝 표시가 영영 안 나온다. `tools/hrun.sh`가 이 방식이다. (도구 정비 2026-11-10, 실행 확인)
- **Bash 도구 timeout은 최대 600초다.** 긴 배치는 `wait-output --timeout 580000`을 여러 번 부른다. (M2-03c-2)
- **`tools/gurun.sh`는 출력을 `$()`로 받기 때문에 pane에 ctrl+c를 보내도 멈추지 않는다.** 자신이 띄운 PID(runs.sh, gurun, node, frida로 띄운 python)를 kill한다. (M2-03c-2)
- **원본 캡처는 약 2분/회(nidda 6000노드, gmu 20000노드)다.** 다른 세션이 Gurobi를 쓰면 `gurun.sh`가 대기하므로 더 걸린다. (M2-03c-2)

- **Codex 세션이 사용량 한도로 끝나면 rollout JSONL(`~/.codex/sessions/YYYY/MM/DD/*<id>.jsonl`)의 `task_complete.error`와 마지막 `custom_tool_call_output`로 중단 지점을 복구한다.** 마지막 출력에 남은 캡처 결과가 미추적 파일로 남아 있을 수 있다. (M3-04b-1)

## 검사기

- **검사기는 `controls`와 `acceptance`로 나누고, `--self-test`와 `--write-manifest`를 둔다.**
  - `controls`: manifest·이전 캡처 투영·3회 일치.
  - `acceptance`: 결과를 보기 전에 고정한 판정 규칙.
  - `--self-test`: 메모리를 변조해 각 검사가 거부하는지 확인한다.
  - 참고 구현은 `tools/mip_retry_basis_check.py`다. (M2-03c-2)
- **양쪽이 모두 null이면 합격이 아니다.** 합성 결과만 있어도 목표 검사가 0이 아닌 값으로 끝나야 한다. 둘 다 변조 항목에 넣는다. (M2-03c-1·03c-2)
- **새 기준 자료 투영기는 기존 검사기를 import해 manifest·3회 일치를 재사용한다.** `tools/mip_retry_rust_expect.py`가 `mip_retry_basis_check`의 `load`·`first`·`check_manifest`를 쓴다. trace 비교는 첫 차이(사건·필드·기준값·실제값)를 출력하고 추가 필드·사건도 실패로 본다. (M2-03d-1)

## git worktree

- **ignore된 `target/`(재생성할 수 있는 탐색 출력)이 남아 있으면 `git worktree remove`에 `--force`가 필요하다.** 미커밋·미추적 파일이 그것뿐인지 먼저 확인한다. (M2-03c-2)
