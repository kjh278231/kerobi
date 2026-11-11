# pi 작업자 설계 (OpenGateway + GLM 5.3 Flash)

실행 스크립트와 로그 검사기를 구현했다. pi 1.5.2에서 잘못된 주소의 인증 실패와 공식 주소의 성공 응답을 확인했다. 스프린트 도입에 필요한 출처 정책과 작업 검증은 §2와 §7에 적었다.

## 1. 목표와 범위

스프린트 하위 작업의 작업자를 pi 코딩 에이전트로 띄우고, 모델은 OpenGateway를 거친 GLM 5.3 Flash를 쓴다. 속도와 비용이 목적이다.

- 바꾸는 것: [위임 절차](handoff/README.md) §5의 "작업자"가 어떤 프로세스인가.
- 바꾸지 않는 것: 점유·branch·worktree·검증 계약·인수인계·검토 절차. 오케스트레이터와 검토자는 지금처럼 Claude 세션이다.
- 만들지 않는 것: pi 안에서 다시 하위 에이전트를 띄우는 확장, 전용 오케스트레이션 서비스, 큐, 대시보드. Herdr와 `tools/hrun.sh`로 충분하다.

## 2. 사용자 결정이 필요한 것

### 2.1 출처 정책 (도입을 막는 항목)

[출처 정책](00_overview.md)은 저장소를 로컬 git으로만 두고 배포·공개를 금지한다. pi 작업자는 읽은 파일을 그대로 게이트웨이에 보낸다. 포팅 작업이면 `../re/decomp/`의 디컴파일 C·asm과 `../re/notes`가 전송 대상에 들어간다.

확인한 사실은 다음과 같다.

- 운영 주체는 Sionic AI다. 팀에서 요청·응답 본문 로깅을 켜면 정책상 본문은 최소 90일간 보존된다. 시험 계정의 로깅 설정은 확인하지 않았다. [OpenGateway 개인정보 처리방침](https://opengateway.ai/legal/privacy-policy)
- 게이트웨이는 모델과 키에 허용된 제공자·리전 후보 중에서 요청을 처리한다. 모델 ID는 제공자 지정에 쓰이지 않는다. [OpenGateway 모델과 라우팅](https://opengateway.ai/docs/gateway/models-and-routing)

선택지:

| 선택 | 내용 | 결과 |
|---|---|---|
| A | 원본 자료 전송 허용 | 모든 하위 작업을 맡길 수 있다. 정책 문서를 고쳐야 한다. |
| B | 원본 자료 전송 금지 (권장 기본값) | `../re/`를 읽지 않는 작업만 맡긴다. §3의 표가 이 기준이다. |
| C | 도입 보류 | 보존 정책을 확인한 뒤 다시 결정한다. |

B에서도 Rust 코드와 작업 문서는 전송된다. Rust 코드는 원본의 2차적 저작물이므로 B도 정책 예외이며 사용자 승인이 필요하다.

B는 공짜 기본값이 아니다. 교훈 기록에 남은 하위 에이전트 용도는 디컴파일 C 읽기와 asm 대조 명세이고([process](lessons/process.md), [tooling](lessons/tooling.md)), 둘 다 `../re/`가 필요하다. B에서는 이 작업을 pi에 맡길 수 없다.

### 2.2 게이트웨이와 모델 ID

OpenGateway AI의 API 기본 주소는 `https://apis.opengateway.ai/v1`이다. 이번에 사용한 모델의 전체 ID는 `z-ai/glm-5.3-flash-ultrafast`이며 `-ultrafast`까지 모델 ID에 포함된다. [OpenGateway Pi 설정](https://opengateway.ai/docs/coding-agents/pi)

### 2.3 무엇을 대체하는가

현재 "하위 에이전트로 스프린트를 돌리는" 방식을 두 가지로 추정했다. 어느 쪽인지에 따라 대체 범위가 다르다.

- Claude Code의 Agent 도구로 띄우는 하위 에이전트(조사·명세 작성). Agent 도구는 외부 모델을 받지 못한다. 이 작업을 GLM으로 옮기려면 Herdr 경로로 바꿔 띄워야 하고, 2.1에서 A가 필요하다.
- Herdr로 띄우는 다른 모델 작업자(handoff §5). 이 문서가 대체하는 경로다.

## 3. 맡길 작업과 남길 작업

이 저장소의 구현은 asm과 비트 단위로 맞춰야 한다. Flash 등급 모델에게는 판정이 기계적으로 가능한 작업만 맡긴다. 아래 구분은 가정이며 0단계 결과로 고친다.

| 작업 | 작업자 | 이유 |
|---|---|---|
| 원본 함수 해석, asm 대조 명세 | Claude | `../re/` 필요, 해석 오류가 뒤에서 드러난다 |
| 검증 계약 작성, 완료 판정, 병합 | Claude | handoff §5 검토 책임 |
| 명세가 확정된 Rust 구현(명세는 작업 문서에 있음) | pi (2.1에서 A 또는 B 승인 시) | 검사기 종료 코드로 판정 가능 |
| 검사기·오라클 비교 스크립트 작성 | pi | 대조군·변조 입력으로 검토 가능 |
| clippy·테스트 실패 수정, 문서 표 갱신 | pi | 범위가 좁고 명령으로 확인 가능 |
| Gurobi 실행이 필요한 캡처 | Claude | 한 프로세스 제한을 pi에서 강제할 수단이 없다(§5) |

## 4. 구성

```
Claude 세션 (오케스트레이터·검토자)
  └─ tools/hrun.sh -- 'tools/pi_worker.sh <프롬프트 파일>'      # Herdr pane
       └─ pi (작업자, work/<하위번호> worktree에서 실행)
            └─ https://apis.opengateway.ai/v1/chat/completions → <MODEL_ID>
```

### 4.1 설치

```bash
npm install -g @earendil-works/pi-coding-agent@<고정 버전>
```

이번 검증에는 설치된 `@earendil-works/pi-coding-agent` 1.5.2를 사용했다. 설치할 때 버전을 고정한다.

### 4.2 공급자 등록

공급자 예시를 `~/.pi/agent/models.json`의 기존 설정에 추가한다. `PI_CODING_AGENT_DIR`를 지정했으면 그 디렉터리의 `models.json`을 사용한다. [공급자 설정 예시](../tools/pi_models.example.json)

- 예시의 모델 ID·문맥·출력·요금은 자리 표시다. `z-ai/glm-5.3-flash-ultrafast`의 공식 Pi 설정은 `contextWindow: 1000000`, `maxTokens: 131072`다. 요금은 모델 페이지에서 확인한다. [OpenGateway Pi 설정](https://opengateway.ai/docs/coding-agents/pi)
- `authHeader: true`로 Bearer 헤더를 보내는 동작을 로컬 대역에서 확인했다. 같은 설정으로 공식 주소에 보낸 요청도 성공했다.
- `compat` 플래그는 넣지 않는다. pi 문서는 "OpenAI 호환"이라는 광고만 보고 켜지 말라고 한다. 0단계에서 실패한 항목만 추가한다.
- 키는 환경변수에만 둔다. 저장소와 `models.json`에 값을 쓰지 않는다.

### 4.3 실행 명령

작업 worktree에서 실행한다. `PI_WORKER_MODEL`에 등록한 모델 ID를 지정한다.

```bash
PI_WORKER_MODEL='<MODEL_ID>' tools/pi_worker.sh 'docs/handoff/prompts/<하위번호>.md'
```

Herdr에서 기다릴 때는 호출자가 같은 명령을 `tools/hrun.sh`로 감싼다.

```bash
tools/hrun.sh --timeout 580000 -- \
  'PI_WORKER_MODEL="<MODEL_ID>" tools/pi_worker.sh "docs/handoff/prompts/<하위번호>.md"'
```

- `PI_WORKER_PROVIDER`의 기본값은 `opengateway`다. `PI_WORKER_MODEL`이 비어 있으면 사용법을 출력하고 종료 코드 `2`를 반환한다.
- 스크립트는 프롬프트 파일을 pi의 stdin으로 전달한다. stdout은 `.pi-run/<이름>.jsonl`에, stderr는 `.pi-run/<이름>.err`에 저장한다. 이름을 생략하면 프롬프트 파일명에서 `.md`를 뺀 값을 쓴다. Git은 `.pi-run/`을 무시한다.
- 스크립트는 `tools/pi_log_check.py`의 종료 코드를 반환한다. 검사기는 파싱 실패, 마지막 assistant 메시지 부재나 오류·중단, 금지 도구 호출을 코드 `1`로 판정한다. 원본 접근이 허용된 작업에서는 `PI_WORKER_ALLOW_RE=1`로 검사기의 원본 접근 판정을 해제한다. Gurobi 명령 검사는 항상 적용한다.
- pi는 `-a`로 이번 실행의 프로젝트 파일을 신뢰한다. `AGENTS.md`와 `.agents/skills`의 실제 로드 여부는 §7의 0단계에서 확인한다.
- Herdr 도움말의 `agent start --kind` 목록에 `pi`가 있다. 이 비대화형 경로에서는 `hrun.sh`가 작업 종료를 기다린다. `hrun.sh`가 `124`를 반환하면 `tools/hrun.sh --wait <pane>`으로 이어서 기다린다.
- 재시도와 자동 압축은 pi 기본값을 쓴다(`retry.maxRetries` 3, `compaction.enabled` true).

### 4.4 프롬프트

handoff §5의 순서를 그대로 쓴다: `목표` · `완료 기준(명령과 기대 종료코드)` · `도달 불가 시 산출물` · `금지` · `보고 형식` · 마지막 줄 한국어 지시. 프롬프트는 작업 branch의 파일로 두어 재실행할 수 있게 한다.

pi 작업자에게만 추가하는 금지:

- Gurobi를 실행하는 명령 금지.
- 2.1에서 B를 고르면 `../re/` 아래 파일 읽기와 `tools/dc.py` 실행 금지. `dc.py`는 명령에 경로가 드러나지 않은 채 `../re/`를 읽는다.
- B에서는 프롬프트에 "작업 문서의 명세가 `AGENTS.md`의 '원본 디컴파일 결과를 먼저 확인' 단계를 대신한다"고 적는다. 자동으로 읽히는 `AGENTS.md`가 반대로 지시하기 때문이다.
- `git push`, `git remote` 변경, main worktree 수정, 공유 `.venv` 설치 금지(기존 금지와 같다).

## 5. pi에 없는 안전장치

| Claude Code에 있는 것 | pi | 대책 |
|---|---|---|
| 권한 확인 | 없음. 도구가 pi 프로세스 권한으로 바로 실행된다 | 작업 worktree에서만 실행, 키 외의 비밀을 환경에 두지 않음 |
| `solver-busy-check` 훅 | 동작하지 않음 | Gurobi 작업을 맡기지 않음. 작업자가 실행 중이면 오케스트레이터도 Gurobi를 실행하지 않음(handoff §5) |
| 샌드박스 | 없음 | 금지 사항은 프롬프트로만 전달된다. 검토 때 `git status`·`git worktree list`·main의 diff로 위반을 확인 |
| 실패 종료 코드 | pi 1.5.2는 HTTP 401 실패를 JSONL에 기록하고 코드 `0`으로 종료했다 | 로그 검사기의 종료 코드로 판정 |

`../re/` 읽기 금지를 프롬프트에만 맡기면 약하다. B를 고르면 작업자를 `../re/`가 보이지 않는 곳에서 실행하는 방법(컨테이너 또는 별도 사용자)을 0단계에서 정한다. 그 전에는 로그에서 `../re/` 경로나 `dc.py`가 나온 `tool_execution_start`가 0건인지 검사한다.

## 6. 완료 판정과 검토

판정은 handoff §5의 검토 절차다. pi의 출력은 판정 근거가 아니다.

1. `hrun.sh`가 돌아오면 작업자 branch의 커밋과 `docs/handoff/M*.md` 갱신을 읽는다.
2. 계약의 완료 기준 명령을 오케스트레이터가 직접 실행해 종료 코드를 확인한다.
3. 검사기 변조 검사(해시 0, 필수 사건 배열 비우기, 목표 0회 입력)를 직접 실행한다.
4. `.pi-run/<하위번호>.jsonl`에서 마지막 assistant `message_end`의 보고와 토큰 사용량을 읽는다. 보고와 2·3의 결과가 다르면 결과를 따른다.

검사기는 마지막 assistant `message_end`의 텍스트와 각 `message_end.message.usage.totalTokens`의 합계를 출력한다. 도구 호출은 `tool_execution_start`로, 도구 오류는 `tool_execution_end.isError`가 `true`인 건으로, 자동 재시도는 `auto_retry_start`로 센다.

## 7. 도입 단계

### 0단계: 연결과 능력 확인 (스프린트에 쓰기 전 필수)

1. 키로 `GET https://apis.opengateway.ai/v1/models`를 호출해 허용된 모델 ID와 문맥 크기를 확인한다.
2. 도구 호출 1회 시험: `pi --mode json -p --no-session --model opengateway/<MODEL_ID> "README.md의 첫 줄을 읽어 그대로 출력"`. `tool_execution_end`가 1건 이상이고 `isError`가 false인지 본다. 게이트웨이 페이지에 도구 호출·스트리밍 지원 문구가 없다.
3. `AGENTS.md`와 `.agents/skills`가 실제로 로드되는지 확인한다(`-a` 유무 각각).
4. 실패 시 종료 코드를 관찰해 기록한다(잘못된 키, 없는 모델).
5. 재현 시험: 이미 병합된 작은 하위 작업 하나를 골라 병합 전 commit에서 같은 프롬프트로 다시 시킨다. 계약의 완료 기준 명령이 통과하는지, 금지 위반이 없는지, 걸린 시간·토큰을 기록한다. 대상은 `../re/`를 읽지 않아도 되는 작업으로 고른다. worktree는 이력을 공유하므로 작업자가 `git show`로 병합된 답을 볼 수 있다. 병합 전 commit까지만 담은 별도 clone에서 실행한다. B에서는 작업자가 `AGENTS.md`와 프롬프트 중 어느 지시를 따랐는지도 기록한다.

합격 조건: 1–4 확인, 5에서 완료 기준 통과와 금지 위반 0건. 불합격이면 §3의 표에서 pi 몫을 줄이거나 도입을 중단한다.

### 1단계: 한 영역에서 시험 사용

한 영역(M1·M2·M3 중 하나)의 하위 작업 2–3개에 쓴다. 작업마다 재작업 횟수와 검토에서 잡힌 오류를 handoff에 적는다.

### 2단계: 문서 반영

1단계 결과가 좋으면 공유 경계 점유를 얻고 다음 문서를 고친다.

- `docs/handoff/README.md` §5: pi 작업자 실행 명령과 §4.4의 추가 금지.
- `docs/README.md`: 이 문서 링크.
- `docs/00_overview.md` §2: 2.1의 결정.
- `docs/handoff/prompts/`: 새 경로다. 위치를 handoff에 적는다.

## 8. 확인한 것과 못 한 것

| 항목 | 상태 | 출처 |
|---|---|---|
| 게이트웨이 주소 `https://apis.opengateway.ai/v1`, Chat Completions, Bearer 인증 | 공식 문서와 실제 성공 응답에서 확인 | [API 레퍼런스](https://opengateway.ai/docs/reference/guides/overview), §8.3 |
| 모델 ID `z-ai/glm-5.3-flash-ultrafast`, 문맥 1,000,000 토큰, 최대 출력 131,072 토큰 | ID는 실제 호출에서 확인. 토큰 한도는 공식 설정값 | [OpenGateway Pi 설정](https://opengateway.ai/docs/coding-agents/pi) |
| 도구 호출, 스트리밍 델타, Herdr 연결 | 별도 실동작 시험 미실행 | §8.3 |
| Herdr의 pi 지원 | `agent start --kind` 목록에 `pi` 표시. 실제 기동과 `hrun.sh` 연결은 미실행 | `herdr agent` 도움말 |
| `models.json` 경로·필드, `apiKey`의 `$NAME` 해석, `compat` 주의 | 공식 문서에서 확인 | https://pi.dev/docs/latest/models-configuration , https://pi.dev/docs/latest/custom-provider |
| `--mode json`, `-p`, `--no-session`, `-a`, `--provider`, `--model`, `--tools` | pi 1.5.2 도움말에서 확인. `-a`는 이번 실행의 프로젝트 파일을 신뢰하고, `--tools`는 쉼표로 구분한 도구 이름을 받는다 | `PI_CODING_AGENT_DIR=<임시 경로> pi --help` |
| 컨텍스트 파일·스킬 탐색과 로드 여부 | 경로는 README 0.81.1 참조. `-a` 유무에 따른 실제 로드는 미확인 | https://cdn.jsdelivr.net/npm/@earendil-works/pi-coding-agent@0.81.1/README.md |
| 재시도·압축 설정 기본값 | 공식 문서에서 확인 | https://pi.dev/docs/latest/settings.md |
| 인증 실패 시 pi의 종료 코드 | 최초 시험 주소의 HTTP 401에도 코드 `0`으로 종료. 키 변수를 비운 별도 실행은 생략 | [인증 실패 로그](../tools/testdata/pi_log/invalid_key.jsonl) |
| 도구 실행 이벤트 | `tool_execution_start.toolCallId`·`toolName`·`args`, `tool_execution_end.toolCallId`·`toolName`·`result`·`isError`를 설치 문서에서 확인 | `$(npm root -g)/@earendil-works/pi-coding-agent/docs/json.md`, 같은 경로의 `message-types.md` |

### 8.1 pi 1.5.2 실패 캡처

저장소 밖의 임시 디렉터리에서 `PI_CODING_AGENT_DIR="$PWD/agent"`를 사용했다. 당시 `models.json`의 주소는 `https://api.opengateway.one/oss/v1`, 모델은 `zai-org/GLM-5.2:fastest`였다. `prompt.txt`에는 `say hi` 한 줄을 넣었다. `PI_CAPTURE_INVALID_KEY`는 시험용으로 만든 잘못된 값이다.

```bash
PI_CODING_AGENT_DIR="$PWD/agent" OPENGATEWAY_API_KEY="$PI_CAPTURE_INVALID_KEY" \
  PI_TELEMETRY=0 timeout 50s pi --offline --no-context-files --no-skills \
  --no-extensions --no-mcp --system-prompt ' ' --mode json -p --no-session -a \
  --provider opengateway --model 'zai-org/GLM-5.2:fastest' \
  --tools read,bash,edit,write,grep,find,ls < prompt.txt > failure.jsonl 2> failure.err
```

- 샌드박스에서 `getaddrinfo EAI_AGAIN api.opengateway.one`이 발생했다. JSONL 37행에 `auto_retry_start` 3건이 남았고 stderr는 비어 있었다. pi는 코드 `0`으로 종료했다. [DNS 실패 로그](../tools/testdata/pi_log/dns_failure.jsonl)
- 네트워크가 허용된 환경에서 같은 명령을 실행하자 `.one` 주소가 HTTP 401과 `Invalid or missing preview API key.`를 반환했다. JSONL은 12행이고 재시도 0건, stderr 0행, pi 종료 코드 `0`이었다. [인증 실패 로그](../tools/testdata/pi_log/invalid_key.jsonl)
- 두 로그에서 `session`, `agent_start`, `turn_start`, `message_start`, `message_end`, `turn_end`, `agent_end`, `agent_settled`를 확인했다. `agent_settled.aborted`는 두 로그에서 `false`다. DNS 실패 로그에는 `entry_appended`, `auto_retry_start`, `auto_retry_end`도 있다. 재시도 시작 필드는 `attempt`, `maxAttempts`, `delayMs`, `errorMessage`다.
- 마지막 `message_end.message`의 `role`은 `assistant`, `content`는 빈 배열, `stopReason`은 `error`다. `errorMessage`가 실패 원인을 담으며 `usage.input`, `output`, `cacheRead`, `cacheWrite`, `totalTokens`는 모두 0이다. 인증 실패 시 `message.model`에 시험 ID의 `:fastest` 접미사가 남았다. 게이트웨이가 요청의 접미사를 처리하는지는 인증된 실행에서 확인해야 한다.
- 저장할 때 임시 작업 디렉터리 경로를 `/tmp/pi-capture`로 바꿨다. 두 로그에 시험용 키 값이 없음을 확인했다.

### 8.2 도구 검증과 인수인계

검증 범위는 `work/pi-worker`의 운영 도구다. 완료 기준은 실제 pi 실패 로그 확보와 아래 명령의 종료 코드다. 합성 자가 시험은 정상, assistant 메시지 누락, 오류·중단, JSONL 파싱 실패, 금지 호출과 `--allow-re` 예외를 검사한다. [작업 지시와 완료 기준](handoff/prompts/pi-worker.md)

| 검증 명령 | 실제 종료 코드 |
|---|---|
| `python3 tools/pi_log_check.py --self-test` | 0 |
| `python3 tools/pi_log_check.py tools/testdata/pi_log/invalid_key.jsonl` | 1 |
| `python3 tools/pi_log_check.py tools/testdata/pi_log/dns_failure.jsonl` | 1 |
| `bash -n tools/pi_worker.sh` | 0 |
| `env -u PI_WORKER_MODEL tools/pi_worker.sh docs/handoff/prompts/pi-worker.md` | 2 |
| `python3 -c "import json;json.load(open('tools/pi_models.example.json'))"` | 0 |

대역 pi로 인자, stdin, stdout·stderr 경로, `PI_WORKER_ALLOW_RE`, 검사기 종료 코드 전달을 확인했다. 기본 설정 경로의 `pi --help`는 `trust.json.lock` 생성 중 읽기 전용 오류가 났으며 임시 `PI_CODING_AGENT_DIR`에서 도움말을 확인했다. [실행 도구 교훈](lessons/tooling.md#herdr실행)

선행 커밋은 `63ad22d`이며 미커밋 선행 변경은 없었다. 실제 로그는 `e5f9dc4`, 검사기는 `7939b8e`, 실행 스크립트와 공급자 예시는 `10041ed`에 있다. 브랜치는 검토 대기 상태다. 이 절에 운영 도구의 인수인계를 기록했다. 다음 검증은 출처 정책과 인증 환경이 결정된 뒤 §7의 0단계를 실행하는 것이다.

### 8.3 공식 주소에서 인증 확인

같은 키와 `z-ai/glm-5.3-flash-ultrafast`를 쓰고 API 주소를 바꿔 시험했다. `api.opengateway.one/oss/v1`은 HTTP 401을 반환했고, `apis.opengateway.ai/v1`은 `say hi`에 응답했다. 공식 주소의 마지막 assistant `stopReason`은 `stop`이었고 pi와 검사기 종료 코드는 각각 `0`이었다.

저장소 밖에서 실행했으며 프로젝트 컨텍스트와 도구를 비활성화했다. 저장소 작업을 맡기기 전에는 출처 정책을 결정하고, 프로젝트 컨텍스트 로드·도구 호출·Herdr 연결을 확인해야 한다.
