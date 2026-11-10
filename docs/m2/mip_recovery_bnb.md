# MIP 해 복원과 B&B 동기화 기록 (2026-11-09)

기준은 main `87ad61e`와 그 위의 기존 미커밋 변경이다. 디컴파일 대상은 Gurobi 13.0.3(`../re/README.md`). 기존 계측은 `tools/mip_state_dump.js`, `tools/mip_rec_check.py`, `tools/uncrush_oracle.py`, `tools/mip_bnb_benchmark.py`다. 변경과 측정 자료는 main worktree에 미커밋이고 별도 미병합 worktree는 없다.

| 단계 | 원본의 확인된 위치 | 현재 구현과 차이 |
|---|---|---|
| MIP presolve | `3cd406` | `presolve_with_state`; 레코드와 생존 열을 보존한다 |
| 루트 LP | `5bbf7f` → `5daa82` → `64da9f` → `5ce67b` → `cda5b` | `solve_original` → `solve_started` → `solve_root_progress_started`; 루트 전파·휴리스틱과 LP 풀이. 상세 `m2_root.md` |
| 트리 | `5bf638`, worker loop | `bnb::solve_started`: 최선 경계 선택, 최대 분수성 분기, 노드마다 모델 복사와 LP 재풀이. 원본의 plunge·strong branching·cut 등은 미구현(`06_branch_and_bound.md`) |
| 원시해 복원 | `36578f` → `363164` | `solve_original`의 역순 레코드 적용과 원본 제약·정수성·목적 검사. 지원하지 않거나 복원이 실패하면 사유를 기록하고 원본 모델을 푼다 |

`363164`(file+0x36c2d0)의 입력 열 매핑은 축약 열을 원 인덱스에 배치한 뒤 최신 레코드부터 연결 리스트를 순회한다. type 9는 `x[i] = x[i] * d2 + d1`, 인덱스는 원래 공간이다. 별도의 경계 보정 분기는 없다. `tools/mip_postsolve_oracle.py`가 직접 원본 함수를 호출해 원 인덱스 2에 입력 2를 놓고 최신 type 9(-2, 5), 고정, 구형 type 9(3, 4)를 적용한 결과 `[0, -7, 35]`를 확인했다. Rust 회귀 검사도 같은 입력과 결과를 쓴다. 원본 함수의 입력은 내부 구조이므로 이 합성 검사는 유효한 인덱스와 레코드를 전제로 한다.

검증: `cargo test --release -p rsolver-mip` 66개, `pytest tests/test_bnb.py -q` 404개 통과. `git diff --check` 통과.

기존 12개 우회 모델의 별도 presolve 레코드 계수(`bench/mip_bnb_records.jsonl`): type 9만 추가로 필요한 모델은 neos-911970(24), markshare2(7), markshare_4_0(4), enlight_hard(200), csched008(238), csched007(246)이다. 나머지는 mas74(type 4 ×2), mas76(4 ×2), mik-250-20-75-4(4 ×4), timtab1(4 ×1), neos-860300(9 ×32, 10 ×136), graphdraw-domain(11 ×20)이다. type 1·2도 함께 나올 수 있다. 이 계수는 별도 presolve 실행 결과이지 각 레코드의 복원 실행 횟수가 아니다.

`363164`의 type 10은 `x[j] = d2*x[k] + d3 + d1*x[j]`이고 type 4는 `35d235`에 위임한다. type 11에 대한 switch 분기는 없다. type 10·11을 원시해 복원에 연결했으며 합성 원본 오라클의 type 1·10·11 혼합 결과 `[0, -7, -13]`을 Rust와 비교했다. `--actual-records`는 실제 presolve 결과의 neos-860300 type 10 레코드 136개와 graphdraw-domain type 11 레코드 20개를 합성 입력으로 원본 함수와 비교한다.

두 모델은 rsolver의 30초 실행에서 해가 없었다. 별도로 `tools/mip_recovery_real_oracle.py`는 rsolver의 축약 모델을 MPS로 내보내 Gurobi(Threads=1, TimeLimit=30)로 풀고, 그 축약해를 `mip_restore_candidate`에 넣어 원본 변수 bound·정수성·행과 목적을 검증한다. neos-860300은 최적 상태의 축약해를 원본 목적 3201로 복원했고 graphdraw-domain은 시간 제한 시점의 축약해를 원본 목적 19848로 복원했다. 두 모델 모두 축약 목적과 복원 목적이 일치했다. 이는 Gurobi가 찾은 각각 한 해의 검증이며 rsolver가 해당 해를 30초 내 찾았다는 뜻은 아니다.

진단 API 연결 후 `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `maturin develop --release` 뒤 `pytest -q` 653개가 통과했다.

기존 축약 레코드의 type 4 플래그는 mas74·mas76이 각각 `[1, 1]`, mik-250-20-75-4가 `[0, 0, 0, 0]`, timtab1이 `[0]`이다. 따라서 LP의 flag 0 복원만 재사용해도 네 모델 모두를 처리할 수는 없다.

### type 4 flag 0 후속

`35d235`(file+0x36e3c0)의 원시해 분할은 `363164`가 기준값을 전달하고 dual·기저 포인터를 비워 호출한다. 기존 LP의 동일한 공통 분할을 flag 0에 재사용했다. flag 1·2의 정수 보정은 구현하지 않아 원본 풀이 우회를 유지한다. `tools/mip_postsolve_oracle.py`는 양·음 배율, 무한 bound와 범위 밖 입력의 네 합성 사례를 `363164`에 직접 넣어 비교하며 Rust 회귀 검사도 같은 출력과 대조한다. 유효 범위 밖 입력 사례는 함수의 선택 규칙 검사일 뿐 실행 가능한 축약해의 사례는 아니다.

같은 조건의 단독 실제 실행(Threads=1, TimeLimit=5): mik-250-20-75-4는 `reduced`, 시간 초과, 목적 0, 경계 -57915.65664001687, 노드 5320, 원본 해 검사 통과였다. timtab1은 `reduced`, 시간 초과, 경계 136374, 노드 5291이었으나 해가 없어 복원 해를 검사하지 못했다. 이 두 실행은 단일 측정이며 아래 type 9 기준 38개 전체 측정을 대체하지 않는다.

검증: `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, 로컬 확장 빌드 후 `PYTHONPATH=python ../.venv/bin/python -m pytest -q` 652개 통과. 같은 worktree에 동시 변경이 관찰되어 공유 가상환경을 수정하는 `maturin develop` 대신 빌드 산출물을 로컬 확장에 복사해 검사했다.

### type 4 flag 1·2와 빈 축약 모델 후속

`35d235`의 정수 분기와 어셈블리 file+0x36e7d1–0x36ec에서 반올림 인자·연산 순서를 확인했다. 공통 분할 후 flag 1은 병합값과 두 정수 열의 범위로 반올림과 보정을 하고, flag 2는 분할된 정수 열을 반올림해 다른 열의 경계를 보정한다. `tools/mip_postsolve_oracle.py --generate-parallel`이 Gurobi에서 flag별 500개를 생성하며, `bench/oracle/mip_postsolve_parallel.txt`의 1,000개 사례가 Rust 결과와 일치했다. 무작위 생성 사례에는 실현 불가능한 중간 입력도 들어 있어 분기 동등성 검사로만 취급한다. 실제 mas74·mas76은 Threads=1, TimeLimit=5에서 축약 경로의 원본 해 검사를 통과했다(각각 시간 초과, 목적 157344.61033).

열이 없는 축약 모델은 기존 루트의 빈 모델 처리로 진행한 뒤 같은 복원·원본 해 검사를 한다. enlight_hard는 축약 경로에서 원본 해와 목적 37을 확인해 OPTIMAL로 반환했다. 고정 변수 하나가 완전히 제거되는 합성 회귀 검사도 있다. 이 변경 이후 38개 측정은 별도로 기록한다.

### 복원 경로 확장 후 38개 재측정

`tools/mip_bnb_benchmark.py --solver rsolver --limit 5 --watchdog 12 --output bench/mip_bnb_recovery_5s.jsonl`로 이전 Gurobi 측정과 같은 Threads=1·5초 조건에서 rsolver만 다시 실행했다. 38개 전부 축약 경로였으며 OPTIMAL 1(enlight_hard)·TIME_LIMIT 37, 외부 감시 중단·오류 0이다. 해를 반환한 22개 모두 원본 모델 제약·정수성·목적 검사를 통과했다. 기존 Gurobi 기준이 최적인 12개에서 유한 경계가 최적 목적을 상대 1e-5보다 많이 위반한 경우는 없었다. 나머지 16개는 해가 없으므로 해당 경로의 실제 복원 해 검증이 아니다. Gurobi의 비교 결과는 이전 5초 측정(최적 6·불가능 1·시간 초과 31)이며 이번에 재실행하지 않았다. 노드 수 등 원시 수치는 JSONL에 있다.

루트·자식 LP 상태와 종료 규칙은 아직 원본과 동기화되지 않았다. 특히 자식 LP 무계는 현재 Numeric으로 보수 중단하고, presolve·전파 내부와 simplex 한 반복 내부에서는 시간 제한을 즉시 확인하지 못한다. 원본의 무계 확정 조건과 중단 지점을 재현·확인하기 전에는 원본 동작을 추정해 대체하지 않는다. 노드별 복사와 LP 재풀이의 비용도 아직 프로파일하지 않았다.

### 자식 LP 무계 원본 조사 (2026-11-10)

`600d25` 디컴파일(`../re/decomp/c/00518170_PRIVATE0000000000600d25.c`)의 상태 5 분기는 조건에 따라 바로 노드 상태 11을 설정하거나, `GRBgetdblattrarray`로 `UnbdRay`를 읽어 검사한 뒤 상태 11 또는 3을 설정한다. 방향 검사에는 1e-13·1e-10과 모델 허용오차가 등장한다. 상태 4는 노드 상태 6으로 바뀐다. `5c239f`(`../re/decomp/c/004e7e20_PRIVATE00000000005c239f.c`)는 MIP가 진행 상태 1일 때 노드 상태 11을 만나면 MIP 상태 5로 바꾼다. 따라서 LP 무계만으로 무조건 MIP 무계를 반환하는 변경은 이 흐름을 재현하지 않는다.

이 문단 작성 당시에는 상태 11의 직접 진입 조건과 방향 검사 전체의 변수 의미, 상태 3 이후 재처리 경로를 확인하지 못했다. 자식 노드에서 이 분기를 확실히 밟는 재현 모델과 원본 오라클도 아직 없다. 이번 조사는 코드나 동작을 바꾸지 않았으며, 현재 Numeric 처리는 그대로 남는다. 다음에는 `600d25`의 입력 상태와 분기 조건을 추적하고 실제 자식 노드 사례를 만들어 상태·무계 방향을 대조해야 한다.

## type 9 적용 후 38개 재측정

`tools/mip_bnb_benchmark.py --limit 5 --watchdog 12 --output bench/mip_bnb_type9_5s.jsonl`, Threads=1, Gurobi와 rsolver를 순차 실행했다. Gurobi 최적 6·불가능 1·시간 초과 31, rsolver 시간 초과 38이고 외부 감시 중단·오류는 없었다. 실제 경로는 축약 31·미지원 레코드 우회 6·빈 축약 모델 우회 1이다. rsolver의 해 21개는 원본 모델 제약·정수성·목적 검사를 통과했고, 기존 기준값이 최적인 12개 모델의 유한 경계는 기준 목적을 상대 1e-5보다 많이 위반하지 않았다. 5초 내 해를 얻지 못한 17개 모델의 복원은 이 검사로 검증되지 않는다. 상태·목적·경계·노드·경로별 원시 결과는 JSONL에 있다.

type 9만 미지원이던 여섯 모델 중 neos-911970, markshare2, markshare_4_0은 축약 경로에서 원본 해·목적 검사를 통과했다. csched007·csched008은 축약 경로였으나 해가 없었고, enlight_hard는 빈 축약 모델이라 원본 풀이로 돌아가 해가 없었다. 따라서 여섯 모델 모두의 복원 검증은 아직 완료되지 않았다.

### M2-03a 후속 조사 (2026-11-10)

[직접 확정 진입 조사](mip_unbounded_entry.md)에서 분기 조건·필드·호출표를 확정했다. 순수 선형 입력은 `600d25`에서 상태 3으로 보류되며, 루트의 무계 확정은 호출자 `5ce67b`가 따로 처리한다. 루트 캐시가 2이면 현재 LP 상태 5를 12로 바꾼다. 직접 상태 11은 추가 제약 문맥의 재풀이와 domain predicate를 통과한 경우다. 호출 없는 합성 상태 13개를 원본 독립 프로세스 3회에서 확인했다. 재풀이·직접 상태 11·ray와 실제 자식 연결은 실행 미검증이다. Rust 변경 없이 M2-03b-1의 선형 재분류·보류 후 재처리 조사로 이어간다.
