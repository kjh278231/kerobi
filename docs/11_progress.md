# 11. 진행 기록과 운영

## 1. 운영 규칙 (2026-11-05 사용자 지시)

오케스트레이터(Herdr 워크스페이스 w4 1번 탭의 Claude 세션)가 M 단계 작업을 탭별 에이전트에게 나눠 맡긴다.

1. **병렬**: 다른 파일·크레이트를 건드리고 따로 검증할 수 있는 단위는 별도 git worktree와 탭에서 동시에 돌린다. 관문 판정은 마일스톤 순서대로 하되, 독립 단위는 앞당겨 진행한다. 브랜치를 main에 합치는 일은 오케스트레이터가 한다.
2. **토큰 한도**: 3시간마다 한도로 멈춘 에이전트를 찾아 이어서 작업을 요청한다.
3. **컨텍스트 70%**: 15분마다 확인한다. 70% 이상이면 순서대로 한다.
   1. `/goal clear`
   2. 하던 작은 단위를 마치고 인수인계 메모(§2 표)를 갱신해 커밋하게 한다.
   3. 같은 탭에서 `/clear`로 새 세션을 연다.
   4. 메모를 읽고 이어 가게 하고, §4의 goal을 다시 건다.
4. **완료 기록**: 마일스톤 안의 단계(A, B, C…)나 관문이 끝나면 §3에 날짜·결과·커밋을 적는다.

공통 규칙(모든 에이전트):
- Gurobi는 한 프로세스만 돌린다(`gurobi_guard.pth`). "already running"이면 기다렸다 다시 시도한다.
- 공유 `.venv`에 `maturin develop`이나 `pip install`은 main 트리만 한다. worktree는 cargo로 빌드한 `.so`를 자기 `python/rsolver`에 두고 `PYTHONPATH=$PWD/python`으로 쓴다.
- worktree는 `crates/lp`의 기존 파일을 고치지 않는다(`lib.rs` 모듈 등록은 허용). 커밋은 자기 브랜치에만, 단계마다 한다.
- `docs/11_progress.md`는 오케스트레이터가 main에서만 고친다. worktree 에이전트는 자기 인수인계 메모에만 적는다.
- 답변은 한국어로 한다.

## 2. 작업 흐름

| 탭 | 폴더 / 브랜치 | 담당 | 인수인계 메모 | goal |
|---|---|---|---|---|
| M1 심플렉스 | `rsolver` / `main` | LP simplex(dual·primal), LU, crash, Quad, LP 드라이버, `crates/core` 삭제 | `docs/10_porting.md` §9 | G1 |
| M1 LP presolve | `rsolver-presolve` / `m1-lp-presolve` | LP presolve, postsolve | `docs/m1_presolve.md` | G2 |
| ↳ postsolve (presolve 에이전트가 띄운 하위 에이전트, Sonnet) | `rsolver-postsolve` / `m1-postsolve` | postsolve `363daa` | `docs/m1_postsolve.md` | G2의 일부 |
| M3 barrier | `rsolver-barrier` / `m3-barrier` | barrier 본체(crossover 제외) | `docs/m3_barrier.md` | G3 |

## 3. 완료 기록

| 날짜 | 단계 | 결과 | 커밋 |
|---|---|---|---|
| 2026-11-05 | M0 관문 | 지문 102/102 | `fc0dcd0` |
| 2026-11-05 | M1 A 스케일링 | 열·행·목적 스케일 64/64 비트 | `503542d` |
| 2026-11-05 | M1 C singleton LU·FTRAN | 반복 0 `xB` 60/60 비트 | `e0eb167` |
| 2026-11-05 | M1 C BTRAN·`d`·DSE 가중치 | 60/60 비트 | `cb66a2b` |
| 2026-11-05 | M1 B 시작 기저 | `vstat` 64/64 | `fff2399` |
| 2026-11-05 | M1 presolve 0–1b | Presolve=0 64/64, PRELIMIT 1–3 64/64 | `87c95d8` (`m1-lp-presolve`) |
| 2026-11-05 | M1 presolve 1c–1g | 열 패스, singleton 행, forcing 행, 중복 행, 바깥 패스. PRELIMIT 1–14 64/64 | `3bed233` (`m1-lp-presolve`) |
| 2026-11-05 | M1 LP 엔진 종단 연결 | dual·primal simplex가 스케일된 작업 공간에서 돈다. 읽은 모델에서 Python `optimize()` 동작 | `ac30617` |
| 2026-11-05 | M3 준비 | 호출 범위 확인(simplex·LU·presolve와 독립, 공유는 스케일링 `07d3ce`뿐), Method=2 기준값, barrier 오라클·모듈 뼈대 | `8405f7c` (`m3-barrier`) |
| 2026-11-05 | M1 dual 반복 경로 | BFRT·cost shifting, 수치 안전장치 사다리, 가중치 리셋, 재분해 뒤 flip. `crates/core` 삭제(만든 LP도 LP 엔진 사용), Markowitz count bucket | `c9fddee`, `33a2342` |
| 2026-11-05 | M1 postsolve 준비 | uncrush 오라클(`363daa` 훅), 저장·종료 규칙과 LP uncrush 경로를 03에 반영 | `e707fea`, `f0622fa` (`m1-lp-presolve`) |
| 2026-11-05 | M3 정규방정식·ordering | 결정성 64/64, small-pivot 규칙의 LDLᵀ, A·Θ·Aᵀ 조립, AAᵀ 패턴 64/64, AMD 순열·symbolic 53/64 | `66054a0` (`m3-barrier`) |
| 2026-11-05 | M1 presolve 2a–2c | 비용 선호점 고정, 함의 중복 행, dual bound 전파. PRELIMIT 1–14 64/64. PreDepRow는 Netlib에서 degen2·degen3만 바꿈 | `eb471f7` (`m1-lp-presolve`) |
| 2026-11-05 | M3 ordering·symbolic | AAᵀ NZ, 순열(AMD + bisection), supernode, Factor NZ/Ops 64/64 | `f1d0841`, `8bc322b` (`m3-barrier`) |
| 2026-11-05 | M3 IPM 프로토타입 | 초기점, predictor, Mehrotra/Gondzio corrector, line search, Gurobi pivot 규칙 LDLᵀ와 refinement | `06308ed` (`m3-barrier`) |
| 2026-11-05 | M1 dual 회귀 수정(israel) | 인공 bound 배율에서 flip 뒤 `x_B` 재계산 | `6013997` |
| 2026-11-05 | M3 IPM 프로토타입 64개 | 상태 64/64, 목적 ≤1e-6 64/64, **BarIterCount 56/64**(±5% 57/64). Barrier statistics 로그 64/64. 불일치 8개는 해법 반올림 누적 | `e905669`, `9b4c612` (`m3-barrier`) |
| 2026-11-05 | M1 dual CHUZR 원본 이식 | 순서 있는 후보 목록(표시·재구축 허용량), BTRAN 비용 점수, 인공 bound 단계(1e19). 반복 0 목록 63/64 | `899c192`, `e412440` |
| 2026-11-05 | M1 presolve 3a–3b | 평행 열·행, PRESPLITROWS, 비용으로 조이는 부등식 행. PRELIMIT 1–17 64/64(기본값 최종 모델은 3/64) | `aba84fe` (`m1-lp-presolve`) |
| 2026-11-05 | M3 Rust IPM | Method=2 경로, 상태·목적 64/64, Barrier statistics 64/64, `2d8ee5` pivot 규칙으로 **BarIterCount 59/64**(92%, G3 P1 기준 90% 넘음), PRIMINF 반복 0 64/64 | `a4ab223`, `ddc3040`, `df38e2a` (`m3-barrier`) |

## 4. goal 원문

- **G1**: M1에서 LP presolve·postsolve를 뺀 나머지를 완료한다. 완료 기준(Presolve=0, Threads=1):
  1. Netlib 91개에서 Method 0/1/기본 모두 상태가 Gurobi와 같고, 목적 상대오차 ≤1e-6이다. 크기 제한 밖 27개는 공개 최적값과 비교한다.
  2. P1: Netlib 64개에서 IterCount가 Gurobi와 같은 인스턴스가 90% 이상이다. 나머지는 ±5%이고 첫 분기점을 기록한다.
  3. P2: work ±5%.
  4. P3: 시간 기하평균 ≤1.2배. 기계가 한가할 때 잰다.
  5. LP 드라이버(`0c3d0d`), dual·primal simplex, Markowitz·FT LU, crash, Quad, 필요한 common을 포팅하고 `crates/core`를 지운다. cargo clippy 경고 0, 테스트 전부 통과.

  범위 밖은 LP presolve·postsolve(`rsolver-presolve`)와 barrier(`rsolver-barrier`)다. Presolve=1 검증과 M1 관문 속도 스냅숏은 presolve를 합친 뒤에 한다.
- **G2**: LP presolve(`390989`)와 postsolve를 완료한다. 완료 기준:
  1. P0: Netlib 64개에서 기본 파라미터의 presolved 모델(`presolved.mps.gz` SHA-256)이 Gurobi와 64/64 같다.
  2. presolved 모델의 해·기저를 postsolve로 원래 공간에 되돌린 결과가 Gurobi와 비트 단위로 같다. 64/64, 오라클로 검증한다.
  3. 미뤄 둔 `47c445`(PreDepRow)를 포함해 기본값에서 도는 reduction을 모두 옮긴다.
  4. cargo clippy 경고 0, 테스트 통과. `docs/03_presolve.md`와 `docs/m1_presolve.md`를 갱신한다.

  범위 밖은 simplex·드라이버(main)와 barrier다.
- **G3**: barrier(Method=2) 본체를 완료한다(crossover 제외). 완료 기준(Presolve=0, Threads=1, Crossover=0):
  1. Netlib 64개에서 상태가 Gurobi와 같고, 목적 상대오차 ≤1e-6이다.
  2. P1: BarIterCount가 Gurobi와 같은 인스턴스가 90% 이상이다. 나머지는 ±5%이고 첫 분기점을 기록한다.
  3. ordering 결과와 Factor NZ가 Gurobi 로그와 같다.
  4. 반복마다 BARRIER 콜백 오라클과 비교하는 도구를 둔다.
  5. cargo clippy 경고 0, 테스트 통과. `docs/05_barrier_pdhg_concurrent.md`와 `docs/m3_barrier.md`를 갱신한다.

  범위 밖은 crossover, PDHG, concurrent, presolve, simplex다.
