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

(작성 예정)

## 7. 판정과 인수인계

(작성 예정)
