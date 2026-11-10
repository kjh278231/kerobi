# M2-03c-2 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 선행 결과는 [M2-03c-1](mip_linear_child_trace.md) §10–13이다.

## 1. 범위와 주장

- 작업 종류: 오라클 구축(읽기 전용 정상 실행 훅). Rust 구현은 제외한다.
- 질문 세 가지:
  1. **입력 basis 출처**: 자연 상태 12 재풀이에서 `cda5b`에 들어가는 vbasis·cbasis는 어디에서 왔는가?
  2. **재풀이 후 VBasis·CBasis**: 재풀이가 끝난 LP의 basis를 원본이 VBasis/CBasis 형식으로 내보내는가? 내보내면 그 값은 무엇인가?
  3. **factor·norm**: 재풀이 simplex가 basis를 실제로 적재하는가? 분해 호출 횟수와 순서는? dual norm 입력이 있는가?
- 사례: M2-03c-1과 같은 입력이다. `neos-3754480-nidda` stress(NodeLimit 6000, 비루트 노드 8287), `gmu-35-50` default(NodeLimit 20000, 루트). snowy 상태 3 사례는 이번 범위에서 뺀다.
- 제외(미검증으로 남김, 결과를 보기 전에 고정):
  - LU 내부 내용(L·U 값, pivot 순서, eta).
  - pricing 가중치 배열 값, pricing 모드 필드, NormAdjust 분기.
  - `ws+0x168` 음수 코드를 사람이 해석해 만든 VBasis. 원본 export 함수의 출력만 VBasis로 인정한다.
  - 상태 10, 자연 5→12, `682037` 진입 predicate(M2-03c 범위에 남김).
- 선행: M2-03c-1 병합본 main `f414f6f`. 미커밋 선행 변경은 없다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 Linux x86-64, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- 디컴파일(`../re/decomp/c/`, 주소는 file offset):

| 함수 | file offset | 근거로 쓰는 부분 |
|---|---|---|
| `5ce67b` | `0x4f0e10` | C:187 `I = N+0x18`. C:262–275 진입 시 `I+0x98` 해제·`I+0xa0=0`. C:290–297 `I+0xb0≠0`이면 `69d7ea`. C:785–933 재풀이: `I+0x98==0` 또는 `I+0xa8≠0` 등이면 `0c3d0d` 경로, 아니면 `cda5b(lp, m, I+0x98, I+0xa0, 0,0,0,0, timer)` 직접 호출. C:1358 LP 상태 2면 `5d4427`, C:1635 상태 9면 `5d3deb` |
| `5ccea5` | `0x4f92d0` | C:204–345 첫 LP. workspace가 없으면 `I+0x98/0xa0/0xa8`로 basis·dual을 적재. 있으면 `cda5b(lp, m, I+0x98, I+0xa0, …)` |
| `69d7ea` | `0x579610` | `I+0xb0` 압축 basis를 `I+0x98`(열, 0/−1/−2/−3)과 `I+0xa0`(행, 0/−1)로 푼다. 열은 2비트, 행은 1비트이고, 32비트 단어 MSB부터 이어지는 비트열이다. 끝에서 `I+0x94=행 수`로 두고 `I+0xb0`를 해제한다 |
| `69d237` | `0x579300` | `I+0x98/0xa0`을 `I+0xb0`에 압축한다. 단어 수는 `(행+2·열+31)>>5`이고, 이후 `I+0x98`를 해제한다 |
| `69e299` | `0x57a210` | `(child, ?, parent)`. 부모 `I+0xb0`를 자식에 memcpy한다. 부모에 `I+0x98`가 있으면 풀린 배열을 복사한다 |
| `5d3deb`·`5d4427` | `0x4f8470`·`0x4f6a50` | LP 뒤 `113d92(lp, I+0x98, I+0xa0)`로 노드에 basis를 저장한다. HasDualNorm이면 `I+0xa8`에 dual norm을 저장한다 |
| `113d92` | `0x1a6300` | 원본 basis export. CBasis는 `ws+0x168[n+i]<0 ? −1 : 0`, VBasis는 `12f01a`→`113674`(`lp+0x64==1`이거나 `ws+0x1d8` bit0=0이면 0x2715) |
| `cda5b` | `0x153cc0` | C:142–190. `2f7e1` 뒤 `ws+0xf0==1`이고 vbasis·cbasis가 모두 있으면 `325b4`로 적재한다. 없으면 `lp+0x238`(`10a423`) 경로를 쓴다 |
| `325b4` | `0xd3150` | 인자 basis를 `ws+0x168`에 적재 |
| `036ade` | `0xd4190` | LU 분해([M1 노트](../m1/m1_notes.md) §6) |

- 시작 경계: 재풀이 블록(`0x4f53a8`)에 도달한 `5ce67b` 호출의 진입. 끝 경계: 그 호출의 반환. 노드 basis 출처는 같은 `I`에 대한 마지막 `69d237`/`69e299` 쓰기와, 그 원천 export까지 실행 중에 연결한다.
- 설정: Threads=1, Seed=0, TimeLimit 없음(M2-03c-1 §11 근거).

## 3. 비교 계약

| 필드·사건 | 의미 | 규칙 | 캡처 |
|---|---|---|---|
| 기존 03c-1 사건 투영 | 새 훅이 궤적을 바꾸지 않음 | 새 캡처에서 03c-1 필드만 뽑은 결과가 저장본 `bench/oracle/mip_linear_child_retry/*_retry_1.json`과 JSON 전체 일치 | 같은 사건 버퍼 |
| `69d7ea` 진입 압축 단어·`I+0x94`·열 수 | 입력 basis의 압축 원천 | 정수 정확 | 진입 읽기 |
| `69d7ea` 반환 뒤 `I+0x98/0xa0` 내용 | 원본 decode 결과 | 정수 정확. Python decoder 재계산과도 일치해야 함(교차 확인) | 반환 읽기 |
| 첫 LP·재풀이 `cda5b` 인자 | 별칭·내용 | 포인터가 `I+0x98/0xa0`과 같은지(참거짓), 내용이 decode 결과와 같은지 정확 비교. **양쪽 null은 합격이 아님** | 진입 읽기 |
| 압축 원천 사슬 | `I+0xb0`를 쓴 마지막 함수·반환 주소·노드 id·내용 해시, 복사면 부모 쪽 사슬 | 해시·정수 정확. 끊긴 사슬은 끊긴 지점과 함께 기록 | 전역 훅 |
| `325b4` 적재 | 재풀이가 인자 basis를 실제 적재 | 호출 여부, 인자 포인터, 적재 직후 `ws+0x168` 비음수 개수 | 진입·반환 |
| `ws+0xf0`, `I+0xa8` null 여부, `cda5b` 인자 5–8 | 적재 조건·norm 입력 | 정수·null 정확 | 재풀이 진입 |
| `036ade` 호출 | 재풀이 중 분해 횟수·순서 | 순서·횟수 정확 | 재풀이 안 사건 |
| 자연 export (`113d92`, `5d3deb`, `5d4427`) | 재풀이 후 원본이 VBasis를 만드는가 | 호출 여부·반환 코드·내용 정확. 호출이 없으면 "원본이 이 경로에서 VBasis를 만들지 않음"으로 판정 | 재풀이 solve 안 훅 |
| export 가능 조건 | `lp+0x64`, `ws+0x1d8`, `lp+0x238` null 여부 | 정수 정확 | 재풀이 반환 |
| 합성 export (별도 실행) | `113d92`를 재풀이 반환 시 직접 호출한 결과 | 반환 코드·배열 정확. **합성으로 표시하고 자연 증거로 쓰지 않음** | NativeFunction |

- 수치 필드는 다루지 않는다. 비교는 모두 정수·포인터 관계·해시다.
- 원본 3회 독립 실행에서 사건 전체가 일치해야 한다. 포인터 값은 저장하지 않고 관계(같음/다름/null)만 저장한다.

## 4. 판정 규칙 (결과 확인 전 고정)

- 질문 1 통과: 두 사례 각각 재풀이 `cda5b` 인자(또는 gmu처럼 인자가 없으면 그 사실과 `lp+0x238` 상태)가 원천까지 3회 같은 사슬로 연결된다.
- 질문 2:
  - 통과: 자연 export가 재풀이 뒤 실행되고 3회 일치한다.
  - "원본 미생성": 자연 export가 실행되지 않으며, 그 사실과 가능 조건이 3회 일치한다. 이 경우 합성 export 결과는 참고 자료일 뿐 질문 2를 통과시키지 않는다.
- 질문 3 통과: `325b4` 적재 여부, `ws+0xf0`, 분해 호출 순서·횟수, norm 입력 null 여부가 3회 일치한다. LU·가중치 내용은 미검증이다.
- 검사기는 대조군(저장본 투영 일치)과 목표 검사를 분리한다. 목표 사건이 0회이거나, 필수 필드가 null끼리 같거나, 합성만 존재하면 0이 아닌 값으로 끝난다.

## 5. 재실행 명령

- 캡처: `tools/mip_retry_basis.js natural|synth MODEL OUTPUT 'PARAMS'`. 원본 프로세스 하나를 Frida로 띄운다.
  - `natural`: 읽기 전용이다. 03c-1 `retry` 모드의 사건을 그대로 만들고 `x:` 사건을 덧붙인다.
  - `synth`: `natural`에 더해 재풀이 블록의 최상위 LP 호출이 반환될 때마다 원본 `113d92`를 별도 버퍼로 직접 부른다(`x:synth_export`, `synthetic: true`).
  - 같은 함수에 리스너를 둘 붙이면 onLeave가 빠졌다(탐색 실행에서 확인). 그래서 주소마다 리스너를 하나만 둔다.
- 전체 재실행: `tools/mip_retry_basis_runs.sh bench/oracle/mip_retry_basis`. 두 사례 × natural·synth × 3회, 총 12개 원본 프로세스를 한 번에 하나씩 돌리며 약 25분 걸린다. TimeLimit은 쓰지 않는다.
- 판정: `python3 tools/mip_retry_basis_check.py --self-test`. controls와 acceptance를 함께 돌리고 변조를 넣어 거부되는지 확인한다. 기대 종료코드는 0이다. `controls`나 `acceptance`만 인자로 줄 수도 있다. manifest는 `python3 tools/mip_retry_basis_check.py --write-manifest`로 다시 만든다.
- 회귀: 솔버 코드를 바꾸지 않으므로 Rust·Python 통합 검사는 대상이 아니다. 기존 03c-1 검사(`tools/mip_linear_child_retry_check.py --self-test`)는 저장 자료가 그대로이므로 계속 0이어야 한다.

## 6. 실행 결과

- 환경: 작업 branch `work/m2-03c-2`, Gurobi 13.0.3 위 해시, Node Frida, 공유 `../.venv`의 gurobipy. 입력 모델 해시와 캡처 14개 파일 해시는 `bench/oracle/mip_retry_basis_manifest.json`에 있다.
- 실행: `tools/mip_retry_basis_runs.sh bench/oracle/mip_retry_basis`. 12개 원본 프로세스가 모두 정상 종료했다(2026-11-10 14:48–15:10).
- 판정: `python3 tools/mip_retry_basis_check.py --self-test`의 종료코드는 0이다.
  - manifest PASS(14개).
  - controls PASS 12건. 12개 캡처 모두 03c-1 사건으로 투영하면 저장본 `*_retry_1.json`과 JSON 전체가 같고, 최종 출력(`DONE 8 6000`·`DONE 8 20000`)도 같다. 같은 사례·모드의 3회는 사건 전체가 같다.
  - acceptance PASS.
  - 메모리 변조 17종을 모두 의도한 검사에서 거부했다. 대상은 바이너리 해시, 빈 사건, 투영, 실행 간 차이, null끼리 같음, 별칭, decode, 사슬, 상태 재설정, 적재, 자연 export 제거, export null, setbasis, 68341b 구간, 합성만 존재, 합성 누락, 가짜 자연 export다.
  - 디스크 캡처 1바이트 변조는 종료코드 1, 복원하면 0이다.
- 첫 실행 실패(보존): 도구 v1로 돈 12회 중 gmu synth 2회차만 사건 1번(`x:solve_basis.provenance`)이 달랐다.
  - 원인: 노드 정보 `I` 포인터로 찾는 마지막 쓰기 기록이 heap 주소 재사용 때문에 다른 노드의 오래된 기록을 가리켰다. 진입 시 `I+0xb0`는 null이었다.
  - 수정: 현재 `I+0xb0` 내용 해시가 기록과 같을 때만 기록을 인정하고, 아니면 `stale`로 표시한다. 수정 뒤 12회를 다시 캡처했다.
  - 실패 파일 두 개와 v1 해시 목록은 `bench/oracle/mip_retry_basis_v1_failure/`에 있다. 판정 규칙은 바꾸지 않았다.

### 질문 1. 입력 basis 출처

**nidda stress (비루트 노드 8287, 3회 동일)**

| 순서 | 사건 | 근거 |
|---|---|---|
| 1 | 노드 8278의 `I+0x98/0xa0`는 `5d4427` 안의 `113d92` export(반환 `0x4f707f`)가 쓴 basis다. `5d4427`은 C:1358에서 LP 상태 2일 때 호출된다(이 조건은 C 근거이며 캡처하지 않았다) | 8278 저장 내용 해시 = 그 export 해시 |
| 2 | `8e4682`가 `69e299`(반환 `0x722780`)로 8278의 풀린 basis를 노드 8287에 복사한다. C 근거: `8e4682` C:56 `local_90 = *(8278의 I)`는 8278과 같은 `M`이다. C:200 `5ff358(0, M)`이 부모 없이 8287을 만들고, C:282–298이 basis를 복사한다. 즉 별도 트리가 아니라 같은 `M`의 부모 없는 노드다. 03c-1의 부모 null·깊이 0은 이 생성 경로와 맞지만, 생성 자체는 캡처하지 않았다 | 복사 결과 해시가 decode 결과와 같다 |
| 3 | `69d237`(반환 `0x722794`)이 압축해 `I+0xb0`(31단어)에 쓰고 `I+0x98`를 해제한다 | 압축 해시 = decode 직전 `I+0xb0` 해시 |
| 4 | `5ce67b` 진입에서 `69d7ea`가 `I+0xb0`를 VBasis 형식으로 푼다. 열 253(0: 100, −1: 137, −2: 16), 행 464(0: 364, −1: 100) | Python decoder 재계산이 원본 출력과 같다 |
| 5 | 첫 LP `cda5b`(`5ccea5`, 반환 `0x4f9626`, method 1)와 재풀이 `cda5b`(`5ce67b` C:929, 반환 `0x4f5b08`, **method 0**)가 둘 다 `I+0x98/0xa0` 포인터를 그대로 받는다. 내용도 같다 | 별칭·내용 정확 비교 |

- 결론: 재풀이는 실패한 LP의 마지막 basis에서 이어 가지 않고 **노드에 저장된 시작 basis**에서 다시 시작한다. 시작 basis는 첫 LP와 같다. 재풀이 method는 C:925 `local_110 = (param_3[3]==0)`로 정해지며 이번 사례에서 0(primal)이었다. 첫 LP는 1이었다.
- `8e4682`(호출자 `8e3397`)가 어떤 휴리스틱인지는 해석하지 않았다. 8287은 `5ce67b` 반환 뒤 C:420 `69f77c`로 정리되는 것으로 보이며, 이 부분은 캡처하지 않았다.

**gmu-35-50 default (루트, 3회 동일)**
- 첫 LP와 재풀이 첫 `0c3d0d` 안의 첫 `cda5b`(`0x14c192`)는 cold start다. vbasis·cbasis 인자, `lp+0x238`, workspace가 모두 없다. 단, `0c3d0d` 안 `cda5b`의 첫 인자가 노드 LP와 같은 객체인지는 기록하지 않았다. 진입 시 LP 상태가 0으로, `lpdriver:enter`의 12와 다르다. 그래서 이 cold start를 노드 LP 자체의 cold start로 해석하지 않는다.
- `0c3d0d` 안의 둘째 `cda5b`(`0x14fe6d`)는 basis와 primal start(인자 5·6)를 받는다. 적재된 header가 인자와 같다. 이 basis는 `0c3d0d` 내부에서 만들어지며(uncrush로 추정) 출처는 추적하지 않았다.

### 질문 2. 재풀이 후 VBasis·CBasis

- **nidda: 원본이 만들지 않는다.** 재풀이 뒤 LP 상태 6(cutoff)이다. 재풀이 블록부터 반환까지 자연 `113d92`·`5d3deb`·`5d4427` 호출이 0회다(3회 동일).
  - export 가능 조건은 `lp+0x64=6`, `ws+0x1d8=7`(bit0=1), `lp+0x238` null이었다.
  - 합성 직접 호출(`113d92` rc 0)의 결과는 열(0: 84, −1: 136, −2: 33), 행(0: 380, −1: 84)이다. 입력 basis와 비교해 열 70칸, 행 64칸이 다르다. 이 값은 참고 자료이며 자연 증거로 쓰지 않는다.
- **gmu: 원본이 만든다.** 재풀이 첫 `0c3d0d`가 반환되면(LP 상태 2) `5d4427`(`5ce67b`의 `0x4f1b84`)이 `113d92`(반환 `0x4f707f`, rc 0)로 export해 노드 `I+0x98/0xa0`에 저장한다. 열 53(0: 30, −1: 14, −2: 9), 행 110(0: 80, −1: 30)이다. 같은 지점의 합성 export와 해시가 같다.
- 이어서 같은 `5ce67b` 호출에서 `68341b`(DegenMoves 문자열 사용, `5ce67b` 반환 `0x4f2a37`)가 실행된다.
  - `10acb2`(`68341b` C:933, 반환 `0x56ac2d`)로 노드 `I+0x98/0xa0`를 `lp+0x238`에 넣는다.
  - 그 뒤 Method 0 `0c3d0d`의 `cda5b`(`0x1502d6`)는 인자 없이 `lp+0x238`에서 basis를 적재한다. 적재된 header는 자연 export와 같다.
  - 즉 둘째 LP는 재풀이 export basis에서 warm start한다. 이 LP 뒤의 합성 export는 열(0: 30, −1: 23)이고 자연 export는 없다.

### 질문 3. 적재·분해·norm

| 사례·호출 | `2f7e1` 뒤 `ws+0xf0` | `325b4` 적재 | `036ade` 분해 횟수 | 인자 5–8·`I+0xa8` |
|---|---|---|---|---|
| nidda 첫 LP | 1 (workspace 신규) | `I+0x98/0xa0`, header = 노드 basis | 4 | 모두 null·null |
| nidda 재풀이 | 12→**1** | 같은 `I+0x98/0xa0`, header = 노드 basis | 5 | 모두 null·null |
| gmu 첫 LP | 1 | 없음 (cold) | 5 | null·null |
| gmu 재풀이 `0c3d0d` 1단계 | 1 | 없음 (cold) | 6 | null |
| gmu 재풀이 `0c3d0d` 2단계 | 1 | 인자 basis | 1 | 5·6 있음, 7·8 null |
| gmu `68341b` Method 0 | 1 | `lp+0x238` = 자연 export | 1 | null |

- nidda 재풀이에서는 `2f7e1`이 상태 12인 workspace의 `ws+0xf0`를 1로 되돌린다(`cda5b` C:182–190 조건). 그러면 `325b4`가 노드 basis를 다시 적재하고 분해를 새로 한다. 재풀이에 들어가는 dual norm은 없다. 인자 7·8이 null이고 `I+0xa8`도 null이다.
- LU 값·pivot 순서, pricing 가중치·모드는 §1에서 미리 정한 대로 미검증이다. 분해 횟수는 호출 횟수일 뿐이며 각 분해의 이유(초기·재분해)는 구분하지 않았다.

## 7. 판정과 인수인계

- 작업 수행 상태: 완료(오라클 구축). 동등성: Rust 미변경으로 **미검증**.
- 통과한 정확한 범위:
  - 질문 1: 두 사례 모두 3회 같은 사슬로 연결됐다. nidda는 export→copy→pack→decode→두 `cda5b`이고, gmu는 cold start다.
  - 질문 2: nidda는 "원본 미생성", gmu는 자연 export 통과다.
  - 질문 3: 적재 여부, 상태 재설정, 분해 호출 횟수, norm 입력 null을 3회 확인했다.
  - 모든 결과는 두 입력·설정의 원본 실행에 한정된다.
- 남은 한계:
  - LU·가중치 내용.
  - gmu `0c3d0d` 내부 2단계 basis의 출처.
  - `8e4682`·`68341b`의 의미.
  - 상태 10, 자연 5→12, `682037` 진입 predicate(M2-03c에 남김).
  - Rust 재풀이를 원본과 맞추는 구현. 현재 Rust는 자식 Numeric을 그대로 둔다.
- 후속 후보: Rust 재풀이 구현이다. 입력은 노드 저장 basis(`I+0x98/0xa0`), method는 `param_3[3]` 규칙, 재풀이 뒤 상태 2이면 basis를 저장한다. 구현 전 이 오라클의 nidda·gmu 사건을 Rust 계측과 맞출 비교 경계를 먼저 정한다.
- 교훈 기록: [원본 사실](../lessons/original.md)의 노드 basis·export·재풀이, [도구](../lessons/tooling.md)의 Frida·Herdr·검사기, [절차](../lessons/process.md).
- 다음 실행 명령: `python3 tools/mip_retry_basis_check.py --self-test`(저장 기준). 원본 재캡처는 `tools/mip_retry_basis_runs.sh <출력>` 후 같은 검사기를 쓰거나 저장본과 비교한다.
