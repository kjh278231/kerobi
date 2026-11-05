# 10. 포팅 방법과 성능 패리티

근거: `re/decomp/c/`(Ghidra 4,358개 + angr 4개 의사코드), `re/notes`, [port_map.tsv](port_map.tsv)(생성: `tools/port_map.py`). 이 문서는 [00 §2](00_overview.md) 출처 정책의 실행 규칙이다.

## 1. "같은 성능"의 정의

시간 하나로는 재지 않는다. 시간은 기계와 컴파일러에 따라 흔들리고, 차이가 나도 원인을 짚지 못한다. 아래 네 단계를 차례로 맞춘다. 앞 단계가 맞지 않으면 뒤 단계 비교는 의미가 없다.

| 단계 | 같아야 하는 것 | 비교 수단 | 합격 |
|---|---|---|---|
| P0 구조 | presolved 모델(행·열·계수·bound·타입), 모델 지문, 로그 앵커 순서 | Gurobi `Model.presolve()` 결과를 MPS로 써서 rsolver presolved 모델과 diff; "Model fingerprint: 0x%08x" | 정확히 같음 |
| P1 궤적 | Threads=1에서 simplex·barrier 반복 수, B&B 노드 수, cut 종류별 수, 해 발견 순서와 목적값 | 콜백 추적(§6)과 `IterCount`·`BarIterCount`·`NodeCount` | 같음이 목표. 차이는 첫 분기점과 원인을 기록하고, 인스턴스별 ±5% |
| P2 work | 단계별 work 단위(presolve, 루트, 트리, 전체) | 로그의 "(%.2f work units)", `Work` 속성 | ±5% |
| P3 시간 | 벽시계 시간 | `re/bench` 같은 기계 재실행 | 기하평균 비 ≤ 1.2, 개별 ≤ 2 |

- P1이 맞으면 같은 알고리즘이 같은 일을 한 것이다. P1·P2가 맞는데 P3가 어긋나면 원인은 커널 속도(메모리 배치, 벡터화, MKL 호출)뿐이다.
- 병렬(Threads>1)은 결정적 모드(work 라운드, 노트 15 §3.12)에서만 P1을 본다.

## 2. 포팅 범위 (`port_map.tsv`)

`tools/port_map.py`가 코어 함수 4,362개 전부를 설계 모듈에 배정한다. 진척은 함수 수가 아니라 패리티 결과(§6)로 잰다.

```
python3 -I rsolver/tools/port_map.py re rsolver .venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so
```

배정 규칙:
1. 노트 10–17이 이름을 든 함수는 그 노트의 모듈로 간다. 언급 수가 가장 많은 노트가 이기고, 그 함수를 표 행으로 설명한 노트는 10표로 센다.
2. 1의 함수가 호출하거나 주소를 넘기는 함수(콜백, 스레드 진입점, 데이터 영역의 함수 포인터 표)는 한 모듈에서만 닿으면 그 모듈, 여러 모듈에서 닿으면 `common`이다.
3. 나머지 중 남은 공개 API에서 닿는 것은 `01_model_api`, 라이선스·WLS·batch·튜닝·서버 계열은 `excluded`(환경 해제 경고 "Continue to use WLS"만 가진 함수는 제외 대상이 아니다: 다목적·concurrent 환경 코드), MKL·OpenSSL은 `extern`, 정적 참조가 전혀 없는 것은 `unreached`다.
4. 노트가 표 행으로 설명하지 않은 함수인데 세 모듈 이상에서 불리면 `common`으로 돌린다(공용 보조 함수).
5. 각 모듈의 진입 함수([00 §4](00_overview.md) 표)와 몇몇 공용 함수는 손으로 고정한다(`OVERRIDE`: 난수 `a7b4d6`·한도 검사 `a83b2f` → common, 파라미터 등록 `ac81d7` → 01, 노드 휴리스틱 디스패처 `8ad3a3` → 07, product score `646052` → 06). 고정이 없으면 노트 17처럼 드라이버를 자주 인용하는 노트가 `c23877`을 가져간다. 스크립트 끝의 자체 검사가 진입 함수의 배정을 확인한다.

2026-11-05 생성 결과:

| 모듈 | 함수 | 노트에 이름 있음 | optimize 경로 | 기계어 KB | 의사코드 줄 |
|---|---:|---:|---:|---:|---:|
| 01_model_api | 569 | 1 | 305 | 400 | 76,732 |
| 02_driver | 169 | 65 | 169 | 307 | 51,260 |
| 03_presolve | 415 | 203 | 403 | 1,341 | 196,867 |
| 04_simplex | 339 | 190 | 335 | 1,457 | 226,720 |
| 05_barrier_pdhg_concurrent | 236 | 125 | 235 | 654 | 104,157 |
| 06_bnb | 273 | 130 | 272 | 635 | 92,765 |
| 07_cuts_heur | 394 | 199 | 394 | 1,445 | 202,101 |
| 08_genconstr_nl | 350 | 112 | 295 | 855 | 143,107 |
| common | 820 | 78 | 820 | 1,085 | 195,537 |
| excluded | 100 | 0 | 13 | 119 | 21,917 |
| extern | 13 | 0 | 0 | 0 | 164 |
| unreached | 684 | 0 | 0 | 728 | 136,520 |

- 포팅 대상은 optimize 경로에서 `excluded`를 뺀 3,228개다. 기계어 7.7MB, 의사코드 약 120만 줄이다.
- `unreached` 684개는 정적 참조가 없다. 대부분 MPS/LP 읽기, JSON, simplex 디버그 코드, 로그 서식 보조다. 패리티 차이가 그쪽을 가리킬 때만 포팅한다. MPS 읽기는 지문(P0)에 영향이 있으므로 M0에서 확인한다.
- 배정은 근사다. 이름 하나를 함수 여러 개가 같이 쓰고(60개 이름, 122개 함수), 간접 호출 표는 가장 가까운 참조 주소로 추정한다. 포팅하다 틀린 배정을 보면 `OVERRIDE`에 넣고 다시 생성한다.
- `common` 820개(의사코드 20만 줄)는 마일스톤마다 그 단계가 처음 부르는 것부터 옮긴다. M1이 가장 많이 가져간다(LU·벡터 연산·정렬·난수·한도 검사).

## 3. 포팅 단위와 절차

함수 하나가 포팅 단위다.

1. **읽기**: `re/decomp/c/<off>_<name>.c`, 노트의 해당 절. 다음 경우에는 objdump를 같이 본다.
   - 점프 테이블(예: `6e8eb3` 노드 선택 규칙 12개)
   - 벡터화 루프(§4)
   - Ghidra가 인자를 빠뜨린 호출(예: `5c239f`:444의 `6e713c` 세 번째 인자, 노트 15 §6.3)
2. **구조체 복원**: `param_1 + 0x470` 같은 오프셋을 이름 있는 Rust 필드로 바꾸고, 필드 doc comment에 원래 오프셋을 적는다(`/// spx+0x470`). env 오프셋은 `re/notes/env_param_offsets.tsv`로 파라미터 이름을 붙인다.
3. **쓰기**: 제어 흐름, 분기 조건, 상수, 연산 순서를 그대로 옮긴다. 바꾸는 것은 이름뿐이다. goto는 `loop`와 labeled `break`로 구조화하되, 블록의 주소 순서가 아니라 실행 순서를 따른다(Ghidra는 goto 대상을 주소 순으로 늘어놓는다, 노트 12 미확인 3).
4. **두 벌 합치기**: double판과 x87 long double판(노트 13 §0.1)처럼 같은 이름을 쓰는 두 벌은 제네릭 함수 하나로 옮긴다. 코드에 출처 태그는 달지 않는다([00 §2](00_overview.md)).
5. **검사**: 단위 테스트를 쓰고, 그 함수가 속한 단계의 차등 검사(§6)를 돌린다.

순서는 마일스톤 범위 안에서 호출 그래프의 **잎부터**다(callee 먼저). `common` 함수는 처음 필요한 마일스톤에서 옮긴다. 노트가 "미확인"으로 남긴 지점은 문헌으로 메우지 않고 해당 함수를 읽어 확정한다. 각 설계 문서의 "포팅으로 확정할 지점" 절이 그 목록이다.

## 4. 수치 충실도 규칙 (P1을 위해)

반복 수와 노드 수가 같으려면 부동소수 결과가 비트 단위로 같아야 한다. 반올림 한 번이 다른 피벗을 고르게 하고, 그때부터 궤적이 갈라진다.

| 규칙 | 이유 |
|---|---|
| 연산 순서를 그대로 둔다. 결합 법칙으로 바꾸지 않고 `mul_add`(FMA)를 쓰지 않는다. 빌드 대상은 기본 `x86-64`(SSE2)로 고정한다 | 원본은 gcc 8.5 빌드(노트 03 §1)이고 FMA 없이 반올림된다 |
| 벡터화 루프는 어셈블리의 레인 구조를 따른다. `ADDPD` 2레인 누적이면 누산기 2개로 짝·홀 원소를 따로 더하고 끝에 합친다. 루프 풀기 단위와 나머지 처리도 같게 한다 | Ghidra 의사코드는 레인 순서를 잃는다. 디컴파일이 실패한 4개 함수도 이 루프 때문이었다 |
| x87 long double판(Quad)은 `F80` 타입(x86_64 `asm!`의 x87 명령)으로 옮긴다. double-double로 대체하지 않는다 | Quad로 넘어간 뒤의 궤적 |
| 난수는 `a7b4d6`과 seed 전달 규칙(예: sub-MIP `Seed += 0x12d8652`)을 그대로 옮긴다. 외부 crate의 RNG를 쓰지 않는다 | 섭동 `rand%250`, 비용 이동, 노드 규칙 3/12·5, 휴리스틱 |
| 정렬은 원본 정렬 함수(`aabbe3` median-of-3 quicksort 등)를 옮겨 쓴다. `sort_unstable`은 쓰지 않는다 | 동률 원소의 순서가 cut 선택과 후보 순서를 정한다 |
| 해시(평행 행·열, cut 중복, NL 공통 부분식)는 원본 해시 함수와 버킷 순회 순서를 그대로 쓴다 | 순회 순서가 reduction 적용 순서를 정한다 |
| `exp/log/sqrt/pow/sin`은 glibc libm을 쓴다. Rust std가 Linux에서 glibc를 부르고, 원본도 `libm.so.6`를 동적 링크한다 | 같은 결과 |
| 밀집 BLAS/LAPACK(`dgemm`, `dtrsm`, `dgetrf`)은 MKL을 링크해 같은 루틴을 부른다(§5) | 커널 속도와 결과 |
| Ghidra의 `(long)double`은 정수 변환이 아닐 수 있다. 지문 해시에서는 비트 재해석이었다(`add rax,[mem]`). 해시·키 계산처럼 결과가 비트 단위로 맞아야 하는 곳은 어셈블리로 확인한다 | 01 §1.1의 실제 사례 |
| 원본이 벽시계로 판단하는 지점(예: LU 재분해의 "경과 시간 > 14×분해 시간", 비결정 concurrent)은 그대로 옮기되 P1에서 그 지점 뒤의 차이를 허용하고 기록한다. work로 판단하는 지점은 정확히 맞춘다 | 원본 자신도 그 지점에서는 실행마다 다를 수 있다 |

## 5. 자료구조, 메모리, 외부 라이브러리

- 자주 쓰는 데이터는 원본 배치를 따른다: 배열 구성, 연결 리스트(MIP presolve), CSC/CSR 쌍, LU 저장 형식, 노드 압축 형식. 캐시 동작이 같아야 P3가 맞는다. 함께 읽히는 배열은 Rust에서도 한 구조체에 둔다.
- 원본이 한 번 크게 잡아 재사용하는 버퍼(예: `spx` 0xf20 바이트, LU 객체, pricing 객체)는 Rust에서도 엔진 수명 동안 재사용한다. 반복마다 `Vec`를 새로 만들지 않는다.
- MKL: 원본은 MKL을 정적 링크한다(노트 03 §4). rsolver는 `intel-mkl-src`(static, lp64)로 같은 라이브러리를 링크한다. 원본의 MKL 스레드 수와 재현성(CBWR) 설정은 미확인이다. barrier를 포팅할 때 `mkl_*` 설정 호출부를 찾아 같게 맞춘다.
- 스레드: 원본의 스레드 생성과 동기화(alive spin, 라운드 경계 병합, worker 상태 0–3)를 그대로 옮긴다. rayon 같은 작업 훔치기 풀은 쓰지 않는다. 일의 배분이 실행마다 달라져 결정성이 깨진다.

## 6. 차등 검사 하네스 (`tools/parity.py`)

```
python tools/parity.py capture gurobipy|rsolver <out_dir> [--work-limit 60] [name ...]
python tools/parity.py compare <ref_dir> <test_dir> [--csv bench/parity.csv]
```

- **라이선스 한계**: `.venv`의 gurobipy는 크기 제한 라이선스다(변수·선형 제약 각 2,000개, 2027-11-29 만료). 기본 대상인 Netlib 64개(`re/bench/results/lp.csv`)와 `miplib_small` 38개는 모두 이 안에 든다. Netlib의 나머지 27개는 공개 최적값과만 비교한다.
- **한 번에 하나**: `.venv`의 `gurobi_guard.pth`가 Gurobi를 한 프로세스만 쓰도록 막는다(라이선스). 캡처는 차례로 돌린다. 다른 세션이 Gurobi를 쓰는 동안에는 끝날 때까지 기다렸다 다시 시도한다.
- **실행**: 두 백엔드 모두 같은 gurobipy 모양 API로 돌린다. Threads=1, WorkLimit(기본 60), 나머지 기본값이다. TimeLimit은 쓰지 않는다. 시간 한도는 실행마다 멈추는 지점이 달라 궤적 비교가 안 된다.
- **기준선**: Gurobi 결과는 `bench/gurobi/`에 있다. 같은 설정으로 두 번 실행해 비교했더니 102개 모두 P0·P1이 일치했다(`bench/gurobi_determinism.csv`, 2026-11-05, 두 실행을 동시에 돌린 부하 상태에서도 같음). 따라서 P1 "같음"은 도달 가능한 목표다. 상태 분포는 LP 64개 OPTIMAL, MIP 12개 OPTIMAL·24개 WORK_LIMIT·2개 INFEASIBLE이다.
- **인스턴스마다 남기는 것**:

  | 파일 | 내용 | 단계 |
  |---|---|---|
  | `summary.json` | Fingerprint, Status, IterCount, BarIterCount, NodeCount, Work, ObjVal, ObjBound, presolved 크기, 로그에서 뽑은 presolve 제거 수·루트 LP(목적값, 반복 수)·"Cutting planes:" 종류별 수 | P0–P2 |
  | `trace.csv.gz` | SIMPLEX(반복마다: ITRCNT, OBJVAL, PRIMINF, DUALINF, ISPERT), BARRIER(반복마다), MIPSOL(해마다: 노드, 목적값, best, bound), MIPNODE(노드 1,000개까지 전부, 그 뒤 1,000개마다: 노드, 상태, best, bound, 해 수). 값은 `repr`로 써서 비트 단위로 비교한다 | P1 |
  | `presolved.mps.gz` | `Model.presolve()` 결과. 압축 해제 내용의 SHA-256으로 비교한다 | P0 |
  | `log.txt` | 전체 로그 | 디버깅 |

- **기록하지 않는 콜백**: PRESOLVE와 MIP 폴링 콜백은 시간 간격으로 불려 실행마다 행 수가 다르다. MIP에서는 루트 LP에도 SIMPLEX 콜백이 오지 않으므로 루트 LP는 로그 값과 노드 0의 MIPNODE 행(루트 cut 패스별 bound)으로 본다.
- **첫 분기점**: 두 추적이 처음 갈라지는 행을 찾아, `port_map.tsv`에서 그 단계를 맡은 모듈의 함수부터 원본과 대조한다. SIMPLEX 콜백은 반복마다 불린다(25fv47: 반복 2,693회에 콜백 2,696회). 결과는 인스턴스별 한 줄로 `bench/parity.csv`에 쌓는다.
- **P3**는 콜백 없는 실행으로 잰다(`re/bench/bench.py`). 콜백이 시간을 늘리기 때문이다.

## 7. 포팅하지 않는 것

- `excluded` 100개: 라이선스 검사, 크기 제한, WLS, 토큰 서버, Compute Server, Cluster Manager, Instant Cloud, batch, 튜닝. 원본이 라이선스 상태로 동작을 바꾸는 분기(예: `bb9b40`, "Solve interrupted (license invalid)")는 옮기지 않고 분기 자체를 없앤다.
- GPU, 녹화·재생, 분산 MIP/concurrent, GBT 기반 Method 선택(`171e36`, 기본 비활성).
- `extern`: MKL은 링크해서 쓰고, OpenSSL·libcurl·mbedTLS는 쓰지 않는다.

## 8. 법적 위치

decomp를 옮긴 코드는 Gurobi 코드의 2차적 저작물이다. 저장소는 비공개로 두고 배포·공개하지 않는다(내부 연구용). 공개가 필요해지면 해당 코드는 이 문서의 규칙이 아니라 클린룸 방식으로 다시 써야 한다([09 §4](09_roadmap.md)).

## 9. 인수인계 메모 (2026-11-05 4차 갱신, 다음 세션용)

작업 방식은 **재구성 + 오라클 비트 검증**이다. 의사코드와 objdump로 함수의 규칙을 읽고, Python 프로토타입(`tools/proto/`)을 Gurobi 오라클과 비트 단위로 맞춘 뒤 Rust로 옮긴다. 의사코드를 줄 단위로 옮겨 쓰는 방식은 호출 인자 손실과 SIMD 조각 때문에 쓰지 않는다. 세부 해독 기록은 [m1_notes.md](m1_notes.md)에 있다.

### 9.1 찾아낸 규칙

- **모델·지문(M0)**: 무한 bound의 내부 값은 ±1e30이다. 지문 해시는 배율을 곱한 double의 비트 패턴을 더한다. MPS 줄은 필드 수로 자유·고정 형식을 정한다(01 §1.5).
- **simplex 작업 배열**: `a' = σ·a·r·s`, `b' = σ·b·r`, `c' = c·s·objscale`, bound는 `/s`다. σ는 `>` 행에서 −1이다. slack은 계수 +1이고 범위는 `<` 행 [0, 1e30], `=` 행 [0, 0]이다.
- **스케일링**: LP·dual의 자동 경로는 ScaleFlag 3이다. 열·행 패스를 최대 10회 하고, 2의 거듭제곱으로 내리며, 한 패스에 2배까지만 바꾼다. 일찍 끝나면 bound 없는 마지막 열 패스를 한 번 더 한다. 목적 스케일은 스케일된 |c|로 위로만 키운다.
- **초기 상태**: 구조 열은 `c_j < 0`이면 상한(−2), 아니면 하한(−1)이다. `u − l ≤ FIXTOL`이면 고정(−4)이다. slack은 기저다.
- **시작 기저**(`1e7531` crash, [m1_notes §5.1](m1_notes.md)): `=` 행을 행 길이로 불안정 정렬(`ab6a5f`)한 순서로 1패스(행 원소가 모두 미표시·`a ≥ 0.1`인 행에 `c/a` 최소 열, 행 열들의 비용 소거) → 2패스(비용 0·자유 열, 길이 < 6) → 3패스(위반 부등식 행에 쌍대 비율 검사). 첫 분해가 기저 위치를 **구조 열은 singleton 피벗 순서, 그 뒤 slack은 행 순서**로 다시 매긴다. 비기저 자유 열은 −3.
- **LU·FTRAN**: 반복 0 기저 64개가 모두 singleton 단계만으로 삼각화된다. L과 U 해법이 모두 열 방향이라, 비트는 피벗 순서와 승수 `a/피벗`, ZERO(1e-13) 건너뛰기로만 정해진다.
- **BTRAN·쌍대값**: y는 c_B 전체를 조밀로 넣은 BTRAN으로 60/60 맞는다(Gurobi가 실제로 조밀로 넘기는지는 미확인). U는 singleton 분해에서 대각뿐이고, 조밀 Lᵀ는 단계 내림차순으로 `|v| > ZERO`일 때만 Lᵀ 행에 흩뿌린다(프로토타입의 열 내적 순서로도 같은 비트). `d_j = c_j − Σ a_ij·y_i`를 열 순서로 계산하고, 상한 비기저는 부호를 뒤집는다. **고정(−4) 구조 열은 `d_j = c_j` 그대로**다(행 방향 비기저 사본에 없음, 347/347 확인). 고정 slack은 계산한다.
- **DSE 초기 가중치**: 정확한 노름 `‖e_pᵀB⁻¹‖²`이다. e_p의 BTRAN 출력은 반복 0에 항상 **희소**이고 행 번호로, Uᵀ는 단계 오름차순, Lᵀ는 단계 공간에서 LU+0x140 = −1(m < 250 등)이면 단계 내림차순 힙, 아니면 Lᵀ DAG 레벨 버킷 순서(버킷 안은 넣은 순서, Lᵀ 행 원소는 단계 내림차순)로 나온다. 제곱합(`1bae9c` L1971–2110)은 그 출력 순서로, 7개 이하면 차례로, 아니면 누산기 8개를 **((0+2)+(4+6)) + ((1+3)+(5+7))**로 합친 뒤(asm `0xc3a26`, 디컴파일러 표시는 틀림) 나머지를 차례로 더한다. 세부는 [m1_notes §6.6](m1_notes.md).
- **오라클 주의**: 자유 열이 있으면 dual 초기화가 자유 열 피벗 루프 안에서 SIMPLEX 콜백을 부른다. 그래서 capri, perold, pilot4, tuff는 반복 0 스냅숏이 xB 계산 전이다. 반복 1 이후로 검증한다.
- **Gurobi 실행**: `gurobi_guard.pth` 때문에 Gurobi는 한 프로세스만 돈다. 다른 세션이 쓰는 중이면 "Gurobi is already running" 오류가 사라질 때까지 기다렸다가 다시 시도한다(5초 간격 재시도 래퍼를 쓰면 편하다).

- **dual 기본 경로(반복 단위)**: Presolve=0·Threads=1의 기본 Method는 64개 모두 dual과 반복 수가 같다(기본 = dual). 기본 ratio test는 SIMPLEXPWRATIO(spx+0x468) = 1이라 **BFRT `1c35c5`**다(Harris `1c137f`는 0일 때). 상수: PIVOTTOL 3e-8, BIGTOL 3e-6, MarkowitzTol 2⁻⁷, PerturbValue 0.0002, 재분해 주기 max(20, ⌊m^0.7⌋). 세부는 [m1_notes §8](m1_notes.md).
- **CHUZR은 점수 최대가 아니다**: 시작 모드(P+0x40 = 0)는 `15d2d`가 만든 순서 목록(P+0x10, 길이 P+0x3c)을 커서(spx+0x2c4)부터 걸으며 표시(spx+0x280)가 0인 첫 위치를 고른다. 목록이 소진되면 다시 만든다. 위반²/w 최대로 고르면 2/60만 맞는다.
- **인공 bound와 BFRT**: dual은 ±1e30을 실제 bound로 쓴다. sc50a 반복 0에서 Gurobi는 비율 0인 열 15를 인공 상한 1e30으로 flip하고(기울기 2.1e30 − 1e30 > 0) 열 3을 넣는다. 1e30 규모의 보폭 뒤에는 x_B를 새로 계산해야 반올림 잔여(1e14–1e16)가 남지 않는다.
- **가중치 폭주**: DSE 가중치가 inf/NaN이 되면 CHUZR가 위반 행을 놓쳐 실행불가능한 x를 OPTIMAL로 낸다(grow22, fit2p에서 실제로 발생). 1e30 초과·NaN이면 1로 되돌린다(Gurobi도 같은 규칙).
- **Netlib 개수**: `re/bench/data/netlib`의 104개 항목 중 MPS는 **91개**(라이선스 안 64 + 밖 27)이고 13개는 빌드 파일이다. 09 로드맵과 goal의 "104개/40개"는 이 항목 수를 센 것이다. 점검은 91개로 한다. 빠진 Netlib 문제(qap8·12·15, stocfor3, truss, pilot.ja, pilot.we 등)는 저장소에 없다.

### 9.2 지금 상태 (2026-11-05)

코드:
- `crates/lp`: `scaling.rs`(64/64 비트), `spx.rs`, `common.rs`(`aabbe3` 정렬), `pricing.rs`(CHUZR 모드 0·표시·비용 가중, 모드 1은 근사), `start.rs`(crash, `vstat` 64/64), `lu.rs`(singleton 단계 원본대로 + Markowitz(단순 탐색, shortcut) + Forrest–Tomlin 갱신, 원본 BTRAN 경로 에뮬레이션), `simplex.rs`(엔진: dual·primal·드라이버 `solve_model`).
- dual은 원본 구조(crash → 자유 열 피벗 → 첫 분해 → 정확한 DSE 노름 → 반복)를 따르지만 반복 내부는 아직 교과서식이다: CHUZR는 위반²/w 최대, ratio test는 Harris 묶음 단위 BFRT + 비용 이동, 섭동은 퇴화 50회 연속이면 1회(난수는 LCG). 원본 규칙이 아니라 반복 경로가 다르다.
- primal은 phase 1(위반 합)·phase 2, Dantzig 가격, Harris 비율 검사의 교과서식이다(`0478d3` 계열 미이식). Method=0과 dual 뒤 정리에 쓴다.
- Python: 읽은 LP와 API로 만든 LP 모두 `optimize()`가 새 엔진을 쓴다. MIP는 M2까지 오류(배낭 테스트 xfail). `crates/core` 삭제 완료. SIMPLEX 콜백은 풀이 뒤 반복 기록으로 재생한다. `Work`는 아직 0이다.
- cargo clippy 경고 0(`crates/lp`는 `needless_range_loop`만 허용), cargo test 5개·pytest 8개 통과(1 xfail).

| 기준 | 현재 | 비고 |
|---|---|---|
| (1) 정확성 Method 1 | Netlib 64개 62/64(bandm·pilot4 시간 초과: dual 경로가 갈린 뒤 교과서식 primal 정리가 정체) | 91개 전체는 미점검(dfl001은 release 300초 초과, pilot·pilot87 미확인) |
| (1) 정확성 Method 0 / 기본 | 미점검 | 기본 = dual 경로. Method 0(교과서식 primal)은 25fv47에서 300초 초과 |
| (2) P1 반복 수 | 11/64 일치, ±5% 안 37/64, 비율 기하평균 1.025. 피벗 경로 전체 일치 2/64, 첫 분기 위치 중앙값 7% | `sh tools/m1check.sh`(→ `bench/lp_check.csv`, `bench/pathcmp.txt`). 반복 수가 같아도 경로는 대부분 일찍 갈린다 |
| (3) P2 work | 미구현 | 작업량 계수 spx+0x148(1.3, m > 10000이면 보정), 커널마다 연산 수 누적 |
| (4) P3 시간 | 미측정 | release에서 25fv47 0.5–0.7초 대 Gurobi 0.09초. 반복마다 O(n+m) 루프와 조밀 FTRAN/BTRAN이 원인 |
| (5) 구조 | LU(Markowitz·FT), crash, dual·primal, 드라이버 일부 | Quad(F80) 미착수, `0c3d0d` 분기(웜스타트·network·sifting·비스케일 재풀이) 미이식, common(난수 `a7b4d6`/`a7b54f`) 미이식 |

### 9.3 다음 단계 (순서대로)

완료(4차 세션, [m1_notes §8.8](m1_notes.md)): 64개 피벗 경로 오라클(`bench/oracle/path1/`, `tools/proto/pathcmp.py all`), 자유 열 루프 원본화(첫 최대 위치, 피벗마다 반복 수, 남은 열 2차 패스, 루프 뒤 재분해: perold 4819 → 1271회, Gurobi 1273), 재분해 주기 m^0.7(Full pricing 때 m^0.65), 재분해마다 위치 재번호(옛 WIP 패치), 행 사본(`a075e`·`a104a`·`a114f`)과 α 벡터 순서(`0a4109`: 1.8 규칙, 처음 닿은 순/열 순, slack은 ρ 순). `48d5f` 전체 명세는 [m1_notes §8.9](m1_notes.md)(미검증).

1. **LU 갱신 뒤 BTRAN 출력 순서**: `tools/proto/alphacmp.py <name>`로 보면 grow7·scsd1·e226·lotfi 모두 BFRT 첫 호출의 α는 순서·비트가 같고, LU 갱신 1회 뒤 호출부터 ρ 순서가 다르다(Gurobi는 희소 출력에 나가는 행이 먼저, 우리 `btran_general`은 조밀 뒤 행 순서). Forrest–Tomlin 갱신 저장(`5b257` L860–975)과 `221173`의 갱신 경로(Uᵀ 희소 → 행 eta → Lᵀ)를 원본대로 옮긴다. 옛 2·3번(Markowitz `036ade`, FT 회계 L+0x184)이 여기에 묶인다. 재분해 시점도 이 회계에 달려 있다.
2. **`48d5f` 이식**(§8.9): 보통의 재분해 뒤에는 flip이 아니라 비용 이동(SHIFT), 첫 분해 뒤 RANDSMALL, 이유 4 등에서 FLIP, 섭동 GLOBAL과 그 발동 조건(0x2c0 % K, 퇴화 비율), RNG(`a7b4d6`·`a7b54f`·`895c5`, common에 둔다). 지금 엔진은 박스 열만 flip하고 나머지 쌍대 위반을 남겨 maros 같은 경로가 흔들렸다. 오라클의 `pert`·`pert_level` 필드로 검증한다.
3. **primal 원본 이식**(`0478d3` 계열): bandm(인공 bound 단계 뒤 상태 5 상황, 지금은 shortcut으로 dual을 이어 감)과 pilot4(섭동 제거 뒤 정리)가 교과서식 primal에서 정체한다. Method 0 정확성에도 필요하다.
4. BFRT 나머지(코퍼스 불일치 18건, 후보 없음 경로), 행 사본의 같은 |a| 분할(`9edd1`), CHUZR 모드 1 열 원소 처리, P2 work, P3 속도(희소 FTRAN/BTRAN), Quad.

### 9.4 기준값·오라클 위치

| 경로 | 내용 | 만드는 법 |
|---|---|---|
| `bench/gurobi_dual_nopre/`, `gurobi_primal_nopre/`, `gurobi_default_nopre/` | Gurobi Presolve=0, Method 1/0/기본, 64개 summary·trace | `tools/parity.py capture gurobipy … --lp-only --param Presolve=0 [--param Method=…]` |
| `bench/gurobi_spx0/` | 반복 0의 스케일·`vstat`·`xB`·`d`·`w`(테스트 기준값) | `tools/spx_oracle.py` |
| `bench/oracle/iter0/`, `iter0-3/` | 반복 0(64개, 행렬 포함), 반복 0–3(작은 LP 12개) 전체 스냅숏 | `tools/spx_oracle.py --iters …` (이제 `price` 필드 포함) |
| `bench/oracle/btran0/` | Gurobi 내부 BTRAN(`9e288`)을 ctypes로 불러 뜬 e_p 출력 | `tools/proto/btran_oracle.py` |
| `bench/oracle/path1/` | 작은 LP 12개의 반복별 진입·이탈·flip 수(dual) | `tools/path_oracle.py <dir> --method 1 name…` |
| `bench/lp_check.csv`(만들 예정) | 91개 × Method 정확성·반복 수·시간 | `tools/lp_check.py` |
| `docs/m1_notes.md` §5–8 | crash, LU, BTRAN·DSE, CHUZR 목록, BFRT 해독 기록 | |

프로토타입 `tools/proto/{lu_singleton,btran_dual,dse_weights}.py`는 `bench/oracle/iter0`을 직접 읽는다(인자 없음).

### 9.5 주의점

- **빌드는 `maturin develop --release`**(PATH에 `~/.cargo/bin` 필요). `.venv`의 maturin은 main 트리만.
- **Gurobi는 한 프로세스만**. 오라클·추적(`fn_trace.js`, `spx_oracle.py`, `tools/bfrt_corpus.sh`)도 Gurobi를 띄우므로 차례로 돌린다.
- **추적기**: `node tools/fn_trace.js <spec.json> <out.jsonl>`. ptrace_scope=1이라 frida.spawn 방식이다. 필드 코드는 파일 머리 주석 참고(`i:`·`P*`·`L*`·`SV`·`AX`·`XR`·`OUT`·`A`/`PA`).
- **반복 비교**: `RSOLVER_SNAP=<file>`로 엔진 스냅숏, `python tools/proto/statecmp.py <name> <spx_oracle.py 출력>`. d는 Gurobi 규약(상한 부호 반전, 기저 변수 잔여 유지)으로 저장된다.
- **점검**: `sh tools/m1check.sh`(Herdr pane, 5분 안팎): lp_check(Method 1) → itstat → pathcmp all. 오라클 비교 `tools/spx_oracle.py`는 이제 LU 갱신 수·CHUZR 허용량(`lu`, `price.allow`)도 남기고, `statecmp.py`는 위치 대신 **변수 기준**으로 x_B·w·목록을 비교한다.
- `git checkout <file>`로 실험을 되돌릴 때 커밋 안 한 다른 변경까지 날아간다(이번 세션에 한 번 겪음). 실험 전 커밋하거나 패치로 저장한다.
- 디버그 출력: `RSOLVER_DEBUG=1`이면 dual 종료 상태와 해의 bound 위반을 stderr로 낸다.
