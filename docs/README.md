# 문서 안내

새 세션·동시 세션·컴퓨터 이동은 [세션 시작과 이동](handoff/README.md)에서 시작한다. 영역별 인수인계에는 다음 하위 작업과 실제 검증 상태가 있다.

브랜치 생성·검증·병합과 선행 변경 정리는 [브랜치 운영 스킬](../.agents/skills/verified-task-branches/SKILL.md)을 따른다. 기존 dirty main을 정리한 이력과 검증은 [기준선 정리 기록](m2/baseline_reconciliation.md)에 있다.

현재 상태와 남은 한계는 [진행 기록](11_progress.md)과 [로드맵 다이어그램](12_roadmap_diagram.md)부터 확인한다. 포팅 절차와 패리티 기준은 [포팅 가이드](10_porting.md), 설계와 출처 정책은 [전체 설계 개요](00_overview.md)에 있다.

M1·M2·M3의 번호별 범위와 완료 확인 방법은 [작업 단위 목록](13_work_units.md)에서 찾는다.

실행할 작업은 [1주 스프린트 작업 목록](14_sprint_plan.md)의 하위 번호로 선택한다. 상태·데이터 소유권과 비교 경계는 [M1 설계 보완](m1/sprint_design.md), [M2 설계 보완](m2/sprint_design.md), [M3 설계 보완](m3/sprint_design.md)에 있다.

모든 스프린트의 착수 계약과 완료 판정은 [동작 동등성 검증 기준](15_sprint_verification.md)을 따른다. 작업 기록은 [검증 계약 양식](templates/sprint_acceptance.md)으로 작성한다.

- `01`–`08`: 모델·API, 드라이버, presolve, simplex, barrier·PDHG, B&B, cut·휴리스틱, 일반 제약의 영역별 설계.
- [09 로드맵](09_roadmap.md): 마일스톤 계획. 현재 상태는 진행 기록과 다이어그램을 기준으로 한다.
- [port_map.tsv](port_map.tsv): 원본 함수와 모듈의 대응표. `tools/port_map.py`가 생성한다.
- [m1/](m1/): LP simplex, presolve·postsolve와 드라이버 작업 기록.
- [m2/](m2/): MIP presolve, 루트·B&B, 해 복원과 측정 기록. 파라미터 오프셋 표도 여기에 있다.
- [m3/](m3/): barrier, ordering, Cholesky와 PDHG 작업 기록.

영역별 문서는 원본 근거, 오라클 사용법, 검증 범위와 남은 한계를 기록한다. 새 상세 기록은 해당 영역 폴더에 둔다.
