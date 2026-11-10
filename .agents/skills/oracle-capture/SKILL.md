---
name: oracle-capture
description: Gurobi 13.0.3 원본을 Frida로 훅해 함수 경계의 상태·사건을 캡처하고, manifest·3회 일치·변조 self-test를 갖춘 검사기로 고정하는 오라클 작업 절차. 새 원본 캡처 경계, 훅 추가, 캡처 검사기 작성, bench/oracle 기준 자료 생성에 사용한다.
---

# 원본 캡처 오라클

캡처 하나는 "원본 바이너리·입력·파라미터·도구가 고정된 상태에서 같은 사건이 3회 나온다"는 증거다. 검사기는 그 증거를 다시 확인하고, 일부러 깨뜨린 자료를 거부해야 한다. 규칙의 근거는 `docs/15_sprint_verification.md` §5–§11, 실패 사례는 `docs/lessons/tooling.md`(Frida 훅·검사기·Herdr)와 `process.md`에 있다.

## 참고 구현

새로 짜기 전에 가장 가까운 것을 복사해 고친다.

| 형태 | 파일 |
|---|---|
| 노드 사건 + 이전 캡처 투영 | `tools/mip_retry_basis.js`, `_runs.sh`, `_check.py`(`controls`/`acceptance`/`tampers`/`--write-manifest`) |
| 상태 배열 + bit 단위 값 | `tools/primal_push_step_capture.js`(실행·해시), `_agent.js`(훅), `_check.py`(`--control`, `--mutation`) |
| 기존 검사기 재사용 투영기 | `tools/mip_retry_rust_expect.py`(`load`·`first`·`check_manifest` import) |

## 순서

각 단계 끝에 작업 branch에 커밋하고 `docs/handoff/M*.md`를 갱신한다.

1. **계약 먼저.** 첫 캡처 전에 `docs/templates/sprint_acceptance.md`로 경계·사례·판정 규칙을 커밋한다. "원본에서 사건이 생기지 않음"도 결론이 될 수 있고, 합성 결과는 참고용이라고 적는다.
2. **도달성과 진입점 확인.** `tools/dc.py <이름|0x주소>`로 C·호출관계·기존 노트를 보고, `tools/dc.py <키> --asm`으로 훅 오프셋을 확인한다. 진입점은 보통 `push %r12`로 시작한다. 함수 중간 훅은 명령 경계, 그 지점으로 오는 jmp 전부, 레지스터 쓰기를 확인한 뒤에만 붙인다. 정상 산술로 불가능한 사건은 계속 찾지 말고 사용자에게 합성 경계로 바꿀지 묻는다.
3. **탐색 캡처 1회.** 필드표를 고정하기 전에 어떤 훅이 실행되는지만 센다.
4. **훅 작성 규칙**
   - 주소 하나에는 `Interceptor.attach` 하나만 붙인다.
   - heap 포인터를 키로 쓰지 않는다. 쓸 때는 내용 해시가 같을 때만 인정한다.
   - Node Frida를 쓴다. venv Python에는 frida가 없다.
   - 캡처기는 바이너리·모델·도구 sha256을 기록하고 기존 출력 덮어쓰기를 거부한다.
   - 궤적 비교에는 TimeLimit 대신 NodeLimit을 쓴다.
5. **실행.** Gurobi는 한 프로세스만 쓴다. 그래서 `tools/gurun.sh`를 거쳐, 독립 프로세스로 3회 돌린다. 한 회에 약 2분이 걸리므로 `tools/hrun.sh --timeout 580000 -- 'tools/<x>_runs.sh <OUT>'`로 돌리고 Bash 도구 timeout은 600000으로 준다. 그 안에 안 끝나 종료 코드 124가 나오면 `tools/hrun.sh --wait <pane>`으로 이어서 기다린다. 멈출 때는 자신이 띄운 PID를 kill한다.
6. **검사기**
   - `controls`: manifest 해시, 3회 사건 일치, 입력·파라미터·출력 줄. 훅을 추가했으면 새 캡처를 이전 캡처 필드로 투영해 JSON 전체가 같은지 비교한다(비간섭 증거).
   - `acceptance`: 결과를 보기 전에 고정한 판정 규칙. 양쪽이 모두 null이면 실패다.
   - `--self-test`: 메모리를 변조해 각 검사가 거부하는지 확인한다. 바이너리 해시, 빈 사건, 투영 차이, 회차 차이, null-null, 대상 필드 각각을 변조한다. `NOT REJECTED`가 하나라도 있으면 실패다.
   - 실패 출력에는 첫 차이(사례·사건·필드·기준값·실제값)를 적는다. 종료 코드 0만으로 통과라고 보지 않는다.
7. **기준 자료.** `bench/oracle/<작업>/`에 캡처, `--write-manifest`로 만든 manifest, 저장소 밖 모델의 해시를 둔다.
8. **기록.** 캡처의 10진수 주소는 `tools/dc.py --dec <값>`으로 변환해 인용한다. 문장마다 근거가 C인지 캡처인지 구분한다. 완료 선언 전에 독립 검토(advisor 또는 다른 모델)를 받는다.

## 완료 판정

오라클 구축 작업은 입력·캡처 경계·3회 재현·검사기 오류 검출·재실행을 확인한 범위까지만 완료다(`15_sprint_verification.md` §11). Rust 구현 일치는 별도 작업이다.
