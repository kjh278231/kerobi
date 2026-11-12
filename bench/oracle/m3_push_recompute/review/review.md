# M3-04c-17 독립 검토

- 검토자: `/root/m3_04c17_review`. 작업자와 별도 세션에서 계약·C·실제 바이너리 asm·agent·합성 호출 틀·검사기를 읽고 아래 검사를 직접 실행했다. 검토 대상 작업 기준 commit은 `9f69dbf`이며 후속 source.asm 보완도 확인했다.
- 판정: **계약 범위 통과, 병합을 막는 문제 없음**. Rust 신규 구현 일치는 이번 작업의 주장이 아니다.

## 원본·ABI 검토

- `134a87@0x1c0070`의 두 구간 `0x1c3872`와 `0x1c3db4`가 새 F+0x18 header를 다시 읽고, rhs 복사·count=-1 설정 후 `09cf9b@0x1205d0`을 호출함을 C와 asm으로 확인했다. 복귀 뒤 raw→basic 복사, basis 순서 shift·폭·분류·PInf 합산, 위반 수 설치와 retry 조건이 검사기와 일치한다.
- pivot 정상 출구 `0x1c3aed`와 round 정상 출구 `0x1c400a`, 두 retry arm `0x1c452b`·`0x1c48b1` 및 retry 쓰기 뒤 `0x1c08d5`가 명령 경계임을 확인했다. round의 r12/r13 초기0은 `0x1c3bd1`·`0x1c3bdc`에서 확인했다. 두 구간의 m=0 경로는 FTRAN을 호출하고 분류 루프만 생략한다.
- 합성 trampoline은 SysV callee-saved 레지스터와 실제 스택을 복원하고 원본 호출 시 16바이트 정렬 frame을 사용한다. round가 `(%rsp)`에 PInf를 저장하므로 출구는 ret 대신 별도 cleanup으로 점프한다. 합성은 FTRAN raw 공급과 로그 helper만 대체하며 실제 LU 수치 동등성의 근거가 아니다.
- Frida의 중간 probe는 함수형 attach이고 이미 있는 `0x1c3aed` probe에 종료 관측을 통합해 중복 attach를 피한다. 자연 계측은 메모리를 읽기만 하며 새 사건을 뺀 투영을 선행15 전체 사건과 비교한다.
- 초기 source.asm이 dc.py 기본 200줄만 담은 문제를 작업자에게 알렸다. 작업자는 `--max 10000`으로 전체4036줄을 보존하고 manifest를 갱신했다. 검토자가 같은 명령을 직접 실행해 말미 공백 정규화 후 파일 전체가 같은지 확인했다.

## 실행 증거

저장소 루트에서 다음 명령을 실행했다.

```bash
python3 tools/push_recompute_check.py --controls --acceptance --self-test --report bench/oracle/m3_push_recompute/review/standard-check.json
tools/gurun.sh node tools/push_recompute_capture.js primal bench/oracle/m3_primal_push_step/interior.lp bench/oracle/m3_push_recompute/review/interior-review-1.json '{}'
tools/gurun.sh ../.venv/bin/python tools/push_recompute_synthetic.py --capture bench/oracle/m3_push_recompute/review/synthetic-review-1.json
python3 bench/oracle/m3_push_recompute/review/review_checks.py
```

- 표준 검사: controls·acceptance·self-test 모두 종료0. 자연 재계산8회(pivot1/round7)·119행, 합성120사례·152행을 재생했고 표준 변조82종을 거부했다.
- 새 원본 실행은 단일 프로세스로 순차2회다. 자연 interior149사건과 합성120사례 각각 저장 기준의 events 전체·환경·원본/도구 식별값과 일치했다. 자연 최종 출력·상태/basis도 일치했다. 독립 PID와 run_id를 확인했다. 캡처 경로·SHA-256은 `review.json`에 고정했다.
- 검토자 추가 오류 주입15종을 전부 거부했다. 바이너리·모델·manifest 모델 해시0, 빈 사건, 자연 목표0회, 필수 rhs 삭제, rhs null-null, basic 수치, PInf의 +0→-0, FTRAN dense count, raw/basic 별칭, FTRAN 사건 순서, 새 header 순서, h 경계의 L/N 및 PInf=1e10 retry 변조다. 원본 자료를 덮어쓰지 않고 메모리 복사본에 주입했다. 첫 차이와 재실행 코드는 `review.json` 및 `review_checks.py`에 있다.

## 한계

자연 자료는 세 소형 모델의 지정 설정이고 자연 pivot 뒤 재계산은 interior1회다. 자연 별칭은 둘 다 false이며 true·free/fixed·정확 동률·retry·m=0은 합성 증거다. 입력 재생에 raw FTRAN을 사용하므로 실제 Rust LU, 전체 push 연결·batch·x87·work·시간은 검증하지 않았다. 초기 합성 복사 길이 누락은 실패 이력으로 보존하고 수정 후 독립3회 및 이번 독립 재캡처를 통과했음을 확인했다.
