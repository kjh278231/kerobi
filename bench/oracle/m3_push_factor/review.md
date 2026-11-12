# M3-04c-15 독립 검토

검토자: `/root/m3_04c15_review`. 최초 검토 기준 구현 commit `4501da9`, 캡처 commit `755d74a`, 당시 미커밋 `push_factor_check.py`. main·공유 venv·원격은 변경하지 않았다.

## 검토 범위와 실행

- 사전 계약과 원본 C `001c0070…134a87.c`1725–1997, asm `0x1c336d–0x1c3465`, `0x1c37ee–0x1c38e3`, `0x1c3aed–0x1c3b79`, `0x1c44ad–0x1c459b`를 직접 확인했다.
- 원본 선택 slice는 원본 코드의 정수합 뒤 cvtsi2sd·minsd·요청 우선·k==limit 비교를 실행한다. wrapper slice는 factor/quality/문맥 helper 대체가 명시됐고, 재계산 직전과 cleanup 전 정해진 출구로 끝난다. trampoline은 원래 스택과 SysV callee-saved 레지스터를 보존하며 fake frame의 배열 포인터를 실제 solver 객체로 주장하지 않는다.
- `python3 tools/push_factor_check.py --controls --acceptance --self-test` 종료0. 최초 자료는 자연 판단28·호출6·skip22, factor16·복원6, 합성22선택+29wrapper=51사례, 변조34종 거부다.
- `tools/gurun.sh node tools/push_factor_capture.js primal bench/oracle/m3_primal_push_step/interior.lp bench/oracle/m3_push_factor/review-interior.json`와 `tools/gurun.sh ../.venv/bin/python tools/push_factor_synthetic.py --capture bench/oracle/m3_push_factor/review-synthetic.json`를 **직렬** 실행했다. 각각137사건·합성51사례가 기준 `events`와 완전 일치하고 자연 최종 결과도 일치했다.
- 추가 직접변조는 `review-extra-tampers.json`에 보존한다. 바이너리0·합성 입력해시0·빈사건·목표0회·품질 gate 전체 제거는 거부했다.

## 최초 검토에서 보완 요청

1. 자연 `factorctl:restored.state.status=-1000`과 `factorctl:call.frame.attempt=99`가 acceptance에서 생존했다. 캡처한 자연 후상태/frame을 전후 사건과 연결하는 검사가 필요하다. 모든3회가 같다는 대조군 검사와 상태 전이 검사는 별개다.
2. 계약은 벡터 내용 보존을 쓰지만 당시 캡처/합성은 pointer boolean만 검사했다. 내용 주장에 맞는 sentinel 내용·hash를 보강하거나 한계를 명시해야 한다.
3. 합성 quality callback이 enter 인자를 검사하지 않았다. caller의 `S+0x2fc` 전달을 주장하려면 비영 sentinel과 호출 인자를 비교해야 한다.
4. `min-q-term` 한 사례는 `5m²`를 `4m²`로 바꾼 수식도 같은 요청을 내므로, q항이 min인 비용의 음/0/양 경계를 보강하도록 권고했다. `sum-convert`도 원래 큰 양수 L/U로는 min의 다른 항 때문에 합산 순서가 선택 결과를 바꾸지 않는다.

## 보강 후 최종 재검토

최종 판정: **통과, 계약 범위의 병합 차단 사항 없음**. 원본 수치 helper·실제 LU·전체 push의 동등성 판정은 아니다.

- 코드 변경을 직접 재검토했다. 자연 select→call의 state/frame, push attempt와 numeric+10, factor leave→restored→quality gate→clear의 상태 연결을 검사한다. 초기 검토에서 생존한 status -1000·attempt99 변조가 이제 각각 첫 다른 필드에서 거부된다.
- 합성은 선택24+wrapper31=**55사례**로 보강됐다. q항이 min인 C19/20/21 비용 경계, 정수합 후 변환을 구별하는 상쇄 L/U, 반환1 이후 request6 보존과 backend numeric 임계값 변경이 포함된다. 음수 U의 상쇄 사례는 실제 nnz 카운터로 도달 가능한 모델 상태가 아니라 명시적 signed 기계 입력이다.
- 합성 wrapper에서 oldalpha/current alpha/spike 세 버퍼에 서로 다른24바이트 sentinel을 넣고 내용 보존을 비교한다. quality callback이 enter7을 검사하고 event에도 기록한다. 자연 factor의 수치 backend는 벡터 내용을 바꿀 수 있으므로 자연 자료의 내용 동등성까지 확장하지 않는다.
- `python3 tools/push_factor_check.py --controls --acceptance --self-test`를 다시 실행해 종료0을 확인했다. 자연271/104/137사건·대조0·합성55사례 독립3회, 자연 판단28·호출6·skip22·복원6, self-test41종 전부 거부다.
- 최종 코드에서 `review-interior.json`과 `review-synthetic.json`을 새로 직렬 재캡처했다. 자연137사건·최종 결과, 합성55사례의 events가 정식 자료와 완전 일치했다. 이 검토의 원본 실행은 초기2회+최종2회=4회이며 마지막 Gurobi 프로세스도 종료했다.
- 별도 직접변조9종(binary0/input0/empty/target0/restored-status/quality-gate누락/call-attempt/sentinel내용/quality-enter)을 재실행해 모두 거부했다. 결과는 `review-extra-tampers.json`에 기록했다. 초기 생존2종이 들어 있는 원자료와 초기 도구/캡처는 `history/initial-capture.tar.gz`에 보존됐다.
- slice 시작·출구 stub은 원본 판단/호출/복원 명령을 대체하지 않는다. 0x1c3872에서 중단하므로 재계산 본문은 실행 증거에 포함하지 않으며, factor·품질·문맥 helper 대체는 synthetic 결과에 명시돼 있다. 숫자 비교는 유한 입력만 대상으로 하고 NaN/Inf/overflow는 제외한다.

품질 helper 수치15a·실제 LU21·재계산 본문17·batch23·x87·전체 push22 동등성은 미검증이다. 이 한계를 작업 기록·스프린트 목록·인수인계에 유지해야 한다. main·공유 venv·원격·작업자 코드에는 검토자가 변경을 가하지 않았다.
