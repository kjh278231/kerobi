# MIPLIB B&B 5초 재측정 (2026-11-09)

대상은 miplib_small 38개, 원본 MPS, Threads=1, 각 풀이기 TimeLimit=5초, 외부 감시 12초다. `87ad61e`의 main worktree에 simplex 마감 확인과 실제 경로 계측을 적용한 미커밋 코드에서 측정했다. Intel Core Ultra 7 155H, Gurobi restricted license. 모델 읽기는 내부 제한 밖이다. 한 번의 짧은 실행이므로 속도 순위로 해석하지 않는다.

Gurobi: 최적 6, 불가능 1, 시간 초과 31. rsolver: 시간 초과 38, 외부 감시 중단·수치 오류·프로세스 오류 0. 유한 시간 내 해는 각각 34개, 21개이며 rsolver 해 21개는 원래 모델의 제약·정수성·목적을 검사했다. 기존 Gurobi 기록이 최적 상태인 12개에 대해 rsolver의 경계가 기준 목적을 상대 1e-5보다 많이 위반한 경우는 없었다. 이는 모든 경계나 최적성의 증명이 아니다.

실제 rsolver 풀이 경로는 축약 모델 26, 미지원 레코드로 원본 모델 직접 풀이 12다. 원본 경로의 다른 사유는 없었다. 별도 레코드 조사에서 이 12개 중 6개는 미지원 종류가 9뿐이었다. type 9는 원래 변수의 이동과 배율 복원이다([근거](m2_spine_d.md)). 이론상 지원 후보이지만 실제로 축약 풀이가 성공할지는 구현·검증 전 알 수 없다.

`cod105`는 이전 2초 측정에서 6초 외부 감시에 걸렸다. presolve와 휴리스틱을 꺼도 재현됐고 simplex 반복 제한을 1로 낮추면 즉시 종료했다. 수정 후 동일 2초 실행은 약 2.04초에 time_limit, 5초 재측정은 약 5.11초에 time_limit이었다. 5초 실행에서 유효한 해 −12를 얻었지만 루트 LP 경계는 미확보다.

시계는 presolve 시작 전에 시작하고 루트·자식 simplex 반복 사이에 마감을 확인한다. presolve, 전파, simplex 초기화나 한 반복 내부의 긴 연산은 중간 중단하지 못한다. 따라서 TimeLimit은 엄격한 벽시각 상한이 아니며 외부 감시는 계속 필요하다. 측정된 rsolver 최대 벽시각(파일 읽기 포함)은 5.11초다.

재실행: `PYTHONPATH=python ../.venv/bin/python tools/mip_bnb_benchmark.py --limit 5 --watchdog 12 --output bench/mip_bnb_5s.jsonl` (Gurobi 잠금 파일에 쓰기 권한 필요). 원시 결과는 `bench/mip_bnb_5s.jsonl`; 이전 측정과 별도 레코드 조사는 [2초 기준표](mip_bnb_baseline.md)에 있다.

| 모델 | Gurobi 상태 | 목적 | 경계 | 노드 | rsolver 상태 | 목적 | 경계 | 노드 | 실제 경로 |
|---|---|---:|---:|---:|---|---:|---:|---:|---|
| ran14x18-disj-8 | time_limit | 3763 | 3484.9723 | 1391 | time_limit | 8089 | 3470.5294 | 266 | reduced |
| gen-ip002 | time_limit | -4770.313 | -4821.8475 | 52152 | time_limit | -2861.4665 | -4822.167 | 8797 | reduced |
| gen-ip054 | time_limit | 6856.2311 | 6785.3082 | 102276 | time_limit | 15687.594 | 6782.2624 | 12702 | reduced |
| ic97_potential | time_limit | 3952 | 3921.6329 | 715 | time_limit | — | 3868 | 601 | reduced |
| pk1 | time_limit | 11 | 3.2927258 | 33866 | time_limit | 675 | 1.8581544 | 6301 | reduced |
| neos859080 | infeasible | — | — | 81 | time_limit | — | 2 | 4559 | reduced |
| neos-911970 | time_limit | 54.76 | 54.68 | 4736 | time_limit | 885.38 | 23.26 | 1170 | unsupported_records |
| markshare2 | time_limit | 46 | -0 | 107781 | time_limit | 203 | 0 | 24816 | unsupported_records |
| mas74 | time_limit | 11877.694 | 10925.49 | 57913 | time_limit | 157344.61 | 10828.699 | 4720 | unsupported_records |
| exp-1-500-5-5 | optimal | 65887 | 65887 | 663 | time_limit | — | 34047.01 | 850 | reduced |
| markshare_4_0 | optimal | 1 | 1 | 4056 | time_limit | 3 | 0 | 67107 | unsupported_records |
| mas76 | time_limit | 40005.054 | 39469.894 | 67966 | time_limit | 157344.61 | 39254.952 | 5592 | unsupported_records |
| neos-860300 | time_limit | 3232 | 3151 | 774 | time_limit | — | 1936.463 | 100 | unsupported_records |
| tr12-30 | time_limit | 130675 | 130367 | 1811 | time_limit | 194146 | 26494.414 | 1237 | reduced |
| neos5 | time_limit | 15 | 14.136364 | 13105 | time_limit | 24 | 13.85 | 3413 | reduced |
| enlight_hard | optimal | 37 | 37 | 0 | time_limit | — | 20 | 11350 | unsupported_records |
| sp150x300d | optimal | 69 | 69 | 1 | time_limit | — | 38 | 2751 | reduced |
| mik-250-20-75-4 | optimal | -52301 | -52301 | 933 | time_limit | 0 | -57974.646 | 4723 | unsupported_records |
| neos17 | optimal | 0.15000258 | 0.14999294 | 1318 | time_limit | 1.2559127 | 0.003332018 | 1196 | reduced |
| neos-4954672-berkel | time_limit | 2724656 | 2210019 | 969 | time_limit | 7017005 | 1458233.2 | 1109 | reduced |
| neos-4338804-snowy | time_limit | 1507 | 1447 | 4414 | time_limit | 3401 | 1447 | 581 | reduced |
| neos-2657525-crna | time_limit | 8.2809932 | 0 | 8356 | time_limit | — | 0 | 1286 | reduced |
| neos-3627168-kasai | time_limit | 988585.62 | 986086.9 | 2082 | time_limit | — | 954625.1 | 120 | reduced |
| neos-3046615-murg | time_limit | 1603 | 1578 | 6907 | time_limit | — | 370 | 11232 | reduced |
| neos-3754480-nidda | time_limit | 13217.934 | -285861.38 | 10202 | time_limit | 27294.032 | -790837.89 | 1225 | reduced |
| csched008 | time_limit | — | 171 | 969 | time_limit | — | 171 | 361 | unsupported_records |
| mad | time_limit | 0.14479986 | 1.9095836e-16 | 18528 | time_limit | 5.9138 | 0 | 6959 | reduced |
| bppc4-08 | time_limit | 54 | 52 | 752 | time_limit | 157 | 51.59 | 179 | reduced |
| glass4 | time_limit | 1.7000142e+09 | 8.00006e+08 | 6609 | time_limit | — | 8.000044e+08 | 4803 | reduced |
| gmu-35-50 | time_limit | -2607449.2 | -2608070.3 | 6086 | time_limit | — | -2608070.3 | 382 | reduced |
| supportcase26 | time_limit | 1814.9292 | 1437.849 | 750 | time_limit | 2201.8794 | 1361.5597 | 2497 | reduced |
| fhnw-binpack4-4 | time_limit | — | 0 | 14469 | time_limit | — | 0 | 2855 | reduced |
| cod105 | time_limit | -12 | -18 | 1 | time_limit | -12 | −∞ (미확보) | 0 | reduced |
| gmu-35-40 | time_limit | -2406091.8 | -2406878.4 | 10202 | time_limit | — | -2406943.6 | 532 | reduced |
| graphdraw-domain | time_limit | 20273 | 13193.864 | 3053 | time_limit | — | 12762.042 | 571 | unsupported_records |
| timtab1 | time_limit | 809350 | 604181.35 | 4323 | time_limit | — | 132567 | 4539 | unsupported_records |
| csched007 | time_limit | — | 312 | 635 | time_limit | — | 275.57639 | 164 | unsupported_records |
| assign1-5-8 | time_limit | 212 | 200 | 4889 | time_limit | 240 | 187.79988 | 459 | reduced |

다음 작업은 type 9 역변환을 합성·실제 모델의 원래 공간 해 검증으로 확인한 뒤 지원하는 것이다. 동시에 presolve와 전파의 중단 지점을 확장하고, 충분히 긴 제한에서 정확한 종료 사례를 확보한다. 노드별 모델 복사·LP 재풀이의 비용은 실제 프로파일 없이 병목이라고 단정하지 않는다.
