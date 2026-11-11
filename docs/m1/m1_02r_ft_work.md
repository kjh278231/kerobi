# M1-02r 검증 계약과 결과: FT 갱신 `05b257` work 지점

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. [M1-02q](m1_02q_work.md) §7의 첫 후속이다. 사용자의 "다음 스프린트 실행해 너가 작업자야" 지시로 착수했다(2026-11-11). M2는 다른 세션이 M2-06e-2를 점유해 M1을 골랐다([시작 절차](../handoff/README.md) §1-3).

## 1. 범위와 주장

- 작업 종류: 구현(새 원본 캡처 없음. M1-02p watchpoint 캡처 재생).
- 완료할 동작: Rust `Lu::update`(FT 갱신)와 재분해 경로가 원본 `05b257`의 work 저장 16지점(FT 갱신 15 + 재분해 경로 0xf461f)을 원본과 같은 순서·양으로 더한다.
  - 갱신 경로(원본 저장 순서): 0xf1beb(η 길이·3) → 0xf1d02(LU+0xe8 내림 탐색 거리·3) → 열 r 원소마다 0xf1f82(행 j 안 r의 위치 p − s) 또는 0xf1f37(길이 1 행이 접두 목록을 떠남, 3) → 0xf2044 | 0xf207a(옛 열 길이·5, 저장 생략 시 carry) → 0xf2521(spike 목록 길이 nS′·3) → 행 r 열마다 0xf3f71(구간 2 p − s) | 0xf2969(구간 1 p − s) 또는 0xf4073(local_80 = 3, 구간 2 p − s) → 0xf2a51(U 행 r 길이·3) → 0xf3eac(빈 열 자리로 바뀐 p의 행, 열마다 구간 1 p − s) → 0xf3ca4(순환 이동 Σ(2·열 길이 + 5), LU+0x1f8) → 0xf3d83(이동 칸 수·2) → 0xf2e3c(nS′·5).
  - 재분해 경로 0xf461f(spike 목록 지우기, (k + k)·x148, k > 0일 때만): `036ade` 저장 뒤, `16b0a` 0xbe793·0xf48ae 앞.
- 주장 범위: kb2 전 구간(재분해 5회와 그 사이 갱신 전부), degen2 피벗 0–355(재분해 F1–F9와 F10 앞 갱신)에서 위 지점과 02q 지점의 저장 열(지점 순서)·각 저장 값이 원본과 비트까지 같다.
- 입력으로 받는 것: 02q와 같이 **재분해 진입 work(spx+0x150)는 원본 캡처 값**(`RSOLVER_WORK_SEED`). 반복 커널·LU 풀이 등 미구현 지점이 있어 Rust work 절대값은 원본과 다르다. 저장 비교는 원본 직전 쓰기 + Rust 양(§3 W).
- 제외와 후속:
  - 원본이 저장하지만 Rust 경로에 없는 경우(관측 0회 예상, 미검증): 새 대각 |S_r| < 1e-13 조기 종료(원본은 0xf1beb만 저장하고 재분해), l ≠ p인데 η가 비고 U 행 r이 남은 오류 종료, 풀 용량 초과 종료(0xf2521 뒤 재할당·재분해), `9e47c` 호출(r ≠ LU+0x1d8, dual은 BTRAN이 η를 미리 둔다), spike 목록에 ≤ ZERO 원소, primal 호출자(`sorder` 없음).
  - `1ca57`·`16b0a` 0xbe793 → M1-02s. LU 풀이 커널 → M1-02t. work를 재분해 판정에 연결하지 않는다(02q와 같음).
- 선행: main `51156d3`(M1-02q 병합). 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca2…0e98f9c`, 주소는 file offset. 모델 `../re/bench/data/netlib/kb2.mps.gz`·`degen2.mps.gz`, Method=1 Presolve=0 Threads=1.
- 근거: [02p 부록 A](m1_02p_formulas.md) 2부 A와 이번에 다시 읽은 `05b257` C:240–1330·C:2006–2033.
  - C:265–315 l(iVar35)·p(iVar52), C:316–420 l ≠ p이면 LU+0x1d0 η 길이 > 0일 때 대각 내적 뒤 0xf1beb(`x148·3.0·n`), η가 비면 U 행 r이 비어야 계속(아니면 오류 종료), l = p이면 η 길이 0. C:441–444 r을 spike 목록에서 빼고 nS′ = 목록 길이 − 1. C:445–455 |S_r| < 1e-13 종료.
  - C:458–512 local_40, q < p 분기의 LU+0xe8 탐색은 찾든 못 찾든(p ≥ e8이면 거리 0) 0xf1d02를 저장.
  - C:513–523 local_80: η 길이 ≥ 1이면 q < t → 1, q ≥ t → 2. η 길이 0이면 nS′ > 0 ∧ l = p ∧ q < t → 3, 아니면 0.
  - C:525–570 열 r의 저장 순 원소마다 길이 1 행(pos ≤ local_40이면 0xf1f37, 아니면 저장 없음) 또는 행 안 위치 0xf1f82.
  - C:597–795 옛 열 길이 cU < nS′이면 0xf2044, 아니면 nS′ > 0일 때 0xf207a, 둘 다 아니면 carry. 그 뒤 0xf2521은 항상 저장.
  - C:851–880 local_80 = 3: U 행 r의 열마다 구간 2 [LU+0x98, LU+0x90) 탐색 0xf4073 → 0xf2a51. C:882–1037 η 길이 ≥ 1: 열마다 local_80 = 1이면 구간 2 0xf3f71, 2이면 구간 1 [LU+0x88, LU+0x98) 0xf2969 → 0xf2a51. local_80 = 0이면 0xf2a51 없음. 양 0이어도 저장한다.
  - C:1043–1086 local_148 = 1(빈 열을 찾아 p를 옮김): ord[p] 행의 열마다 구간 1 탐색 0xf3eac.
  - C:1129–1218 l > p: 순환 단계마다 acc = (acc + 2·열 길이) + 5, 0xf3ca4(`acc·LU+0x1f8`), 0xf3d83(`(k·2.0)·x148`, k = 이동 칸 수). l ≤ p이면 둘 다 없음.
  - C:1225–1316 새 열 삽입 뒤 0xf2e3c(`nS′·5.0·x148`) 항상 저장. C:1329 LU+0x1c8 길이 = 0.
  - C:2006–2033 재분해 경로: LU+0x1c8 길이 k > 0이면 값 지우기 뒤 0xf461f(`(k + k)·x148`). 갱신이 끝나면 0이므로 k > 0은 FTRAN 뒤 갱신 대신 재분해한 때뿐이다.
- 캡처 순서(02p `kb2_w_1`·`degen2_w_1`): 갱신 지점은 위 순서, 0xf461f는 `036ade` 저장(0xdcb7b 포함) 뒤 0xbe793 앞. 원본 k(`8b473` 호출 순번)는 Rust `iter`(pivot 시점)와 같다(02q A).
- 관측 횟수(kb2/degen2, 02p `sites.tsv`): 0xf1beb 25/422, 0xf1d02 19/373, 0xf1f37 1/97, 0xf1f82 53/945, 0xf2044 33/601, 0xf207a 2/19, 0xf2521 35/621, 0xf2969 19/545, 0xf2a51 35/528, 0xf2e3c 35/621, 0xf3ca4 25/510, 0xf3d83 25/510, 0xf3eac 1/103, 0xf3f71 63/769, 0xf4073 5/41, 0xf461f 3/13(degen2는 F10 이후 포함 전체 실행 수).

## 3. 비교 계약

02q 검사기 `tools/m1_factor_work_check.py`를 넓힌다(같은 W 규칙, 지점과 창 확대). 02q 검사 M·N·A·K·F와 변조·변이는 그대로 통과해야 한다.

| 검사 | 규칙 |
|---|---|
| W 저장 | 정렬된 재분해 n개와, n < 전체이면 재분해 n + 1의 원본 k 앞까지(degen2 k < 356)의 SITES(02q 17 + 이번 16) 원본 쓰기 열과 Rust 저장 열(각 저장에 Rust iter 표식, 같은 k 범위)이 지점 순서까지 같고, 각 저장에서 원본 직전 쓰기 값에 Rust 양을 순서대로 `a + v`로 더한 값이 원본 쓰기와 비트 같다. 첫 불일치는 사례·저장 번호·k·지점·원본 양 `(v − prev)/1.3`을 출력한다 |
| G 도달 | 범위 안 갱신 지점 16개가 두 모델 합쳐 모두 1회 이상 저장되고, 갱신(0xf2e3c) 수가 kb2 35·degen2 ≥ 300이다 |
| T 변조 | 02q 11종 + 갱신 지점 값 한 단위, 갱신 저장 하나 삭제, 0xf461f 지점 바꿈, 원본 0xf1f82 쓰기 한 비트. 모두 거부 |
| R 변이 | 02q 6종 + 이번 변이(아래). 생존하면 판별 불가로 기록 |

이번 변이(첫 실행 전에 고정, 기대: 모두 거부. 거부할 사례는 관측 횟수로 예상):

| 변이 | 예상 |
|---|---|
| 0xf1f82 p − s를 행 길이로 | kb2 |
| 0xf2044 조건 `<` → `<=` | degen2(0xf207a 19회 중 cU = nS′) 또는 생존 |
| 0xf2969·0xf3f71 구간 시작 바꿈(h ↔ h − umid) | kb2 |
| 0xf3ca4 +5 생략 | kb2 |
| 0xf3d83 이동 칸 수 + 1 | kb2 |
| 0xf461f k + 1(r 포함 길이 대신 다른 길이) | kb2 |
| 0xf2a51 local_80 = 0일 때도 저장 | kb2 또는 degen2 |
| 0xf1beb 결합 `3·(x148·n)` | 생존 예상(양 끝자리, 02q 판별력 한계) |

- 합격: W·G·T 통과, R 변이 거부(결합 변이는 생존해도 판별 불가로 기록), 02q 검사 전체 통과, 회귀(§5) 통과.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 원본 출력 |
|---|---|---|---|
| kb2 Method=1 Presolve=0 Threads=1 | 실제 | 모든 갱신 지점(0xf1f37·0xf3eac 각 1회), local_80 = 1·2·3, 0xf207a | `bench/oracle/m1_02p/kb2_w_1.json.gz`, 02q `kb2_fw_*` |
| degen2 같은 설정 | 실제 | 같은 지점의 많은 실행, 0xf461f 재분해 이유 10 | `bench/oracle/m1_02p/degen2_w_1.json.gz`, 02q `degen2_fw_*` |

- 02p 캡처는 3회 바이트 동일(02p §6), 02q가 비간섭을 확인했다. 새 Gurobi 실행은 없다.
- 실행 0회 예상(미검증으로 남김): §1 제외 목록.

## 5. 재실행 명령

- 비교: `tools/build_local.sh && PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_factor_work_check.py --self-test`, `--mutate`로 R.
- 회귀: `cargo test -p rsolver-lp`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, M1 검사기 `--self-test`(`m1_lsolve`·`m1_update`·`m1_usolve`는 `--rust`), worktree 빌드 pytest, Netlib Method=1 `tools/proto/pathcmp.py all` 출력이 기준선 main `51156d3` 빌드와 바이트 동일(work는 판정에 쓰지 않음).

## 6. 실행 결과

- 실행: worktree `../rsolver-m1-02r`(branch `work/m1-02r`, 기준 main `51156d3`), 기준선 빌드 `../rsolver-m1-02r-base`(detached `51156d3`). 원본 자료는 02p `bench/oracle/m1_02p/{kb2,degen2}_w_1.json.gz`(검사기가 SHA-256 고정)와 02q `fw` 캡처. Gurobi 실행 0회.
- Rust 변경:
  - `Lu::update`가 `&mut Work`를 받는다. 양을 계산하는 곳(열 r 루프, LU+0xe8 탐색, η, 행 r 제거, C:871 맞바꿈, p 이동, 순환 이동)에서 지점 묶음별로 모으고, 끝에서 원본 순서로 저장한다. 갱신 상태를 바꾸는 코드의 순서는 그대로다(pathcmp 불변). 새 대각이 ZERO 이하이면 0xf1beb만 저장하고 false(원본 C:445–455 종료와 같음. 전에는 상태를 끝까지 바꾼 뒤 false였는데 호출자가 곧 재분해하므로 결과 같음).
  - 0xf2a51은 원본 local_80을 따른다: η가 있을 때, 또는 η 없이 l = p이고 q < t이며 spike가 있을 때. Rust의 C:871 맞바꿈 분기(`!others.is_empty() && q < t`)는 l ≠ p이고 U 행 r이 빈 경우도 들어오지만 그때는 저장하지 않는다.
  - `Simplex::pivot`이 FTRAN spike 목록 길이(`sorder.len()`, 없으면 ZERO 넘는 원소 수)를 `spike_ns`에 두고, 갱신이 끝나면 0. 재분해(`refactor`)는 `036ade` 저장 뒤 `spike_ns` > 0이면 0xf461f를 저장한다.
  - `Work.k`(pivot의 `iter`)를 저장마다 덤프한다(`RSOLVER_WORK` 저장 항목 `[지점, 값, [양], k]`).
- 검사기 `tools/m1_factor_work_check.py`(02q 검사기를 넓힘): `--self-test` 종료 0.

```text
kb2: N 05b257 entries 40, A aligned refactors 5/5, K terms 42 (general 37), W stores 422 {… 0xf1beb 25, 0xf1d02 19,
     0xf1f37 1, 0xf1f82 53, 0xf2044 33, 0xf207a 2, 0xf2521 35, 0xf2969 19, 0xf2a51 35, 0xf2e3c 35, 0xf3ca4 25,
     0xf3d83 25, 0xf3eac 1, 0xf3f71 63, 0xf4073 5, 0xf461f 3 …}, F 0x158 5
degen2: alignment ends at refactor 10 (head: Rust iteration 359, original k 356)
degen2: … A aligned refactors 9/16, W stores 3184 {… 0xf1beb 177, 0xf1d02 203, 0xf1f37 79, 0xf1f82 349, 0xf2044 337,
     0xf207a 10, 0xf2521 348, 0xf2969 140, 0xf2a51 283, 0xf2e3c 348, 0xf3ca4 237, 0xf3d83 237, 0xf3eac 18,
     0xf3f71 266, 0xf4073 41, 0xf461f 7 …}, F 0x158 9, W range k < 356
ACCEPTANCE PASS (+ self-test)
```

- W: kb2 전 구간 저장 422개(갱신 35회), degen2 k < 356 저장 3,184개(갱신 348회, 재분해 9회)의 지점 순서·k·값 비트가 첫 실행부터 원본과 같다. 0xf2044·0xf207a·0xf2521 수의 차(337 + 10 = 347 < 348)는 cU = nS′ = 0의 carry 1회다. 16지점 모두 비교됐다(G).
- 이것으로 확인한 상태: 행 U 안 순서(0xf1f82, 행마다 r의 자리), U 열 구간 경계와 구간 안 순서(0xf3f71·0xf2969·0xf4073·0xf3eac), η 길이, LU+0xe8 탐색 거리, 순환 이동 단계의 열 길이와 칸 수, spike 목록 길이 nS′(`sp.len()`이 원본과 같음), 재분해 이유 10의 spike 길이. 02m·02n이 순서를 맞춘 상태를 work 양으로 다시 판별한 셈이다.
- T(`bench/oracle/m1_02q/check_mutate_02r.txt`): 02q 11종 + 갱신 양 한 단위(t_ftadd), 갱신 저장 하나 삭제(t_ftdrop), 0xf461f 지점 바꿈(t_461f), 원본 0xf1f82 쓰기 한 비트(t_wft), 저장 k + 1(t_k) 모두 거부.
- R(`--mutate`, 같은 파일, 최종 코드로 재실행): 02q 6종 + 이번 9종 중 8종 거부, 결합 변이(`3·(x148·n)`) 생존(사전 표의 예상대로 판별 불가). 거부 위치: f1f82 행 길이 kb2 저장 92, f2044 `<=` degen2 저장 544(carry 사례), 구간 시작 바꿈 kb2 77, f3ca4 +5 생략 kb2 192, f3d83 + 1 kb2 61, f461f + 1 kb2 37, f2a51 local_80 = 0 degen2 47, f1f37 `pos ≤ local_40` 조건 제거 kb2 306(검토 뒤 추가. 길이 1 행이 pos > local_40이라 저장하지 않는 분기가 실행됨).
- 회귀(worktree 빌드):
  - `cargo test -p rsolver-lp` 42 통과(1 무시), `cargo test --release --workspace` 125 통과·11 무시, `cargo clippy --workspace --all-targets -- -D warnings` 0(첫 실행 `type_complexity` 1건 → `work::Store` 별칭).
  - M1 검사기 12종 `--self-test`(`m1_lsolve`·`m1_update`·`m1_usolve`는 `--rust`) 종료 0, `m1_bfrt_rust` ACCEPTANCE PASS, pytest 657 통과.
  - Netlib: `tools/proto/pathcmp.py all` 출력이 기준선 빌드와 바이트 동일(64모델, 완전 일치 27/64, 중앙 0.752). `tools/lp_check.py --methods 1,0,-1 --timeout 120`: 정답 90/91 ×3, IterCount 31/0/31, 273행의 상태·목적값·반복 수가 기준선과 같다(시간 열만 다름. 총 153.4 → 161.3초, 부하 통제 없는 참고값).
- 계약 변경(추가만, 축소 없음): W에서 저장마다 k도 같아야 하고 변조 `t_k`(저장 k + 1)를 더했다. 검토 뒤 변이 `f1f37 without local_40 test`를 더했다(첫 실행 뒤 추가임을 밝힌다).
- 독립 검토 1회(advisor): 코드 수정 지적 없음. 반영: 0xf1f37 무저장 분기 변이 추가와 최종 코드 `--mutate` 재실행, 지점 수 표기(15 + 1), nS′ 근거 문장 정정, 02p 부록 A 정정, 인수인계 주의 추가.

## 7. 판정과 인수인계

- 작업 수행 상태: 완료.
- 동등성: **통과**(계약 범위). kb2 전 구간·degen2 피벗 0–355에서 FT 갱신 15지점과 재분해 경로 0xf461f의 저장 순서·값이 원본과 비트 같고, 02q 지점도 그대로 같다. 재분해 진입 work(0x150)는 원본 값을 입력으로 넣었다.
- 미검증: 새 대각 |S_r| < 1e-13 종료(실행 0회, Rust는 0xf1beb만 저장하도록 옮김), l ≠ p에서 η가 비고 U 행 r이 남은 오류 종료, 풀 용량 초과 종료, `9e47c` 호출(r ≠ LU+0x1d8), spike 목록의 ≤ ZERO 원소, primal 호출자(`sorder` 없음, spike 길이를 ZERO 넘는 원소 수로 둠), 재분해 이유 10 밖에서 spike가 남는 재분해, 0xf1beb 곱 결합(변이 생존), degen2 피벗 356 이후.
- 후속:
  - **M1-02s**: 반복 커널(`1bae9c`·`1b7106`·`28c93`·`8b473`·`a4109` 등)·`05b257` 뒤 처리, `1ca57` 0xbf981/0xbfed2, `16b0a` 0xbe793. 같은 검사기에 지점을 더한다.
  - **M1-02t**(02k 뒤)·**M1-02u** 그대로. 모두 끝나면 work를 재분해 판정에 연결하고 `RSOLVER_WORK_SEED` 없이 degen2 피벗 356을 대조.
- 다음 명령: `tools/build_local.sh && PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_factor_work_check.py --self-test`, 새 지점은 `SITES`(또는 묶음 집합)와 G에 더한다.
- 갱신: `docs/handoff/M1.md`, `docs/14_sprint_plan.md`, `docs/11_progress.md`, `docs/12_roadmap_diagram.md`.
- 교훈: `docs/lessons/original.md`(dual work 누적 절: FT 갱신 저장 순서와 local_80, 0xf461f), `docs/lessons/tooling.md`(코드 순서와 저장 순서가 다르면 버퍼로 모아 원본 순서로 내보내기, 덤프에 반복 표식).
