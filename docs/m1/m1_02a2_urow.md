# M1-02a-2 검증 계약과 결과: U 행 저장 순서의 생성 규칙

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. [M1-02a-1](m1_02a1_tau.md) §7의 후속 조사이고, 사용자의 "다음 스프린트 진행" 지시로 착수했다(2026-11-11). M2는 M2-03d-14를 다른 세션이 먼저 점유해 진행 중이라 [시작 절차](../handoff/README.md) §1-3 순서대로 M1을 골랐다.

## 1. 범위와 주장

- 작업 종류: 조사. Rust 솔버 코드는 바꾸지 않는다. 임시 계측은 패치 파일로 남기고 되돌린다.
- 질문: 원본 LU의 행 방향 U(시작 LU+0xb0, 끝 LU+0xb8, 열 번호 LU+0xd0, 값 LU+0xc8)에서 한 행 안의 원소 순서는 어떻게 생기는가. M1-02a-1은 τ 모으기가 이 저장 순서로 더한다는 것을 보였고, 그 순서를 캡처에서 그대로 가져다 썼다. M1-02d(구현)가 Rust에서 같은 순서를 만들려면 아래 두 규칙이 필요하다.
  1. FT 갱신(`05b257`): 나가는 열의 원소를 각 행에서 지울 때, 행 r을 U에서 뺄 때(row eta), 새 열 원소를 넣을 때 순서가 어떻게 바뀌는가.
  2. 인수분해(`036ade`, 그 안의 `1da0b2` singleton 단계) 직후 각 행의 초기 순서.
- 완료 기준:
  - 갱신: C 근거로 규칙을 적고, 02a-1 캡처의 갱신 34회 전부에서 직전 캡처 U 행에 그 규칙을 적용한 결과가 다음 캡처 U 행과 원소 순서·값 비트까지 같다. 변이(안정 삭제, 앞에 넣기, row eta 때 행 r을 비우지 않기)는 거부되어야 한다.
  - 초기 순서: C의 쓰기 지점에서 끌어낸 규칙이, 같은 입력 상태(같은 기저·같은 입력 열 순서)의 Rust 인수분해 정보로 피벗 15·26 재분해 직후의 캡처 U 행 전부를 순서까지 재현한다. 오름차순이 아닌 행이 적어도 하나 재현돼야 한다. 피벗 0·6 재분해는 U 행이 없어 판별 자료가 아니다.
  - 초기 순서를 재현하지 못하면 "답함"으로 적지 않는다. 행 종류(singleton·kernel, 접두·꼬리)별로 맞는 후보와 틀리는 행을 보고하고 후속 번호를 둔다.
- 제외 범위와 후속:
  - Rust 수정은 M1-02d다.
  - 조밀 전환(dgetrf, 036ade L1542/L1762)과 2×2 블록 단계는 kb2에서 실행되지 않으면 C만이다.
  - kb2 밖 모델은 이번 판정에 넣지 않는다(재캡처 필요).
- 선행: main `7924343`(M1-02a-1 병합). 미커밋 선행 변경 없음. 원본 자료는 02a-1 캡처 `bench/oracle/m1_02a1_tau/kb2_{1,2,3}.json.gz`(gz SHA-256 `64cc9dea89f809b601f7cb2e11636cc632f382318ceeaec0273a599687bdf393`)와 저장 오라클 `bench/oracle/iter0-30/kb2.jsonl.gz`다. 이번에는 Gurobi를 실행하지 않는다(새 캡처가 필요해지면 계약을 바꾸고 기록한다).

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 모델 `../re/bench/data/netlib/kb2.mps.gz`(Method=1, Presolve=0, Threads=1).
- 원본 위치(C 읽기, 착수 시점):

| 위치 | 내용 |
|---|---|
| `05b257` `0xf15c0` C:243–264 | 갱신 경로의 배열: LU+0xd0 열 번호, +0xc8 값, +0xb0 시작, +0xb8 끝, +0xc0 용량, +0xf8 접두 행 표시, +0xf0 접두 행 목록, +0xec 그 개수 |
| `05b257` C:527–567 | 나가는 열(LU+0x88/0x90/0xa8의 열 원소 행마다): 행 U에서 그 열 위치를 찾고, 끝을 1 줄인 뒤 마지막 원소를 그 자리로 옮긴다. 1원소 행은 접두 행 목록에서 뺀다 |
| `05b257` C:1240–1304 | 새 열 원소(spike 목록 순서)마다 행 U 끝에 붙인다(`idx[b8] = 열`, `b8++`). 용량이 차면 행을 memcpy로 옮긴다(순서 유지) |
| `05b257` C:2570–2700 | 재분해 직후: LU+0xb8[i] = LU+0xb0[i+1](행이 꽉 찬 연속 배치), LU+0xf8 = −1(앞 LU+0xdc개), 0,1,…(LU+0x100까지), −2(꼬리) |
| `036ade` `0xd4190` | 행 방향 U가 주 출력이다. L5420–5470은 행 방향 U(LU+0xb0/0xd0/0xc8)를 전치해 열 방향 U(LU+0x88/0xa8/0xa0)를 만든다. 행 U 쓰기 지점 L1086, L1250, L2127, L2391(희소), L3765·L3800·L5195(조밀). L5373에서 열 번호를 `local_c8`로 바꾼다. 인자 param_12 = LU+0x20 |
| `1da0b2` | singleton 단계. param_12·param_15를 받는다(미독) |

- 시작·종료 경계: 02a-1과 같다(각 피벗의 `09d112` 진입에서 읽은 LU 상태). 피벗 k와 k+1 사이에 `05b257` 갱신 한 번(LU+0x18c가 1 증가) 또는 재분해(0이 됨)가 있다.
- 입력: 피벗 k의 행 U(위치별 열 번호·값), 나가는 위치 r(=02a-1 사건 `r`), LU+0x180(row eta 개수). 출력: 피벗 k+1의 행 U.
- 재분해 입력: 재분해 직전 기저(위치 → 변수). 저장 오라클의 `vstat`는 재분해 뒤 위치를 단계 순서로 다시 매긴 값이다(스냅숏 14→15에서 둘 다 기저인 변수 42개 중 36개의 위치가 바뀜). 그래서 직전 기저는 스냅숏 k−1의 `vstat`에 피벗 k−1의 교환(위치 r에 들어오는 q)을 적용해 만든다. Rust 인수분해 결과의 단계 순서가 스냅숏 k의 위치와 같아야 같은 입력으로 인정한다.

## 3. 비교 계약

새 `tools/m1_urow_check.py`가 판정한다. 하나라도 실패하면 종료코드 1이다.

| 검사 | 규칙 |
|---|---|
| M 입력 | 02a-1 캡처 3개의 gz SHA-256이 위 값이고 사건 목록이 서로 같다. 저장 오라클·Rust 덤프·그 생성 패치의 SHA-256 고정 |
| G 갱신 | 갱신 34회 각각: 피벗 k 행 U에 (1) 열 r을 가진 행마다 그 원소 자리에 마지막 원소를 옮기고 끝을 줄임, (2) LU+0x180이 1 늘면 행 r을 비움, (3) 피벗 k+1에서 열 r을 가진 행에 그 원소를 끝에 붙임을 적용한 결과 = 피벗 k+1 행 U(모든 위치, 열 번호 순서와 값 비트) |
| V 변이 | 안정 삭제, 앞에 붙이기, row eta 때 행 r 유지를 각각 G에 넣으면 실패해야 한다 |
| L 같은 입력 | 피벗 15·26: 직전 기저의 Rust 인수분해 단계 순서로 매긴 위치 = 스냅숏 15·26 `vstat` 위치, U 원소 집합·값 비트·대각 = 캡처 |
| F 초기 순서 | 피벗 15·26: §6에서 C로 정한 초기 순서 규칙을 Rust 인수분해 정보에 적용한 행 U 순서 = 캡처 행 U 순서(모든 위치) |
| T 목표 | G에서 마지막 원소 옮기기가 안정 삭제와 다른 갱신 ≥ 1, row eta 갱신 ≥ 1. F에서 오름차순이 아닌 캡처 행 ≥ 1을 재현. 하나라도 0이면 실패 |

- G의 한계: 새 열이 어느 행에 들어가는지와 row eta 여부는 다음 캡처(열 r 소속, LU+0x180)에서 읽는다. G가 판정하는 것은 원소의 자리(순서)뿐이다. 값은 옮긴 원소의 값 비트가 따라가는지와 새 원소 값이 캡처와 같은지만 본다.
- 정수·순서는 정확 비교, 값은 u64 비트. 허용오차 없음.
- 자기검사(`--self-test`): 캡처 해시, 피벗 하나의 행 두 원소 교환(G), 행 r 비우기 표시 변조(G), Rust 덤프 값 한 비트(L), Rust 위치 하나 교환(L), 캡처 재분해 행 두 원소 교환(F)을 모두 거부해야 한다.
- 초기 순서를 재현하지 못하면 F는 실패로 남기고 조사 결과를 §7에 적는다(판정 규칙을 넓히지 않는다).

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 | 원본 증거 |
|---|---|---|---|---|
| kb2 Method=1, Presolve=0, Threads=1 | 원본 실제 실행(02a-1 캡처, 피벗 0–37) | 갱신 34회(row eta 있음·없음), 재분해 4회(U 행 있음 2회) | `kb2.mps.gz` | `bench/oracle/m1_02a1_tau/kb2_{1,2,3}.json.gz`, `bench/oracle/iter0-30/kb2.jsonl.gz` |

- 적용하지 않은 범주: 다른 모델(재캡처 필요), 조밀 전환(실행에 없으면 C만).

## 5. 재실행 명령

- 판정: `python3 tools/m1_urow_check.py`, 자기검사 `--self-test`.
- Rust 덤프 재생성(기준 자료, 필요할 때만): 깨끗한 작업 트리에서 `git apply bench/oracle/m1_02a2_urow/rust_factor_dump.patch && tools/build_local.sh && PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_urow_check.py --write-dump && gzip -n bench/oracle/m1_02a2_urow/rust_factor_kb2.jsonl`, 그 뒤 `git checkout crates/ && tools/build_local.sh`. 덤프는 `Lu::factor_tol`이 kernel 뒤에 단계(행·위치·대각)·`ppos`·`kernel_u`(목록 순서)·`ucol`을 한 줄로 적는다. 호출은 `_core.row_costs(직전 기저 vstat)`다.
- 회귀: Rust 미변경이면 `python3 tools/m1_tau_check.py`는 다시 돌리지 않아도 된다(같은 기준 자료). 돌릴 수 있으면 같은 빌드에서 돌린다.

## 6. 실행 결과

### 6.1 탐색(계약 확정 전후, 판정 근거 아님)

- 계약 전: 스크래치 모의로 갱신 34회를 "마지막 원소로 채우는 삭제, row eta 때 행 r 비우기, 끝에 붙이기"로 재현했다(안정 삭제는 6회 실패). 재분해 직후 행은 단계 번호로 오름차순이 아닌 것이 있었다(피벗 15 행 32: [41, 42, 39]).
- 저장 오라클 `vstat`는 재분해 뒤 위치가 단계 순서로 다시 매겨진다(스냅숏 14→15에서 42개 중 36개 이동, 25→26에서 40개). 그래서 직전 기저(스냅숏 k−1 + 피벗 k−1 교환)로 바꿔 읽었다. 그 기저의 위치(재분해 전 위치)로 캡처 행 U를 다시 쓰면, 접두 행(LU+0xf8 ≥ 0) 23개는 모두 오름차순, 꼬리 행 8개는 모두 내림차순이었다.
- C 읽기:
  - `1da0b2` C:530–560: singleton 단계 뒤 남은 열을 위치 오름차순으로 돌며, 열 원소의 행이 이미 피벗됐으면(param_18[행] = 단계) 그 단계의 U 행 끝(`local_70[단계]++`)에 붙인다. 아니면 활성 행 사본에 넣는다. `local_70`은 이 뒤 C:568–597에서 시작 위치로 되돌린다. 인자 param_21 = LU+0x20, param_22 = 036ade param_15(`*param_22` = LU+0xd0, `[1]` = LU+0xc8)다.
  - `036ade` C:600–690: kernel 활성 행렬을 위치 오름차순으로 만들며 원소 노드(+0 값, +0xc 열, +0x10 다음)를 행 목록 머리에 넣는다(`node+0x10 = head[행]; head[행] = node`). C:2258–2265: fill 원소도 새 노드를 같은 방식으로 머리에 넣는다(C:1936–1948·C:2190–2206의 머리 저장은 죽은 노드를 빼며 순서를 유지하는 재연결). C:2366–2393·C:2116–2128: 피벗 행의 U 행을 목록 머리부터 걸으며 쓴다. 그래서 fill이 없으면 위치 내림차순이다. L5373이 열 번호를 단계로 바꾼다(`local_c8`). L5420–5470은 행 U를 전치해 열 U를 만든다.
  - `05b257` C:970–1037: row eta이면 LU+0x180을 1 올리고, 행 r 원소마다 열 U에서 r을 빼고, `LU+0xb8[r] = LU+0xb0[r]`로 행 r을 비운다. `09e47c`는 LU+0xb0/0xb8/0xd0/0xc8/0xf8을 읽기만 한다(그 기저로 쓰는 문 없음). `05b257`에서 행 배열 기저(`local_68`·`local_70`·`local_138`·`local_130`)로 직접 쓰는 문은 C:565–568, C:1037, C:1304–1305뿐이고, 별칭(C:1044–1045의 `dVar49`·`lVar12`)으로 C:1277–1306의 행 이동(memcpy, 순서 유지)과 끝 갱신을 쓴다. C:1531·C:1677은 `local_138` 변수를 다른 분기에서 재사용하며 이번에 따라가지 않았다.
  - Rust `Lu::kernel`은 목록을 뒤집어 저장하고(`push` = 앞에 넣기), 위치 오름차순으로 만들고, `take_alive`가 머리부터의 순서를 돌려준다. 그래서 `kernel_u[단계]`가 원본 목록 순서의 후보다. 현재 `finish()`는 이 순서를 버리고 단계 오름차순으로 `urow`를 다시 만든다.

### 6.2 판정

- 실행: worktree `../rsolver-m1-02a-2`, 기준 main `7924343`(뒤에 main `7f86824` 병합). 계약 `4aa5a7c`, 검사기·덤프 `60f90b1`. x86-64 Linux(WSL2). Gurobi는 실행하지 않았다.
- 기준 자료: 02a-1 캡처 3개(gz SHA-256 `64cc9dea…`, 세 회 같음), 저장 오라클 `007d5a55…`, Rust 인수분해 덤프 `bench/oracle/m1_02a2_urow/rust_factor_kb2.jsonl.gz` `7294c10c…`, 생성 패치 `rust_factor_dump.patch` `fa509686…`. 패치를 적용한 빌드에서 §5 명령으로 덤프를 다시 만들면 gz SHA-256이 같다(실행 확인). 그 뒤 `crates/`는 되돌렸다(`git diff crates/` 비어 있음).
- `python3 tools/m1_urow_check.py --self-test` 종료 0(출력 그대로):

```text
M: 3 captures (pinned gz sha256, equal events), stored oracle, Rust factor dump and its patch pinned
G: 05b257 rule = next captured rows on all 34 updates (row etas 24, stable removal would differ on 6)
V: stable: rejected (G pivot 13->14 (r 18, eta 1): rows [9, 10, 17, 19] differ, first rule [6, 15] vs capture [)
V: prepend: rejected (G pivot 1->2 (r 14, eta 0): rows [29, 36, 37, 38] differ, first rule [14, 12] vs capture [)
V: keep_r: rejected (G pivot 9->10 (r 24, eta 1): rows [24] differ, first rule [16, 30] vs capture [])
L: pivots (15, 26): Rust factor of the pre-factor basis renumbers to the snapshot positions; U sets, value bits and diagonal = capture; singleton steps = prefix t
F: initial order rule = captured rows at pivots (15, 26) (multi-entry rows: singleton 23, kernel 8; not ascending in step 18)
F variant rust_now: rejected (F pivot 15: steps [32, 33, 40] differ, rule [39, 41, 42] vs capture [41, 42, 39])
F variant all_singleton: rejected (F pivot 15: steps [40] differ, rule [41, 42] vs capture [42, 41])
F variant kernel_desc: NOT DISCRIMINATED
T: targets met; kernel rows whose list order differs from descending position (fill): 0
rejected: capture sha256: M: bench/oracle/m1_02a1_tau/kb2_2.json.gz sha256 0000000000000000000000000000000000000000000000000000000000000
rejected: update: two entries of a pivot 12 row exchanged: G pivot 11->12 (r 6, eta 1): rows [9] differ, first rule [18, 6] vs capture [6, 18]
rejected: update: row eta flag dropped: G pivot 9->10 (r 24, eta 0): rows [24] differ, first rule [16, 30] vs capture []
rejected: update: value bit of a carried entry at pivot 14: G pivot 13->14 (r 18, eta 1): rows [6] differ, first rule [15, 18] vs capture [15, 18]
rejected: dump: one U value bit: L pivot 15: U sets/values != capture
rejected: dump: two step positions exchanged: L pivot 26: Rust step order != snapshot 26 positions
rejected: refactor: two entries of a pivot 26 row exchanged (G 26->27 sees it first): G pivot 26->27 (r 41, eta 1): rows [10] differ, first rule [32, 33, 34, 41] vs capture [33, 32, 34, 41]
rejected: dump: kernel list order of a pivot 15 row reversed (F only): F pivot 15: steps [40] differ, rule [41, 42] vs capture [42, 41]
PASS
```

- 계약 변경(결과 확인 뒤, 넓힌 것 없음): L에 "Rust singleton 단계 수 = 접두 t"를 더했다. F에 변이 3개를 더했다(현재 Rust 순서·모두 singleton 규칙은 거부되어야 함, kernel 위치 내림차순은 kb2에 fill 행이 없어 판별 안 됨으로 기록). 자기검사에 F만 보는 변조(덤프 kernel 목록 순서 뒤집기)와 옮겨진 원소 값 한 비트 변조(G)를 더해 8종이 됐다. 계약의 "캡처 재분해 행 두 원소 교환" 변조는 G(피벗 26→27)가 먼저 거부한다.
- 회귀: 같은 worktree 깨끗한 빌드(`tools/build_local.sh`)에서 `PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_tau_check.py --self-test` PASS. Rust를 바꾸지 않아 `cargo test`·clippy·pytest는 실행하지 않았다.
- 실행하지 못한 검사: 다른 모델(재캡처 필요), fill이 있는 kernel 행, singleton 열에 원소가 있는 singleton 행.

## 7. 판정과 인수인계

- 작업 수행 상태: **완료(조사)**. 두 질문 모두 C 근거와 캡처 재현으로 답했다.
- 답:
  - **FT 갱신**(`05b257`): 나가는 열 r의 원소는 각 행에서 그 자리에 행의 마지막 원소를 옮기고 끝을 줄여 지운다(C:556–567). row eta(LU+0x180 +1)이면 행 r을 비운다(C:970–1037). 새 열 원소는 각 행 끝에 붙인다(C:1302–1304, 용량이 차면 순서 그대로 옮김). `09e47c`는 순서를 바꾸지 않는다. 갱신 34회 전부 재현.
  - **인수분해 직후**(`036ade`·`1da0b2`): singleton 단계 행의 kernel 열 원소는 열의 재분해 전 기저 위치 오름차순이다(1da0b2 C:530–560은 그 시점에 아직 피벗되지 않은 열만 쓴다). kb2는 두 재분해 모두 t = singleton 수라 singleton 행에 singleton 열 원소가 없다. kernel 단계 행은 활성 행 목록의 머리부터 순서다. 목록은 위치 오름차순으로 만들며 앞에 넣고 fill도 앞에 넣으므로(036ade C:600–690, C:2258–2265) fill이 없으면 위치 내림차순이고, Rust `kernel_u[단계]` 순서와 같다. 그 뒤 열 번호를 단계로 바꾼다. 피벗 15·26의 행 U 전부(여러 원소 행 31개, 단계 오름차순이 아닌 행 18개) 재현.
  - Rust와의 차이: `Lu::finish()`는 모든 행을 단계 오름차순으로 만들고, `Lu::update`는 `retain`(안정 삭제)으로 지운다. 둘 다 원본과 다르다(F `rust_now`·V `stable` 거부).
- 동등성: 해당 없음(조사). Rust 구현은 불일치로 남는다.
- 남은 한계:
  - kb2 한 모델, 재분해 2회(U 행 있음)·갱신 34회. 원본 캡처는 02a-1 것을 그대로 썼다.
  - kernel 행의 fill 원소 순서(앞에 넣기)는 C(C:2258–2265)와 Rust 포팅 근거뿐이다. kb2의 kernel 행 8개는 fill이 없어 "위치 내림차순"과 판별되지 않는다.
  - singleton 행이 singleton 열에도 원소를 갖는 경우(t ≠ singleton 수)는 실행에 없고 규칙도 정하지 않았다. M1-02d는 이 경우를 미확인으로 두고 구현한다. kb2는 두 재분해 모두 t = singleton 수이고 singleton 행 원소가 모두 kernel 열이다. 다른 쓰기 지점(1da0b2 C:662, 2×2 블록 C:798·C:1120–1158)은 C만이다.
  - 조밀 전환(036ade L3765·L3800·L5195)과 2×2 블록 단계는 실행에 없다(C만).
  - G는 새 열의 소속 행과 row eta 여부를 다음 캡처에서 읽는다(§3).
  - `05b257`의 다른 경로(C:1531·C:1677의 `local_138` 재사용 분기 등)는 따라가지 않았다. 갱신 규칙은 위 쓰기 지점과 34회 재현이 근거다.
- 후속:
  - **M1-02d**(구현): (1) `finish()`가 singleton 단계 행을 재분해 전 위치 오름차순, kernel 단계 행을 `kernel_u` 순서로 두게 한다. (2) `update`의 열 삭제를 마지막 원소로 채우는 삭제로 바꾼다. (3) τ U 단계를 원본 규칙(꼬리 행 흩뿌림 + 접두 행 저장 순서 모으기)으로 계산한다. 판정은 `tools/m1_tau_check.py` T 차이 0, 이 검사기의 규칙을 Rust 덤프 대신 Rust `urow`에 직접 적용한 대조, `statecmp kb2` 반복 12 이후, Method=1 Netlib 회귀다. 첫 명령은 `grep -n "urow" crates/lp/src/lu.rs`다.
  - **M1-02c**(검증)는 그대로다.
  - (선택) 다른 Netlib 모델에서 `tools/m1_tau_path.js`를 다시 캡처하면 fill이 있는 kernel 행과 singleton 열 원소를 판별할 수 있다.
- 작업 branch·commit: `work/m1-02a-2`, worktree `../rsolver-m1-02a-2`. 미커밋 없음(이 기록 커밋 기준). 병합 상태는 [M1 인수인계](../handoff/M1.md)와 main 이력으로 확인한다.
- 갱신 위치: [M1 인수인계](../handoff/M1.md), [스프린트 목록](../14_sprint_plan.md) M1-02a-2 행, [작업 단위](../13_work_units.md) M1-02, [진행 기록](../11_progress.md)·[로드맵](../12_roadmap_diagram.md) 맨 위.
- 교훈 기록: [원본 사실](../lessons/original.md) "U 행 저장 순서", [도구](../lessons/tooling.md) "재분해 뒤 vstat는 다시 매긴 위치".
