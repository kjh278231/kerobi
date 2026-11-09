# 12. 로드맵 다이어그램

상태: ✅ 완료 · 🔄 진행 · 🔎 에이전트 검증 완료·병합 대기 · ⏳ 예정 (2026-11-09 기준. main에 병합된 구현과 남은 작업을 표시한다. 세부 기록은 [11_progress.md](11_progress.md), 심플렉스 최신 검증은 [10_porting.md §9.2](10_porting.md))

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
        DRV["🔄 LP 드라이버 G5<br/>파라미터 548/548 · 로그 102/102"]:::doing
        GATE1["⏳ M1 관문<br/>병합 + 속도 비교표"]:::todo
    end

    subgraph M3["M3 내부점·1차법"]
        BAR["✅ Barrier<br/>반복 수 64/64"]:::done
        PDHG["✅ PDHG<br/>반복 수 64/64"]:::done
        XO["⏳ Crossover·Concurrent<br/>심플렉스 warm-start 입구 구현됨"]:::todo
    end

    subgraph M2["M2 MIP 엔진"]
        MPRE["✅ MIP presolve G6<br/>모델·sweep·레코드 별도 검증 38/38<br/>main 병합"]:::done
        BNB["🔄 B&B 진행 중<br/>분기·노드 탐색·종료 구현<br/>검증 후 main 병합"]:::doing
        CUT["⏳ Cut separator"]:::todo
        HEU["🔄 휴리스틱<br/>main 구현"]:::doing
        PROP["🔄 전파<br/>main 구현"]:::doing
        POST["🔄 MIP 해 복원<br/>고정·치환 지원, 나머지 직접 풀이"]:::doing
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

- 상세 기록의 위치는 [문서 안내](README.md)에서 찾을 수 있다.

- `rsolver-barrier`의 `m2-mip-presolve`에는 B&B, 루트 전파·휴리스틱과 Python MIP 풀이가 있다. presolve 연결과 고정·치환 해 복원은 `13ce2fe`에 커밋됐고 `3e5c9d4`로 main에 병합됐다. 진행 문서 변경은 미커밋이다.
- 다른 MIP 복원 레코드와 빈 축약 모델은 원래 모델 직접 풀이로 돌아간다. LP presolve→simplex→postsolve 연결, MIP 콜백과 cut separator는 남아 있다.
- G6의 모델·레코드·PRELIMIT 저장 변화점 및 최종 모델 38/38을 2026-11-09 별도 검증했다. 38개에서 실행되지 않은 하위 풀이는 미구현이며 `3e5c9d4`로 main에 병합됐다. 범위는 [m2_presolve.md](m2/m2_presolve.md)에 있다.
