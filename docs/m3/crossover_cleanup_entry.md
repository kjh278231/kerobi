# M3-04f-2 crossover cleanup 진입 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 선행: [04f-1 해 설치](primal_push_install.md), [진입 경계](crossover_entry_contract.md), [push 분기](primal_push_branches.md).

## 1. 범위와 주장

- 작업 종류: **조사 + 오라클 구축**. 구현(Rust)은 하지 않는다.
- 조사 질문:
  1. `13f716`(Crossover 본체)은 push 반환(`retry` = `local_318`, signed `remaining` = `local_314`)과 Crossover 파라미터(env+0x2ba0)로 cleanup simplex `0cda5b`의 알고리즘(primal 0 / dual 1)과 basis 인자 유무를 어떻게 고르는가.
  2. 두 번째 시도 뒤에도 retry가 남는 경우(C 1363–1521) 상태 배열·remaining을 어떻게 재설정하는가.
  3. `0cda5b`가 basis 인자를 받았을 때 `0325b4`가 workspace 상태 배열(S+0x168)·x·slack·superbasic 목록(S+0x440·0x448·0x450)에 하는 일, 그리고 그 진입 조건(S+0xf0 = 1, D+0x324 < 1).
  4. 위 경계를 원본 실행에서 재현 가능하게 캡처하고 C 규칙 재계산으로 검사하는 오라클 방법.
- 포함: double(S+0x28 = 0), Threads = 1, mode 0(barrier 뒤 crossover), Presolve = 0, Method = 2, Crossover ∈ {−1, 1, 2, 3, 4}의 Netlib 소형 45 자연 실행. **범위 추가(탐색 뒤, 판정 전)**: Netlib 45가 모두 remaining = 0이라 04f-1에서 remaining > 0이 자연으로 나온 1b 사례(arms·retry, PStart/DStart가 있어 `0c3d0d` → `13f716` **mode 1**)와 interior를 Crossover 5값으로 더한다. push 뒤 선택 코드(C 1363–1690)는 mode를 읽지 않는다(mode는 S+0x2d8 초기값 C 366–384와 로그 필드 C 1338만 바꿈). mode 1 진입 구조 자체는 계속 제외한다. 자연에서 안 나오는 분기는 `0x1bf2ee` slot 편집 합성(`synthetic.json`)으로 따로 캡처한다.
- 제외와 후속: cleanup simplex 본체(`0cda5b` 반복)와 최종 basis·해의 일치(04f-3 이후 구현 단위), x87(S+0x28 ≠ 0), mode 1(warm start `0c3d0d` 경로), Crossover = 5·threads ≥ 2(`9b54f1`), 재시작(`Restart crossover`, 04g), 할당 실패, 시간·work, Rust 구현.
- 선행 commit: main `02b60e3`. 미커밋 선행 변경 없음(main의 미추적 `.claude/settings.local.json`은 무관).

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 ELF file offset(Ghidra = +0x100000).
- `13f716` = `../re/decomp/c/001bc5d0_PRIVATE000000000013f716.c`: push 시도 루프 C 851–1220, 뒤처리 C 1225–1362(S+0x2d8 재계산·로그·`a83b2f`·`08b129`), retry 잔존 재설정 C 1363–1521, cleanup 선택 C 1523–1690. asm `0x1bf2ee`(retry 검사) – `0x1bf6c2`(Crossover 읽기) – `0x1bf6f2`(선택 결과 r14d) – `0x1bf781`·`0x1bf7b9`·`0x1bfa4f`(`0cda5b` 세 호출).
- 스택 slot(위 asm): `rsp+0x50` retry, `+0x54` remaining, `+0x230` S, `+0x2d0` local_98(재시작 수), `+0x2d8` OptimalityTol(local_90), `+0x250` n+m, `+0x60` local_308(S+0x168), `rbx` x(n+m 연속, slack은 +n).
- `0cda5b` = `../re/decomp/c/00153cc0_PRIVATE00000000000cda5b.c` C 142–191: `02f7e1(model, alg)` → S+0xf0 = 1이고 vbasis·cbasis가 모두 있고 `D+0x324 < 1`이면 `0325b4(S, vbasis, cbasis, x, slack)`.
- `0325b4` = `../re/decomp/c/000d3150_PRIVATE00000000000325b4.c` 전체(688줄).
- 시작 경계: `0x1bf2ee`(push 시도 루프와 뒤처리가 끝난 뒤 retry 검사). 종료 경계: `0cda5b` 진입(인자)과 그 안 `0325b4` 반환(설치 상태). cleanup simplex 반복은 비교하지 않는다(반환 상태·최종 VBasis/CBasis는 참고 기록).
- 중간 훅 주의: `0x1bfae2`(`251b4f` call 직후)는 인자 push 4개 때문에 rsp가 0x20 내려간 상태라 `rsp+0x230`이 S가 아니다(탐색에서 access violation). `add $0x20,%rsp` 뒤 `0x1bfae6`에 붙였다.

### 2.1 조사 답(C·asm 근거, §6에서 원본 캡처로 확인)

**Q1 push 반환 소비와 cleanup 선택.** `local_314`(remaining)는 진입에서 −1로 시작하고(C 154) primal push만 쓴다. dual push는 retry만 쓴다. 시도 루프(C 851–1220, attempt `local_310` 0·1)가 끝나면 retry가 남았을 때 Q2를 거친 뒤, `S+0x180 = 1`, `local_98 = 2`(재시작 2회째)면 `S+0x178 += 10`을 하고 선택 bool B(r14d)로 `0cda5b`를 부른다(C 1523–1690, asm 0x1bf6c2–0x1bf7b9·0x1bfa59–0x1bfbda).

| Crossover(env+0x2ba0) | 조건 | B | `0cda5b` 호출 |
|---|---|---|---|
| −1(자동) | retry ≠ 0 | 0 | `(model, 0, 0…)` |
| −1 | remaining > 0 | 1 | `(model, 1, 0…)` dual, basis 인자 없음 |
| −1 | remaining ≤ 0, S+0x2d8 = 0(±0, NaN 아님), PInf S+0x120 > 0, S+0x28 = 0 | 쌍대 검사 | 검사 참이면 dual, 거짓이면 아래 primal |
| −1 | 그 밖(S+0x28 ≠ 0 포함) | 0 | primal |
| 1·3(`(c & ~2) = 1`) | — | 0 | primal |
| 2·4 | remaining < 0이면 0, 아니면 1 | | |

B = 0인 primal은 remaining = 0이면 `(model, 0, 0, 0, 0, 0, 0, 0, timer)`, remaining ≠ 0이면 `(model, 0, S+0x168, S+0x168 + 4n, x, x + 8n, 0, 0, timer)`로 **workspace 자기 상태 배열과 push가 쓴 x·slack을 basis 인자로 넘긴다**. dual(B = 1)은 remaining이 양수여도 basis 인자를 넘기지 않는다. 쌍대 검사(asm 0x1bfab8–0x1bfbda): `251b4f(S, S+0x1e0)`로 d(n+m)를 다시 계산하고, 기저(vstat ≥ 0)는 |d| 최댓값, 비기저는 −3 건너뜀, −4는 0, −1·−2는 둘 다 max(0, −d)를 위반으로 보고 위반이 OptimalityTol 이상인 것만 최댓값에 넣는다. 두 최댓값의 합 < OptimalityTol이면 dual. S+0x2d8은 push 뒤(retry = 0, S+0x28 = 0) C 1225–1316이 FeasibilityTol 이상인 비기저 위반 합 + 기저 |d| 합으로 다시 쓴 값이다(이번 검사에서는 입력으로만 기록).

**Q2 retry 잔존(두 시도 뒤에도 retry ≠ 0).** env+0x2340 ≠ 0이면 model+100 = −1006으로 cleanup 없이 종료(C 1365–1368, 미관측). 아니면 env+0x24a0(PerturbValue)을 진입값으로 되돌리고 S+0x454 = 0, 행 i는 basic 위치 i(header[i] = n + i, slack basis), 열 j는 D+0x390/0x398 bound로 `min(max(FIXTOL', 0), FeasTol) < ub − lb`이면 (lb > −I 또는 ub < I)일 때 `lb + ub ≤ 2x`면 −2 아니면 −1, 둘 다 무한이면 −3, 그 밖은 −4. remaining = 0, S+0x184 = 0(C 1363–1521). 따라서 retry 잔존은 auto면 primal, Crossover 2·4면 dual로 basis 인자 없이 cleanup한다.

**Q3 `0325b4` 설치(`0cda5b` C 178–191: `02f7e1` 뒤 S+0xf0 = 1, vbasis·cbasis 둘 다, D+0x324 < 1).** crossover에서는 `13f716`이 S+0x4d8 = 1(C 357)로 두므로 스케일 경로가 꺼지고 x·slack은 바뀌지 않는다. 순서: ① vbasis n개·cbasis m개를 S+0x168로 복사(이 경로에서는 자기 자신), ② `a0011(S+0x1d0)`, ③ D+0x3c = 1이면 D+0x408 flag 열의 −2를 −1로 바꾸고 x 사본의 그 열 부호를 뒤집음(미관측), ④ 비기저 열(vbasis < 0): FIXTOL(S+0x3b0) < ub − lb가 아니면 −4, |x − lb| < τ(S+0x380)면 −1, |x − ub| < τ면 −2, 둘 다 τ 이상이면 열 길이(S+0x78) ≠ 0 또는 |c| > OptTol(S+0x388)일 때 −3(superbasic)이고 |x| > 1e-13이면 목록에 (j, x) 추가, 아니면(빈 열·작은 비용) 원래 상태 유지, ⑤ 비기저 행(cbasis < 0): FIXTOL < ub − lb가 아니면 −4, |s − lb| ≥ τ면 −3·목록 추가(ub 검사와 1e-13 검사 없음), 아니면 −1, ⑥ S+0x450 = 목록 수, S+0x440 = 위치, S+0x448 = 값(S+0x288 작업 목록은 0으로 지움), (S+0x470 factor가 있으면 그것, 없으면 S+0x498)+0xc = 0. 기저 위치(≥ 0)는 그대로 둔다.

**Q4 오라클 방법.** 위 7개 지점의 읽기 전용 훅(§3)과 `0x1bf2ee` slot 편집 합성, 검사기 `tools/xo_cleanup_check.py`의 C 규칙 재계산(선택·쌍대 검사·retry 재설정·`0325b4`)이다. 후속 구현은 이 사건을 Rust `crossover` cleanup 선택·설치 함수의 기대값으로 그대로 쓸 수 있다.

## 3. 비교 계약

| 필드·사건 | 의미 | 비교 규칙 | 캡처 |
|---|---|---|---|
| `sel:enter` | retry, remaining, Crossover, S+0x2d8, S+0x120, S+0x28, local_98, OptTol, vstat(n+m), S+0x184 | 정확·비트 | `0x1bf2ee` 훅 |
| `sel:reset` | retry 잔존 재설정 뒤 vstat·remaining·S+0x184·env+0x24a0 | C 1363–1521 재계산과 정확 | `0x1bf6c2` 훅 |
| `sel:dual` | auto 쌍대 검사(`251b4f` 뒤 d(n+m), vstat) | C 1530–1651 재계산과 비트 | `0x1bfae6` 훅 |
| `sel:decide` | 선택 bool(r14d), S+0x180·S+0x178 | §4 규칙 재계산과 정확 | `0x1bf6f2` 훅 |
| `cda5b:enter` | alg, vbasis/cbasis/x/slack 포인터의 별칭(S+0x168·+4n·x·x+8n 여부) | 정확 | `0x153cc0`(호출자 13f716) |
| `i325b4:enter/leave` | vstat·x·slack 전후, S+0x450·0x440·0x448, lb·ub·τ·FIXTOL·OptTol·열 길이·c, S+0x4d8, sense flag | C 규칙 재계산과 정확·비트 | `0xd3150` 진입·반환 |
| `cda5b:leave` | 반환, model+100, S+0xf0 | 기록(비교 대상 아님) | 반환 훅 |

- 오차 허용 없음. 재현성: 사례당 독립 3회의 사건열 동일.
- 시간·work: 비교하지 않는다(Threads = 1, 로그 끔).

## 4. 사례

| 사례 | 실제/합성 | 노리는 분기 |
|---|---|---|
| Netlib 45 × Crossover −1 | 실제 | auto: remaining 양수 → dual, 0·음수 → 쌍대 검사 또는 primal |
| Netlib 45 × Crossover 1·2 | 실제 | dual-first 뒤 primal push; 1 → primal, 2 → dual(remaining ≥ 0) |
| Netlib 45 × Crossover 3·4 | 실제 | primal-first; 3 → primal, 4 → dual(remaining ≥ 0) |

자연에서 나오지 않는 분기(retry 잔존 재설정, 쌍대 검사 참, superbasic 설치 등)는 미관측으로 적고, 필요하면 합성 편집 추가를 결과 확인 전 결정한다.

## 5. 재실행 명령

- 캡처: `tools/hrun.sh --timeout 590000 -- 'python3 tools/xo_cleanup_runs.py --reps 3'`(작업 worktree, Gurobi 한 프로세스).
- 검사: `python3 tools/xo_cleanup_check.py --controls --acceptance`, `--self-test`.

## 6. 실행 결과

- 환경: x86-64 WSL2 Linux, Gurobi 13.0.3(위 SHA-256), Threads = 1·Seed = 0·OutputFlag = 0·Presolve = 0·Method = 2·LPWarmStart = 2(arms·retry는 ScaleFlag = 0과 PStart/DStart). 계약 `b7e1146`, 합성·검사기 `8c4ebf1`, 결과는 이 branch의 다음 commit.
- 원본 실행: 정식 **768회 모두 성공**(Netlib 45 × Crossover 5값 × 3 = 675, 특수 자연 arms·retry·interior × 5값 × 3 = 45, 합성 16 × 3 = 48). 정식 전 탐색 31회(scratchpad, 기준 자료 아님) 중 4회가 `0x1bfae2` 훅 오류로 실패해 `0x1bfae6`으로 고쳤다. 첫 agent(v1)로 찍은 Netlib 675회는 3회 재현·규칙 일치였으나 retry 재설정 입력(D bound·FIXTOL)이 없어 근거로 쓰기 전에 버리고 새 agent로 다시 찍었다(총 원본 실행 1,478회).
- 재현성(`--controls`): 256사례 × 3회 사건열·최종 VBasis/CBasis 동일.
- C 규칙 재계산(`--acceptance`): 768실행 전부 통과. 분기 집계(실행 수):
  - Netlib 45 자연은 Crossover 5값 모두 remaining = 0·retry 0이다. auto·1·3은 primal(basis 인자 없음), 2·4는 dual이며 각 135회.
  - 자연 remaining > 0(arms·retry, Crossover 1–4): 1·3 → primal + basis 설치, 2·4 → dual(인자 없음). 자연 remaining < 0(Crossover −1, arms·retry): arms는 S+0x2d8 ≠ 0이라 primal + 설치, retry는 S+0x2d8 = 0·PInf > 0이라 **자연 쌍대 검사 참 → dual**.
  - 합성: auto remaining 양수 → dual, 음수 → primal + 설치, Crossover 4 음수 → primal + 설치, 쌍대 검사 강제 7종(참 5·거짓 1: scsd1 → primal, remaining −1과 결합 1), retry 잔존 재설정 3종(auto → primal, 3 → primal, 2 → dual).
  - 쌍대 검사 24회(참 21·거짓 3), retry 재설정 9회(열 −1/−2 재분류·행 slack basis·remaining 0·env+0x24a0 복원 일치), `0325b4` 설치 27회: 상태 변화 342개(모두 −1/−2 → −3), superbasic 목록 345개 위치·값 비트 일치, x·slack 보존. arms Crossover 3은 push가 −1/−2로 놓은 후보 20개가 bound에서 τ 이상 떨어져 있어 전부 superbasic이 됐다.
  - `0cda5b` 호출 지점 3곳·alg·인자 별칭(S+0x168, +4n, x, x+8n)·S+0x180 = 1·S+0x178이 모두 규칙과 같다. cleanup은 768회 모두 반환 0·model 상태 2.
- 검사기 판별력(`--self-test`): 자료 변조 13종 모두 거부. 첫 실행에서 2종이 생존했는데, 둘 다 변조가 잘못 설계된 것이었다(①첫 push의 retry는 소비되지 않아 마지막 push로 옮김, ②d = +1.0은 하한 비기저에서 위반이 아니어서 기저 d로 옮김). 규칙 변이 12종 중 7종 거부. 생존 5종은 자료에 해당 상황이 없어서다. 쌍대 합이 OptTol과 같은 동률(`dual_le`), OptTol 미만 비기저 위반이 결과를 바꾸는 경우(`dual_no_tol_skip`), 빈 열·|c| ≤ OptTol 비기저(`no_cost_gate`), |x| ≤ 1e-13 superbasic(`no_tiny`), ub에 붙은 행 slack(`row_ub`)이 그 상황이다. 결과 `bench/validation/m3_xo_cleanup/check.txt`.
- 미관측(C·asm만): env+0x2340 ≠ 0의 −1006 종료, D+0x3c = 1 sense flag(열 부호 반전), D+0x324 ≥ 1에서 설치 생략, S+0x28 ≠ 0(x87), local_98 = 2(재시작)의 S+0x178 += 10, 위 생존 변이 5개의 분기, `0325b4` 할당 실패.

## 7. 판정과 인수인계

- 작업 수행 상태: 완료(조사 + 오라클). 동등성: Rust 구현이 없으므로 **미검증**(오라클은 원본 재현성과 C 규칙 재계산 일치를 확인).
- 확인한 범위: double·Threads = 1에서 push 반환(retry·remaining) → Crossover 값별 cleanup 알고리즘·basis 인자 선택, auto 쌍대 검사, retry 잔존 재설정, `0325b4` 상태 배열 설치(스케일 없는 crossover 경로)가 768 원본 실행에서 §2.1 규칙과 정확·비트 일치했다.
- 후속: **M3-04f-3**(Rust cleanup 선택·retry 재설정·`0325b4` 설치 이식, 이 자료를 기대값으로 사용). 그 뒤 cleanup simplex 본체 연결(최종 basis 비교)과 04g(재시작·local_98). 재실행: `python3 tools/xo_cleanup_check.py --controls --acceptance --self-test`. 원본 재캡처: `python3 tools/xo_cleanup_runs.py --cases --reps 3 && python3 tools/xo_cleanup_runs.py --reps 3`(새 디렉터리에서; 기존 출력은 건너뜀).
- 교훈: `docs/lessons/original.md`(crossover cleanup 선택과 basis 설치), `docs/lessons/tooling.md`(call 직후 probe의 rsp slot).
