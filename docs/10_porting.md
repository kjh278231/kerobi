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

`tools/port_map.py`가 코어 함수 4,362개 전부를 설계 모듈에 배정하고, Rust 코드의 `port:` 태그를 세어 진척도를 낸다.

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
4. **태그**: 함수 doc comment에 `/// port: 6e8eb3@0x5b29d0`를 단다. 여러 원본을 합치면 태그도 여러 개다. double판과 x87 long double판(노트 13 §0.1)처럼 같은 이름을 쓰는 두 벌은 제네릭 함수 하나로 옮기고 태그 두 개를 단다.
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
| 원본이 벽시계로 판단하는 지점(예: LU 재분해의 "경과 시간 > 14×분해 시간", 비결정 concurrent)은 그대로 옮기되 P1에서 그 지점 뒤의 차이를 허용하고 기록한다. work로 판단하는 지점은 정확히 맞춘다 | 원본 자신도 그 지점에서는 실행마다 다를 수 있다 |

## 5. 자료구조, 메모리, 외부 라이브러리

- 자주 쓰는 데이터는 원본 배치를 따른다: 배열 구성, 연결 리스트(MIP presolve), CSC/CSR 쌍, LU 저장 형식, 노드 압축 형식. 캐시 동작이 같아야 P3가 맞는다. 함께 읽히는 배열은 Rust에서도 한 구조체에 둔다.
- 원본이 한 번 크게 잡아 재사용하는 버퍼(예: `spx` 0xf20 바이트, LU 객체, pricing 객체)는 Rust에서도 엔진 수명 동안 재사용한다. 반복마다 `Vec`를 새로 만들지 않는다.
- MKL: 원본은 MKL을 정적 링크한다(노트 03 §4). rsolver는 `intel-mkl-src`(static, lp64)로 같은 라이브러리를 링크한다. 원본의 MKL 스레드 수와 재현성(CBWR) 설정은 미확인이다. barrier를 포팅할 때 `mkl_*` 설정 호출부를 찾아 같게 맞춘다.
- 스레드: 원본의 스레드 생성과 동기화(alive spin, 라운드 경계 병합, worker 상태 0–3)를 그대로 옮긴다. rayon 같은 작업 훔치기 풀은 쓰지 않는다. 일의 배분이 실행마다 달라져 결정성이 깨진다.

## 6. 차등 검사 하네스 (`tools/parity.py`, M0에서 만든다)

- **라이선스 한계**: `.venv`의 gurobipy는 pip 크기 제한 라이선스다(변수·선형 제약 각 2,000개, `re/bench/bench.py:GRB_LIMIT`). Netlib 64개(`lp.csv` 전부)와 `miplib_small` 38개 전부가 이 안에 들어가므로 실시간 나란히 실행이 된다. Netlib의 나머지 40개는 저장된 공개 최적값과 비교만 한다.
- **실행**: 같은 MPS, 같은 파라미터(Threads=1, Seed=0, 나머지 기본값)로 Gurobi와 rsolver를 돌린다.
- **수집**:

  | where | 값 |
  |---|---|
  | PRESOLVE(1) | PRE_COLDEL, PRE_ROWDEL, PRE_SENCHG, PRE_BNDCHG, PRE_COECHG |
  | SIMPLEX(2) | SPX_ITRCNT, SPX_OBJVAL, SPX_PRIMINF, SPX_DUALINF, SPX_ISPERT |
  | BARRIER(7) | BARRIER_ITRCNT, BARRIER_PRIMOBJ, BARRIER_DUALOBJ, BARRIER_PRIMINF, BARRIER_DUALINF, BARRIER_COMPL |
  | MIP(3) / MIPSOL(4) | MIP_NODCNT, MIP_OBJBST, MIP_OBJBND, MIP_SOLCNT, MIP_CUTCNT, MIP_ITRCNT / MIPSOL_OBJ, MIPSOL_NODCNT |
  | 로그 | "Presolved: …", "Root relaxation: …", "Cutting planes:" 절, "Explored … nodes", work 단위 |
  | 모델 | `Model.presolve()`를 MPS로 쓴 것(P0) |

- **첫 분기점**: 두 추적이 처음 갈라지는 지점(예: SPX_ITRCNT k에서 목적값이 다름)을 찾아, 그 구간의 함수부터 원본과 대조한다. 결과는 인스턴스별 한 줄로 `rsolver/bench/parity.csv`에 쌓는다.
- Gurobi 콜백은 매 반복이 아니라 간격을 두고 불린다. 그래서 반복 수가 같은 지점끼리만 비교한다.

## 7. 포팅하지 않는 것

- `excluded` 100개: 라이선스 검사, 크기 제한, WLS, 토큰 서버, Compute Server, Cluster Manager, Instant Cloud, batch, 튜닝. 원본이 라이선스 상태로 동작을 바꾸는 분기(예: `bb9b40`, "Solve interrupted (license invalid)")는 옮기지 않고 분기 자체를 없앤다.
- GPU, 녹화·재생, 분산 MIP/concurrent, GBT 기반 Method 선택(`171e36`, 기본 비활성).
- `extern`: MKL은 링크해서 쓰고, OpenSSL·libcurl·mbedTLS는 쓰지 않는다.

## 8. 법적 위치

decomp를 옮긴 코드는 Gurobi 코드의 2차적 저작물이다. 저장소는 비공개로 두고 배포·공개하지 않는다(내부 연구용). 공개가 필요해지면 해당 코드는 이 문서의 규칙이 아니라 클린룸 방식으로 다시 써야 한다([09 §4](09_roadmap.md)).
