# M3-04c-12 독립 검토

- 검토자: 별도 에이전트 `/root/m3_04c12_review`(주작업자와 독립, 읽기 전용).
- 대상: main `ec76fd6` 대비 `work/m3-04c-12`의 조사 문서·스프린트/설계·집계 도구·자료·검증 기록. 계약 `41b6476`, 도구 첫 기록 `035856c` 이후 최종 작업 diff를 검토했다.
- 판정: **통과, 병합을 막는 문제 없음**. 조사 완료와 구현 동등성 미검증을 구분한 범위가 적절하다. 원본 새 실행 0회.

## 독립 확인

- `134a87` C1725–2418 및 `25615c@0x1c5310` 전체를 읽고 필요한 asm과 대조했다. 지원 집합의 factor 생략, raw count 누적·dense 압축, failure>=3 보류, 기존 요청·주기 동률·비용식, 반환1 반복과 batch 추가 호출, 재분해 뒤 재계산 및 round 순서를 확인했다.
- `python3 tools/push_loop_survey.py`를 재실행하고 저장 `survey.json`과 바이트 동일함을 확인했다. 대표18기록·공급236·고갈38·보류18·factor73(요청0=26, 10=47, 반환 모두0)·header변경33.
- 기존 `chain.check`에 바이너리 해시0·모델 해시0·사건 비움·step/ratio/retry 제거로 만든 목표0회 입력을 각각 직접 주입해 모두 거부되는지 확인했다. 원본 자료는 변경하지 않았다.
- `python3 tools/push_chain_oracle.py --check`, `--self-test`(8종) 종료0.
- `python3 tools/push_step_chain_rust.py --compare` 독립 재실행: 236단계·5,144필드 비트 일치, 재분해 뒤 비교 제외6 유지.
- `verification.json`의 도구·survey 해시와 실제 파일을 대조하고 `git diff --check`를 통과했다.

## 반영한 보정

- 집계의 `last_supply`를 `push:enter`에서 초기화했다. 한 기록에 여러 push가 생기면 뒤 push의 초기 factor를 앞 push 후처리로 잘못 분류할 수 있었다. 현재18기록은 각각 push1회라 집계값은 변하지 않았다. 도구 해시와 survey를 갱신했다.
- 공급 반환 sparse만으로 내부 FTRAN dense 미실행을 주장하지 않도록 raw 표현 미수집을 명시했다. 공급자 압축 전 음수 count 누적, support=-1 정규화, failure skip와 caller 실패 증가의 차이도 문서에 남겼다.
- 공급자 C 범위를573–686, 요청0 기록을 C1996으로 보정했다.

## 한계

신규 동작 오라클·Rust 구현·수치 backend·오류/batch/x87 동등성을 검토 통과로 승격하지 않는다. 인계·병합·worktree 정리·점유 해제는 주작업자가 수행하고 main 이력·세션 상태로 최종 확인한다.
