# pi-worker: pi 작업자 실행 도구 구현

설계는 `docs/16_pi_worker_design.md`다. 먼저 읽는다. 이 작업은 솔버 코드가 아니라 운영 도구다. `crates/`, `python/`은 건드리지 않는다.

## 목표

1. `tools/pi_log_check.py <log.jsonl> [--allow-re]`: pi의 `--mode json` 출력(JSONL)을 읽는 검사기. 표준 라이브러리만 쓴다.
   - 출력: 마지막 assistant `message_end`의 텍스트, 토큰 사용량 합계, 도구 호출 수, 도구 오류 수, 자동 재시도 수.
   - 종료 1: assistant `message_end`가 없음, 마지막 assistant 메시지가 오류·중단으로 끝남, 파싱할 수 없는 줄이 있음.
   - 종료 1: 금지 도구 호출이 있음. 금지는 Gurobi 실행(`gurobi`, `gurun.sh`, `grbgetkey`가 명령에 있음)과, `--allow-re`가 없을 때 원본 자료 접근(도구 인자에 `re/decomp`, `re/notes`, `../re/`, `dc.py`가 있음)이다. 위반한 호출을 한 줄씩 출력한다.
   - 종료 0: 그 밖의 경우.
   - `--self-test`: 합성 로그로 위 규칙을 `assert`로 확인한다. 합격 1건, 각 거부 사유마다 1건 이상.
   - 이벤트·필드 이름은 추측하지 않는다. 설치된 pi 1.5.2가 실제로 내는 JSONL을 캡처해 `tools/testdata/pi_log/`에 두고 그 형식에 맞춘다(아래 "실제 로그" 참조).
2. `tools/pi_worker.sh <프롬프트 파일> [이름]`: 현재 디렉터리에서 pi 작업자를 한 번 실행한다.
   - 모델은 환경변수 `PI_WORKER_MODEL`(필수), 공급자는 `PI_WORKER_PROVIDER`(기본 `opengateway`). 없으면 사용법을 출력하고 종료 2.
   - `mkdir -p .pi-run`, 프롬프트는 stdin으로 넣는다. 로그는 `.pi-run/<이름>.jsonl`, stderr는 `.pi-run/<이름>.err`.
   - pi 인자: `--mode json -p --no-session -a --provider … --model … --tools read,bash,edit,write,grep,find,ls`. `pi --help`로 각 플래그를 확인한다.
   - pi가 끝나면 `tools/pi_log_check.py`를 실행하고 그 종료 코드로 끝낸다. `PI_WORKER_ALLOW_RE=1`이면 `--allow-re`를 넘긴다.
   - Herdr는 이 스크립트 안에서 다루지 않는다. 호출자가 `tools/hrun.sh -- 'tools/pi_worker.sh …'`로 감싼다.
3. `tools/pi_models.example.json`: 설계 §4.2의 공급자 등록 예시. 모델 ID·문맥·요금은 자리 표시로 둔다. 키 값을 쓰지 않는다.
4. `.gitignore`에 `.pi-run/` 추가.
5. `docs/16_pi_worker_design.md` 갱신: §4.3을 `tools/pi_worker.sh` 사용법으로 바꾸고, §8에 이번에 실제로 확인한 사실(플래그, 이벤트 이름, 키 없이 실행했을 때의 종료 코드)과 여전히 미확인인 항목을 적는다. Herdr `agent start`가 kind `pi`를 지원한다는 사실도 반영한다(`herdr agent` 출력에서 확인됨).

## 실제 로그

OpenGateway 키는 없다. 다음만 한다.

- `PI_CODING_AGENT_DIR=<저장소 밖 임시 디렉터리>`에 `models.json`을 두고 실행한다. `~/.pi`를 만들거나 고치지 않는다.
- 잘못된 키로 한 번 실행해 실패 로그와 종료 코드를 얻는다. 모델 ID는 `zai-org/GLM-5.2:fastest`를 쓴다. 프롬프트는 "say hi" 한 줄이다. 저장소 파일 내용을 프롬프트나 `@파일`로 보내지 않는다.
- 캡처한 실패 로그에서 키처럼 보이는 값을 지우고 `tools/testdata/pi_log/`에 둔다.
- 성공 로그의 형식은 설치된 패키지의 문서(`$(npm root -g)/@earendil-works/pi-coding-agent/docs/json.md`)에서 확인한다. 실제 성공 실행은 하지 않는다.

## 완료 기준 (명령과 기대 종료 코드)

```bash
python3 tools/pi_log_check.py --self-test                      # 0
python3 tools/pi_log_check.py tools/testdata/pi_log/<실패 로그> # 1
bash -n tools/pi_worker.sh                                      # 0
env -u PI_WORKER_MODEL tools/pi_worker.sh docs/handoff/prompts/pi-worker.md   # 2
python3 -c "import json;json.load(open('tools/pi_models.example.json'))"      # 0
git status --short                                              # 출력 없음 (모두 커밋)
```

합성 로그만 있고 실제 pi가 낸 로그가 한 건도 없으면 완료가 아니다.

## 도달 불가 시 산출물

pi가 키 없이 로그를 한 줄도 내지 않거나 네트워크가 막혀 있으면, 시도한 명령·출력·종료 코드를 `docs/16_pi_worker_design.md` §8에 적고 검사기는 문서의 형식만으로 만든 뒤 "실제 로그 미확인"이라고 적는다. 완료라고 보고하지 않는다.

## 금지

- main worktree(`../rsolver`) 수정, `git push`, remote 변경, 병합.
- 공유 `.venv` 설치, `cargo` 빌드, Gurobi 실행, `tools/session.py claim/release`.
- `../re/` 읽기, `tools/dc.py` 실행. 이 작업에는 필요 없다.
- npm 패키지 추가 설치, `~/.pi`·`~/.codex`·`~/.claude` 수정.
- 다른 에이전트 실행, Herdr pane·agent 조작.
- `docs/handoff/README.md`, `docs/README.md`, `docs/00_overview.md`, `AGENTS.md` 수정(2단계 몫이다).
- 실패에 맞춰 검사기의 합격 조건을 넓히는 것.

## 진행

단계마다 `work/pi-worker`에 커밋한다: 실제 로그 캡처, 검사기, 실행 스크립트, 문서. 파일을 스크립트로 치환할 때는 대상 문자열이 있는지 `assert`로 확인한다.

## 보고 형식

마지막 메시지에 다음을 적는다: 커밋 목록, 완료 기준 각 명령의 실제 종료 코드, 실제 로그에서 확인한 이벤트·필드 이름, 키 없이 실행했을 때 pi의 종료 코드, 확인하지 못한 것.

모든 출력·문서는 한국어, 코드 식별자는 유지.
