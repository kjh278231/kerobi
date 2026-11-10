# M2-06b 검증 계약과 결과: 전역 노드 힙 Rust 이식

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 원본 규칙과 추적은 [M2-06a](mip_node_select_spec.md)다. M2 인수인계의 첫 다음 작업이고, 사용자의 "다음 스프린트 진행" 지시로 착수했다(2026-11-11).

## 1. 범위와 주장

- 작업 종류: 구현(L1 저장 기준 재생).
- 완료 후 주장할 것: `crates/mip/src/node_heap.rs`의 `NodeHeap`(삽입 `push`, 꺼내기 `pop`, cutoff 가지치기 `prune`, `T+0x170` 값 `min_bound`)에 06a 네 사례 run 1의 최상위 트리 힙 연산열을 넣으면, 원본 추적의 모든 힙 스냅숏(배열 순서의 노드 id·키·bound 비트), 힙 크기, `T+0x170`, 꺼낸 노드, 가지치기가 지운 노드 순서가 같다.
- 포함 분기: 동률 키의 삽입·pop(markshare_4_0, pk1, neos5), 동률 없는 대조군(tr12-30), 자식 하나인 sift-down, 가지치기 뒤 재힙화, 재시작 구간의 전체 가지치기와 새 트리, 트리 해제.
- 입력 전제: 연산열은 원본 추적의 사건(`ins` 키·bound 비트, `enter`·`leave`의 pop 수, `prune`·`acct`·`pruned`, `tnew`·`tfree`)에서만 만든다. Python 힙 모델(`mip_node_select_check.py` `Heap`)의 재생 결과는 입력에도 기대값에도 쓰지 않는다.
- cutoff: 원본 cutoff(xmm0)는 캡처에 없다. 가지치기마다 지운 노드 bound의 최솟값을 cutoff로 합성한다(지운 노드가 없으면 +∞). 06a §3이 남은 bound 최댓값 < 지운 bound 최솟값을 이미 검사했다. 그래서 `cutoff ≤ bound` 등호 쪽은 C 근거만 있고 합성 cutoff로 거부되는 변이는 원본 증거가 아니다.
- 제외 범위와 후속:
  - `bnb.rs` 연결: Rust B&B에는 plunge 목록(06c·06d)과 키 식이 없어 연결해도 원본 궤적이 되지 않는다. 03d-12·03d-14처럼 이식만 하고 연결은 뒤로 미룬다.
  - 키 식(`627f3e`, 06a §8.2): 부모 LP 분수 수(`I+0x08`→`+0x80`)와 `M+0xa4`의 재캡처가 필요하다. 후속 M2-06b-2.
  - status 10 둘째 힙(`T+0x150`), node file(`6e4db7`·`70262a`), 병렬 worker 힙, 지연 목록: 06a 사례에서 실행 0회. 입력 생성에서 status 10 삽입·지연 추가가 0회임을 검사한다.
- 선행: main `64e16f9`(M2-06a 병합 포함), 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 file offset.
- 이번에 다시 읽은 C(`re/decomp/c/`):

| 함수 | 위치 | 옮긴 규칙 |
|---|---|---|
| `6e5802` `0x5b1e70` | C:85–125(일반 힙) | 끝에 붙이고 `parent.key <= key`면 멈춤, 아니면 부모를 내림. 끝나면 `T+0x170` = 루트 bound |
| `6e8eb3` `0x5b29d0` | C:226–294 | 비었으면 `T+0x170 = 1e100`. 루트를 빼고 끝 원소를 루트에 둠. sift-down: 자식 둘이면 `m = (k2 <= k1) ? k2 : k1`, `x <= m`이면 멈춤, `k2 <= k1`이면 오른쪽. 자식 하나면 `k1 <= x && x != k1`일 때만 내림. 끝나면 `T+0x170` = 루트 bound 또는 1e100 |
| `6e460c` `0x5aed60` | C:49–104 | 배열 순서로 `cutoff < b || cutoff == b`이면 `606036`·`5f9d4e(·,0)`(지움), 아니면 앞으로 모음. i = n−1부터 0까지 같은 sift-down. `T+0x170` = 루트 bound 또는 1e100 |

- 시작·끝: 트리별 힙 상태. 사건 하나는 원본 함수 호출 하나(`6e5802` 진입, `6e8eb3`의 pop 하나, `6e460c` 한 번)다.
- 입력: 삽입 노드의 id·키(`I+0x50`)·bound(`I+0x18`) 비트, pop 시점, 합성 cutoff.
- 출력: pop한 노드 id, 가지치기가 지운 노드 id 순서, 스냅숏 지점의 배열·크기·`min_bound`.

## 3. 비교 계약

`tools/mip_node_heap_rust.py`가 판정한다. 하나라도 실패하면 종료 코드 1이다.

| 검사 | 규칙 |
|---|---|
| M 입력 | 네 사례 run 1의 SHA-256이 `bench/oracle/m2_06a/manifest.json`과 같다(A1이 run 2·3을 같게 만든다) |
| X 재생 입력 | 사건에서만 만든다. pop은 같은 선택의 첫 `in_sel` 삽입 직전, 없으면 `leave`에서 `leave.pops − enter.pops`번. 입력 끝 `end` 줄로 잘린 입력은 재생 실패 |
| P pop | 선택마다 꺼낸 id 열 = 그 선택의 지연 추가(`defer`) id 열 + `leave.node` id(있으면) |
| R 가지치기 | 지운 id 열 = 그 `prune`과 다음 사건 사이 `acct`(`from` `6e460c`) id 열, 순서까지 |
| S 스냅숏 | `enter`·`leave`: 크기 `heap_n`, `heap_min`(`T+0x170`) 비트, `heap`이 있으면 배열(id·키·bound 비트) 전체. `pruned`: 배열 전체와 `heap_min`. `tfree`: 크기 |
| K 개수 | 사례별 pop·삽입·가지치기 노드·배열 스냅숏 수가 06a 표와 같다: pop 216·44·52·27(합 339), 삽입 407·1521·1260·1501(4,689), 가지치기 노드 25·549·447·608(1,629), 배열 스냅숏 435·91·106·96(728) |
| T 변조 | Rust 출력 변조(배열 두 항목 교환, pop id, 지운 순서 교환, 줄 삭제, `min_bound` 비트) 5종이 각각 거부된다 |
| V 변이 | 아래 코드 변이를 작업 복사본에 넣어 다시 돌린다. 사례별 거부·생존이 표의 기대와 같아야 한다(실행 전 고정) |

| 변이 | 근거 | 기대(markshare_4_0, pk1, neos5, tr12-30) |
|---|---|---|
| V1 pop 동률에서 왼쪽 자식 | 06a A3 V1 | 거부, 거부, 거부, 생존 |
| V2 삽입 동률에서 계속 올림 | 06a A3 V2 | 거부, 거부, 거부, 생존 |
| V3 pop 동률에서 계속 내림 | 06a A3 V3 | 거부, 거부, 거부, 생존 |
| P2 가지치기 비안정 삭제(끝 원소로 채움) | 06a A3 P2 | 거부 4 |
| C1 가지치기 `cutoff < b`만 지움 | 합성 cutoff(원본 증거 아님) | 거부 4 |
| H1 재힙화를 0부터 n−1로 | 보고만 | 보고 |
| O1 자식 하나에서 같아도 내림 | 보고만 | 보고 |

- 부동소수는 모두 u64 비트로 비교한다. 비교식은 C의 부동소수 비교를 그대로 써서 ±0은 같은 키다. NaN 키·bound는 06a 자료에 없다(입력 생성에서 검사).
- 노드 id는 `N+0x00`이다. 원본 id와 Rust 항목을 같은 값으로 맞춘다.

## 4. 사례

06a §4의 네 사례 run 1(`bench/oracle/m2_06a/<사례>_1.json.gz`). 모두 실제 모델, Threads=1, Seed=0, NodeLimit=2000. 동률 pop은 215/216, 42/44, 12/52, 0/27이다.

- 적용하지 않은 범주: status 10 힙, 지연 목록, node file(실행 0회, §1).

## 5. 재실행 명령

- 판정: `python3 tools/mip_node_heap_rust.py`(입력 생성 → `cargo test --release -p rsolver-mip --test node_heap_l1 -- --ignored` → M·X·P·R·S·K, 기대 `PASS`, 종료 0).
- 판별력: `python3 tools/mip_node_heap_rust.py --self-test`(위 + T·V, 작업 복사본 빌드로 수 분).
- 회귀: `cargo test -p rsolver-mip`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`. 06a 검사기 `python3 tools/mip_node_select_check.py`.
- Gurobi 캡처는 하지 않는다(06a 자료 재사용). Python 패키지·`bnb.rs`를 바꾸지 않아 pytest는 해당 없음.

## 6. 실행 결과

- 실행: 코드 commit `64ef45e`(기준 main `64e16f9`), x86-64 WSL2. 입력은 06a manifest의 run 1 네 파일(markshare_4_0 `27035e7c…`, pk1 `13a6a756…`, neos5 `584288a8…`, tr12-30 `c2423572…`). Gurobi는 실행하지 않았다.
- `python3 tools/mip_node_heap_rust.py` 종료 0, 첫 실행부터 차이 없음:

| 사례 | pop | 삽입 | 가지치기 노드 | 배열 스냅숏 | 스냅숏(크기·`T+0x170`) |
|---|---:|---:|---:|---:|---:|
| markshare_4_0 | 216 | 407 | 25 | 435 | 4,004 |
| pk1 | 44 | 1,521 | 549 | 91 | 3,972 |
| neos5 | 52 | 1,260 | 447 | 106 | 3,957 |
| tr12-30 | 27 | 1,501 | 608 | 96 | 3,887 |

- `--self-test` 종료 0. T 변조 5종 거부(배열 두 항목 교환, pop id, 지운 순서, 줄 삭제, `T+0x170` 비트). V 변이(첫 차이는 `target/mip_node_heap_rust/self_test.log`):

| 변이 | markshare_4_0 | pk1 | neos5 | tr12-30 | 기대와 |
|---|---|---|---|---|---|
| V1 pop 동률 왼쪽 | 거부 | 거부 | 거부 | 생존 | 같음 |
| V2 삽입 동률 올림 | 거부 | 거부 | 거부 | 생존 | 같음 |
| V3 pop 동률 내림 | 거부 | 거부 | 거부 | 생존 | 같음 |
| P2 비안정 삭제 | 거부 | 거부 | 거부 | 거부 | 같음 |
| C1 `cutoff < b`만 | 거부 | 거부 | 거부 | 거부 | 같음(합성 cutoff) |
| H1 재힙화 0부터 | 거부 | 거부 | 거부 | 거부 | 보고만 |
| O1 자식 하나 동률 내림 | 거부 | 생존 | 생존 | 생존 | 보고만 |

- 첫 self-test는 "배열 교환" 변조를 원본 배열이 없는 스냅숏(pop 없는 선택)에 넣어 실패했다. 검사기가 아니라 변조 위치 문제라 원본 배열이 있는 스냅숏을 고르게 고쳤다. 판정 규칙·기대는 바꾸지 않았다.
- clippy `double_comparisons`로 `cutoff < b || cutoff == b`를 `cutoff <= b`로 바꿨다(NaN 포함 같은 의미). 바꾼 코드로 판정·self-test를 다시 돌렸다.
- 회귀: `cargo clippy --workspace --all-targets -- -D warnings` 0, `cargo test --release --workspace` 통과 123·실패 0·무시 6(새 단위 테스트 1, 새 무시 테스트 `node_heap_l1` 1), `python3 tools/mip_node_select_check.py` `ACCEPTANCE PASS`.
- 실행하지 않은 검사: pytest(Python·`crates/py`·`bnb.rs` 미변경), Gurobi 캡처(06a 자료 재사용).

## 7. 판정과 인수인계

- 작업 수행 상태: **완료(구현 L1)**.
- 동등성: **통과(L1, 계약 범위)**. 06a 네 사례의 최상위 트리 힙 연산(삽입 4,689, pop 339, 가지치기 50회·1,629 노드)을 `NodeHeap`에 넣은 결과가 원본의 꺼낸 노드, 지운 순서, 배열 스냅숏 728개, 크기·`T+0x170` 스냅숏 15,820개와 비트까지 같다. 동률 규칙 V1–V3은 동률 있는 3사례에서 거부된다.
- 한계:
  - `bnb.rs` 미연결. Rust B&B는 여전히 `Vec` 최소 bound `swap_remove`다. 연결은 plunge 목록(06c·06d)과 키 식이 생긴 뒤가 맞다.
  - 키 식(`627f3e`) 미구현·미대조. 이번 입력의 키는 캡처 비트다. 후속 M2-06b-2.
  - 가지치기 `cutoff ≤ bound`의 등호 쪽은 C 근거만 있다(합성 cutoff). 자식 하나 sift-down의 엄격 비교는 markshare_4_0에서만 판별된다.
  - status 10 둘째 힙, node file, 병렬 worker 힙, 지연 목록은 구현하지 않았다.
- 후속: M2-06b-2(키 식 재캡처·Rust 이식), M2-06c(plunge 규칙·꼬리 유지 판정), 06e(pseudocost).
- 교훈: `docs/lessons/tooling.md`(변조는 기대값이 있는 필드에 넣기).
