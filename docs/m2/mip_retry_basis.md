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

(실행 후 기록)

## 7. 판정과 인수인계

(실행 후 기록)
