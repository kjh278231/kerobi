# 12. 로드맵 다이어그램

상태: ✅ 완료 · 🔄 진행 · 🔎 에이전트 검증 완료·병합 대기 · ⏳ 예정 (2026-11-10 기준. main에 병합된 구현과 남은 작업을 표시한다. 세부 기록은 [11_progress.md](11_progress.md), 심플렉스 최신 검증은 [10_porting.md §9.2](10_porting.md))

M1·M2·M3의 세부 작업 번호, 상태와 확인 방법은 [작업 단위 목록](13_work_units.md)에 있다. G1~G6은 기존 검증 관문이다.

M2-03a 직접 진입 조건 조사는 완료했다. 순수 선형 LP 무계 입력의 노드 보류와 루트 호출자의 별도 무계 확정을 구분했고, 호출 없는 원본 합성 상태 13개×3회를 확인했다. 직접 상태 11·ray·실제 자식 연결과 Rust 동등성은 미검증이다. 다음은 M2-03b-1의 상태 재분류·보류 재처리 조사다. [상세 결과](m2/mip_unbounded_entry.md). 작업 branch `sprint/m2-03a`는 미병합이고 main에 복사한 결과와 기존 변경은 미커밋이다.

스프린트 구현 완료는 [동작 동등성 검증 기준](15_sprint_verification.md)의 원본 구간 계약으로 판정한다. 시간·집계 결과만으로 내부 동작 일치를 판정하지 않으며 작업 수행 완료·동등성 판정·전체 관문을 구분한다. 기준 문서와 계약 양식, 시작 지침 연결은 main worktree에 미커밋이다.

24개 번호는 상위 작업이며 실행할 항목은 [1주 스프린트 하위 작업](14_sprint_plan.md)에서 선택한다. 원본 미해독 영역은 조사로 설계를 확정한 뒤 구현 작업을 추가한다. 다음 착수 후보는 M2-03b-1이다. 이 계획과 영역별 설계 보완은 main worktree에 미커밋이다.

새 세션은 [인수인계 안내](handoff/README.md)와 영역별 다음 작업을 읽는다. 같은 저장소의 세 영역 점유와 오프라인 컴퓨터 이동 도구를 추가했다. 컴퓨터 간 자동 동기화는 아직 없고 기존 원격 금지 정책을 유지한다. 설정과 인수인계 문서는 main worktree에 미커밋이다.

운영 상태: main과 M2-03a 독립 worktree가 있다. 이전 로컬 브랜치는 모두 병합됐으며 이번 `sprint/m2-03a`는 미병합이다. LP 드라이버 병합 후 Rust·Python 통합 테스트를 통과했으며 수치 한계가 남는다. 백업과 문서의 미커밋 상태는 [진행 기록](11_progress.md)을 참고한다. 이 운영 상태 변경은 미커밋이다.

```mermaid
flowchart TD
    classDef done fill:#c8f7c5,stroke:#2e7d32,color:#1b5e20
    classDef doing fill:#ffe0b2,stroke:#ef6c00,color:#e65100
    classDef pending fill:#bbdefb,stroke:#1565c0,color:#0d47a1
    classDef todo fill:#eeeeee,stroke:#9e9e9e,color:#424242

    M0["✅ M0 뼈대<br/>MPS·지문 102/102"]:::done

    subgraph M1["M1 LP 엔진"]
        PRE["✅ LP presolve·postsolve<br/>64/64"]:::done
        SPX["🔄 심플렉스 G1<br/>정답 64/64 · 반복 수 24/64"]:::doing
        DRV["🔄 LP 드라이버 G5<br/>main 병합 · 수치 차이 확인 남음"]:::doing
        GATE1["⏳ M1 관문<br/>병합 + 속도 비교표"]:::todo
    end

    subgraph M3["M3 내부점·1차법"]
        BAR["✅ Barrier<br/>반복 수 64/64"]:::done
        PDHG["✅ PDHG<br/>반복 수 64/64"]:::done
        XO["⏳ Crossover·Concurrent<br/>심플렉스 warm-start 입구 구현됨"]:::todo
    end

    subgraph M2["M2 MIP 엔진"]
        MPRE["✅ MIP presolve G6<br/>모델·sweep·레코드 별도 검증 38/38<br/>main 병합"]:::done
        BNB["🔄 B&B 진행 중<br/>38개 첫 기준 측정<br/>시간 제한·중단 조사"]:::doing
        CUT["⏳ Cut separator"]:::todo
        HEU["🔄 휴리스틱<br/>main 구현"]:::doing
        PROP["🔄 전파<br/>main 구현"]:::doing
        POST["🔄 MIP 해 복원<br/>type 1·2·4·9·10·11 지원"]:::doing
        GATE2["⏳ M2 관문"]:::todo
    end

    M4["⏳ M4 일반 제약·SOS·2차"]:::todo
    M5["⏳ M5 NL·다목적·해 풀"]:::todo
    M6["⏳ M6 병렬 B&B·성능"]:::todo

    M0 --> PRE & SPX & BAR & PDHG & MPRE
    PRE --> DRV
    BAR & PDHG --> DRV
    SPX -- "warm-start 입구" --> DRV
    SPX & DRV --> GATE1
    GATE1 --> XO
    BAR --> XO
    MPRE --> BNB
    BNB --> CUT & HEU & PROP & POST
    CUT & HEU & PROP & POST --> GATE2
    MPRE --> M4
    GATE2 --> M4 --> M5 --> M6
```

## 현재 작업 범위

- 상세 작업 기록은 `docs/m1/`, `docs/m2/`, `docs/m3/`로 정리했으며 [문서 안내](README.md)에서 찾을 수 있다. 기존 작업은 main worktree에 미커밋이다.

- 현재 MIP 후속은 M2-03 자식 LP 무계, M2-04 시간 제한, M2-05 해 복원으로 구분한다. [작업 단위 목록](13_work_units.md)은 main worktree에 미커밋이다.

- 자식 LP 무계의 직접 확정 조건·필드·호출표를 M2-03a에서 확정했다. 선형 입력은 노드 상태 3으로 보류되고 루트 무계는 호출자가 별도 처리한다. 호출 없는 원본 합성 상태 13개×3회를 확인했으며 ray·직접 상태 11·실제 자식 연결은 실행 미검증이다. Numeric 처리는 유지한다. [조사 기록](m2/mip_unbounded_entry.md)은 작업 branch에 선택 커밋하고 main에는 미커밋으로 복사한다.

- `AGENTS.md`에 원본 복원 기준과 프로젝트 탐색·검증 경로를 정리했다. 지침은 main worktree에 미커밋이며 별도 미병합 worktree는 없다.
- B&B, 루트 전파·휴리스틱과 Python MIP 풀이가 main에 있다. presolve 연결과 고정·치환 해 복원은 `13ce2fe`에 커밋됐고 `3e5c9d4`로 main에 병합됐다. 진행 문서 변경은 미커밋이다.
- type 4 flag 1·2와 type 10·11, 빈 축약 모델 처리를 추가했다. type 4 정수 분기는 합성 원본 오라클 1,000개와 일치하고 mas74·mas76 원본 해 검사를 통과했다. 빈 축약 enlight_hard는 최적해 37을 복원했다. type 10·11 대상 두 모델은 Gurobi가 축약 모델에서 구한 해를 원본 공간으로 복원해 검사했다. rsolver는 두 모델에서 30초 내 해를 찾지 못했다. 38개 재측정은 전부 축약 경로, 최적 1·시간 초과 37, 원본 해 검사 22개 통과다. 그 외 미지원 레코드와 복원 검증 실패 시 원본 모델로 우회한다. [작업 기록](m2/mip_recovery_bnb.md). 변경은 main worktree에 미커밋이며 별도 미병합 worktree는 없다.
- G6의 모델·레코드·PRELIMIT 저장 변화점 및 최종 모델 38/38을 2026-11-09 별도 검증했다. 38개에서 실행되지 않은 하위 풀이는 미구현이며 `3e5c9d4`로 main에 병합됐다. 범위는 [m2_presolve.md](m2/m2_presolve.md)에 있다.

- MIPLIB 38개 첫 측정에서 rsolver 37개 시간 초과, 1개 외부 감시 중단이었다. 전체 시간 제한과 실제 복원 우회 경로 계측이 남았다. 결과와 새 도구는 main worktree에 미커밋이다: [기준표](m2/mip_bnb_baseline.md).

- 5초 재측정에서 rsolver 38개가 시간 제한으로 반환됐고, 실제 경로는 축약 26·미지원 레코드 우회 12였다. simplex 반복 시간 제한을 추가했으며 presolve 내부 중단은 남았다. 이 측정은 type 9 지원 전 결과다. [결과](m2/mip_bnb_5s.md)는 main worktree에 미커밋이다.
