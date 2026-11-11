# pi 작업자 설계 (OpenGateway + GLM 5.3 Flash)

상태: **설계 초안, 미구현·미검증.** 작성 2026-11-11. 이 문서의 어떤 항목도 실제 실행으로 확인하지 않았다. §2의 사용자 결정 3건과 §7의 0단계가 끝나기 전에는 스프린트에 쓰지 않는다.

## 1. 목표와 범위

스프린트 하위 작업의 작업자를 pi 코딩 에이전트로 띄우고, 모델은 OpenGateway를 거친 GLM 5.3 Flash를 쓴다. 속도와 비용이 목적이다.

- 바꾸는 것: [위임 절차](handoff/README.md) §5의 "작업자"가 어떤 프로세스인가.
- 바꾸지 않는 것: 점유·branch·worktree·검증 계약·인수인계·검토 절차. 오케스트레이터와 검토자는 지금처럼 Claude 세션이다.
- 만들지 않는 것: pi 안에서 다시 하위 에이전트를 띄우는 확장, 전용 오케스트레이션 서비스, 큐, 대시보드. Herdr와 `tools/hrun.sh`로 충분하다.

## 2. 사용자 결정이 필요한 것

### 2.1 출처 정책 (도입을 막는 항목)

[출처 정책](00_overview.md)은 저장소를 로컬 git으로만 두고 배포·공개를 금지한다. pi 작업자는 읽은 파일을 그대로 게이트웨이에 보낸다. 포팅 작업이면 `../re/decomp/`의 디컴파일 C·asm과 `../re/notes`가 전송 대상에 들어간다.

확인한 사실은 다음과 같다.

- 게이트웨이 소개 페이지에 운영 주체, 데이터 보존·로그 정책이 없다(미확인).
- 요청은 "13개 라이브 제공자" 중 하나로 라우팅되고 장애 시 다른 제공자로 넘어간다. 어느 제공자가 받는지 호출자가 고르지 못한다.

선택지:

| 선택 | 내용 | 결과 |
|---|---|---|
| A | 원본 자료 전송 허용 | 모든 하위 작업을 맡길 수 있다. 정책 문서를 고쳐야 한다. |
| B | 원본 자료 전송 금지 (권장 기본값) | `../re/`를 읽지 않는 작업만 맡긴다. §3의 표가 이 기준이다. |
| C | 도입 보류 | 보존 정책을 확인한 뒤 다시 결정한다. |

B에서도 Rust 코드와 작업 문서는 전송된다. Rust 코드는 원본의 2차적 저작물이므로 B도 정책 예외이며 사용자 승인이 필요하다.

B는 공짜 기본값이 아니다. 교훈 기록에 남은 하위 에이전트 용도는 디컴파일 C 읽기와 asm 대조 명세이고([process](lessons/process.md), [tooling](lessons/tooling.md)), 둘 다 `../re/`가 필요하다. B에서는 이 작업을 pi에 맡길 수 없다.

### 2.2 게이트웨이와 모델 ID

- "OpenGateway AI"를 `api.opengateway.one`으로 가정했다. 다른 서비스면 §4의 주소·키 이름이 달라진다.
- 인증 없이 받은 `/models` 목록 29건에 GLM 5.3 Flash가 없다. GLM은 5.2, 5.1, 5만 있다. 이 목록은 Claude Code 호환 별칭만 보여주는 것으로 보이고, 전체 목록은 키가 있어야 확인할 수 있다.
- `:ultrafast`라는 라우팅 접미사는 페이지에 없다. 있는 것은 `:preferred`, `:fastest`, `:cheapest`다. "ultrafast"가 `:fastest`를 뜻하는지, 별도 모델 변형인지 정해야 한다.

모델 ID는 추측해서 적지 않는다. 아래에서는 `<MODEL_ID>`로 둔다.

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
  └─ tools/hrun.sh -- 'pi --mode json -p ... "<프롬프트 파일>"'   # Herdr pane
       └─ pi (작업자, work/<하위번호> worktree에서 실행)
            └─ https://api.opengateway.one/oss/v1/chat/completions → <MODEL_ID>
```

### 4.1 설치

```bash
npm install -g @earendil-works/pi-coding-agent@<고정 버전>
```

패키지 이름은 pi 문서의 import 경로에서 확인했다. 설치 명령과 최신 버전 번호는 문서에서 찾지 못했다(0.81.1 README는 존재). 버전을 고정한다. 0.73.1 예제와 현재 문서의 JSON 이벤트 이름이 다르다(`tool_result_end` → `tool_execution_end`).

### 4.2 공급자 등록

`~/.pi/agent/models.json`:

```json
{
  "$schema": "https://pi.dev/schemas/models.schema.json",
  "providers": {
    "opengateway": {
      "baseUrl": "https://api.opengateway.one/oss/v1",
      "api": "openai-completions",
      "apiKey": "$OPENGATEWAY_API_KEY",
      "authHeader": true,
      "models": [
        {
          "id": "<MODEL_ID>",
          "name": "GLM 5.3 Flash",
          "reasoning": false,
          "input": ["text"],
          "contextWindow": 0,
          "maxTokens": 0,
          "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 }
        }
      ]
    }
  }
}
```

- `0`으로 둔 값은 자리 표시다. 키로 받은 `/models` 응답과 요금표로 채운다. 2차 출처는 문맥 약 1M 토큰, 출력 최대 131,072 토큰을 말하지만 게이트웨이 경유 값은 미확인이다.
- `authHeader`의 기본값은 `false`다. `apiKey`만으로 Bearer 헤더가 붙는지는 0단계에서 확인하고 필요 없으면 지운다.
- `compat` 플래그는 넣지 않는다. pi 문서는 "OpenAI 호환"이라는 광고만 보고 켜지 말라고 한다. 0단계에서 실패한 항목만 추가한다.
- 키는 환경변수에만 둔다. 저장소와 `models.json`에 값을 쓰지 않는다.

### 4.3 실행 명령

```bash
cd <work/하위번호 worktree>
mkdir -p .pi-run
tools/hrun.sh --timeout 580000 -- 'pi --mode json -p --no-session -a \
  --provider opengateway --model "<MODEL_ID>" \
  --tools read,bash,edit,write,grep,find,ls \
  "아래 지시를 수행" < docs/handoff/prompts/<하위번호>.md \
  > .pi-run/<하위번호>.jsonl 2> .pi-run/<하위번호>.err'
```

- `hrun.sh`는 호출한 디렉터리에서 pane을 만들고 명령의 종료 코드를 돌려준다. 580초 안에 끝나지 않으면 124로 돌아오며 `--wait <pane>`으로 다시 기다린다. 이 명령 자체는 실행해 보지 않았다.
- 프롬프트는 stdin으로 넣는다. pi는 파이프로 받은 stdin을 첫 프롬프트 앞에 붙인다. 따옴표가 겹치지 않는다.
- `--provider`와 `--model`을 나눠 쓴다. pi의 `--model`은 `provider/id`와 `:<thinking>` 접미사를 해석하는데, 게이트웨이 ID가 `<org>/<Model>:policy` 꼴이라 한 인자로 쓰면 잘못 쪼개질 수 있다. 정책 접미사가 게이트웨이까지 전달되는지 0단계에서 확인한다.
- `herdr agent start`는 쓰지 않는다. 대화형 에이전트용이고 pi가 지원 종류인지 확인하지 않았다. 비대화형 실행이므로 handoff §5의 `herdr agent wait` 대신 `hrun.sh`의 대기를 쓴다.

- `-a`: print·json 모드는 프로젝트 신뢰 기본값이 "ask"이고, 이 값은 프로젝트 자원을 무시한다. `-a` 없이 `AGENTS.md`와 `.agents/skills`가 읽히는지 0단계에서 확인한다.
- pi는 `AGENTS.md`(또는 `CLAUDE.md`)를 cwd에서 상위로 올라가며 모두 읽고, 스킬은 `.agents/skills`와 `~/.agents/skills`에서 찾는다. 기존 지침과 `verified-task-branches`·`oracle-capture` 스킬을 그대로 쓴다. pi 전용 지침 파일은 만들지 않는다.
- `.pi-run/`은 ignore 대상이다. 실행 로그이고 증거가 아니다. 증거는 작업자가 worktree에 커밋한 파일이다.
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
| 실패 종료 코드 | 실행 모드의 종료 코드가 문서에 없음 | pi의 종료 코드를 완료 판정에 쓰지 않는다 |

`../re/` 읽기 금지를 프롬프트에만 맡기면 약하다. B를 고르면 작업자를 `../re/`가 보이지 않는 곳에서 실행하는 방법(컨테이너 또는 별도 사용자)을 0단계에서 정한다. 그 전에는 로그에서 `../re/` 경로나 `dc.py`가 나온 `tool_execution_start`가 0건인지 검사한다.

## 6. 완료 판정과 검토

판정은 handoff §5의 검토 절차다. pi의 출력은 판정 근거가 아니다.

1. `hrun.sh`가 돌아오면 작업자 branch의 커밋과 `docs/handoff/M*.md` 갱신을 읽는다.
2. 계약의 완료 기준 명령을 오케스트레이터가 직접 실행해 종료 코드를 확인한다.
3. 검사기 변조 검사(해시 0, 필수 사건 배열 비우기, 목표 0회 입력)를 직접 실행한다.
4. `.pi-run/<하위번호>.jsonl`에서 마지막 assistant `message_end`의 보고와 토큰 사용량을 읽는다. 보고와 2·3의 결과가 다르면 결과를 따른다.

로그에서 읽을 것: 마지막 `message_end`(role이 assistant)의 텍스트, 각 `message_end`의 `usage`, `tool_execution_end`의 `isError`, `auto_retry_start` 횟수.

## 7. 도입 단계

### 0단계: 연결과 능력 확인 (스프린트에 쓰기 전 필수)

1. 키로 `GET https://api.opengateway.one/oss/v1/models`를 호출해 `<MODEL_ID>`, 문맥 크기, 라우팅 접미사를 확인한다.
2. 도구 호출 1회 시험: `pi --mode json -p --no-session --model opengateway/<MODEL_ID> "README.md의 첫 줄을 읽어 그대로 출력"`. `tool_execution_end`가 1건 이상이고 `isError`가 false인지 본다. 게이트웨이 페이지에 도구 호출·스트리밍 지원 문구가 없다.
3. `AGENTS.md`와 `.agents/skills`가 실제로 로드되는지 확인한다(`-a` 유무 각각).
4. 실패 시 종료 코드를 관찰해 기록한다(잘못된 키, 없는 모델).
5. 재현 시험: 이미 병합된 작은 하위 작업 하나를 골라 병합 전 commit에서 같은 프롬프트로 다시 시킨다. 계약의 완료 기준 명령이 통과하는지, 금지 위반이 없는지, 걸린 시간·토큰을 기록한다. 대상은 `../re/`를 읽지 않아도 되는 작업으로 고른다. worktree는 이력을 공유하므로 작업자가 `git show`로 병합된 답을 볼 수 있다. 병합 전 commit까지만 담은 별도 clone에서 실행한다. B에서는 작업자가 `AGENTS.md`와 프롬프트 중 어느 지시를 따랐는지도 기록한다.

합격 조건: 1–4 확인, 5에서 완료 기준 통과와 금지 위반 0건. 불합격이면 §3의 표에서 pi 몫을 줄이거나 도입을 중단한다.

### 1단계: 한 영역에서 시험 사용

한 영역(M1·M2·M3 중 하나)의 하위 작업 2–3개에 쓴다. 작업마다 재작업 횟수와 검토에서 잡힌 오류를 handoff에 적는다.

### 2단계: 문서 반영

1단계 결과가 좋으면 다음을 고친다. 모두 공유 경계이므로 `claim shared`가 필요하다. 이 문서에서는 고치지 않았다.

- `docs/handoff/README.md` §5: pi 작업자 실행 명령과 §4.4의 추가 금지.
- `docs/README.md`: 이 문서 링크.
- `docs/00_overview.md` §2: 2.1의 결정.
- `.gitignore`: `.pi-run/`. 그 전까지는 `session.py export`가 이 폴더를 스냅숏에 담는다.
- `docs/handoff/prompts/`: 새 경로다. 위치를 handoff에 적는다.

## 8. 확인한 것과 못 한 것

| 항목 | 상태 | 출처 |
|---|---|---|
| 게이트웨이 주소 `https://api.opengateway.one/oss/v1`, OpenAI 호환 chat completions, `Authorization: Bearer $OPENGATEWAY_API_KEY` | 페이지에서 확인 | https://opengateway-web.pages.dev/ |
| 모델 ID 형식 `<org>/<Model>[:policy]`, 정책 `:preferred`·`:fastest`·`:cheapest` | 페이지에서 확인 | 위와 같음 |
| GLM 5.3 Flash의 게이트웨이 모델 ID, `:ultrafast` | **미확인** (공개 목록·페이지에 없음) | `GET /oss/v1/models` 무인증 응답 29건 |
| 도구 호출·스트리밍 지원, rate limit, 요금, 보존 정책, 운영 주체 | **미확인** | 페이지에 문구 없음 |
| GLM 5.3 Flash 문맥 약 1M 토큰 | 2차 출처만 | https://pricepertoken.com/pricing-page/model/z-ai-glm-5.3-flash , https://aimlapi.com/models/glm-5-3-flash |
| GLM 5.3 Flash 요금 | **미확인** (검색 요약에 서로 다른 값, 원문 미열람) | 없음 |
| §4.3 실행 명령, Herdr의 pi 지원 여부 | **미확인** (실행하지 않음) | `tools/hrun.sh` 읽기만 함 |
| `models.json` 경로·필드, `apiKey`의 `$NAME` 해석, `compat` 주의 | 공식 문서에서 확인 | https://pi.dev/docs/latest/models-configuration , https://pi.dev/docs/latest/custom-provider |
| `-p`, `--mode json`, `--no-session`, `--tools`, `--model provider/id`, JSON 이벤트 이름 | 공식 문서에서 확인 | https://pi.dev/docs/latest/cli.md , https://pi.dev/docs/latest/json.md |
| 컨텍스트 파일·스킬 탐색 경로, 권한 확인 없음, 프로젝트 신뢰 `-a` | README 0.81.1에서 확인 | https://cdn.jsdelivr.net/npm/@earendil-works/pi-coding-agent@0.81.1/README.md |
| 재시도·압축 설정 기본값 | 공식 문서에서 확인 | https://pi.dev/docs/latest/settings.md |
| 실행 모드의 실패 종료 코드 | **미확인** (문서에 없음) | 위 cli.md |
| pi 설치, 키 발급, 실제 호출 | **실행하지 않음** | 이 컴퓨터에 pi와 키가 없다 |
