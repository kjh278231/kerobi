# M2-06c-2 검증 계약과 결과: plunge 규칙 1·2·5·6 선택 점수

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 앞선 작업은 [M2-06c](mip_plunge_rule.md)(규칙 번호·RNG·규칙 3/12·꼬리, 기준 자료 `bench/oracle/m2_06c`)다. M2 인수인계의 첫 다음 작업이고, 사용자의 "다음 작업 이어서 진행" 지시로 착수했다(2026-11-11, 작업자는 이 세션).

## 1. 범위와 주장

- 작업 종류: 조사(저장 캡처 재생). 새 원본 캡처·Gurobi 실행 없음. Rust·Python 패키지는 바꾸지 않는다.
- 조사 질문: `6e8eb3`이 규칙 1·2·5·6일 때 plunge 목록에서 어느 항목을 고르는가(점수식·조상 비교·동률·RNG)와 work.
- 완료 후 주장할 것(재생 = §2의 규칙을 Python으로 계산, 입력은 06c 캡처):
  - S2: 규칙 1·2·5·6 호출마다 재생 첨자 = `pick.idx`(그 뒤 06c가 확인한 대로 선택 노드 = 반환 노드).
  - W: 모든 규칙(1·2·3·4·5·6·7·12)의 규칙 본문 work 증가 `pick.work − rule.work`가 재생과 비트까지 같다.
- 입력: `pick.list`(id, est, depth, `I+0x41`), `rule.st`(RNG 슬롯), `rule.work`. 규칙 1·6의 조상 `5fb13c(it, maxd − 2)`는 같은 호출의 꼬리 `anc` 사건(선택 노드 `sel`과 남은 항목 `it`)의 id 사슬을 노드 id로 찾아 다시 걷는다. 꼬리의 d′ = depth_sel − δ(δ ≥ 2) ≤ maxd − 2이므로 사슬은 필요한 깊이를 덮는다.
- 재생 불가로 분류하는 것(실패와 따로 센다): 꼬리 조상이 null이라 `it` 사슬이 없는 규칙 1·6 호출(maxd < 2면 모든 조상이 null이라 재생 가능), 사슬이 훅 상한 64개에서 잘린 경우. 이 밖에 사슬이 없으면 실패다. 재생 불가 수는 06c의 null-anc 호출 수(58) 이하여야 한다.
- 제외 범위와 후속:
  - 규칙 4·7 점수: `I+0x34`(분기 변수)·`I+0x38`(분기 값)·`I+0x40`(방향 바이트)과 incumbent x(`5e6f57`)가 06c 캡처에 없다. 식은 §2에 asm 해석으로만 남기고 work만 비교한다. 후속 **M2-06c-3**(새 캡처).
  - 규칙 9(BranchDir ≠ 0)·11(목적 간격 −1e100, 전부 정수): 06c 사례 0회. asm만.
  - 규칙 5 뽑기 뒤 RNG 상태: 06c 캡처에 없다. 그 호출 안에서 다시 읽지 않고 다음 호출은 다시 시드하므로 이후 판단에는 영향이 없다(§2.1). 미검증으로 남긴다.
  - 제외 항목(`I+0x41`) 건너뛰기: 06c 사례 0회(asm만).
- 선행: main `8409a67`(M2-06c 병합). 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`, 주소는 file offset. 함수 `6e8eb3` = `0x5b29d0`, 점프 테이블과 기호는 [M2-06c §2](mip_plunge_rule.md#2-원본과-상태-경계)와 같다.
- 시작 경계: dispatch 직전 probe `0x5b361d`(06c `rule` 사건). 이때 minEst m(xmm0), best = −1e100(xmm1), `0x40(%rsp)` = −1, work는 minEst 3n을 더한 뒤 값이다. 종료 경계: `0x5b44a9`(06c `pick` 사건, −1이면 0으로 바꾼 뒤).
- 공통: `0x5b3541`–`0x5b35fc`의 minEst와 work 3n은 **모든 규칙**에서 dispatch 전에 실행된다.
- 아래 식은 objdump(`0x5b3647`–`0x5b44a9`)를 읽은 것이다. `skip` = `I+0x41 ≠ 0`. "엄격 최대"는 `comisd`/`cmova`(sc > best일 때만 첨자 갱신)이고 best는 `maxsd`(같은 값)다.

```text
규칙 2 (0x5b412e–0x5b426f, 2개씩 풀어 씀)
  idx = argmax_strict_{j !skip} (−est_j);  work += 3.0·n
규칙 6 (0x5b3bc1–0x5b3daa)
  maxd = 0, anc = null
  for j: !skip and depth_j > maxd → maxd = depth_j; anc = 5fb13c(N_j, depth_j − 2)    # 첫 최심 항목의 조부모
  work += 3.0·n
  for j !skip: v = −est_j; v > best → (5fb13c(N_j, maxd − 2) == anc → idx = j, best = v)   # best는 일치할 때만
  work += 3.0·n
규칙 1 (0x5b4274–0x5b448c): 규칙 6과 같은 maxd·anc(검사 순서만 depth 먼저)
  work += 3.0·n
  for j !skip: sc = (double)(−depth_j) − (est_j − m)/1e8; sc > best → (5fb13c(N_j, maxd − 2) == anc → idx = j, best = sc)
  work += 4.0·n
규칙 5 (0x5b3daf–0x5b3e19, 0x5b4a87)
  r = a7b4d6(&slot)   # slot = 0x70(%rsp), LCG 1회, tempering 31비트 정수
  s = r mod n (idiv);  j = s, s+1, …(mod n) 중 첫 !skip → idx;  모두 skip이면 0.  work 없음
규칙 4 (0x5b3e1e–0x5b3fa6, asm만)
  x = 5e6f57(M)   # 해 풀 최선해 x (M+0x3220, +0x50 첨자)
  for j !skip and I+0x34 ≥ 0: b = (I+0x40 == '<') ? floor(I+0x38) : ceil(I+0x38)
     sc = (double)depth_j − |x[I+0x34] − b| / 100;  엄격 최대
  work += 7.0·n
규칙 7 (0x5b39fc–0x5b3bbc, asm만)
  sc = (double)((int8)I+0x40 + 10·depth_j) − (est_j − m)/1e8;  엄격 최대;  work += 5.0·n
규칙 9 (0x5b3910–0x5b39f7, asm만, r10d = env+0x27e0 BranchDir)
  base = (double)(10·depth_j) − (est_j − m)/1e8;  sc = (r10d == −1) ? base − (double)(int8)I+0x40 : base + (double)(int8)I+0x40
  엄격 최대;  work += 5.0·n
규칙 11 (0x5b37ca–0x5b390b, asm만)
  sc = (double)(int)I+0x90;  엄격 최대;  work += 3.0·n
규칙 8·10: 본문 없이 idx = 0 (0x5b4499).
```

- `5fb13c(N, d)`: `(d | depth_N − d) < 0`이면 null, 아니면 부모(`N+0x10`)를 depth_N − d번 걷는다(널에서 멈춤). 시작 노드의 I만 읽는다. null == null도 일치다(maxd < 2).
- 상수: `0x2a6e410` 3.0, `0x2a6e418` 1e8, `0x2a6e428` 4.0, `0x2a6e430` 5.0, `0x2a6e438` 7.0, `0x2a6e3d0` 100.0, `0x2a6e370` 부호 비트.
- `a7b4d6` = 06c의 `a7b54f` 앞단과 같은 LCG·tempering(`crates/lp/src/common.rs` `Rng::next_int`). 규칙 5는 k % 13 = 0 절에서만 정해지므로 같은 호출에서 홀수 k 뽑기와 겹치지 않고, `rule.st`가 곧 시드다.

## 3. 비교 계약(판정 규칙, 첫 재생 전 고정)

검사기 `tools/mip_plunge_score_check.py`(06c 검사기의 C0·로더·R·G·T를 그대로 부른 뒤 아래를 더한다).

| 비교 | 대상 | 규칙 |
|---|---|---|
| S2 | 규칙 1·2·5·6 호출의 `pick.idx` | 재생 첨자와 정확히 일치 |
| W | 모든 규칙의 `pick.work − rule.work` | 규칙별 식(2·5·4·7·3/12는 한 번, 1·6은 두 번 더함)으로 비트 일치. work 포인터 null이면 둘 다 null |
| 사슬 | 규칙 1·6의 조상 | 꼬리 `anc` 사슬을 id로 찾아 걷는다. 사슬의 시작 id·depth가 `pick.list`와 같아야 한다 |

- **합격 조건**: S2·W 전부 일치, 재생 불가 수 ≤ 58이고 모두 null-anc 또는 사슬 상한 사유, 그리고 사례 합계에서 다음이 모두 1회 이상: 규칙 1·2·5·6 재생 호출, 규칙 1과 규칙 6 각각에서 조상 비교가 선택을 바꾼 호출(W1 기준으로 다른 첨자), 규칙 5에서 s ≠ 0인 호출, 규칙 4·7 work 비교 호출.
- **변종(결과 전 고정)**: 재생에 적용해 거부 사례를 보고한다. 생존은 "그 절은 실행으로 판별되지 않음"이다.
  W1 조상 비교 끔, W2 maxd·anc를 마지막 최심 항목으로, W3 조상 깊이 maxd − 1, W4 maxd − 3, W5 null 조상은 불일치, W6 규칙 1 depth 부호 +, W7 규칙 1 q 부호 +, W8 best를 조상 불일치 때도 갱신, W9 규칙 2 동률은 뒤 항목, W10 규칙 5 시작 = trunc(u·n)(u = 실수 뽑기), W11 규칙 1·6 work를 한 번에 (6n·7n).
- **self-test(변조)**: 규칙 1·2·5·6 각각의 `pick.idx` 변경, 규칙 6 호출의 `it` 사슬 조상 id 변경, 규칙 1 호출의 est 변경, `rule.st` 변경(규칙 5), 규칙 4·7 `pick.work` 변경, 규칙 1·6 호출 전부 삭제(합격 0회), 06c 바이너리 해시 0. 하나라도 거부되지 않으면 실패.
- 수치: 모든 double 비트 비교. 원본 반복 실행의 재현성은 06c의 사례마다 3회 gz 바이트 동일(manifest)을 재사용한다(06c C0를 다시 실행).

## 4. 사례

06c 기준 자료 그대로(`bench/oracle/m2_06c`, 5사례 × 3회, 재캡처 없음): markshare_4_0·pk1·neos5·tr12-30(NodeLimit 2000)·pk1-pool(+PoolSearchMode=2). 06c 표의 규칙 분포: 1 2,346, 2 2,914, 5 239, 6 2,000, 4 347, 7 38. 재생은 run 1만(3회 동일은 C0가 확인).

## 5. 재실행 명령

- 판정: `python3 tools/mip_plunge_score_check.py`(기대: 06c C0·R/G/S/T PASS, 사례별 S2/W PASS와 재생 불가 수, `ACCEPTANCE PASS`, 종료 0).
- 판별력: `python3 tools/mip_plunge_score_check.py --self-test`(`ACCEPTANCE PASS (+ self-test)`, 종료 0).
- 06c 검사기 회귀: `python3 tools/mip_plunge_rule_check.py --self-test`.
- Rust·Python 미변경: cargo·pytest 해당 없음. Gurobi 실행 없음.

## 6. 실행 결과

- 실행 commit `b3758c6`(검사기, rebase 전 `217e995`·내용 같음, rebase 뒤 다시 실행), main `497233f` 위. 입력은 `bench/oracle/m2_06c`(manifest 해시 확인), run 1만 재생. Gurobi 실행 없음.
- `python3 tools/mip_plunge_score_check.py --self-test` 종료 0(`ACCEPTANCE PASS (+ self-test)`). 06c C0·R/G/S/T도 같은 실행에서 통과했다. `python3 tools/mip_plunge_rule_check.py --self-test`도 종료 0(06c 회귀).
- 재생(첫 실행에서 전부 일치):

| 사례 | 규칙 1 | 규칙 2 | 규칙 5 (시작 ≠ 0) | 규칙 6 | 조상 비교가 선택을 바꿈 (1 / 6) | 재생 불가 (null 꼬리 조상) | work 비교 호출 |
|---|---:|---:|---:|---:|---|---:|---:|
| markshare_4_0 | 0 | 312 | 144 (82) | 251 | 0 / 36 | 0 | 2000 |
| pk1 | 557 | 687 | 32 (16) | 390 | 483 / 171 | 7 | 1984 |
| neos5 | 474 | 717 | 31 (19) | 345 | 370 / 91 | 8 | 1977 |
| tr12-30 | 693 | 575 | 0 | 520 | 566 / 208 | 7 | 1922 |
| pk1-pool | 597 | 623 | 32 (28) | 485 | 518 / 224 | 12 | 1981 |
| 합계 | 2,321 | 2,914 | 239 (145) | 1,991 | 1,937 / 730 | 34 (규칙 1 25·규칙 6 9) | 9,864 |

  규칙 1·2·5·6 선택 7,465호출이 `pick.idx`와 같다. work는 9,864호출 전부(규칙 1·2·3·4·5·6·7·12)가 비트까지 같다. 재생 불가 34는 모두 꼬리 조상이 null인 호출이고(상한 58 이하), 사슬 상한(64)으로 잘린 경우는 0이다.
- 변종(결과 전 고정):

| 변종 | 결과 | 비고 |
|---|---|---|
| W1 조상 비교 끔 | 5사례 거부 | |
| W3 조상 깊이 maxd − 1, W4 maxd − 3 | 5사례 거부 | |
| W8 불일치 때도 best 갱신 | 5사례 거부 | |
| W6 규칙 1 depth 부호 +, W7 q 부호 + | 규칙 1이 있는 4사례 거부 | |
| W10 규칙 5 시작 = trunc(u·n) | 규칙 5가 있는 4사례 거부 | |
| W5 null 조상은 불일치 | pk1-pool 거부 | maxd < 2인 규칙 1·6 호출 22 |
| W9 규칙 2 동률은 뒤 항목 | neos5 거부 | 최소 est 동률 2호출 |
| W2 마지막 최심 항목 | 생존 | 최심 항목이 둘 이상인 3,981호출에서 첫·마지막 항목의 조부모가 늘 같았다 |
| W11 work를 한 번에 더함 | 생존 | 6n·7n 한 번과 두 번 더하기의 반올림이 모두 같았다 |

- self-test(변조 11종 모두 거부): 규칙 1·2·5·6 `pick.idx`, 규칙 6 꼬리 사슬의 조상 id, 규칙 1 est, 규칙 5 RNG 슬롯, 규칙 4·7 work, 규칙 1·6 호출 전부 삭제(합격 0회), 06c 바이너리 해시 0. 첫 판에서 RNG 슬롯 변조가 생존했다. 고른 첫 규칙 5 호출에서 LCG 한 단계를 더해도 r mod n이 같았다. 변조 대상을 "한 단계 더하면 시작이 바뀌는 첫 규칙 5 호출"로 바꿨다(판정 규칙은 그대로).
- 실행하지 않은 검사: cargo·pytest(Rust·Python 미변경), 원본 재캡처(06c 자료 재사용, 3회 동일은 06c C0로 다시 확인).
- 계약 변경: 없음. 검사기에서 규칙 1·6의 첫 반복(maxd 탐색)은 새 최댓값마다 `5fb13c`를 부르지만 마지막 결과만 남는다. 재생은 마지막 최심 항목에서 한 번만 조상을 구한다(중간 호출은 꼬리 사슬보다 얕은 깊이를 물어 재생 불가가 되므로).

## 7. 판정과 인수인계

- 작업 수행 상태: 완료(조사).
- 동등성(규칙 재생): 통과. 06c 5사례 캡처에서 규칙 1·2·5·6 선택 7,465호출과 모든 규칙의 본문 work 9,864호출이 원본과 같다.
- 통과 범위와 한계:
  - 실행 확인(변종 거부까지): 규칙 1·6의 조상 비교와 깊이 maxd − 2, 일치할 때만 best 갱신, null == null, 규칙 1의 −depth·−q, 규칙 2의 엄격 최소 est(동률 앞 항목, 2호출), 규칙 5의 정수 뽑기 r mod n.
  - 실행 확인(판별 변종 생존): 첫 최심 항목(마지막과 조부모가 늘 같음), work를 두 번 더함(한 번과 반올림 같음).
  - 미검증: 재생 불가 34호출(null 꼬리 조상), 제외 항목 건너뛰기(0회), 규칙 5의 순환 검색(제외 항목 0이라 항상 시작 항목), 규칙 5 뒤 RNG 상태(캡처 없음, 이후 판단에 쓰이지 않음).
  - asm만: 규칙 4·7 점수(work만 비교), 규칙 9·11(0회), 규칙 8·10.
- 후속:
  - **M2-06c-3**(조사·오라클): 규칙 4·7 점수. 새 캡처 도구(06c 도구는 manifest 해시로 고정되어 바꾸지 않는다)로 `pick` 시점의 `I+0x34`·`I+0x38`·`I+0x40`과 `5e6f57` 해, 규칙 본문 `5fb13c` 호출(`0x5b3c2c`·`0x5b3d1f`·`0x5b42e6`·`0x5b4402` 반환 지점)과 규칙 5 뒤 RNG 상태를 더한다. 규칙 9는 BranchDir = ±1 사례로 닿는다.
  - **M2-06d**(구현): 06c·06c-2 규칙(번호·RNG·1·2·3·5·6·12·꼬리)을 Rust로 옮긴다.
  - 다음 명령: `python3 tools/session.py status` 뒤 M2-06d 또는 M2-06c-3. 검사기를 바꾸면 `python3 tools/mip_plunge_score_check.py --self-test`.
- 작업 branch `work/m2-06c-2`, 기준 main `497233f`. 커밋·병합 상태는 M2 인수인계를 따른다.
- 교훈: `docs/lessons/original.md`(규칙 1·6은 최심 항목 조부모와 같은 조상인 항목만 고름, minEst는 모든 규칙에서 계산), `docs/lessons/tooling.md`(조상 질의를 꼬리 사슬에서 재사용할 때 중간 질의는 덮이지 않음).
