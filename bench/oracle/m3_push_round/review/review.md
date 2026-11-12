# M3-04c-19 독립 검토

검토자 `/root/m3_04c19_review`. 최초 검토 기준 `58368f8`, 작업자와 별도로 C/asm·훅·합성 ABI·검사기를 읽고 직접 실행했다. Rust 재빌드/소스 변이는 수행하지 않았다. 최초에는 BTRAN caller 연결 누락3건으로 보류했으며, 작업자의 보강 뒤 최종 판정은 **통과**다.

## 원본·재현

- `tools/dc.py 134a87`, C1143–1176/1999–2062/2226–2340과 전체 asm의 호출·분기·읽기/쓰기·출구를 대조했다. 전체4036줄 asm을 바이너리에서 다시 생성했다. 저장본과 ret 한 줄 말미공백만 달랐고 말미공백 정리 뒤 전체 일치했다(`source.asm`).
- `python3 tools/push_round_check.py --controls --acceptance --self-test --report bench/oracle/m3_push_round/review/initial-checks.json`: 종료0, 자연751사건·합성106사례·변조154종 거부.
- 작업자 Gurobi 실행 종료를 확인한 뒤 아래 두 명령을 순차 실행했다. 원본 신규 실행2회다.

```bash
tools/gurun.sh node tools/push_round_capture.js primal bench/oracle/m3_push_retry/retry.lp bench/oracle/m3_push_round/review/retry.json "$(cat bench/oracle/m3_push_retry/retry.extra.json)"
tools/gurun.sh ../.venv/bin/python tools/push_round_synthetic.py --capture bench/oracle/m3_push_round/review/synthetic.json
```

- retry161사건·합성106사례가 정식 `*-1.json`과 pid/run_id를 제외한 **전체 필드 동일**했다. `recapture-comparison.json`에 결과를 저장했다.
- 합성은 원본 명령을 그대로 실행하고 종료 stub과 BTRAN/factor/context backend만 대체한다. pass 종료 stub이 total/cursor/deferred를 저장하므로 이 저장 자체는 원본 동등성 증거가 아니다. 원본 round/dense 압축에서는 실제 저장을 실행한다. 첫 pass support0의 live edx 누락 실패와 수정 이력은 보존되어 있다.

## 자연 범위

자연 roundctl 사건207건(arms127/retry49/interior31), 승인36건(22/5/9), 보류6건(arms/retry 각3), 고갈9회, round factor7회·재계산7회·종료3회다. 두 모델의 보류 실패는 각각1→2→3으로 진행한다. BTRAN22회(17/4/1)는 **전부 희소 count=1**이며 count0/다수/dense는 합성 제어 경계다. 자연과 합성의 실행 증거를 수치 backend의 동등성·전체 루프 구현으로 승격할 수 없다.

## 직접 변조

`python3 bench/oracle/m3_push_round/review/tampers.py` 결과 `tampers-initial.json`:

- 바이너리0·빈사건·round 목표0·요청10 변경·승인수 변경·보류 실패 증가 제거는 거부했다.
- 첫 BTRAN prepare.selected.leave와 enter.selected.leave/unit row를 함께 다른 행으로 바꾼 동일3회 자료가 controls+acceptance에 생존했다.
- 첫 BTRAN prepare/applied의 request를 함께6으로 바꾼 동일3회 자료가 생존했다.
- 두 번째 BTRAN prepare의 기존 표시를 모두0으로 지우고 applied를 raw indices만 남긴 동일3회 자료가 생존했다.

prepare를 직전 accepted/step:applied의 selected·P·request와 연결하고, applied를 다음 factorctl:select/공급자 문맥과 연결해야 한다. 작업자에게 이3건을 병합 전 보강 사항으로 전달했다. 캡처는 올바르며 재캡처가 필요한 지적은 아니다.

## 최종 재검토

작업자는 prepare를 직전 applied의 step·selected·전체 round 상태에 연결하고, 매 공급자 진입 P를 직전 출력 P와 대조하며, sparse applied를 factorctl:select의 요청·상태·지원·시도/실패 게이트와 연결했다. 원본 agent·합성 도구·캡처 내용은 바꾸지 않았다. 초기 생존3건은 검사기 self-test에도 추가됐다.

- 수정 diff를 다시 읽고 `python3 tools/push_round_check.py --controls --acceptance --self-test --report bench/oracle/m3_push_round/review/final-checks.json`을 직접 실행했다. 종료0, 변조157종 모두 거부.
- 같은9개 직접 변조를 다시 실행해 모두 거부됨을 확인했다(`tampers-final.json`). 이전에 생존한 caller leave/request/기존marks3건은 prepare의 직전 applied 연결 검사에서 첫 차이를 출력했다. 초기 생존 기록 `tampers-initial.json`은 보존한다.
- 독립 재캡처 retry161사건·합성106사례는 수정 전후 도구/캡처가 같으므로 그대로 유효하다. 원본 재실행은2회이며 추가 실행은 하지 않았다.
- 검토자는 제품/도구/main/공유venv를 수정하지 않았고 review 디렉터리에만 자료를 썼다. 현재 원본 오라클의 계약 범위에서 병합을 막는 문제는 없다. Rust L1(20), 실제 LU(21), 연결(22), batch(23), x87·전체work/P3 동등성은 이번 검토의 주장 밖이다.

저장하는 독립 asm의 ret 말미공백은 저장소 diff 규칙에 따라 정리했다. 명령과 전체4036줄 비교 결과는 그대로다.

최종 검사기 SHA256 `215c48a35f75d6f04702908dd1844c872850c2bb869f1f6dd2db0777e93c0f70`.
