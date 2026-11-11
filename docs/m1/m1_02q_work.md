# M1-02q 검증 계약과 결과: Rust dual work 틀과 재분해 지점

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. [M1-02p](m1_02p_work.md) §8의 첫 후속이다. 사용자의 "다음 스프린트 진행해. 너가 작업자야" 지시로 착수했다(2026-11-11). M2는 다른 세션이 M2-06d-3을 점유해 M1을 골랐다([시작 절차](../handoff/README.md) §1-3).

## 1. 범위와 주장

- 작업 종류: 구현(+ Markowitz 누적 항 확인용 새 읽기 전용 캡처).
- 완료할 동작: Rust에 dual work 계수기(`06268b` param_2)를 두고 아래 지점이 원본과 같은 양을 같은 순서로 더하게 한다. 쓰기 지점 표식(원본 트랩 pc)과 값 열을 덤프한다.
  - 분해 `036ade` 5지점: 0xd4ded(`5nb + 100 + 3q`, 그 직전 work = spx+0x150), 0xd5300(`10nb + 3·nnzB + 7·act`), 0xd85e6(Markowitz 누적 acc), 0xdc7b2(`2(2nb + L + U)·f`), 0xdcb7b(`2(L + nb)`, L 레벨 분기), 그리고 spx+0x158 이동 평균. 1e8 넘는 acc 중간 반영(0xd5e39)은 같은 식으로 넣는다(미관측).
  - 재분해 뒤 `05b257` 5지점: 0xf48ae(3m·LU+0x1f8), 0xf4f9b(9m), 0xf4ffd(LU+0xdc), 0xf507a(2·max(LU+0xdc, t)), 0xf51af(m − e8 − 1), 저장을 건너뛸 때 레지스터로 넘기는 부분합 포함. d ≥ m 저장 0xf515b는 미관측.
  - `9e60b` 3지점(재분해 안 cost mode ≥ 1): 0x1327dc 또는 0x1329e6(진입 mode가 2인지)에 U 행 원소 수·LU+0x1f8, 0x1329c1 3m·LU+0x1f8.
  - 모델 상수: `534f7` 0xe84b9(2·nnz(A)), `1bae9c` 0xc391f(DSE 초기화 m, dual 시작 1회).
- 주장 범위: 원본 경로와 같은 Rust 실행 구간(kb2 재분해 5회 전부, degen2 재분해 F1–F9 = 피벗 0–355)에서 위 지점의 저장 순서·양이 원본 쓰기와 비트까지 같고, spx+0x158이 재분해마다 비트까지 같다.
- 입력으로 받는 것(Rust가 아직 계산하지 못함): **재분해 진입 work(= spx+0x150)는 원본 캡처 값을 넣는다**(`RSOLVER_WORK_SEED`). 다른 지점(반복 커널 등)이 미구현이라 Rust 자체 work 절대값은 원본과 다르다. 저장 비교는 원본의 직전 쓰기 값에 Rust 양을 원본 순서로 더해 원본 쓰기와 비트 비교한다(§3 W).
- 제외와 후속:
  - `05b257` 0xf461f(재분해 경로의 spike 지우기, spike nnz는 FT 갱신 상태)는 FT 묶음 M1-02r로 둔다(02p 표의 05b257 21 = FT 16 + 재분해 5 기준).
  - `1ca57` 0xbf981/0xbfed2(재분해마다): 양이 spx+0x338(섭동 단계)·지점이 spx+0x2cc에 달려 Rust 상태 대응이 필요하다 → M1-02s(02p가 모델 상수 묶음에 넣었던 것을 옮긴다).
  - `16b0a` 0xbe793(재분해 경로의 DSE 갱신) → M1-02s.
  - Rust work를 재분해 판정(`06268b` LAB_001f6eb5)에 연결하지 않는다. 일부 지점만 있는 work로 판정하면 경로가 바뀐다. 연결은 M1-02q–u가 모두 끝난 뒤.
  - degen2 F10–F16(피벗 356 이후)은 Rust 경로가 이미 달라 이번 비교 범위 밖이다(미검증).
- 선행: main `8569900`. 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca2…0e98f9c`, 주소는 file offset(ghidra − 0x100000). 모델 `../re/bench/data/netlib/degen2.mps.gz`, `kb2.mps.gz`, Method=1 Presolve=0 Threads=1.
- 식의 근거: [02p 부록 A](m1_02p_formulas.md) 2부 A·B·C·H와 이번에 다시 읽은 C·asm.
  - `036ade` C:296–330(nnzB), C:535–541(0x150, q = (P0 + local_100) − local_110, `1da0b2` 뒤), C:543–585(act, 0이면 1), C:813–822(acc > 1e8 반영, asm 0xd5e08–0xd5e35), C:924·C:1072(singleton 항 `(cnt + 1)·10`), C:2564(일반 피벗 항 `ns·6·bound + 10·min(Σ열 카운트, Σ행 카운트)`, asm 0xd6e92–0xd6ea4·0xd7157–0xd78eb), C:1531–1533(0xd85e6), C:5741–5754(f, 0xdc7b2), C:5913–5936(0xdcb7b), C:5753–5772(0x158: Δ = 마지막 036ade 쓰기 뒤 − 0x150, 옛 ≤ 0이면 Δ 아니면 (Δ + 옛)·0.5).
  - `05b257` C:2545–2729(재분해 반환 경로, 저장 생략 시 부분합을 넘김), `9e60b` C:246–251·C:345–347.
  - `534f7` C:1512–1514, `1bae9c` C:2205.
- 새 캡처 `tools/m1_factor_work.js`(읽기 전용): `06268b` 안 `036ade` 호출마다 basis(param_5 = nb, param_11 heads, spx+0x390 Markowitz tol)와 누적 항 사건. 036ade의 param_1·param_2가 double이라 param_3 = args[0]이다(첫 탐색 실행이 args[10]에서 접근 위반으로 확인). 항: 0xd6077 열 singleton `cnt = r15`, 0xd647b 행 singleton `cnt = *(int*)(rdx + 4·r13)`, 0xd6ea4 일반 피벗 `ns = *(int*)(rsp+0x70)`(bound는 xmm4라 Frida로 읽지 못한다, [tooling](../lessons/tooling.md)), 0xd7157 `ecx`·`r10d`(피벗 행 다른 원소의 열 카운트 합과 피벗 열 다른 원소의 행 카운트 합. 첫 판독과 도구 주석은 둘을 반대로 적었고, 첫 비교에서 K가 짝 순서만 다르게 나와 바로잡았다. 도구는 캡처 해시 고정 때문에 주석을 고치지 않았다). 카운트 배열은 원소 수 − 1 표기다. `05b257` 재분해 진입(06268b C:183·188)마다 (k, 이유, work 비트).
- 피벗 번호 k: `8b473` 호출 지점 0xc58d5(02n·02p와 같다).
- Rust 쪽: `RSOLVER_WORK=<파일>`이면 재분해마다 JSON 줄 하나(반복, head, tol, 0x150, 0x158, 누적 항, 그 사이 저장 열 (지점, 값, 더한 양들)). `RSOLVER_WORK_SEED=<파일>`이면 재분해 진입에서 work를 파일의 다음 값으로 둔다(검증용).

## 3. 비교 계약

검사기 `tools/m1_factor_work_check.py`. double은 IEEE 비트(hex)로 비교한다. 정수·순서는 정확 비교.

| 검사 | 규칙 |
|---|---|
| M 입력 | 새 캡처 degen2·kb2 각 3회 gz SHA-256 동일, `ok`, 바이너리·모델·도구 해시, errors 없음. 02p 캡처 해시 고정 |
| N 비간섭 | 새 캡처의 `05b257` 재분해 진입 열 (k, 이유, work 비트)과 출력 줄이 02p run 1의 U 사건(이유 ≠ 0)과 같다 |
| A 정렬 | Rust 재분해 i의 head(위치 순)·반복 번호·Markowitz tol이 캡처 F_i와 같다. 첫 불일치에서 비교 범위를 끊고 그 지점을 출력한다. 범위는 kb2 5/5, degen2 9 이상이어야 한다 |
| K 누적 항 | 재분해마다 항 열(종류, cnt / ns, ecx, r10d)이 캡처 T 사건과 순서까지 같다. 0xd85c7(반영 지점) 1회 |
| W 저장 | 정렬 범위의 구현 지점 쓰기 열(지점 순서)이 캡처 W 열(02p run 1, 구현 지점만, 같은 재분해 창)과 같고, 각 쓰기에서 원본 직전 쓰기 값(없으면 E)에 Rust의 더한 양들을 순서대로 `a + v`로 더한 값이 원본 쓰기 값과 비트 같다 |
| F 필드 | Rust 0x150 = 넣은 진입 work(입력, 정보), Rust 0x158 = 02p V 사건(05b257 반환)의 0x158 비트 |
| T 변조 | (`--self-test`) 바이너리 해시 0, 빈 사건, F 사건 0개(대상 0회), N work 한 비트, head 한 칸, 항 cnt + 1, Rust 더한 양 한 비트, 0x158 한 비트, 저장 지점 바꿈. 모두 거부 |
| R 변이 | 임시 사본에 Rust 식 변이(0xd4ded 100 생략, acc 일반 항 min → max, 0x158 평균 0.5 생략, 9e60b 지점 선택 반대, 0xf507a 조건 반대)를 넣어 빌드하면 검사가 실패한다(최종 코드에 남기지 않음) |

- bound는 캡처하지 못한다. 0xd85e6 쓰기가 비트 같으면 acc 합 = Σ 항이 같으므로 bound는 합으로만 확인된다고 기록한다.
- 0x150은 입력이므로 일치를 주장하지 않는다. Δ와 0x158 이동 평균이 원본과 같은지가 대상이다.
- 합격: M·N·A·K·W·F·T 통과, R 변이 모두 거부(생존하면 판별 불가로 기록). 회귀(§5) 통과.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 | 원본 출력 |
|---|---|---|---|---|
| kb2 Method=1 Presolve=0 Threads=1 | 실제 | m < 250(L 레벨 없음), 일반 피벗, cost mode 0 | `kb2.mps.gz` | `bench/oracle/m1_02q/kb2_fw_{1,2,3}.json.gz`, W는 `bench/oracle/m1_02p/kb2_w_1.json.gz` |
| degen2 같은 설정 | 실제 | m ≥ 250(0xdcb7b), 9e60b mode 1·2(F9부터 2), 0xf4ffd·0xf507a 저장, singleton 두 종류 | `degen2.mps.gz` | `bench/oracle/m1_02q/degen2_fw_{1,2,3}.json.gz`, `m1_02p/degen2_w_1.json.gz` |

- 실행 0회(미검증으로 남김): acc > 1e8 중간 반영, f ≠ 1(24(L + U) > 2e6), 0xf515b(d ≥ m이고 t < m − 1), work 포인터 null, 특이 basis 재시도, 조밀 전환(dgetrf), quad(spx+0x148 = 1.95), env+0x2454 = 0, cost mode 끄기(iterations > 2m).
- 탐색 실행(기준 아님): 도구 첫 판 kb2 1회(args 위치 오류), 고친 판 kb2·degen2 각 1회. 기준은 최종 도구로 새 파일에 3회씩 다시 캡처한다.

## 5. 재실행 명령

- 원본 캡처: `for m in kb2 degen2; do for i in 1 2 3; do tools/gurun.sh node tools/m1_factor_work.js ../re/bench/data/netlib/$m.mps.gz bench/oracle/m1_02q/${m}_fw_$i.json.gz; done; done`(Herdr `tools/hrun.sh`).
- 비교: `tools/build_local.sh && PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_factor_work_check.py`, `--self-test`로 T, `--mutate`로 R.
- 회귀: `cargo test -p rsolver-lp`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, M1 검사기(`m1_update_check --rust` 등) `--self-test`, worktree 빌드 pytest, Netlib Method=1 경로·결과가 기준선 main `8569900`과 같음(work는 판정에 쓰지 않으므로 변화 없어야 한다).

## 6. 실행 결과

- 실행: worktree `../rsolver-m1-02q`(branch `work/m1-02q`, 기준 main `8569900`), 기준선 빌드 `../rsolver-m1-02q-base`(detached `8569900`). 원본 `bench/oracle/m1_02q/{kb2,degen2}_fw_{1,2,3}.json.gz`(gz SHA-256 kb2 `61dc5447…`, degen2 `b0dd087a…`, 각 3회 바이트 동일). Gurobi: 기준 캡처 6회 + 탐색 3회(kb2 2, degen2 1).
- `tools/build_local.sh && PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_factor_work_check.py --self-test` 종료 0.

```text
kb2: N 05b257 entries 40, A aligned refactors 5/5, K terms 42 (general 37), W stores 43 {0xe84b9 1, 0xd4ded 5, 0xd5300 5, 0xd85e6 5,
     0xdc7b2 5, 0xf48ae 5, 0xf4f9b 5, 0xf51af 5, 0xc391f 1, 0xf4ffd 3, 0xf507a 3}, F 0x158 5
degen2: alignment ends at refactor 10 (head: Rust iteration 359, original k 356)
degen2: N 05b257 entries 637, A aligned refactors 9/16, K terms 157 (general 113), W stores 104 {0xe84b9 1, 0xd4ded 9, 0xd5300 9,
     0xd85e6 9, 0xdc7b2 9, 0xdcb7b 9, 0xf48ae 9, 0x1327dc 8, 0x1329c1 9, 0xf4f9b 9, 0xf51af 9, 0xc391f 1, 0xf4ffd 6, 0xf507a 6, 0x1329e6 1}, F 0x158 9
(both) refactor 1 basis = original as a set, position order differs (Rust start.rs renumbers before it)
T: 11 tampers rejected (binary hash, empty events, no F events, 05b257 entry work, head entry, refactor 1 head member,
   term cnt + 1, Rust amount + 1 unit, 0x158 bit, store site swapped, original write bit)
```

- `--mutate`(`bench/oracle/m1_02q/check_mutate.txt`): 변이 6종 모두 거부. 0xd4ded 100 생략(W kb2 저장 1), 일반 항 min → max(W kb2 0xd85e6), 0x158 0.5 생략(F kb2 재분해 2), 9e60b 지점 반대(W degen2 저장 7), 0xf507a 조건 `d ≤ m`(W kb2 저장 7), q에서 U 생략(W kb2 0xd4ded).
- N: 새 도구의 `05b257` 진입 40·637회 (k, 이유, work 비트)와 출력 줄이 02p 캡처와 같다. 명령 probe가 경로·work를 바꾸지 않았다.
- A: kb2 5/5, degen2 9/16(F10은 원본 k 356, Rust 반복 359: 02n의 work 조건 재분해, 이번 범위 밖). F2부터 head는 위치 순서까지, Markowitz tol(0.0078125)·반복 번호가 같다. **F1(dual 시작 첫 분해)은 같은 basis 집합이지만 위치 순서가 다르다**: 원본은 crash basis를 행 순서(바꾼 slack의 행 위치, `[22, 23, 24, 25, 45, 27, …]`)로 F1에서 분해하고 그 뒤 위치를 다시 매긴다. Rust `start.rs:159–165`는 그 재번호(구조 열 피벗 순 + slack 행 순)를 미리 적용해 두고 `dual_start`에서 다시 분해하므로 F1 입력이 원본 F1의 출력 순서(`[38, 37, 29, 27, …]` = 원본 F2 입력)다(코드 확인). F2부터 head·W·K가 같고 F1 W(5지점)도 같다(Markowitz 항 0개). F1에 커널이 있는 모델에서는 피벗 순서가 달라질 수 있다(§7 후속).
- K: 일반 피벗 항 150개(ns, `ecx`, `r10d`)와 singleton 항 49개가 순서까지 같다. 첫 비교에서 일반 항이 모두 (`ecx`, `r10d`) 짝만 뒤바뀌어 나와(min이라 W는 이미 통과) 의미를 바로잡았다: `ecx` = 피벗 행 다른 원소의 열 카운트 합, `r10d` = 피벗 열 다른 원소의 행 카운트 합. 1e8 중간 반영 0회.
- W: kb2 43·degen2 104 저장이 지점 순서와 값 비트까지 같다(원본 직전 쓰기 + Rust 양). F1–F9 0xd85e6이 같으므로 bound(캡처 불가)를 쓴 Σ ns·6·bound도 합으로 같다.
- F: spx+0x158 kb2 5·degen2 9회 비트 일치. 0x150은 넣은 값(입력)이다.
- 판별력 한계: 더한 양의 마지막 비트 하나 차이는 work 합(10⁴–10⁶)의 반올림에 묻혀 관측되지 않는다(첫 변조 `t_add`가 통과). 실제 식 오류는 정수 단위 × 1.3로 달라지므로 변조를 "양 + 1단위"로 바꿨다.
- 검토 반영(advisor): m = 0이면 `m − e8 − 1`이 usize 아래로 넘어가 `saturating_sub`로 고침(원본은 0을 더함), 검사기에 Rust가 읽는 모델 파일 해시 대조와 F1 집합 변조(`t_head1`) 추가, F1 원인을 `start.rs` 코드로 확인해 서술 정정.
- 회귀(worktree 빌드):
  - `cargo test -p rsolver-lp` 42 통과(1 무시), `cargo test --release --workspace` 125 통과·11 무시, `cargo clippy --workspace --all-targets -- -D warnings` 0(첫 실행은 `assign_op_pattern` 1건 → `w += a`로 고침. IEEE 덧셈 교환이라 비트 같음, 검사기 재통과).
  - M1 검사기 11종 `--self-test`(`m1_lsolve`·`m1_update`·`m1_usolve`는 `--rust`) 종료 0, `m1_bfrt_rust` ACCEPTANCE PASS.
  - worktree 빌드 pytest 657 통과.
  - Netlib: `tools/proto/pathcmp.py all` 출력이 기준선 빌드와 바이트 동일(64모델, 완전 일치 27/64, 중앙 0.752). `tools/lp_check.py --methods 1,0,-1 --timeout 120`: 정답 90/91 ×3, IterCount 31/0/31. 기준선 빌드로 같은 명령을 돌려 91모델 × 3방식 273행의 상태·목적값(출력 repr)·반복 수·정답 판정이 모두 같다(총 시간 161.7 → 162.8초, 비 1.006, 부하 통제 없는 참고값).
- 계약 변경(착수 뒤, 이유): A의 F1을 집합 비교로 둠(위 F1 위치 순서. 범위 축소가 아니라 기존 start.rs 차이의 보고), 변조 `t_add`를 1비트에서 1단위로(위 판별력 한계), K의 `ecx`/`r10d` 의미 정정(도구 주석은 캡처 해시 고정 때문에 그대로 두고 여기서 정정).

## 7. 판정과 인수인계

- 작업 수행 상태: 완료.
- 동등성: **통과**(계약 범위). kb2 재분해 5회·degen2 재분해 F1–F9에서 이번에 옮긴 17지점 중 관측된 15지점의 저장 순서·값과 spx+0x158이 원본과 비트 같고, 036ade Markowitz 항이 순서까지 같다. 재분해 진입 work(0x150)는 원본 값을 입력으로 넣었다.
- 미검증: 0xd5e39(acc > 1e8)·0xf515b(d ≥ m, t < m − 1) 실행 0회, LU+0xdc = 0의 0xf4f9b carry(14/14 저장), m ≥ 250인데 L 레벨 없음(0xdcb7b 생략), m = 0, f ≠ 1, 9e60b mode 끄기, 특이 basis 재시도, 조밀 전환, quad, env+0x2454 = 0, 여유 열 있는 dual 시작의 0xe84b9 순서, degen2 F10–F16. bound는 합으로만. Rust work 절대값(반복 커널·LU 풀이·`1ca57`·`16b0a`·0xf461f 등 미구현)과 재분해 판정 연결은 범위 밖.
- 후속:
  - **M1-02r**(구현): FT 갱신 `05b257` 16지점(재분해 경로의 spike 지우기 0xf461f 포함, 02p 분류).
  - **M1-02s**: 02p 목록에 더해 `1ca57` 0xbf981/0xbfed2(spx+0x2cc 지점 선택, spx+0x338 섭동 단계 = Rust `pt_lvl` 대응 확인)와 재분해 경로 `16b0a` 0xbe793.
  - dual 시작 F1 입력 순서: Rust `start.rs`가 첫 분해의 재번호를 미리 적용한다. F1을 원본처럼 행 순서로 분해하게 바꾸는 일은 M1-02c 또는 새 번호(현재 두 모델에서 F1 결과·work는 같다).
  - M1-02t·u 그대로. 모두 끝나면 Rust work를 재분해 판정에 연결하고 `RSOLVER_WORK_SEED` 없이 degen2 피벗 356을 02n 검사기 `--rust`로 대조.
- 주의: `RSOLVER_WORK_SEED`는 재분해 진입에서 w를 덮어쓴다. work를 판정에 연결하기 전에 검증용으로만 남기거나 없앤다. main worktree 공유 `.venv` 확장은 이 변경 전 빌드다.
- 다음 명령: `tools/build_local.sh && PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_factor_work_check.py --self-test`(Rust work를 바꾸면), 새 지점은 `SITES`와 W 비교에 더한다.
- branch `work/m1-02q`, worktree `../rsolver-m1-02q`. 병합은 main의 `Merge M1-02q` 이력.
- 갱신: `docs/handoff/M1.md`, `docs/14_sprint_plan.md`(02q 완료, 02r·02s 범위 추가), `docs/11_progress.md`, `docs/12_roadmap_diagram.md`.
- 교훈: `docs/lessons/original.md`(dual work 누적 절: 036ade 항 의미·필드, 0x158에 0xdcb7b 포함, start basis 위치 순서), `docs/lessons/tooling.md`(검사기: 합으로만 보이는 양의 변조 크기, 캡처 도구 주석 수정은 해시 고정을 깬다).
