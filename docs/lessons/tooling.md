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

## 원본 주소 인용

- **캡처에 10진수로 남은 반환 주소는 `hex()`로 변환해 인용한다.** 손으로 변환하다 `0x722794`를 `0x722894`로 문서에 적었다(검토에서 발견). 함수 매핑도 같은 값으로 `../re/decomp/index.tsv`를 bisect해 구한다. (M2-03c-2)
- **문장마다 근거가 C인지 캡처인지 구분한다.** C로 추정한 "별도 트리"를 캡처한 사실처럼 썼다가 고쳤다. (M2-03c-2)

## Herdr·실행

- **`pane wait-output --match`는 화면에 다시 찍힌 명령줄에도 맞는다.** 끝 표시는 `--regex "EXIT=[0-9]"`처럼 숫자까지 넣어 기다린다. (M2-03c-2)
- **Bash 도구 timeout은 최대 600초다.** 긴 배치는 `wait-output --timeout 580000`을 여러 번 부른다. (M2-03c-2)
- **`tools/gurun.sh`는 출력을 `$()`로 받기 때문에 pane에 ctrl+c를 보내도 멈추지 않는다.** 자신이 띄운 PID(runs.sh, gurun, node, frida로 띄운 python)를 kill한다. (M2-03c-2)
- **원본 캡처는 약 2분/회(nidda 6000노드, gmu 20000노드)다.** 다른 세션이 Gurobi를 쓰면 `gurun.sh`가 대기하므로 더 걸린다. (M2-03c-2)

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
