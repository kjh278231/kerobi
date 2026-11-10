# M2-06b-2 검증 계약과 결과: 힙 키 식(`627f3e`)

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 원본 규칙과 06a 추적은 [M2-06a](mip_node_select_spec.md), Rust 힙은 [M2-06b](mip_node_heap_rust.md)다. M2 인수인계의 첫 다음 작업이고, 사용자의 "다음 스프린트 진행" 지시로 착수했다(2026-11-11).

## 1. 범위와 주장

- 작업 종류: 조사·오라클·구현(L1 저장 기준 재생).
- 완료 후 주장할 것:
  - 원본이 분기 자식의 힙 키 `I+0x50`을 쓰는 곳 전부와 각 식의 입력(§2).
  - `M+0xa4`·`M+0x40`의 의미(§2.3).
  - Rust `node_heap::child_key`·`deep_key`가 캡처한 모든 키 쓰기의 입력에서 원본 키와 비트까지 같은 값을 낸다(실행된 경로·분기 한정).
- 포함 분기: 경로 A의 `M+0x40 ≠ −1e100` 쪽(비율 < 1과 = 1 포화, `|b| < 1`), 경로 A의 `M+0x40 == −1e100` 쪽(자연 목적 0 모델), 경로 B의 일반 쪽.
- 입력 전제: 키 입력은 원본 실행 중 쓰기 직전 probe에서 읽은 메모리·정수 레지스터 값이다. 키는 `627f3e` 반환 때 `I+0x50`에서 읽고, 같은 노드의 `6e5802` 삽입 키와 다시 맞춘다.
- 제외 범위와 후속:
  - 깊이 항(부모 depth > 100·ncols): NodeLimit 2000에서 나올 수 없다. C·asm만으로 옮긴다. 탐색 sweep 38모델 500노드에서도 0회.
  - 경로 B의 −1e100 식, 경로 C·D·E: 탐색 sweep에서 B −1e100·C·D·E 0회. Rust 재생 입력에는 넣지 않는다(나오면 검사기가 실패한다). D는 A와 식이 같아 `child_key`·`deep_key`로 쓸 수 있지만 실행 증거가 없다. B −1e100·C·E는 Rust에 옮기지 않았다(C·asm 기록만).
  - `max(1, M+0xa4)`의 1 쪽: 캡처에서 `M+0xa4 ≥ 1`만 나왔다(sweep 포함). 변이는 생존이 기대값이다.
  - `bnb.rs` 연결: plunge 목록(06c·06d) 전이라 06b처럼 이식만 한다.
  - 루트 키(`5daa82`가 넣는 노드): 06a 캡처상 bound와 같다. 이번 범위 밖(쓰는 곳 미조사).
- 선행: main `51e12e2`(M2-06b 병합 포함), 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 file offset.
- 06a §8.2는 쓰는 곳을 `627f3e` 4곳으로 적었다. asm에서 `0x50(%r..)` 쓰기를 전부 찾고, `5ff358`(노드 생성)의 호출자 중 `627f3e`가 부르는 `6406da`·`63c7ae`도 쓰는 것을 찾았다. 둘 다 `627f3e`만 부른다(`index.tsv` 호출 그래프). `5ff358`의 나머지 호출자 11개(`5daa82`, `5e1548`, `6a6563`, `7005f4`, `7aef0b`, `8e3397`, `8e4682`, `8e68e0`, `8f39d7`, `8faaa4`, `9447ae`)와 `5ff358` 자체의 C에는 `+0x50`(`[10]`) 직접 쓰기가 없다. 하위 함수를 거친 쓰기는 조사하지 않았다. 그래서 "쓰는 곳 전부"는 이 범위의 주장이고, 실행으로는 A5(힙에 들어간 루트 밖 노드의 키가 모두 이 경로에서 옴)가 뒷받침한다.

### 2.1 키 쓰기 경로

b는 자식 bound `I+0x18`(쓰기 직전 값), n은 부모 LP 결과의 분수 수다.

| 경로 | 함수·쓰기 지점 | 일반 식(`M+0x40 ≠ −1e100`) | `M+0x40 == −1e100` | 깊이 항 | probe 지점과 입력 |
|---|---|---|---|---|---|
| A | `627f3e` `0x533be5`/`0x533bed`(C:309–335) | `b + min(n/max(1,a4), 1)·2⁻⁴⁸·max(\|b\|,1)` | `n`(`0x533e3e`) | 있음 | `0x533b2c`: M=rbp, b=r9+0x18, n=`rsp+0x10`의 double, 부모 I=rdi(+0x2c depth, +0x08→+0x80), ncols=`[rsp+0x40]`→+0xe8→+0xc, 노드=`[rsp+0xc0][r14]` |
| B | `627f3e` `0x533560`/`0x5335c8`(C:613–660) | 같은 식, n=`rsp+0x48`의 int, b=rdx+0x18(두 자식 공통) | `child+0x90 / (int)rsp+0x20 + n`(`0x533678`·`0x533654`) | 없음 | `0x5334fe`: M=rcx, 자식 I=r10·r8 |
| C | `627f3e` `0x532451`/`0x53245b` | `b` | `+0.0` | 없음 | `0x53244d`: M=r12, I=r9, 노드=rbp[r13] |
| D | `6406da` `0x534ca5`/`0x534cad`(C:320–348) | A와 같음, n=`rsp+0x80`의 double | `n`(`0x534e6c`) | 있음 | `0x534bf1`: M=r13, I=r10, 부모 I=rdi, ncols=`[rsp+0x88]`→+0xe8→+0xc |
| E | `63c7ae` `0x5356b2`…`0x535705`(C:270–297) | `b`(원천 I=r8의 +0x18) | `(double)(int)rsp+0x28` | 없음 | `0x5356ae`: M=r14, 노드=`[rsp+0x50][0,1]` |

깊이 항: `(double)부모 depth > (double)ncols·100`이면 `key += max(|key|,1)·2⁻⁴⁵`(`0x533ba0`–`0x533be5`). 상수는 `.rodata` `0x2a6c8f0` = 2⁻⁴⁸, `0x2a6c900` = 2⁻⁴⁵, `0x2a6c8f8` = 100, `0x2a6c948` = 1.0, `0x2a6c8e8` = −1e100.

asm 연산 순서(A): `t = n / (double)max(1,a4)`, `t = minsd(t, 1)`, `t *= 2⁻⁴⁸`, `t *= maxsd(|b|, 1)`, `key = t + b`. 2⁻⁴⁸ 곱은 정확하므로 곱 순서 변이는 판별할 수 없다(검사하지 않음).

### 2.2 경계와 상태

- 시작: probe(쓰기 직전, `M+0x40` 비교 전). 끝: 같은 `627f3e`·`6406da`·`63c7ae` 호출의 반환. 키는 반환 때 I+0x50에서 읽는다(xmm은 Frida에서 못 읽는다).
- 노드 id(`N+0x00`)는 probe에서 읽는다. 탐색에서 일부 자식의 N 구조체가 반환 전에 해제·재사용됐다(fhnw-binpack4-4 등, N+0x00이 할당자 값). 그래서 키는 I로 읽고, N이 바뀐 기록은 `moved`로 표시한다. 이 노드도 뒤의 삽입 키와 맞는지로 확인한다.
- 노드 id는 M마다 따로 센다. 결합은 같은 중첩 깊이(nest 1)의 가장 최근 쓰기로 한다.
- 함수형 probe만 쓴다([교훈](../lessons/tooling.md)). 대상에 아무것도 쓰지 않는다.

### 2.3 `M+0xa4`·`M+0x40`의 의미

- M은 `5daa82`가 할당하는 MIP 구조체(크기 0x3620)다. C:169에서 `M+0xa4 = 2000000000`.
- `M+0xa4`: `654502` C:302가 루트 LP 결과(`I+0x08`)의 `+0x80`(분수 수)을 쓴다. `M+0xa0`이 2e9면 같은 값을 첫 값으로 쓴다. 그러니 키 비율은 "부모 분수 수 / 루트 분수 수"다(C). 캡처 증거는 §6에 둔다(부모 depth 0 호출에서 `n == a4`의 비율, 한 실행 안에서 값이 바뀜).
- `M+0x40`: `5daa82` C:1649–1787이 목적 계수로 계산하는 목적 간격이다. 비연속 변수의 \|c\| > 1e-10 계수가 없고 연속 변수 계수도 없으면 −1e100(C:1753), 하나면 그 \|c\|(C:1748), 둘 이상이면 `a7b5d4`·`a7bea8` 결과(또는 0, C:1757–1765), 연속 변수 계수가 있으면 0이다. 뒤의 보정(C:1765–1787)은 읽기만 했다. 목적이 0인 fhnw-binpack4-4 최상위 M에서 −1e100인지로 확인한다. sweep에서 목적이 0이 아닌 bppc4-08(55회)·mas76(28회)에도 −1e100이 나왔는데 모두 nest 2(sub-MIP의 M)였다. sub-MIP 목적은 확인하지 않았다.

## 3. 비교 계약

`tools/mip_node_key_check.py`가 판정한다. 하나라도 실패하면 종료 코드 1이다.

| 검사 | 규칙 |
|---|---|
| A1 재현성 | 사례마다 원본 3회 gz 바이트가 같다. 모든 실행 `ok`, `errors` 없음, `tool_sha256`이 현재 도구와 같다. `--write-manifest`로 기록한 manifest의 SHA-256과 같다 |
| A2 경로 | 사례별 경로·분기 실행 횟수가 §4 기대(0 아님/0)와 같다. `probe_outside` 0 |
| A3 입력 교차 | 경로 A의 스택 n이 부모 LP `+0x80`과 같다(전부). 부모 N의 +0x18 = rdi |
| A4 Python 식 | §2.1 식으로 모든 키 쓰기의 키 비트를 재현한다(실행된 경로 전부, 이 식은 Rust와 독립) |
| A5 결합 | 최상위 트리의 루트 밖 삽입마다 nest 1의 같은 id 쓰기가 있고 키 비트가 같다. 삽입 bound 비트 = 반환 때 bound |
| A6 비간섭 | 06a 네 사례: `ins` 사건 열(in_sel, from, tree, 노드 8필드, heap_n)과 출력 줄이 06a run 1과 같다 |
| R Rust | 모든 A·B 키 쓰기 입력을 `crates/mip/tests/node_key_l1.rs`(무시 테스트)로 `child_key`·`deep_key`에 넣어 키 비트가 원본과 같다. 입력 끝 `end` 줄 |
| T 변조 | (`--self-test`) 캡처 사본에서 키 비트 하나, a4 하나, 삽입 키 하나, 06a 비교용 삽입 하나를 바꾸면 각각 A4·R, A4·R, A5, A6이 실패 |
| V 변이 | (`--self-test`) 작업 공간 사본의 `node_heap.rs`를 바꿔 사례별 거부·생존이 아래와 같다 |

| 변이 | 기대(거부 사례) | 근거(탐색, 판정 근거 아님) |
|---|---|---|
| K1 `min(·,1)` 제거 | ~~fhnw 빼고 5사례~~ → markshare_4_0, neos5, tr12-30, neos-860300(§6 계약 변경) | 처음 근거 "비율 ≥ 1"은 비율 = 1을 포함해 틀렸다. 바꾼 기대는 n > a4가 있는 사례 |
| K2 2⁻⁴⁸ → 2⁻⁴⁵ | fhnw 빼고 5사례 | fhnw는 −1e100 쪽만 |
| K3 `max(\|b\|,1)` → `\|b\|` | markshare_4_0, pk1 | 유한 `M+0x40`에서 \|b\| < 1은 두 사례뿐 |
| K4 −1e100 분기 무시(일반 식) | fhnw | |
| K5 `max(1,a4)` → `a4` | 없음(생존) | a4 ≥ 1 |
| K6 깊이 항 제거 | 없음(생존) | 깊이 조건 0회 |

- 수치: 모든 키는 비트 비교. NaN·무한대는 입력에 없음(검사기가 확인하지 않음, 식 입력이 유한임을 A4가 비트로 대신 확인).
- 시간·병렬: Threads=1, NodeLimit 2000, 타이머 분기 없음(06a와 같은 설정).
- 캡처의 `new` 사건(`5ff358` 생성자)은 진단용이고 판정에 쓰지 않는다.

## 4. 사례

| 사례 | 실제/합성 | 노리는 분기 | 기대 경로 |
|---|---|---|---|
| markshare_4_0 | 실제 | 06a 비교, \|b\| < 1, 비율 포화 | A 일반 |
| pk1 | 실제 | 06a 비교, \|b\| < 1 | A 일반 |
| neos5 | 실제 | 06a 비교, 실행 중 a4 변화 | A 일반 |
| tr12-30 | 실제 | 06a 비교, 실행 중 a4 변화 | A 일반 |
| fhnw-binpack4-4 | 실제(목적 0) | `M+0x40 == −1e100` | A −1e100 |
| neos-860300 | 실제 | 경로 B | A 일반, B 일반 |

설정은 모두 `{"NodeLimit":2000}`(Threads 1, Seed 0). 모델은 `../re/bench/data/miplib/`. 자료 `bench/oracle/m2_06b2/<사례>_<1..3>.json.gz`, manifest `bench/oracle/m2_06b2/manifest.json`.

적용하지 않은 범주: 깊이 항·B −1e100·C·D·E(0회, §1), 오류 반환(`627f3e` rc ≠ 0 쓰기 0회를 A2가 확인하지 않음, 기록만).

## 5. 재실행 명령

```sh
for m in markshare_4_0 pk1 neos5 tr12-30 fhnw-binpack4-4 neos-860300; do for r in 1 2 3; do
  tools/gurun.sh node tools/mip_node_key.js ../re/bench/data/miplib/$m.mps.gz bench/oracle/m2_06b2/${m}_$r.json.gz '{"NodeLimit":2000}'
done; done
python3 tools/mip_node_key_check.py --write-manifest   # 처음 한 번
python3 tools/mip_node_key_check.py                    # A1–A6, R
python3 tools/mip_node_key_check.py --self-test        # + T, V
```

회귀: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --release --workspace`, `python3 tools/mip_node_heap_rust.py --self-test`(같은 파일 수정), `python3 tools/mip_node_select_check.py`.

## 6. 실행 결과

- 환경: branch `work/m2-06b-2`(기준 main `51e12e2`), WSL2 x86-64, Threads 1. 도구 SHA-256과 자료 해시는 `bench/oracle/m2_06b2/manifest.json`.
- 탐색(판정 근거 아님, 자료는 저장하지 않음):
  - 06a 4사례 첫 캡처: 경로 A만 실행, 키 13,104개가 §2.1 식과 같았다.
  - 38모델 sweep(NodeLimit 500, TimeLimit 120): 키 쓰기 42,213회(A 42,197, B 16)가 식과 비트까지 같고, 삽입 12,732건이 결합됐다. C·D·E·B −1e100·깊이 항·a4 < 1은 0회. A −1e100은 fhnw-binpack4-4(최상위)와 bppc4-08·mas76(일부 호출)에서 857회.
  - 첫 도구는 자식 N의 id를 반환 때 읽었다. fhnw에서 N이 반환 전에 해제·재사용돼 id가 할당자 값(예 `0x21df6`)이 되면서 삽입 22건이 결합되지 않았다. id를 probe 시점에 읽도록 고쳤다(§2.2).
- 원본 3회: 6사례 모두 gz 바이트 동일(A1).
- 기본 판정 `python3 tools/mip_node_key_check.py` 종료 0(`ACCEPTANCE PASS`):

| 사례 | 키 쓰기 | 비율 < 1 | 비율 > 1 | \|b\| < 1(유한) | 결합한 삽입 | 최상위 a4 순서 | moved |
|---|---|---|---|---|---|---|---|
| markshare_4_0 | A 2,196 | 2,125 | 8 | 1,915 | 406 | 6 | 0 |
| pk1 | A 3,751 | 3,435 | 0 | 418 | 1,519 | 15 | 0 |
| neos5 | A 3,203 | 3,183 | 2 | 0 | 1,258 | 35 → 36 | 0 |
| tr12-30 | A 3,954 | 3,908 | 17 | 0 | 1,498 | 161 → 116 | 0 |
| fhnw-binpack4-4 | A −1e100 3,318 | (1,060) | 0 | (3,318) | 591 | 2 → 4 | 188 |
| neos-860300 | A 920, B 40 | 912 | 2 | 0 | 351 | 212 | 0 |

  - 합계 17,382개 키가 Python 식(A4)과 Rust(R) 모두 원본과 비트까지 같다. A3: 경로 A의 스택 n이 전부 부모 LP `+0x80`과 같다. A6: 06a 4사례의 삽입 열·출력이 06a run 1과 같다.
  - `M+0xa4` 증거: 최상위에서 부모 depth가 0인 호출 20회 모두 n = a4다. a4는 한 실행 안에서 바뀐다(neos5 35 → 36, tr12-30 161 → 116, fhnw 2 → 4). 그래서 정수 변수 수 같은 정적 개수가 아니다. 루트 LP를 다시 풀 때마다 갱신된다는 C(`654502` C:302)와 맞는다. `654502` 쓰기 자체는 훅하지 않았다.
  - `M+0x40` 증거: 목적 0인 fhnw-binpack4-4의 최상위 호출은 전부 −1e100, 나머지 5사례의 최상위에는 −1e100이 없다(A2).
- `--self-test` 종료 0: 변조 4종 거부(키 비트 → A4·R, a4+1 → A4·R, 삽입 키 → A5, 06a 삽입 → A6). 변이 K1–K6이 아래 기대와 같다.
  - K1(`min` 제거): markshare_4_0, neos5, tr12-30, neos-860300 거부. pk1·fhnw 생존.
  - K2(2⁻⁴⁸ → 2⁻⁴⁵): fhnw 빼고 거부.
  - K3(`max(|b|,1)` → `|b|`): markshare_4_0, pk1 거부.
  - K4(−1e100 분기 무시): fhnw 거부.
  - K5(`max(1,a4)` → `a4`)·K6(깊이 항 제거): 전 사례 생존.
- 계약 변경: K1 기대는 처음에 "fhnw 빼고 5사례"였다. 탐색 통계가 n = a4(비율이 정확히 1이라 `min`과 무관)를 함께 셌기 때문이다. 첫 self-test에서 pk1 생존으로 실패했다(pk1은 n > a4가 0회, n = a4가 316회). 기대를 n > a4 사례(위 표 "비율 > 1" > 0)로 바꾸고 검사기 주석에 남겼다.
- 회귀: `cargo clippy --workspace --all-targets -- -D warnings` 0, `cargo test --release --workspace` 124 통과·무시 7(새 `node_key_l1` 포함), `python3 tools/mip_node_heap_rust.py --self-test` PASS, `python3 tools/mip_node_select_check.py` `ACCEPTANCE PASS`. pytest는 실행하지 않았다(Python 패키지 미변경).

## 7. 판정과 인수인계

- 작업 수행 상태: 완료.
- 동등성: 통과(L1, 비트). 범위는 경로 A의 두 분기(유한 `M+0x40`, −1e100)와 경로 B의 유한 분기다. 비율 < 1·> 1·= 1, \|b\| < 1을 포함한다.
- 남은 한계:
  - C·asm만: 깊이 항, `max(1, a4)`의 1 쪽, 경로 B −1e100·C·E(Rust 미이식), D(A와 같은 함수로 덮지만 실행 0회).
  - 루트 키(`5daa82`)를 쓰는 곳은 찾지 않았다.
  - `M+0x40` 계산의 보정 단계(`5daa82` C:1765–1787)는 의미만 읽었다.
  - `bnb.rs` 미연결.
- 후속: M2-06c(plunge 규칙·꼬리 유지 판정), M2-06e(pseudocost, 10b 선행 사슬). 키를 `bnb.rs`에 쓰려면 부모 LP 분수 수와 루트 분수 수를 Rust 노드 LP 결과에 둬야 한다(06d 또는 연결 작업).
- 다음 명령: `node_heap.rs`를 바꾸면 `python3 tools/mip_node_key_check.py --self-test`와 `python3 tools/mip_node_heap_rust.py --self-test`.
- 교훈: [original.md](../lessons/original.md)(키 쓰기 5경로와 `M+0xa4`·`M+0x40` 의미), [tooling.md](../lessons/tooling.md)(반환 전에 해제되는 자식 N).
