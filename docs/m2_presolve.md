# M2 MIP presolve 작업 노트 (`3cd406`)

MIP presolve 포팅 중에 알아낸 사실. 확정된 내용은 [03](03_presolve.md)로 옮긴다. 작업 방식은 LP presolve([m1_presolve.md](m1_presolve.md))와 같다: 재구성 + 오라클 비트 검증, 단계 오라클은 `GURO_PAR_PRELIMIT`.

## 0. 현재 상태 (2026-11-06)

| G6 기준 | 상태 |
|---|---|
| (1) miplib_small 38개 기본값 presolved 모델 38/38 | **38/38**(2026-11-06, `578fd2`의 정확 knapsack `6b5e11`/`6b56d2` 포팅, 브랜치 `m2-knap`; mip_rec_check 38/38, sweep 기본값 38/38) |
| (2) 기본값에서 도는 MIP reduction 전부(probing·대칭성·연결 성분 포함) | 시작 전. 목록은 §2 |
| (3) MIP postsolve 레코드 | 시작 전 |
| (4) clippy 0, 테스트, 03·이 메모 갱신 | 시작 전 |

운영 규칙: 이 브랜치(`m2-mip-presolve`)에만 커밋, `crates/lp` 기존 파일 수정 금지(MIP 코드는 새 크레이트 `crates/mip`), `docs/11_progress.md` 수정 금지, 공유 `.venv` 설치 금지(`tools/build_local.sh` + `PYTHONPATH=$PWD/python`), Gurobi는 한 프로세스(`tools/gurun.sh`). 하위 에이전트는 브랜치를 붙인 worktree에서만. 크레이트 수준의 파라미터 표(이름·기본값·범위)는 G5(`m1-lp-driver`)가 만든다. `docs/m2_param_offsets.tsv`는 오라클·도구용이고, `crates/mip`에는 MIP presolve가 읽는 필드만 가진 `Params`(LP `presolve::Params`처럼 `from_pairs`)만 둔다(2026-11-06 오케스트레이터 지시). 사용량 한도 때문에 하위 에이전트는 동시에 3개 이하로 두고, 끝난 것은 병합한 뒤 worktree·브랜치를 지운다(같은 날 지시). 토큰 여유가 없어 이후 하위 에이전트는 **한 번에 하나만** 돌린다(2026-11-06 사용자 지시). roundend(대칭 끝, `594c0c` 시작 직전)와 outer-c(two_rows 등)는 일시 중지했고, 각 worktree의 커밋·작업 파일에서 이어 간다.

### 진행 중인 하위 작업 (2026-11-06 시작)
| worktree / 브랜치 | 일 | 산출물 |
|---|---|---|
| ~~`m2-skel`~~ (완료, 병합 `ef28a7e`) | 뼈대: `crates/mip`(상태·생성자 MIP 분기·시작부·마무리·`3b0917`), MPS 정수 MARKER, py 분기. 다시 확인: Presolve=0·PRELIMIT 0/1/2 38/38, 3은 36/38(`5a9282` 미포팅). 카운트: k=1 정수 bound 반올림, k=2 `36d89a`, k=3 `5a9282`, k=4 첫 `3e39a6` | [m2_presolve_state.md](m2_presolve_state.md) |
| ~~`m2-spine-b`~~ (완료, 병합) | 패스 루프 해독. phase(0..1) > round(0..4) > 패스 p(전역) > 내부 i(−5부터). 카운터는 ebx. 내부 반복 7카운트, 바깥 패스 22 + [p<3] + [round==0 && k==0] + [k==0]. 인라인 I→B·PREPARTOBJ·함의 bound·쌍대 이진화 후보 | [m2_spine_b.md](m2_spine_b.md) |
| ~~`m2-spine-c`~~ (완료, 병합 `e492342`) | 라운드 후 단계 해독. 라운드 0–4 × 바깥 반복 2, PRELIMIT 카운트 round 0은 19, 이후 16. 호출 없이 세는 곳 목록 | [m2_spine_c.md](m2_spine_c.md) |
| ~~`m2-spine-d`~~ (완료, 병합) | FINAL 단계 해독. 두 번째 probing 뒤 ρ > 1e-4면 라운드 루프 재시작(최대 1회), 마무리 사슬 `52cba4`→`521497`→`470936`→`57999f`→{`36ddf2`+inline 쌍대 고정}→inline implied int. 카운트 C1–C23 | [m2_spine_d.md](m2_spine_d.md) |
| ~~`m2-probe`~~ (완료, 병합 `54125bd`) | probing `4d3a1d` 전체 포팅(`probe*.rs`, `conflict.rs`, `subst.rs`). 엔진 62/62 패스 비트 일치(`tools/probe_check.py`), 쓰기 13호출 일치(`tools/probe_apply_check.py`). sweep에서 `probe` 첫 불일치 0 | [m2_red_probe.md](m2_red_probe.md) |
| `../rsolver-m2-outer-b` / `m2-outer-b` | `bd6d2e3`까지 병합(`4030c9`, `40d2bc`, `36ee92`, `3e40d1`, `37f484` MIP, `418542`, 덤프 각 78/78). 이 worktree는 사용자가 맡긴 Codex 에이전트가 마무리 중(`37f484` 단계 C 쌍 집계, `3bb6fe`). 끝나면 다시 병합 | [m2_red_outer_b.md](m2_red_outer_b.md) |
| ~~`m2-outer-c`~~ (완료, 병합 `4358dd1`) | `45feaa`(+`45d7ec`), `5466a7`, `47c445` mode 2, inline 쌍대 이진화. 덤프 78/78·78/78·47/47·78/78 | [m2_red_outer_c.md](m2_red_outer_c.md) |
| ~~`m2-final`~~ (완료) | FINAL 단계: inline implied int 척도, `470936` sparsify, `57999f`, inline 쌍대 고정 포팅(덤프 각 38/38, 합성 쌍대 고정 2/2). `52cba4`·`521497`은 하위 풀이(`5307f4`/`522aa2`) 미포팅 shortcut(38개에서 안 돎) | [m2_red_final.md](m2_red_final.md) |
| ~~`m2-roundend-b`~~ (완료, 병합 `641aa0f`) | `838543`, `423e35`, `5aaaf7`, `579e67`, `5a1293`, `5a04fc` 포팅(덤프 전부 일치). `419ffc`, `5a4d8b`, `5a5eff`는 stub(38개에서 작업량만 바뀜) | [m2_red_roundend_b.md](m2_red_roundend_b.md) |
| ~~`m2-clique`~~ (완료, 병합 `3ece196`) | `451bed`, `45885d`, `52be10`, `46a62d` 포팅. 덤프 비교 각 78/78, 레코드 38/38. 덤프에 +0xf0/+0xf4(prelude에서 센 이진·정수 수)를 더함 | [m2_red_clique.md](m2_red_clique.md) |
| `../rsolver-m2-roundend` / `m2-roundend` (일시 중지) | 병합(`ad481f7`): 라운드 끝 A + 대칭 `5577ba` MIP 분기(`5507cb`, use 블록, type 19, 덤프 38/38 f4 포함). 남은 일 `594c0c`(작업 파일 `inner.rs` 수정 중에 멈춤) | [m2_red_roundend.md](m2_red_roundend.md) |
| `../rsolver-m2-outer-a` / `m2-outer-a` (대기) | 병합: `374521`, `5431ee`, `3e5363`, `57627e`, `3ff744` mode 0–3(+열 집계, `add_cols`/`add_rows`), `578fd2`(+`6aec61` mode 2, `6b4738`) — 덤프 전부 일치, `578fd2`만 66/78. 남은 일: `6b5e11`(11–512개 이진 행 정확 knapsack, asm 3,491줄)과 `6b56d2` 포팅 → gmu 둘의 첫 불일치(k=28). 노트 §5.3에서 이어 감 | [m2_red_outer_a.md](m2_red_outer_a.md) |
| ~~`m2-inner`~~ (완료, 병합 `a4dcbf1`) | 내부 루프 7개 + inline 함의 bound 포팅. 덤프 비교 147–225/호출 전부 일치(`tools/mip_dump_check.py`), 레코드 38/38 바이트 일치(`tools/mip_rec_check.py`, `CoreModel.mip_post_records`). 공용 보조 API는 `helpers.rs`(m2_red_inner.md §11) | [m2_red_inner.md](m2_red_inner.md) |

### 제어 흐름 (2026-11-06, `crates/mip/src/driver.rs`)
spine 노트 b·c·d대로 `3cd406`의 단계(≤2) × 라운드(≤5) × 패스 × 내부 반복, 라운드 끝, FINAL을 옮겼다. 카운트 62곳(reduction 없이 세는 I→B·PREPARTOBJ·함의 bound·쌍대 이진화·패스 끝·FINAL 쌍대 고정 포함), timer 28개(시작마다 +1, 한도 B·0.5B·2B·4B·3B), 스냅숏 셋과 진척도 판정 셋, inline I→B·PREPARTOBJ·열 타입 통계·두 번째 probing 판정. stub(`stubs.rs`)은 모두 포팅되어 지웠다(마지막: `419ffc`, `5a4d8b`, `5a5eff`). 확인: 아무것도 안 바뀌는 5개 모델의 총 카운트가 Gurobi 호출 기록과 같다(neos5 62, 두 번째 probing이 없는 모델 61).

### 남은 묶음 (아직 담당 없음, 동시 3개 한도 안에서 차례로)
- 그 뒤: `594c0c`(roundend, `inner.rs` 작업 파일), `6b5e11`/`6b56d2`(outer-a 노트 §5.3), `52cba4`/`521497` 성분·블록 탐색 포팅(G6 기준 (2)).

### 다음 단계 (순서)
1. 위 네 브랜치를 `m2-mip-presolve`에 합친다. 구간 노트로 `3cd406` 제어 흐름 전체를 reduction stub(카운트만)으로 옮긴다. stub이 Gurobi와 같은 k에서 카운트하는지는 sweep의 "바뀐 k"와 reduction 위치가 맞는지로 확인한다.
2. 내부 루프 reduction(`3e39a6`, `36ddf2`, `37168e`, `36c5cf`, `56f793`, `56c80e`, `36e6b6`)을 먼저 옮긴다. 모든 모델이 이 루프를 지난다. 검증은 PRELIMIT 체크포인트와 상태 오라클(`mip_state_dump.js`).
3. 바깥 reduction을 묶음으로 하위 에이전트에 나눈다(각자 worktree·브랜치, 새 파일). 앞 reduction이 아직 없어도 상태 오라클로 진입 상태를 넣어 한 reduction씩 비트 비교할 수 있게 상태 적재 함수(`State::from_dump`)를 둔다.
4. probing `4d3a1d`, 대칭 `5577ba`(LP 포트와 MIP 분기 차이), 연결 성분 `52cba4`, sparsify `470936`, 마무리 inline 블록.
5. postsolve 레코드 비교(레코드 목록 오라클), 작업량, clippy·테스트·문서.

## 1. 호출 그래프와 실제 호출: MIP presolve는 LP를 풀지 않는다 (2026-11-06)

### 1.1 P0 경로
`Model.presolve()` = `GRBpresolvemodel`(`999702`). 연속 모델이면 `390989`(LP presolve), MIP(`a7f063` ≠ 0)이면 `3db19d` → `3db15a`(재시도 래퍼: "Presolved model has numerical issues" / "Q matrix is non-PSD after presolve substitutions"면 PREQSUBST=0·AGEQ=0으로 한 번 더) → `3cd406(model, probeMode, mipctx=0, 1, timer, …)`. 비볼록 연속 모델도 `3db19d`로 간다("Continuous model is non-convex -- presolving as a MIP").

### 1.2 정적 호출 그래프 (`re/decomp/callgraph.tsv`)
`3cd406`의 직접 callee는 158개, 닿는 함수는 2,767개다(04_simplex 318개, 05_barrier 223개, 06_bnb 258개, 07_cuts 377개 포함). `390989`도 같은 2,767개에 닿는다. 둘 다 아래 탈출 간선을 통해 드라이버 `c23877`이 든 거대한 강연결 성분에 들어가기 때문이다(`3cd406 → 5b58f4 → 98b671 → c23877 → 390989`). 그래서 정적 도달만으로는 판정이 안 되고, 탈출 간선을 하나씩 보고 실제 호출을 쟀다.

| 탈출 간선(직접 callee → solver 쪽) | 조건·의미 | 기본값 38개에서 |
|---|---|---|
| `5b58f4` → `98b671` → `c23877`/`7a0103` | 해 기반 축약(SOLREDUCTION). 문제 유형(+0x08) == 11, DualReductions, state+0x244 == 0, env+0x2590 ≥ 1, model+0x1b8의 해 정보가 있을 때(3cd406 L1668–1675) | 0회 |
| `521497` → `522aa2` → `98b671` | 연결 성분 뒤 SOS·QC·일반 제약 블록 | `521497` 38/38, `522aa2` 0회 |
| `99ceb1`, `49f868`, `58b53e`, `4b2054`, `b44b00`, `b651e5`, `9951ed`, `9a4ba6`, `312e55`, `317cdd`, `451b10`, `ad531f` | 모델 생성(`3b0917`, L9765) 뒤 PWL·일반 제약·QP/QC PSD 검사·PreQLinearize·MIQCP 형식(barrier Cholesky `2d8e22`/`313802`, 스케일링 `7d3ce`로 이어짐) | 0회 |
| `451195` → … → `300f28` | QC presolve(state+0x27c ≠ 0) | 0회 |
| `4d3a1d` → `ac761d` | probing의 스레드 분할 | `4d3a1d` 38/38(61회), `ac761d` 0회 |
| `47c445` → `d07d1` | simplex 반복 로그. spx 문맥(param_3)이 있을 때만(LP와 같음) | `47c445` 38/38, `d07d1` 0회 |
| `4962bd` → `5eda00` | 재시작 문맥에서 행 3–7개 남은 모델 직접 풀기 | 0회 |
| `a71757` → `7d3ce` | 결과 계수 통계(numIssue 판정). 스케일링은 부르지 않음 | `a71757` 38/38, `7d3ce` 0회 |
| `3ba5ac` → `9a91f3` | 콜백 디스패처(where=PRESOLVE). 사용자 콜백이 없으면 바로 돌아온다 | 7,718회(호출만) |
| `838543` → `83e123` | cut 모듈의 mod-k/zero-half 보조 루틴 재사용(PRESS2MIP·PREPROBE0·PRECOVERS 블록) | 15개 모델 3,860회 |

### 1.3 실제 호출 (`tools/presolve_cov.js`)
Frida로 38개 모델의 기본값 `Model.presolve()`(Threads=1, WorkLimit 60, `tools/parity.py`와 같은 설정)를 돌리며 `3cd406`에서 닿는 함수 중 common·01_model_api·excluded를 뺀 1,883개에 훅을 걸었다.
- **04_simplex는 한 번도 불리지 않는다.** simplex 초기화 `1e7531`, LP 드라이버 `c3d0d`, 메인 드라이버 `c23877`, barrier `2d8e22`·`13f716`, 스케일링 `7d3ce` 모두 0회.
- 불린 비-presolve 함수: 02_driver의 파라미터·로그·모델 질의 헬퍼(`a7f063`, `a7f0ad`, `a7f821`, `bb8b1f` 등, 모델마다 1–6회), `9a91f3`(콜백 디스패처), `ac7c92`/`ac7e13`(모델 질의), `83e123`(위 표), `3bbf87`·`3bde19`·`535d61`·`536369`(평행 처리 `3ff744`·`470936`·`594c0c`의 보조), `57b1ce`(SOS 검사), `ac4062`(해시, probing·대칭성), `b3ee9d`.
- 결론: **MIP presolve는 LP 풀이·simplex·barrier를 부르지 않는다.** 범위 밖(B&B, cut, 휴리스틱, simplex)과 겹치는 것은 `838543`이 쓰는 cut 모듈 보조 루틴뿐이다.

### 1.4 LP presolve(`390989`)와 공유하는 함수
`3cd406`과 `390989`가 함께 부르는 직접 callee: `3a484d`(상태 생성), `3b0917`(모델 생성), `3b8cf6`, `3ba5ac`/`3ba9de`(콜백·예산), `3c546c`/`3c5595`(스냅숏·진척도), `3ff744`(평행), `47c445`(PreDepRow), `5577ba`(대칭), `470936`(sparsify), `536971`/`5369b8`/`5383d0`/`538f30`(추적기), `312e55`, `42872b`, `451195`, `9a4ba6`, `88c65` 등.

기본값 38개에서 실제로 불리고 `crates/lp/src/presolve/`에 이미 포팅된 함수: `3a484d`, `534a8b`, 추적기(`5369b8`, `53763f`, `5379f1`, `537da8`, `53802c`, `538353`, `538f30`, `5399fc`, `53a487`, `53af0f`), `3ba5ac`, `3ba9de`, `3c546c`, `3c5595`, `3ff744`, `36dce1`, `47c445`, `5577ba`, `ac4062`, `3b0917`, `5a4d8b`, `5a5eff`, `3a0e1b`, 집계기 `37f484`와 그 보조(`370047`, `36fcb5`, `36f5c4`, `3773aa`, `377cb6`, `3791fd`, `395c97`, `39b9e0`, `53d139`, `53d18c`, `53d2b0`, `53d3d9`), `470936`(호출 지점만).

주의: LP 포트는 CSR 상태(state+0xe0 = 1) 분기만 옮겼다. MIP는 연결 리스트 상태라 `3ff744`의 콜백 묶음, `3a484d`의 MIP 분기(세 번째 인자 1), `3b0917`의 정수 타입 처리가 다르다. LP 하위 모듈은 모두 private(`mod state;` 등)이고 `crates/lp` 기존 파일은 고칠 수 없으므로, **MIP는 `crates/mip`에 자기 상태를 두고 LP 포트의 코드를 옮겨 와 MIP 분기를 더한다**(03 §1의 "따로 포팅" 결정과 같다).

## 2. 기본값에서 도는 reduction (coverage, 38개 전부에서 불림)

`3cd406`의 직접 호출 순서(neos5, 한 패스. 여러 패스·라운드 모델은 가운데가 반복된다). 크기는 기계어 바이트, P는 LP 포트에 있음.

| 단계 | 함수(크기) |
|---|---|
| 시작 | `3a484d`(36,718 P), `5b84b1`, `36d89a`(531) 빈·자유 행, `370047`(702 P) activity, `5a9282`(2,465) |
| 내부 루프 | `3e39a6`(9,318) 열 패스, `36ddf2`(1,965) 행 정리, `37168e`(4,058) bound 강화, `36c5cf`(2,503) singleton 행, `56f793`(2,349) 계수 강화, `56c80e`(6,228) doubleton 집계, `36e6b6`(770) GC |
| 바깥 패스 | `3e5363`(5,451) XOR, `3ff744`(16,559 P) 평행 ×3, `57627e`(10,175) implied int, `578fd2`(1,281) knapsack, `374521`(12,783) 쌍대 bound, `5431ee`(3,175) 함의 SCC, `4030c9`(21,516) 지배 열, `414c03`→`40d2bc`(17,951) 행 비영 소거, `36ee92`(357) 압축, `3e40d1`(2,622) 부등식→등식, `37168e`, `37f484`(34,685 P) 집계, `414c53`→`40d2bc` 열 비영 소거, `53af0f` + `3dcc78`(1,990) ×행, `418542`(2,645) 쌍대 집계, `57b1ce`, `45885d`(13,962)→`451bed`(15,214) clique, `52be10`(5,691), `46a62d`(20,983), `5466a7`(11,233) VUB 병합, `45feaa`(21,340) two-row, `47c445`(34,430 P) |
| 라운드 후 | `58d4ce`(361), `5577ba`(2,900 P) 대칭, `5a9282`, `5ab429`(1,113), `59c193`(7,021), `55a779`(10,702) 집계 패스, `594c0c`(11,746), `423e35`(7,059), `419ffc`(7,235), `5a1293`(3,594), `5a4d8b`/`5a5eff`(P), `5aaaf7`(1,221), `838543`(5,003) ×3, `579e67`(2,881), `4d3a1d`(40,233) probing, `5a04fc`(6,792) |
| 마무리 | `4d3a1d`, `52cba4`(2,377) 연결 성분, `521497`(5,790), `470936`(12,999 P) sparsify, `57999f`(492), `36ddf2`, `36d89a`, `3b0917`(23,326 P) 모델 생성, `a71757`(8,388) 계수 통계, `3b8cf6` 해제 |
| 일부 모델만 | `3dd6f1`(30개) activity 증분, `39ab33`(20개), `3a061e`(7개, type 9), `3df916`(2개), `564ce0`(1개) |

- 불린 presolve 함수(직접·간접) 158개, 기계어 722KB. 그중 LP 포트에 없는 것 459KB.
- 불리지 않은 직접 callee(훅을 건 것 중): `37c34c` AGEQ, `3c2f36` 쌍대 정수→이진, `414ca3` clique 부분집합, `416036` 행 포함, `4178ae` 쌍대 slack, `42872b`/`451195`/`56b360`/`564ce0`(Q·QC), `490ab6`/`4962bd` 작은 모델 직접 풀기, `54807f` 대칭 변환, `57b2bf`/`586049` SOS, `585a6e`, `58b41b`, `5a7c0f`, `5b1161` NL, `5b58f4` 해 기반, `9ee0cf` PWL, 일반 제약 블록(`b6fd6f`, `b832dc`, `b84382`).
- "불림"은 reduction이 모델을 바꿨다는 뜻이 아니다. 무엇이 실제로 모델을 바꾸는지는 PRELIMIT 체크포인트 차이로 잰다(§3).

## 2a. 도구

| 도구 | 내용 |
|---|---|
| `node tools/presolve_cov.js <진입 함수> <out.json> [name ...]` | Frida coverage. 모델마다 함수별 호출 수, 비-presolve 함수의 첫 호출 backtrace, 진입 함수의 직접 호출 순서(`seq`: [함수, 복귀 주소]). 기본 모델은 MIPLIB 38개. Gurobi 한 프로세스(`tools/gurun.sh`로 감싼다) |
| `tools/prelimit_sweep.py <out> [name ...]` | 모델마다 PRELIMIT k = 0, 1, 2, …의 presolved 모델. 앞 k와 달라진 k의 MPS만 남기고 `sweep.tsv`(k, 해시, 바뀜)를 쓴다. 기본값 결과와 40번 연달아 같으면 멈춘다. 38개 약 2분 |
| `node tools/mip_state_dump.js <model> <함수> <n번째\|all> <out.json> [Param=Value ...]` | 상태 오라클. `5b84b1(env, state+0x3f0)`에서 상태 포인터를 잡고, 지정 함수의 n번째 호출 진입·복귀 때 연결 리스트(행·열 순서 그대로 [행, 열, 값]), lb/ub/obj/vtype, rhs/sense, 개수, activity, 카운터(+0x3f8부터 14개), 작업량, 레코드 목록(state+0x370, push 순서로 [type, 원시 바이트, rec 주소, data 주소])을 JSON으로 덤프한다. MIP type 1 레코드는 16바이트 {열 i32, 패딩, 값 f64}로 LP(쌍대 정보 포함)와 다르다(glass4). reduction 하나를 따로 맞출 때 쓴다(진입 상태를 넣고 돌려 복귀 상태와 비교). glass4 `3b0917` 진입에서 행 390·열 316·nnz 1803이 presolved 모델과 같음을 확인 |
| `PYTHONPATH=python python tools/sweep_check.py <sweep> [-v] [name ...]` | 포트를 sweep의 모든 변화점 k와 기본값 결과에 비교해, 모델마다 첫 불일치 k와 그 k가 포트 추적(`CoreModel.mip_presolve_trace`)에서 어느 reduction인지 알려 준다. Gurobi 불필요 |
| `docs/m2_param_offsets.tsv` | `ac81d7` 등록 표의 파라미터 415개(이름, env 오프셋)와 getParamInfo 기본값·범위 |
| `tools/presolve_oracle.py <dir> --param GURO_PAR_PRELIMIT=k <name ...>` | LP와 같은 PRELIMIT 체크포인트. `3cd406`도 reduction 호출마다 카운터(local_68)를 올리고 state+0x1dc와 비교해 마무리(`LAB_004b2415`)로 간다(L1651) |

확인(2026-11-06): `presolve_oracle.py` 기본값 출력이 `bench/gurobi/<이름>/presolved.mps.gz`와 38/38 바이트 단위로 같다. PRELIMIT k = 0, 1, 2 출력은 38/38이 Presolve=0 출력과 같고, k = 3에서 2개, 5에서 18개, 10에서 29개가 갈라진다. 기본값과 같은 것은 k = 0에서 5개(presolve가 아무것도 안 바꾸는 모델)다.

## 3. PRELIMIT sweep: 출력을 바꾸는 호출 (2026-11-06)

`tools/prelimit_sweep.py`로 38개 모델의 k = 0, 1, 2, …를 떴다. 모델마다 출력이 바뀐 k(괄호는 바뀐 내용: 행·열 수 증감, sense, 계수, 목적, nz, bound, 정수 표시, rhs 개수).

- 출력이 한 번도 안 바뀌는 모델 5개: cod105, exp-1-500-5-5, gen-ip002, neos5, pk1 (기본값 결과 = Presolve=0 결과).
- 공통 위치: k = 3–4(열 제거, 첫 내부 루프의 열 패스로 보임), 6(bound, 30개 모델), 7(행 제거), 9(계수), 10–13, 20(정수 표시 변경: csched007/008 `int246`/`int238`, markshare `int7`/`int4`), 21–25, 28(계수), 31–33, 36(sense), 38, 43–45, 76 등.
- 바뀐 횟수가 많은 모델: gmu-35-40·gmu-35-50 26, tr12-30 23, csched008 16, sp150x300d 15, timtab1·neos-4954672-berkel 14, csched007 13, neos-860300 12.
- 전체 표: 세션 스크래치 `sweep_summary.tsv`(`sweep_an.py`로 다시 만든다). k와 reduction의 대응은 `3cd406` 카운트 지점 해독(구간별 노트 `m2_spine_*.md`) 뒤에 채운다.
