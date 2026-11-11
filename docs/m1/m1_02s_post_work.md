# M1-02s 검증 계약과 결과: `05b257` 뒤 처리 work 지점(`1bae9c`·`1b7106`·`1ca57`·`16b0a`)

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. [M1-02r](m1_02r_ft_work.md) §7의 첫 후속이다. 사용자의 "다음 스프린트 진행해. 너가 작업자야" 지시로 착수했다(2026-11-11). M2는 다른 세션이 M2-06g를 점유해 M1을 골랐다([시작 절차](../handoff/README.md) §1-3).

## 1. 범위와 주장

- 작업 종류: 구현(새 원본 캡처 없음. M1-02p watchpoint 캡처 재생).
- 범위 나눔(착수 전, 결과 전): 스프린트 표의 M1-02s는 반복 커널(`a4109`·`1c35c5`·`28c93`·`1c8477`·`2896a3`·`15d2d`, 19지점)과 `05b257` 뒤 처리를 함께 묶었다. 반복 커널은 검증 열이 "단계별 후보 수·분할 길이 캡처 대조"라 새 캡처가 필요한 다른 종류의 일이어서 **M1-02v**로 옮긴다. 이번 M1-02s는 뒤 처리 13지점만 한다. 착수 전에 02p 캡처의 지점별 양 `(v − prev)/1.3`을 풀어 아래 입력 해석이 맞는지 미리 보았다(§2 끝, 탐침).
- 완료할 동작: Rust dual이 아래 13지점의 work 저장을 원본과 같은 순서·양으로 더한다.

| 묶음 | 지점(트랩 pc) | 양(·x148) | 원본 | Rust 위치 |
|---|---|---|---|---|
| 피벗마다(갱신 경로) | 0xc0410 | 3·N, N = α_q 목록 길이(피벗 뺀), β ≠ 0일 때만 | `1bae9c` C:420–462 | dual 루프, `pivot` 갱신 뒤 |
| 〃 | 0xc06c7(P+0x40 = 0) · 0xc0c24(P+0x40 ≠ 0, spx+0x2c8 = 0) · 0xc09bd(P+0x40 ≠ 0, spx+0x2c8 ≠ 0, 8·N) | 7·N | C:544–777 | 〃, `price_touch` 앞 |
| 목록 재구성(mode 0, P+0x28 < 0, P+0x38 ≥ 0) | 0xc26ec | 3·c0 + ([P+0x44 = 1] + 5)·(m − c0), c0 = 가능 위치 수. 옛 목록 길이 P+0x3c > 0일 때만 저장, 아니면 carry | C:1000–1048 | `price_touch` → 재구성 |
| 〃 | 0xc2761 | 2k + s, k = 옛 목록에서 남긴 수, s = 훑은 수(= 옛 길이). m > 0일 때 저장 | 〃 | 〃 |
| 〃 | 0xc2842 | 2q, q = 새로 붙는 위반 위치 수 | 〃 | 〃 |
| 〃 | 0xc2ad4 | 합친 목록 길이 | C:1060–1260 | 〃 |
| 재분해 뒤 | 0xc142b | 4m + S + S, S = 0이 아닌 bound에 있는 비기저 구조 열의 길이 합(vstat −1·−4는 l, −2·−5는 u) | C:1432–1592 | `reinvert_with`·`dual_start` |
| 〃 | 0xc4329(`1b7106`, P+0x40 = 0) · 0xc1fe3(P+0x40 ≠ 0) | 5m + 4·nInf. mode 0은 새 목록 길이, mode 1은 위반 위치 수 | `1b7106` C:146, `1bae9c` C:2262–2470 | `price_full` |
| 〃 | 0xbf981(spx+0x2cc = 0) · 0xbfed2(spx+0x2cc ≠ 0, spx+0x490 = 0) | (m+n)·(2 + [spx+0x338 ≠ 0]) | `1ca57` C:139–141·312–316 | `reinvert_with`·`dual_start` 끝 |
| 재분해 경로(FTRAN 뒤 갱신 대신) | 0xbe793 | (3·x148)·N, β ≠ 0일 때만 | `16b0a` C:46–48, `05b257` C:2466–2479 | `refactor`, 0xf461f 뒤 |

- 주장 범위: kb2(02r과 같은 범위: 재분해 5회의 `05b257` 반환까지), degen2 피벗 0–355에서 위 지점과 02q·02r 지점의 저장 열(지점 순서)·각 저장 값이 원본과 비트까지 같다.
- 입력으로 받는 것: 02q·02r처럼 재분해 진입 work(spx+0x150)는 원본 캡처 값(`RSOLVER_WORK_SEED`). 반복 커널·LU 풀이 지점이 없어 Rust work 절대값은 원본과 다르다. 저장 비교는 원본 직전 쓰기 + Rust 양(W 규칙).
- 대응 가정(검사로 판별): P+0x40 = Rust `pr.mode`, P+0x44 = `pr.cost_mode`, spx+0x2c8 = `c8`, spx+0x2cc = `big_m`, spx+0x338 = `pt_lvl`, P+0x3c·P+0x10 = `pr.list`. 원본 저장 순서상 `48d5f`(섭동·flip, 0xe7000) 뒤에 0xc142b가 오므로 S는 섭동·flip 뒤 vstat으로 센다.
- 제외와 후속:
  - 실행 0회 예상(미검증): 0xc09bd(mode 1에서 spx+0x2c8 ≠ 0), 0xc0410·0xbe793의 β = 0 분기(저장 없음), `1ca57`의 두 번째 `1bae9c` 호출(P+0x3c = 0, `118725`), 0xbfed2의 spx+0x490 ≠ 0, 0xc26ec·0xc2761의 carry 분기(옛 목록 빔·m = 0), `17248`(DSE 모드 2).
  - **M1-02v**(새 번호): 반복 커널 `a4109`·`1c35c5`·`28c93`·`1c8477`·`2896a3`·`15d2d`(19지점). `15d2d`는 CHUZR 쪽 목록 재구성이라 이번 재구성과 같은 Rust `rebuild`를 쓰지만 지점·식이 다르다.
  - M1-02t·M1-02u 그대로. work를 재분해 판정에 연결하지 않는다.
- 선행: main `f7ac92b`(M1-02r 병합). 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca2…0e98f9c`, 주소는 file offset. 모델 `../re/bench/data/netlib/kb2.mps.gz`·`degen2.mps.gz`, Method=1 Presolve=0 Threads=1.
- 근거(이번에 다시 읽음):
  - `1bae9c` C:130–170: spx+0x2cc·0x2c8(local_b0)·P+0x44(local_108) 읽기. C:184 갱신 경로 조건 LU+0x18c ≠ 0 ∧ spx+0x290 = 0 ∧ P+0x40 ≠ −1.
  - C:420–462(0xc0410): spx+0x3ec ≠ 0이고 β(spx+0x310) ≠ 0이면 w_r과 α_q 목록(spx+0x210) 원소마다 w 갱신, 루프 수·3. β = 0이면 w_r = 1e-8, 저장 없음.
  - C:486–777: P+0x40 ≠ 0이면 mode 1 x_B 갱신 루프(local_b0 = 0이면 0xc0c24 7.0, 아니면 0xc09bd 8.0, asm 상수 2a5d0f0 = 7.0·2a5d108 = 8.0), P+0x40 = 0이면 mode 0 루프 뒤 0xc06c7 7.0. 루프 수 = α_q 목록 길이. 그 뒤 P+0x28 < 0이면 P+0x38 감소, 음수면 이유 3, 아니면 재구성.
  - C:799–1260(재구성): m 위치를 돌며 가능 위치에 표시 2(c0 셈), 위반 위치는 표시 ≠ 0이면 1. 0xc26ec 양을 P+0x3c > 0일 때 저장(아니면 레지스터). 옛 목록(P+0x10, 길이 P+0x3c)에서 표시 0인 첫 원소부터 점수가 직전 남긴 것보다 크면 표시 1, 아니면 남김(k). 끝까지 훑어 s = P+0x3c. 0xc2761 `((k+k)+s)`. 표시 1 위치를 새 목록으로 모아(q) 0xc2842 `w + (q+q)·x148`, P+0x3c = k + q, `aabbe3` 정렬, 병합 뒤 0xc2ad4 길이.
  - C:1432–1592(0xc142b): 구조 열 j < n마다 vstat −1·−4면 −l, −2·−5면 −u를 보고 0이 아니면 열 길이(spx+0x78)를 S에 더한다. `9cf9b` FTRAN 뒤 `((m·4 + S) + S)·x148 + w`.
  - C:2262–2470: P+0x40 = 0이면 `1b7106`(0xc4329, param_4 = m, 새 목록 길이 = P+0x3c), 아니면 위반 위치 수 uVar36(P+0x3c에 저장한 뒤 문턱으로 압축) → 0xc1fe3 `(m·5 + uVar36·4)·x148 + w`.
  - `1ca57` C:42–139: `1bae9c`가 local_40을 세우고 spx+0x294 = 0일 때 목적값 루프 뒤 spx+0x2cc = 0이면 0xbf981 `((m+n)·([0x338 ≠ 0] + 2))·x148 + w`, 아니면 0xbfed2(같은 양).
  - `16b0a` C:1–58: β(param_2) ≠ 0이면 w 갱신, `x148·3.0·N + w`(asm `(3.0·x148)·N`). `05b257` C:2466–2479: spx+0x290 = 0 ∧ spx+0x2fc ≥ 0 ∧ 이탈 위치 ≥ 0 ∧ spx+0x3ec ∉ {0, 2}일 때 재분해 경로에서 호출.
- 캡처 순서(02p `kb2_w_1`·`degen2_w_1`): 갱신 경로 `05b257`(V0) → 0xe7000 → 0xc0410 → 0xc06c7|0xc0c24 → [0xc26ec → 0xc2761 → 0xc2842 → 0xc2ad4]. 재분해 경로 … 0xf461f → 0xbe793 → 0xf48ae … V → 0x12de50 → 0xe7000 → 0x1219d6 → 0xc142b → [0xc391f(시작만)] → 0xc4329|0xc1fe3 → 0xbf981|0xbfed2.
- 탐침(착수 전, 02p 캡처의 양): kb2 0xc0410/3 = 0xc06c7·0xc0c24/7(35피벗 모두), (m+n) = 84에 0xbf981 168·252(k = 15부터 252: 그 재분해 안에서 섭동이 켜짐), 0xc142b S = 1·1·2·4(·6), 0xc4329 nInf 1, 0xc1fe3 nInf 14·6·10. degen2 0xbfed2 4회 뒤 k = 131(U4, 인공 bound 끝)부터 0xbf981, 0xc26ec c0 137 등(P+0x44 = 1로 맞음).

## 3. 비교 계약

02q·02r 검사기 `tools/m1_factor_work_check.py`를 넓힌다(같은 W 규칙·범위). 02q·02r 검사 M·N·A·K·W·F·G와 변조·변이는 그대로 통과해야 한다.

| 검사 | 규칙 |
|---|---|
| W 저장 | SITES(02q 17 + 02r 16 + 이번 13)의 원본 쓰기 열과 Rust 저장 열이 지점 순서·k까지 같고, 각 저장에서 원본 직전 쓰기 값에 Rust 양을 순서대로 `a + v`로 더한 값이 원본 쓰기와 비트 같다(02r과 같음) |
| G2 도달 | 범위 안 이번 지점 중 실행 예상 12개(0xc09bd 제외)가 두 모델 합쳐 1회 이상 저장되고, 모델마다 0xc0410 수 = 0xf2e3c(갱신) 수 |
| T 변조 | 02r 16종 + 이번: 0xc0410 양 한 단위, 0xc142b 저장 삭제, 0xc06c7 ↔ 0xc0c24 지점 바꿈, 원본 0xc1fe3 쓰기 한 비트, 0xbe793 저장 삭제. 모두 거부 |
| R 변이 | 02r 15종 + 이번(아래). 생존하면 판별 불가로 기록 |

이번 변이(첫 실행 전에 고정. 거부할 사례는 탐침으로 예상):

| 변이 | 예상 |
|---|---|
| 0xc0410 N에 피벗 포함(N + 1) | kb2 |
| 0xc06c7·0xc0c24 mode 판정 뒤집기 | kb2 |
| 0xc142b S에 bound 0인 열도 셈 | kb2 또는 degen2 |
| 0xc142b를 섭동·flip 앞 vstat으로 | 생존 가능(섭동이 vstat을 옮긴 재분해가 범위에 있어야 판별) |
| 0xc1fe3 nInf = 문턱 뒤 목록 길이 | kb2 또는 생존 |
| 0xbf981 플래그 0 고정 | kb2(k = 15) |
| 0xc2761 `k + s`(2k 대신 k) | degen2 |
| 0xc26ec 옛 목록이 비어도 저장 | 생존 가능(옛 목록 빈 재구성이 있어야 판별) |
| 0xbe793 결합 `(N·3)·x148` | 생존 예상(양 끝자리, 02q 판별력 한계) |

- 합격: W·G·G2·T 통과, R 변이 거부(생존 예상으로 적은 것은 생존해도 판별 불가로 기록), 02q·02r 검사 전체 통과, 회귀(§5) 통과.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 원본 출력 |
|---|---|---|---|
| kb2 Method=1 Presolve=0 Threads=1 | 실제 | mode 0 → 1 전환(k = 6), 0xc4329·0xc1fe3, 섭동 켜짐(k = 15) 뒤 0xbf981 ×3, 0xbe793 3회 | `bench/oracle/m1_02p/kb2_w_1.json.gz`, 02q `kb2_fw_*` |
| degen2 같은 설정 | 실제 | 인공 bound 단계(0xbfed2, S 큼), mode 0 재구성 51회, mode 1 전환(k = 278), 0xbe793 7회 | `bench/oracle/m1_02p/degen2_w_1.json.gz`, 02q `degen2_fw_*` |

- 범위 안 기대 횟수(kb2/degen2): 0xc0410 35/348, 0xc06c7 5/272, 0xc0c24 30/76, 0xc142b 4/9, 0xc4329 1/7, 0xc1fe3 3/2, 0xbf981 4/5, 0xbfed2 0/4, 0xc26ec·0xc2761·0xc2842·0xc2ad4 0/51, 0xbe793 3/7. kb2는 다섯째 재분해 V까지라 그 뒤의 0xc142b·0xc1fe3·0xbf981(k = 38)은 범위 밖이다.
- 02p 캡처는 3회 바이트 동일(02p §6), 02q가 비간섭을 확인했다. 새 Gurobi 실행은 없다.

## 5. 재실행 명령

- 비교: `tools/build_local.sh && PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_factor_work_check.py --self-test`, `--mutate`로 R.
- 회귀: `cargo test -p rsolver-lp`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, M1 검사기 `--self-test`(`m1_lsolve`·`m1_update`·`m1_usolve`는 `--rust`), worktree 빌드 pytest, Netlib Method=1 `tools/proto/pathcmp.py all` 출력이 기준선 main `f7ac92b` 빌드와 바이트 동일(work는 판정에 쓰지 않음), `tools/lp_check.py --methods 1,0,-1 --timeout 120` 기준선과 같은 행.

## 6. 실행 결과

- 실행: worktree `../rsolver-m1-02s`(branch `work/m1-02s`, 기준 main `f7ac92b`), 기준선 빌드 `../rsolver-m1-02s-base`(detached `f7ac92b`). 원본 자료는 02p `bench/oracle/m1_02p/{kb2,degen2}_w_1.json.gz`(검사기가 SHA-256 고정)와 02q `fw` 캡처. Gurobi 실행 0회.
- Rust 변경(`crates/lp/src/simplex.rs`, `pricing.rs`):
  - dual 루프: `pivot` 앞에 `dse_n = (β ≠ 0).then(α_q 목록 길이)`. 갱신 경로면 0xc0410(β ≠ 0일 때)과 x_B 지점(`pr.mode` = 0 → 0xc06c7, mode 1이고 `c8` 거짓 → 0xc0c24, 참 → 0xc09bd 8·N)을 `price_touch` 앞에 저장한다. 재분해 경로면 `refactor`가 0xf461f 뒤에 `dse_n`을 받아 0xbe793 `(X148·3)·N`.
  - `price_touch`의 재구성: `Pricing::rebuild`가 `Rebuilt { feasible, scanned, kept, added }`를 돌려주고 0xc26ec(옛 목록이 비면 carry)·0xc2761·0xc2842·0xc2ad4. **c0는 `c8`이 거짓이면 m, 참이면 가능 위치 수**(C:799–855의 카운터 위치. 첫 실행에서 발견, 아래).
  - 재분해 뒤(`reinvert_with`의 dual 쪽 끝, `dual_start`): 섭동·flip 뒤 `work_xb`(0xc142b) → `price_full`(mode 0이면 0xc4329 = 5m + 4·목록 길이, mode 1이면 0xc1fe3 = 5m + 4·위반 위치 수) → `work_obj`(0xbf981, `big_m`이면 0xbfed2, 플래그 `pt_lvl ≠ 0`). `dual_start`는 0xc142b를 0xc391f 앞에 둔다. `dual_resume`에도 같은 세 지점을 두었다(범위 밖, 미검증).
- 검사기 `tools/m1_factor_work_check.py`: SITES에 `POST` 14지점(0xc09bd 포함), G2(0xc0410 수 = 갱신 수, `POST_RUN` 13지점 도달), 변조 5·변이 9 추가.
- 첫 실행: kb2 통과, degen2 저장 1194(k = 138, 인공 bound가 끝난 뒤 첫 재구성)에서 0xc26ec가 Rust 1977·원본 1332(= 3·444)로 실패. 원본 C:799–855(spx+0x2c8 = 0 루프)는 `iVar60++`가 if/else 밖이라 c0 = m이다. 가능 위치만 세는 것은 spx+0x2c8 ≠ 0 루프뿐이다(그 앞 51회 중 k < 131은 인공 bound 단계라 맞았다). 고친 뒤 `--self-test` 종료 0:

```text
kb2:    W stores 507  {0xc0410 35, 0xc06c7 5, 0xc0c24 30, 0xc142b 4, 0xc4329 1, 0xc1fe3 3, 0xbf981 4, 0xbe793 3, …}
degen2: W stores 4118 {0xc0410 348, 0xc06c7 272, 0xc0c24 76, 0xc142b 9, 0xc4329 7, 0xc1fe3 2, 0xbf981 5, 0xbfed2 4,
                      0xc26ec 51, 0xc2761 51, 0xc2842 51, 0xc2ad4 51, 0xbe793 7, …}, W range k < 356
ACCEPTANCE PASS (+ self-test)
```

- W: 지점별 횟수가 §4의 기대 횟수와 모두 같고, 02q·02r 지점까지 포함한 kb2 507·degen2 4,118 저장의 지점 순서·k·값 비트가 원본과 같다. 0xc09bd 0회(예상).
- T(`bench/oracle/m1_02q/check_mutate_02s.txt`): 02r 16종 + 이번 5종(t_c0410·t_c142b·t_mode·t_wc1fe3·t_be793) 모두 거부.
- R(같은 파일, 최종 코드로 재실행): 02r 15종 + 이번 9종 중 이번 6종 거부(c0410 N + 1 kb2 저장 16, mode 뒤집기 kb2 17, S에 bound 0 열 kb2 8, nInf = 목록 길이 kb2 155, 플래그 0 kb2 156, `k + s` degen2 22). 생존 3종은 사전 표의 예상대로 판별 불가로 기록한다:
  - 0xc142b를 섭동·flip 앞 vstat으로: 범위 안에 섭동·flip이 S를 바꾼 재분해가 없다. 첫 `--mutate`의 이 변이는 저장을 옮기지 않고 하나 더 끼워 지점 중복으로만 거부됐다. 옮기는 변이로 고쳐 다시 돌리니 생존했다(첫 실행 뒤 변이 수정임을 밝힌다).
  - 0xc26ec를 옛 목록이 비어도 저장: 범위 안 재구성 51회 모두 옛 목록이 비지 않았다.
  - 0xbe793 결합 `(N·3)·x148`: 양 끝자리가 합의 반올림에 묻힌다(02q·02r의 결합 변이와 같음).
- 회귀(worktree 빌드):
  - `cargo test -p rsolver-lp` 42 통과(1 무시), `cargo test --release --workspace` 125 통과·11 무시, `cargo clippy --workspace --all-targets -- -D warnings` 0.
  - M1 검사기 12종 `--self-test`(`m1_lsolve`·`m1_update`·`m1_usolve`는 `--rust`) 종료 0, `m1_bfrt_rust` ACCEPTANCE PASS, pytest 657 통과.
  - Netlib: `tools/proto/pathcmp.py all` 출력이 기준선 빌드와 바이트 동일(완전 일치 27/64, 중앙 0.752). `tools/lp_check.py --methods 1,0,-1 --timeout 120`: 정답 90/91 ×3, IterCount 31/0/31, 273행의 상태·목적값·반복 수가 기준선과 같다(시간 열 제외).
- 계약 변경: 변이 "0xc142b 섭동 앞"의 구현을 고쳤다(위). 범위·비교 규칙은 바꾸지 않았다.

## 7. 판정과 인수인계

- 작업 수행 상태: 완료.
- 동등성: **통과**(계약 범위). kb2(재분해 5회의 `05b257` 반환까지)·degen2 피벗 0–355에서 `05b257` 뒤 처리 13지점과 02q·02r 지점의 저장 순서·값이 원본과 비트 같다. 재분해 진입 work(0x150)는 원본 값을 입력으로 넣었다.
- 미검증: 0xc09bd(mode 1·spx+0x2c8 ≠ 0, 실행 0회), β = 0 분기(0xc0410·0xbe793 저장 없음), 0xc26ec carry 분기(옛 목록 빔, 변이 생존), 0xc142b의 섭동·flip 뒤 시점(변이 생존), `1ca57` 두 번째 `1bae9c` 호출·spx+0x490 ≠ 0, `17248`(DSE 모드 2), `dual_resume` 경로, kb2 다섯째 재분해 뒤(k = 38) 저장, degen2 피벗 356 이후. spx+0x338 = `pt_lvl ≠ 0`, spx+0x2c8 = `c8`은 kb2·degen2 범위에서만 판별했다.
- 후속:
  - **M1-02v**: 반복 커널 `a4109`·`1c35c5`·`28c93`·`1c8477`·`2896a3`·`15d2d`(19지점). `15d2d`는 이번 `Rebuilt` 수를 CHUZR 쪽 재구성에 쓰되 식·지점이 다르다.
  - **M1-02t**(02k 뒤)·**M1-02u** 그대로. 모두 끝나면 work를 재분해 판정에 연결하고 `RSOLVER_WORK_SEED` 없이 degen2 피벗 356을 대조.
- 다음 명령: `tools/build_local.sh && PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_factor_work_check.py --self-test`, 새 지점은 `SITES`와 도달 집합에 더한다.
- 갱신: `docs/handoff/M1.md`, `docs/14_sprint_plan.md`, `docs/11_progress.md`, `docs/12_roadmap_diagram.md`.
- 교훈: `docs/lessons/original.md`(dual work 누적 절: 뒤 처리 세 함정), `docs/lessons/tooling.md`(캡처 양 탐침).
