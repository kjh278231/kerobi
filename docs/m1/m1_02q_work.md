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

(작성 예정)

## 7. 판정과 인수인계

(작성 예정)
