# M3-04c-21g push 재분해 `036ade` factor 생성(주입점 제거) 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 원본 기준 자료는 [M3-04c-21e 지연 LU 연쇄 오라클](primal_push_lu_chain_oracle.md), 연결 대상은 [M3-04c-21f 직렬 backend](primal_push_lu_rust.md)다.

## 1. 범위와 주장

- 작업 종류: **구현**. 기준 main `fcdcb9d`(21f 병합), branch `work/m3-04c-21g`, worktree `../rsolver-m3-04c-21g`. 선행 미커밋 없음. main의 미추적 `.claude/settings.local.json`은 무관하며 보존한다. 사용자의 "다음 M3 스프린트 진행해 너가 작업자야" 지시로 이 세션이 직접 작업한다(작업자 선택 질문 생략).
- 완료할 동작: push 안 재분해(`05b257` 요청≠0)에서 `036ade` 분해와 그 뒤 factor 설정(`05b257` C:2570–2760)이 만드는 factor를 **원본 배열 표현**(21f `LuState`: 순열 F+0x20…0x48, 대각, L 열 F+0x58/0x60/0x68과 레벨, L 행 F+0x70/0x78/0x80과 레벨·bucket, U 열·행의 풀 위치, 스칼라, 접두 목록 F+0xf0/0xf8, F+0x1a0)으로 Rust가 생성한다. 21f 연쇄의 재분해 주입을 이 생성으로 바꿔, push의 첫 factor 사건 뒤에는 원본 factor를 한 번도 넣지 않고 207연산을 재생한다.
- 방식(계약에서 선택): M1 `Lu::factor_tol`(`1da0b2` singleton 단계 + `036ade` Markowitz 커널, 이미 원본 피벗 순서를 재현하도록 이식됨)을 그대로 부르고, 결과를 원본 배열로 옮기는 adapter `crossover_refactor`를 새로 만든다. `036ade` 직접 이식(6,037줄)은 하지 않는다. M1 `lu.rs`는 **동작을 바꾸지 않고** 내부 결과를 읽는 접근자만 더한다(공유 M1 파일 변경, 수치 경로 불변을 workspace 회귀로 확인).
- 사전 탐색(계약 전, 결과 판정에 쓰지 않음): 19개 재분해 입력(push 진입 CSC·재분해 진입 header·Markowitz S+0x390 = 0.1)으로 M1 분해를 돌려 원본 `lu:refactor` 반환의 피벗 행(F+0x20)·기저 위치(F+0x28)·대각 비트와 비교했다. 18회 일치, **capri 23회째 재분해만 커널 첫 단계(140)에서 다르다**. 원본 140·141단계는 행 {217,218}×기저 위치 {152,206}의 2×2 블록 피벗이다(`1da0b2` C:686–1510: 남은 활성 ≥ 50이고 S+0x4e4 ≥ 0이면 개수 2 행을 해시로 짝지어 블록 피벗, S+0x4e0/0x4e4를 갱신). M1은 이 단계가 없다(`docs/m1/m1_notes.md` §6.1 항목 7).
- 포함: 재분해 19회(arms4·retry3·interior4·adlittle3·scsd1 2·capri3) 중 2×2 블록 단계가 피벗하지 않는 경우. 커널(Markowitz) 있는 재분해 9회(interior 5·9, adlittle 1·2·14, scsd1 1·2, capri 1·2)와 L 레벨·Lᵀ 레벨(capri) 포함.
- **예상 실패(사전 명시)**: capri 23회째 재분해(2×2 블록). Rust는 이 재분해에서 원본과 다른 factor를 만들고, 그 뒤 capri 연산은 실패로 센다. 범위를 줄이는 것이 아니라 원인과 함께 실패로 기록하며 통과 수에 넣지 않는다. 2×2 블록 단계 이식(S+0x4e0/0x4e4 상태, M1 simplex 경로에도 영향)은 후속 **M3-04c-21h**(M1 조율)로 등록한다.
- 제외 범위: 조밀 전환(MKL `dgetrf`, 채움 > 0.36·n²)·1e99 피벗 상한·특이 기저(반환≠0)·풀 부족 재할당(자연 0회, M1 shortcut 그대로이며 발생하면 Rust 결과가 달라져 실패로 드러남), 첫 분해(push 밖 F1), work(`036ade` work·F190·F+0x200은 기록만, 21f와 같음), pricing 경로·루프22·품질15a·batch23·x87·P3.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 21e manifest의 캡처·도구 해시를 그대로 검사한다.
- C: `../re/decomp/c/000d4190_PRIVATE0000000000036ade.c`(분해·저장 배치·L/Lᵀ 레벨·F+0x104/0x108/0x110, C:5355–5930), `000df800_PRIVATE00000000001da0b2.c`(singleton 단계, 2×2 블록 C:686–1510), `000f15c0_PRIVATE000000000005b257.c` C:1336–1475(재분해 인자: `036ade(ZERO=S+0x3a8, Markowitz=S+0x390, S, …, m=S+0x64, n=S+0x68, CSC=S+0x70/78/80/88, header=F+0x18, …)`, F+0x160=0·F+0x168=F+0x158−1 직전 설정)과 C:2540–2770(분해 뒤 F40/F48 별칭·identity·F1a0·U 끝·접두·F+0xe8). M1 대응 근거는 `docs/m1/m1_02a2_urow.md`·`m1_02d_tau.md`·`m1_02h_aq_order.md`·`m1_02j_lsolve.md`·`m1_02m_usolve.md`와 `docs/lessons/original.md`의 U 행 순서·접두 목록·L 레벨 항목.
- 시작 경계: 재분해 `05b257` 진입(요청≠0). 입력: push 진입 시 캡처한 CSC(S+0x70…0x88, `push:enter.model`), 재분해 진입 header(`lu:refactor` enter), ZERO(F+0x10), Markowitz(S+0x390, `factor:enter.factor_ctl.state.markowitz`), 그리고 `036ade`가 쓰지 않는 할당·호출자 상태(풀 크기 F+0x158, 정렬 pool_mod16, 갱신 한도 F+0x188, work 배율 F+0x1f8 등 §6에서 C로 확정한 목록)는 **직전 Rust 상태**에서 이어받는다. push의 첫 재분해에서는 이 할당 상태만 그 `lu:enter` factor에서 읽는다(분해 전 상태라 원본 출력이 아님).
- 종료 경계: 재분해 `05b257` 반환. 재번호(21f `renumber`)는 21f와 같은 push 상태 입력(`036ade` 반환 순열 o/n 대신 **Rust가 만든 피벗 위치**)으로 한다. 즉 o/n도 이번에는 Rust 생성값이며, 원본 o/n과 비교한다.

## 3. 비교 계약

| 대상 | 규칙 |
|---|---|
| `lu:refactor` leave | Rust 피벗 행 순서(F+0x20)·F+0x30·o(F+0x28)·n(F+0x38) = 원본 |
| 재분해 `lu:leave` factor | 21f의 factor 스냅숏 전체(perm·diag·L·Lrow(levels·lev·lst·bucket)·U 열/행과 풀 위치·scalars(xd8…x1e0, pool_mod16)·prefix·eta_oldest·etas·spike·scratch), header·이탈 위치·vstat·spike·cache·F1d8·버퍼 교환(21f와 같음) |
| 이후 연산 | 21f 계약 §3 그대로(FTRAN·BTRAN·FT 갱신 스냅숏·출력) |

- 정수·목록·순서 정확, f64는 ±0 포함 비트. 미지값·adapter 오류는 실패. 원본 factor는 비교기만 읽고 Rust 입력에 넣지 않는다(§2의 첫 재분해 할당 상태 제외). 비교 제외는 21f와 같은 F190·work·x200.
- 원본 재현성은 21e 정식 21회(7사례 각 독립3회 동일)를 재사용한다. 새 원본 실행은 0회가 목표다.
- 판별력: (a) trace 변조(재분해 factor의 L 순서·U 행 순서·풀 위치·스칼라·접두·레벨), (b) Rust 소스 변이(임시 Cargo 사본): L 저장 순서 뒤집기, U 열 순서, 풀 채움 식(x160), 열 기준, F+0x104/0x108/0x110 식, 접두 목록 순서, L/Lᵀ 레벨 계산, F+0xe8 탐색, F1a0 초기화, Markowitz 문턱(0.1 → 기본 2⁻⁷) 변이. **반드시 거부**: 주입 경로로 되돌리는 변이는 의미가 없으므로 대신 "직전 분해 factor 유지"(재분해 생략) 변이를 넣는다. 생존은 따로 센다.
- 21f 주입 재생(`--compare`)은 회귀로 유지한다(주입 모드와 생성 모드를 같은 하네스에서 선택).

## 4. 사례와 원본 자료

| 사례 | 재분해(커널 단계 수) | 비고 |
|---|---|---|
| arms | 4 (0) | 삼각 |
| retry | 3 (0) | 삼각 |
| interior | 4 (5·9회째 커널 3·4) | FT 연쇄 |
| adlittle | 3 (21·22·31) | FT 11 연쇄 |
| scsd1 | 2 (7·11) | |
| capri | 3 (64·64·131) | L/Lᵀ 레벨, 23회째 2×2 블록 = **예상 실패** |

- 자료: `bench/oracle/m3_push_lu_chain/*-1.json(.gz)`(21e 정식), Netlib 모델은 캡처 안 CSC를 쓴다(원본 모델 파일 불필요).

## 5. 재실행 명령

- 기준 재검사: `python3 tools/push_lu_check.py --controls --acceptance`, 21f `python3 tools/push_lu_rust.py --compare`.
- 생성 재생: `python3 tools/push_lu_rust.py --write --generate` → `cargo test -p rsolver-lp --test push_lu` → `python3 tools/push_lu_rust.py --compare --generate`.
- 오류 검출: `--self-test --generate`(trace 변조), `--compare --generate --mutate-rust <새 결과경로>`(소스 변이).
- 회귀: `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, 21b/21d `--compare`, py_compile, `git diff --check`. M1 `lu.rs`에 접근자만 더하므로 solver 동작 불변을 workspace 테스트로 확인하고, 확장·pytest·Netlib·P3는 solver 경로 미변경으로 제외한다(바뀌면 실행).

## 6. 실행 결과

- 환경: WSL2 x86-64 Linux 6.6.87.2, Python 3.10, Rust stable. 계약 `a88c0f1`. 원본 실행 **0회**(21e 정식 `*-1` 재사용, `push_lu_check --controls --acceptance`로 3회 동일·해시 재확인). 증거 `bench/oracle/m3_push_refactor_rust/`(입력 `input.txt` 213연산, `comparison.txt`, `self-test.txt`, `rust-mutations.json/.txt`, manifest, `regression/`).
- 구현: `crates/lp/src/crossover_refactor.rs`(M1 `Lu::factor_tol` → 원본 배열 adapter), `lu.rs`에 읽기 전용 `parts()`/`LuParts`와 저장 필드 `s1`(LU+0xd8)·`s3`(LU+0x100) 추가(M1 계산·분기 불변, workspace 회귀 통과), 하네스 `push_lu.rs`의 `M`/`G` 연산과 `refactor_chain_trace`, `tools/push_lu_rust.py --generate`.
- **연쇄 재생**(`--generate --compare` 종료0): 213연산(모델 6·재분해 19·FTRAN 120·BTRAN 32·FT 36) 중 **211 통과**, 비교 필드 1,234,852. 재분해 생성 **18/19 비트 일치**(삼각 9·커널 9, L/Lᵀ 레벨 2): 순열 F+0x20/0x28/0x30/0x38(`036ade` 반환)·대각·L 열/행·레벨·lst·U 열/행과 풀 위치·스칼라 전부(xd8·xdc·xe4·xe8·x100·x104·x108·x110·x128·x140·x160·x168·x1e0 등)·접두·F1a0·재번호. 그 뒤 FT·FTRAN·BTRAN이 생성 상태 위에서 21f와 같은 분기 수로 원본과 일치한다. push 첫 factor 사건 뒤 원본 factor 주입은 0회다.
- **예상 실패 2연산**(계약 §1 사전 명시): capri#813 재분해의 `G.perm.x20[140]`(원본 218, Rust 149: 원본 140·141단계는 2×2 블록 {217,218}×{152,206})과 그 뒤 capri#821 FTRAN(`L.beg[142]` 290≠291). 비교기는 예상 실패가 정확히 그 재분해에서 시작하는지까지 판정한다.

| 사례 | 재분해(생성 일치) | 그 뒤 연산 | 비고 |
|---|---|---|---|
| arms | 4/4 | FTRAN 28·BTRAN 17 | 삼각 |
| retry | 3/3 | FTRAN 10·BTRAN 4 | 삼각 |
| interior | 4/4 | FTRAN 12·BTRAN 1·FT 5 | 커널 2회(5·9회째) |
| adlittle | 3/3 | FTRAN 24·BTRAN 2·FT 11 | 커널 3회, t=s3≠M1 prefix(아래) |
| scsd1 | 2/2 | FTRAN 13·BTRAN 1 | 커널 2회 |
| capri | 2/3 | FTRAN 32/33·BTRAN 7·FT 20 | 커널·L/Lᵀ 레벨, 23회째 2×2 블록 실패 |

- **trace 변조**(`--generate --self-test` 종료0): 42종 모두 판정 실패로 거부(연산 누락·머리 해시·오류 주입, 재분해의 `036ade` 순열 x20/o·대각·L 열 안 순서·L 값·L 시작·L/Lᵀ 레벨·L lst 마지막 칸·Lᵀ 행 안 순서·Lᵀ lst·U 열 행 순서·U 열 경계·U 행 순서·U 행 시작·스칼라 18종·접두 목록/자리·F1a0·header·이탈 위치·vstat·x40 별칭, 이후 FTRAN 출력·FT 대각·BTRAN cache). 첫 시도에서 "L 열 순서 뒤집기"가 통과했다(interior 5회째 L 행 [7,6,7]이 회문). 열 안 서로 다른 두 원소를 맞바꾸는 변조로 고쳐 거부를 확인했다.
- **Rust 소스 변이**(`crossover_refactor.rs`, 임시 Cargo 사본, 23종): **모두 거부**. 계약 필수 `refactor-skipped`(직전 factor 유지)는 arms#423 x20에서 거부. Markowitz 0.1 → 기본 2⁻⁷은 capri#2 피벗 215에서 거부(push의 S+0x390 = 0.1이 피벗을 가른다). 그 밖에 L/Lᵀ/U 행·열 순서 뒤집기, L 행을 단계 대신 행 번호로, 풀 채움 U만, 열 기준, t = M1 prefix, 꼬리 경계 엄격, xe8 조건 반전, F+0xf8 경계, x108/x110 교환, 빈 범위 1.0, x1e0 대각 제외, xd8/xe4 출처, lst 마지막 칸 누적, Lᵀ lst 길이, F1a0 0, spike 0 불변식 제거, o 대신 행 순열.
- **회귀**([요약](../../bench/oracle/m3_push_refactor_rust/regression/summary.txt)): 17명령 모두 종료0: 21e `push_lu_check --controls --acceptance`, 21f `--write`(입력 파일 불변 `git diff --exit-code`)·`--compare`·`--self-test`(45종), 생성 `--write`·`--compare`·`--self-test`, `cargo test --test push_lu`(2), 21b `push_ftran` test·compare, 21d `push_btran` test·compare, py_compile, clippy, `cargo test --release --workspace` **139통과·0실패·ignored12**(새 테스트 1), `git diff --check`. 하네스 변이 함수를 일반화했으므로 21f 소스 변이도 다시 돌렸다(`regression/lu-mutations-21f.txt`, 아래 §6 끝).
- 실행하지 않은 검사: M1 `lu.rs`는 읽기 접근자와 저장 필드만 더해 solver 수치 경로가 바뀌지 않으므로 확장 설치·pytest·Netlib·P3는 제외했다. 원본 캡처 0회. 독립 검토자는 쓰지 않았다(사용자 지시로 이 세션이 작업자·마스터를 겸함).
- **계약 변경**(결과 확인 중, 이유 기록):
  1. 할당 4필드(F+0x158·pool_mod16·F+0x188·F+0x1f8)의 출처: 계약은 첫 재분해 `lu:enter` factor였으나 21e가 push 첫 `05b257` 진입 factor를 캡처하지 않았다(미초기화 첫 호출, 21e §6). `036ade`는 풀 부족 재시도(C:2592–2620: 용량 1.4배·−1 반환, `05b257`이 다시 호출)에서만 F+0x158을 바꾸므로, 그 재분해의 `036ade` 반환이 1회임을 비교기가 확인한 뒤 첫 재분해 `lu:leave`에서 이 4필드만 읽는다. 다른 factor 필드는 읽지 않는다.
  2. Lᵀ lst(F+0x138): 원본 배열은 m+1칸이고 `lst[levels+1]`은 마지막 레벨의 개수다(아래). 캡처는 L은 levels+2칸, Lᵀ는 levels+1칸을 담으므로 backend는 Lᵀ lst를 levels+1칸으로 둔다(`221173`은 그 이상을 읽지 않는다).
  3. F+0x1a8 bucket: 재분해 직후 원본에는 `036ade` scratch 잔류값(1.0의 상위 32비트 1072693248 등)이 남는다. 읽는 쪽(FTRAN A2·BTRAN 레벨 경로)이 쓰기 전에 0으로 둔다. Rust는 이 칸을 미지값(`BUCKET_UNKNOWN`)으로 두고 비교에서 빼며 센다(재분해 비교 34칸). 미지값이 읽히면 출력이 달라져 드러난다. 이후 FTRAN/BTRAN 스냅숏에서는 미지 칸이 남지 않았다.
  4. push 첫 재분해의 spike 값 버퍼는 21f 주입과 같은 C 불변식(목록 밖 칸 0)으로 둔다. 처음에 미지값으로 두었더니 adlittle#191 FT가 그 칸을 읽어 멈췄다. 두 모드의 풀·스냅숏 전체가 같음을 진단 덤프로 확인한 뒤 원인을 찾았다.

## 7. 판정과 인수인계

- 작업 수행: **구현 완료(계약 범위, 예상 실패 1재분해 포함)**. 동등성: push 자연 7사례 재분해 19회 중 2×2 블록 단계가 피벗하지 않는 **18회**에서 `036ade` 분해와 분해 뒤 factor 설정을 Rust가 원본 배열 표현으로 생성해 factor 전체가 **비트 일치(통과)**. 원본 factor 주입 없이 그 위의 FT 36·FTRAN 119·BTRAN 32가 일치한다. capri 23회째 재분해(2×2 블록)와 그 뒤 FTRAN 1회는 **실패**(사전 명시, 통과로 세지 않음).
- 새로 확정한 원본 사실: F+0x100(t)는 `1da0b2`의 열 singleton 단계 3 끝(s3, C:1514–1532)이다. M1 `Lu::finish`는 t를 "앞쪽 빈 U 열 수"로 세서 단계 4 열이 비면 s3보다 커진다(adlittle 1·2회째 재분해에서 36 대 35). 이 차이는 M1 simplex의 FT 갱신·FTRAN 경로에도 영향을 줄 수 있으나 M1 동작은 바꾸지 않았다(아래 후속). `lst[levels+1]`은 마지막 레벨 개수, x1e0 = m + nnz(L), x160 = 2·(nnz(L)+nnz(U)), 재분해 직후 F+0x1a8은 scratch 잔류값.
- 미검증: 2×2 블록 단계(capri 23), 조밀 전환 dgetrf, 특이 기저, 풀 부족 재시도, 1e99 피벗 상한(자연 0회), work. M1 커널의 shortcut이 다른 모델에서 드러날 가능성.
- 범위 밖: 루프 설치22·품질15a·batch23·x87·P3.
- 후속:
  - **M3-04c-21h**(M1 조율): `1da0b2` 2×2 블록 단계(C:686–1510, S+0x4e0/0x4e4 상태와 갱신 규칙 C:1498–1510)를 M1 `Lu`에 이식하고 capri 23 재분해를 닫는다. M1 simplex 경로에도 영향이 있어 M1 점유와 회귀(pathcmp·Netlib)가 필요하다. 같은 작업에서 M1 t(prefix) = s3 정정 여부도 M1과 정한다.
  - 다음 M3 후보: **M3-04c-22**(초기화된 직렬 push 루프 연결) 또는 21h.
- 재실행: `python3 tools/push_lu_rust.py --generate --write` → `cargo test -p rsolver-lp --test push_lu` → `python3 tools/push_lu_rust.py --generate --compare` / `--generate --self-test` / `--generate --mutate-rust <새 경로>`. 21f 주입 모드는 `--generate` 없이 그대로다.
- branch `work/m3-04c-21g`, worktree `../rsolver-m3-04c-21g`, 선행 미커밋 없음. 최종 commit·병합은 main `Merge M3-04c-21g` 이력을 따른다. main 미추적 `.claude/settings.local.json` 보존. 원격/업로드 없음.
- 교훈: `docs/lessons/original.md`(t = s3와 M1 prefix 차이·lst 마지막 칸·F+0x1a8 잔류·풀 재시도), `tooling.md`(회문 변조·주입 모드와 생성 모드의 상태 덤프 비교).
