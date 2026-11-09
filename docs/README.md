# 문서 안내

현재 상태와 남은 한계는 [진행 기록](11_progress.md)과 [로드맵 다이어그램](12_roadmap_diagram.md)부터 확인한다. 포팅 절차와 패리티 기준은 [포팅 가이드](10_porting.md), 설계와 출처 정책은 [전체 설계 개요](00_overview.md)에 있다.

- `01`–`08`: 모델·API, 드라이버, presolve, simplex, barrier·PDHG, B&B, cut·휴리스틱, 일반 제약의 영역별 설계.
- [09 로드맵](09_roadmap.md): 마일스톤 계획. 현재 상태는 진행 기록과 다이어그램을 기준으로 한다.
- [port_map.tsv](port_map.tsv): 원본 함수와 모듈의 대응표. `tools/port_map.py`가 생성한다.
- [m1/](m1/): LP simplex, presolve·postsolve와 드라이버 작업 기록.
- [m2/](m2/): MIP presolve, 루트·B&B, 해 복원과 측정 기록. 파라미터 오프셋 표도 여기에 있다.
- [m3/](m3/): barrier, ordering, Cholesky와 PDHG 작업 기록.

영역별 문서는 원본 근거, 오라클 사용법, 검증 범위와 남은 한계를 기록한다. 새 상세 기록은 해당 영역 폴더에 둔다.
