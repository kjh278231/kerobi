# 12. 로드맵 다이어그램

상태: ✅ 완료 · 🔄 진행 · 🔎 에이전트 검증 완료·병합 대기 · ⏳ 예정 (2026-11-06 기준. 세부 기록은 [11_progress.md](11_progress.md), 심플렉스 최신 검증은 [10_porting.md §9.2](10_porting.md))

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
        MPRE["🔎 MIP presolve G6<br/>모델·sweep·레코드 38/38<br/>별도 검증·main 병합 대기"]:::pending
        BNB["⏳ B&B 본체"]:::todo
        CUT["⏳ Cut separator"]:::todo
        HEU["⏳ 휴리스틱"]:::todo
        PROP["⏳ 전파"]:::todo
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
    GATE1 & MPRE --> BNB
    BNB --> CUT & HEU & PROP
    CUT & HEU & PROP --> GATE2
    MPRE --> M4
    GATE2 --> M4 --> M5 --> M6
```

## 병렬 구간

- 지금: 심플렉스와 LP 드라이버는 진행 중이다. MIP presolve는 에이전트 검증을 마쳤고, 별도 검증과 main 병합을 기다린다.
- M1 관문 뒤: Crossover·Concurrent와 B&B 본체를 함께 시작한다.
- B&B 골격 뒤: Cut, 휴리스틱, 전파를 각자 다른 탭에서 진행한다.
- MIP presolve 검증·병합 뒤: M4 일반 제약 변환은 B&B와 겹쳐 진행할 수 있다.

G6의 38/38은 `m2-mip-presolve`의 `2f28f77`에서 에이전트가 보고한 결과다. 미검증 경로와 미구현 처리의 한계는 [11_progress.md §3](11_progress.md#3-완료-기록)에 기록했다.
