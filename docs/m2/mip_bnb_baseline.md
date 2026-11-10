# MIPLIB B&B 기준 측정 (2026-11-09)

대상 `miplib_small.csv` 38개, 코드 main `87ad61e`, Intel Core Ultra 7 155H, 두 풀이기 Threads=1, TimeLimit=2초, 프로세스별 외부 감시 6초. 원본 MPS를 읽고 기본 presolve를 사용했다. Gurobi restricted license 사용. 한 번의 짧은 실행으로 성능 비율을 판단하지 않는다.

`elapsed`는 파일 읽기를 포함한 벽시각이다. rsolver의 TimeLimit 시계는 presolve 뒤 B&B에서 시작하며 개별 LP 도중에도 확인하지 않아 Gurobi와 전체 시간 제한이 완전히 동등하지 않다. `watchdog_timeout`은 프로세스를 강제 종료한 것으로 상태·경계·노드를 알 수 없다. 노드 집계 기준도 두 엔진에서 다를 수 있다.

Gurobi 5개 최적, 1개 불가능, 32개 시간 초과. rsolver 37개 시간 초과, 1개 외부 감시 시간 초과. 해를 반환한 모델은 각각 34개와 20개이며 rsolver 해 20개는 원래 모델의 제약·정수성·목적값을 검사했다. 기존 Gurobi 최적 기준값이 있는 사례에서 rsolver 경계가 기준값을 상대 1e-5 넘어서 위반한 사례는 없었다. 시간 초과 상태의 서로 다른 목적값은 불일치 판정이 아니다.

복원 레코드는 동일 모델을 별도로 presolve한 결과다. 12/38에서 미지원 유형 4·9·10·11이 나왔다. 이들은 원본 직접 풀이 우회 후보이며 실제 B&B 경로 계측은 아니다. presolve 실패, 빈 축약 모델, 해 복원 검증 실패에 의한 우회는 이 수치에 포함됐는지 알 수 없다. 종류별 레코드 수는 9: 751, 10: 136, 11: 20, 4: 9다. 유형 9의 의미와 안전한 복원부터 확인한다.

측정 재실행: `PYTHONPATH=python ../.venv/bin/python tools/mip_bnb_benchmark.py --limit 2 --watchdog 6` (Gurobi 잠금에 쓰기 권한 필요). 레코드 재측정: `PYTHONPATH=python ../.venv/bin/python tools/mip_bnb_benchmark.py --records --watchdog 6`. 원시 결과는 `bench/mip_bnb_baseline.jsonl`, `bench/mip_bnb_records.jsonl`에 있다. `neos859080` Gurobi는 최초 측정 도구의 무한값 JSON 직렬화 오류를 고친 뒤 같은 조건으로 재실행했다.

| 모델 | Gurobi 상태 | 목적 | 경계 | 노드 | rsolver 상태 | 목적 | 경계 | 노드 | 미지원 레코드 |
|---|---|---:|---:|---:|---|---:|---:|---:|---|
| ran14x18-disj-8 | time_limit | 3763 | 3482.8993 | 554 | time_limit | 8089 | 3462.7016 | 103 | — |
| gen-ip002 | time_limit | -4770.313 | -4827.7779 | 17142 | time_limit | -2861.4665 | -4825.2533 | 3779 | — |
| gen-ip054 | time_limit | 6856.2311 | 6779.4317 | 32264 | time_limit | 15687.594 | 6779.4762 | 5167 | — |
| ic97_potential | time_limit | 3953 | 3918.785 | 638 | time_limit | — | 3868 | 248 | — |
| pk1 | time_limit | 14 | 0 | 8908 | time_limit | 675 | 0.56109334 | 2595 | — |
| neos859080 | infeasible | — | — | 81 | time_limit | — | 2 | 1903 | — |
| neos-911970 | time_limit | 54.83 | 53.64 | 746 | time_limit | 885.38 | 23.26 | 465 | 9 |
| markshare2 | time_limit | 46 | -0 | 36471 | time_limit | 224 | 0 | 11755 | 9 |
| mas74 | time_limit | 11945.825 | 10826.303 | 15303 | time_limit | 157344.61 | 10774.34 | 1901 | 4 |
| exp-1-500-5-5 | time_limit | 67743 | 65646 | 629 | time_limit | — | 33328.574 | 351 | — |
| markshare_4_0 | optimal | 1 | 1 | 4056 | time_limit | 3 | 0 | 29061 | 9 |
| mas76 | time_limit | 40005.054 | 39270.134 | 15680 | time_limit | 157344.61 | 39192.496 | 2349 | 4 |
| neos-860300 | time_limit | 3445 | 2337 | 1 | time_limit | — | 1865.6049 | 36 | 10,9 |
| tr12-30 | time_limit | 132320 | 130310 | 587 | time_limit | 194146 | 26271.576 | 512 | — |
| neos5 | time_limit | 15 | 14.035714 | 10036 | time_limit | 24 | 13.75 | 1329 | — |
| enlight_hard | optimal | 37 | 37 | 0 | time_limit | — | 20 | 4647 | 9 |
| sp150x300d | optimal | 69 | 69 | 1 | time_limit | — | 38 | 1068 | — |
| mik-250-20-75-4 | optimal | -52301 | -52301 | 933 | time_limit | 0 | -58423.661 | 1925 | 4 |
| neos17 | optimal | 0.15000258 | 0.14999294 | 1318 | time_limit | 1.2559127 | 0.0028329171 | 481 | — |
| neos-4954672-berkel | time_limit | 2833410 | 2172826 | 620 | time_limit | 7017005 | 1448392.9 | 443 | — |
| neos-4338804-snowy | time_limit | 1518 | 1447 | 1552 | time_limit | 3401 | 1447 | 236 | — |
| neos-2657525-crna | time_limit | 8.2809932 | 0 | 2977 | time_limit | — | 0 | 535 | — |
| neos-3627168-kasai | time_limit | 988585.62 | 984437.77 | 627 | time_limit | — | 954573.56 | 49 | — |
| neos-3046615-murg | time_limit | 1639 | 1569 | 784 | time_limit | — | 335 | 4624 | — |
| neos-3754480-nidda | time_limit | 13542.663 | -340145.65 | 3814 | time_limit | 27294.032 | -855936.81 | 483 | — |
| csched008 | time_limit | — | 171 | 602 | time_limit | — | 171 | 147 | 9 |
| mad | time_limit | 0.21119999 | 0 | 7761 | time_limit | 5.9138 | 0 | 2883 | — |
| bppc4-08 | time_limit | 55 | 52 | 618 | time_limit | 157 | 51.59 | 68 | — |
| glass4 | time_limit | 1.8000147e+09 | 8.000054e+08 | 2526 | time_limit | — | 8.000042e+08 | 1985 | — |
| gmu-35-50 | time_limit | -2607060.8 | -2608070.3 | 2328 | time_limit | — | -2608070.3 | 156 | — |
| supportcase26 | time_limit | 1912.5921 | 1432.4507 | 706 | time_limit | 2201.8794 | 1340.882 | 976 | — |
| fhnw-binpack4-4 | time_limit | — | 0 | 5713 | time_limit | — | 0 | 1011 | — |
| cod105 | time_limit | -12 | -1024 | 1 | watchdog_timeout | — | — | — | — |
| gmu-35-40 | time_limit | -2406079.6 | -2406887.3 | 3974 | time_limit | — | -2406943.6 | 218 | — |
| graphdraw-domain | time_limit | 21302 | 13036.537 | 738 | time_limit | — | 12735.275 | 233 | 11 |
| timtab1 | time_limit | 890622 | 565112.2 | 896 | time_limit | — | 121459 | 1906 | 4 |
| csched007 | time_limit | — | 303 | 521 | time_limit | — | 274.42778 | 68 | 9 |
| assign1-5-8 | time_limit | 213 | 197 | 1204 | time_limit | 240 | 187.05 | 177 | — |

다음 순서: `cod105`가 제한을 넘는 구간을 분리하고 시간 제한 전파를 검증한다. 실제 원본 직접 풀이 경로와 사유를 계측해 복원 우회 비율을 확정한다. 이후 동일한 전체 벽시각 제한을 구현하고 더 긴 제한으로 정확성·성능을 재측정한 뒤 노드 LP 재풀이 비용을 분석한다. 자식 노드 무계 LP의 Numeric 처리와 노드 풀 확장 한계는 이번 결과에서 재현됐다고 볼 수 없다.
