# M3-04c-21f 직렬 numeric backend 연쇄(FT 갱신·재분해 재번호) Rust 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 원본 기준 자료는 [M3-04c-21e 지연 LU 연쇄 오라클](primal_push_lu_chain_oracle.md), 연결하는 adapter는 [21b FTRAN](primal_push_ftran_rust.md)·[21d BTRAN](primal_push_btran_rust.md)이다.

## 1. 범위와 주장

- 작업 종류: **구현**. 기준 main `05f2e97`(21e 병합), branch `work/m3-04c-21f`, worktree `../rsolver-m3-04c-21f`. 선행 미커밋 없음. main의 미추적 `.claude/settings.local.json`은 무관하며 보존한다. 사용자의 "다음 M3 스프린트 진행해 너가 작업자야" 지시로 이 세션이 직접 작업한다(작업자 선택 질문 생략).
- 완료할 동작: 원본 factor 배열 표현(F+0x20 … F+0x1d8, U·eta 공용 풀 위치 포함)을 상태로 갖는 Rust 직렬 backend `crossover_lu`가 push 안의 factor 연산 사건열을 순서대로 처리한다.
  - **FT 갱신**(`05b257` 요청0·S+0x290=0, C:144–1335): 구간 i52/i35, 행 eta 판정과 `09e47c` 호출(= 입력 e_r·출력 cache F1d0인 `21a941`, §2), cache 내적의 8-lane 결합으로 새 대각, spike 목록에서 r 제거, 새 접두 경계 local_40·LU+0xe8 탐색, 열 r 제거와 LU+0xf0 목록, 풀 여유 local_158 계산, 행 eta 레코드(풀 F+0x160 위치)·F1a0 최소 갱신·F180/F184, 행 r 제거(local_80 1/2/3), 빈 열 승격, 순열(교환·순환·회전), t=local_40+1, 새 열의 풀 재배치와 tail/prefix 배치, 꽉 찬 U 행의 풀 재배치(여유 min(max(4,len/2), F188−F18c)), LU+0xe8 갱신, F18c+1, spike count 0, 공통 종료(cache count 0·F1d8=−1).
  - **재분해의 push 상태 재번호**(C:2006–2090, 2765–2770): spike 목록 값 +0·count 0, `r'=n[r]`, `header'[p]=header[o[p]]`, vstat 재번호, spike idx/header 버퍼 교환 별칭, cache count 0·F1d8=−1.
  - **FTRAN·BTRAN 연결**: 공급 wrapper `09d008`의 spike clear(목록 칸 +0·count 0) 뒤 21b `ftran`, 21d `btran`을 Rust가 들고 있는 factor·spike·cache·F1d8·level bucket으로 부른다.
- **주입점(원본 입력)**: 재분해 호출의 `036ade` 분해와 그 뒤 factor 설정(C:2570–2760: 순열 identity·F40/F48·F1a0·U 끝·접두 목록·밀도 필드 등)은 이번 범위에서 옮기지 않는다. 각 재분해 `lu:leave`의 factor(header·spike·cache·F1d8 제외)를 원본에서 읽어 Rust 상태를 다시 세우고, U 열 배열의 풀 기준(F+0xa8 − F+0x148)은 `036ade` C:362·5378(F+0xd0 = 풀 시작, F+0xa8 = 풀 + local_1c0)에 따라 재분해 직후 F+0x160/2로 둔다(21e 자료의 FT 열 재배치 24회 모두 이 값과 일치, 결과에 다시 센다). 재분해 사이(FT 갱신·FTRAN·BTRAN 연쇄)에는 **원본 중간 factor를 다시 넣지 않는다**. `036ade` 분해 수치 생성과 C:2570–2760 설정은 후속 **M3-04c-21g**로 분리한다(6,037줄 함수, M1 `Lu::factor`는 자체 표현이라 별도 adapter가 필요).
- 포함 분기와 관측 목표(21e 집계): FT 36(행 eta 26·구간 자명 10·F180 +1 23), 재분해 19, FTRAN(eta>0 41 포함)·BTRAN(지원 32), 7사례(arms·retry·interior·adlittle·scsd1·capri, 대조군은 push 사건0).
- 제외 범위: `036ade`와 C:2570–2760(21g), FT 실패·재시도 경로(|S_r|<1e-13, 풀 부족 재할당 `cd9cc`·`1f2069`, 요청 6/11·반환1, cache 0인데 U 행 r 비어 있지 않음; 자연 0회이며 Rust는 오류로 멈춘다), pricing 객체 경로(S+0x478/0x480≠0, `9eb18`·`9e60b`), F190·work(S+0x?·F+0x200 누적은 기록만, 21e 계약과 같음), 루프 설치22·품질15a·batch23·x87·P3. M1 공유 코드(`lu.rs`)는 바꾸지 않는다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 21e manifest의 바이너리·캡처·도구 해시를 그대로 검사한다(`push_lu_check.check_files`).
- C(직접 읽음): `../re/decomp/c/000f15c0_PRIVATE000000000005b257.c` C:1–1335(FT 경로 전체), C:1990–2095·2540–2610·2755–2772(재분해 뒤 spike·재번호·identity·종료), `00131760_PRIVATE000000000009e47c.c` 전체, `00120680_PRIVATE000000000009d008.c`(spike clear), `000d4190_PRIVATE0000000000036ade.c` C:340–365·5355–5385(풀 기준 포인터). asm: `tools/dc.py 05b257 --asm --range 0xf19c0,0xf1c01`(cache 내적: lane 쌍 (0,1)(2,3)(4,5)(6,7), 결합 `((l0+l2)+(l4+l6)) + ((l1+l3)+(l5+l7))`, 꼬리 순차; 21b `eta_dot`과 같은 형태).
- `09e47c` = `21a941`: 두 함수의 F108≤5 heap 경로(입력 행 U 위치 < t면 바로 나눗셈·출력·흩기, 아니면 값 누적·표시, min-heap, `m−top ≤ 6n`이면 순차 전환, pop·순차에서 `2·len` work)와 F108>5 순차 경로(flag −1/−2/≥0 분기, `3c + 2(m−low)`)가 C 줄 단위로 같다. 차이는 입력이 e_r(count 1·값 1.0, S+0x220에 씀), 출력이 cache F1d0, U 행 r이 비면 cache count 0으로 조기 반환(work 없음), F+0x200을 0으로 시작하는 것뿐이다. 그래서 21d `kernel`을 이 입력·출력으로 부른다.
- cache 내적의 peel: cache 값 버퍼 주소의 mod 16이 필요하지만 21e 캡처에 없다(F1d0 버퍼는 별도 할당). **가정**: 16바이트 정렬(peel 없음). cache count ≥ 8인 FT가 있으므로(21e 13회) 이 가정이 틀리면 대각 비트 차이로 드러난다. 미캡처 가정으로 기록한다.
- 풀 좌표: U 행(F+0xd0/0xc8)은 풀 시작, U 열(F+0xa8/0xa0)은 풀 + cb(위 주입점), eta 레코드는 21e의 `idx_off`(F+0x148 기준). eta 값의 peel = `(pool_mod16[1] + 8·val_off) mod 16 = 8`. 풀 상한 F+0x168(초과면 원본은 재할당 경로, 제외 범위).
- 행 끝 경계 F+0xc0(꽉 찬 행 판정)은 캡처되지 않는다. 재분해 뒤 C:2580–2593이 `F+0xb8[i] = F+0xc0[i] = F+0xb0[i+1]`로 두므로 주입 시 F+0xc0 = 행 끝으로 시작하고, 이후는 Rust가 재배치에서 계산한다. push의 첫 factor 사건이 재분해가 아니면 그 push는 시작할 수 없어 실패로 센다.
- spike 값 버퍼: 목록 밖 칸은 0이라는 C 불변식(FTRAN spike 저장·`09d008` clear·FT C:1250 지우기·재분해 C:2006–2030이 목록 칸만 쓰고 지움)에 따라 재분해 주입 때 전 칸 +0으로 둔다(가정, 기록).
- 시작 경계: 사례의 각 push에서 첫 `lu:leave`(재분해) 직후. 종료: push의 마지막 factor 연산 반환. 연산 입력 중 push 상태(공급/재계산 FTRAN 입력 벡터·save·spike 인자, BTRAN 입력·out count, FT의 이탈 위치 r, 재분해 진입 header·vstat·r, `036ade` 반환 순열 o/n)는 원본 사건에서 읽는다. factor·spike·cache·F1d8·level bucket·scratch는 Rust 상태만 쓴다.

## 3. 비교 계약

| 대상 | 규칙 |
|---|---|
| FTRAN `ftran:enter`/`ftran:leave` | factor 스냅숏(header 제외 공통 키 전부: perm·diag·L·U 열/행과 풀 위치·scalars·prefix·eta_oldest·etas(offset·mod16 포함)·spike·scratch)과 leave의 output·input_after·alias |
| BTRAN `btran:enter`/`btran:leave` | 위 factor 스냅숏 + Lrow(bucket 포함)·cache·x1d8, leave의 output·input_after·alias |
| FT `lu:enter`(요청0) | factor 스냅숏 전체(header 제외) |
| FT `lu:roweta` | 호출 유무, 행 인자, leave의 cache(count·목록·값) |
| FT `lu:leave` | 반환 0, factor 스냅숏 전체(header 포함, header는 진입과 같아야 함) |
| 재분해 `lu:leave` | header, 이탈 위치(S+0x2f8), vstat 전체, spike count, cache count·F1d8, 버퍼 교환 불리언(spike idx = 진입 header·header = 진입 spike idx) |

- 정수·목록·순서 정확, f64는 ±0 포함 little-endian 비트. 미지값·NaN·무한대·adapter 오류는 실패. 원본 기대값은 Python 비교기만 읽고 Rust 입력에는 넣지 않는다(주입점·push 상태 입력 제외).
- 스냅숏 비교에서 F190(x190)·work·x200은 제외한다(21e와 같음). `L`·`Lrow`의 levels·lev·lst는 비교하되 FTRAN/BTRAN이 바꾸지 않는 값이다.
- 원본 재현성은 21e 정식 21회(7사례 각 독립3회 동일)를 재사용한다. 이번 원본 실행은 0회가 목표다.
- 판별력: (a) 재생 trace 변조(사건 누락·factor 필드 비트·풀 위치·eta offset·cache·header·vstat), (b) Rust 소스 변이(임시 Cargo 사본): **pivot마다 재분해**(FT 대신 주입 상태 유지)와 **무조건 행 eta**(구간 판정 무시) 변이는 반드시 거부돼야 하며, 그 밖에 local_40 분기·순환 조건·풀 여유 식·tail/prefix 배치·F1a0 최소·8-lane 결합·spike clear 생략을 바꾼다. 생존은 따로 센다.

## 4. 사례와 원본 자료

| 사례 | 출처(21e 정식 `*-1`, 2·3과 동일 검증됨) | push 안 05b257 | 노리는 분기 |
|---|---|---:|---|
| arms | `arms-1.json` | 4 | 재분해 4, 지원 BTRAN |
| retry | `retry-1.json` | 3 | 재분해 3 |
| interior | `interior-1.json` | 9 | FT 5(행 eta 3), 재분해 4, eta>0 FTRAN |
| adlittle | `adlittle-1.json.gz` | 14 | FT 11 연쇄(eta 6까지), cache 경로 BTRAN |
| scsd1 | `scsd1-1.json.gz` | 2 | 재분해 2 |
| capri | `capri-1.json.gz` | 23 | FT 20 연쇄(eta 14까지, cache ≥8 내적), level L/Lᵀ |

- 21b가 구현하지 않은 FTRAN 경로(`21b9de` 레벨 A2·경로 B, `21574d`)가 Netlib 사례에서 나오면 그 push는 adapter 오류로 실패한다. 이 경우 실패로 기록하고 범위를 조용히 줄이지 않는다.

## 5. 재실행 명령

- 기준 재검사: `python3 tools/push_lu_check.py --controls --acceptance`.
- 구현 재생: `python3 tools/push_lu_rust.py --write` → `cargo test -p rsolver-lp --test push_lu` → `python3 tools/push_lu_rust.py --compare`.
- 오류 검출: `--self-test`(trace 변조), `--compare --mutate-rust <새 결과경로>`(소스 변이).
- 회귀: `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, 21b `push_ftran_rust.py --compare`, 21d `push_btran_rust.py --compare`, py_compile, `git diff --check`. solver·확장 경로는 바뀌지 않으므로 pytest·Netlib·P3는 제외하고 이유를 기록한다.

## 6. 실행 결과

- 환경: WSL2 x86-64 Linux 6.6.87.2, Python 3.10, Rust stable(clippy 1.99). 계약 `17e4179`, 첫 동작 `4b3af39`(capri A2 미구현으로 실패, 아래), A2·별칭·미지칸 `78bb095`, 이후 clippy 정리·변이 도구. 원본 실행 **0회**(21e 정식 `*-1` 재사용, `push_lu_check --controls`로 3회 동일·해시 재확인). 증거 `bench/oracle/m3_push_lu_rust/`(입력 `input.txt`, `comparison.txt`, `self-test.txt`, `rust-mutations.json/.txt`, manifest, 회귀 기록).
- **연쇄 재생**(`--compare` 종료0): 7사례(대조군 사건0) push 1회씩, 연산 **207개**(재분해 19·FTRAN 120·BTRAN 32·FT 36) 전부 원본과 비트/정확 일치, 비교 필드 1,257,225개. 재분해 사이에는 원본 factor를 다시 넣지 않았다.

| 사례 | 재분해 / FTRAN / BTRAN / FT | 최대 eta 수 |
|---|---|---:|
| arms | 4 / 28 / 17 / 0 | 0 |
| retry | 3 / 10 / 4 / 0 | 0 |
| interior | 4 / 12 / 1 / 5 | 2 |
| adlittle | 3 / 24 / 2 / 11 | 6 |
| scsd1 | 2 / 13 / 1 / 0 | 0 |
| capri | 3 / 33 / 7 / 20 | 14 |

| 분기(관측) | 수 | 비고 |
|---|---:|---|
| FT 행 eta(`09e47c`) / 구간 자명 / cache 재사용 | 26(eta 생성 23) / 10 / **0** | 행 eta 26 중 cache 0개 3 |
| FT local_80 0/1/2/3 | 7/17/6/6 | 1 = 접두 r, 2 = 꼬리 r, 3 = eta 없이 r을 꼬리로 |
| FT 빈 열 승격 / 순환 / 새 열 재배치 / 행 재배치 | 14 / 21 / 33 / 174행 | 열 재배치 33회 모두 열 기준 = 분해 직후 F+0x160/2 |
| FT cache 내적 8-lane / 순차 | 11 / 25 | peel 유무는 판별 불가(아래 변이) |
| FTRAN eta>0 / R 희소 목록 / R 희소→조밀 | 41 / 13 / 3 | 21b 자연 미관측이던 R 희소 경로를 이번에 비교 |
| FTRAN `21b9de` A1 / A2(레벨) / `214344` | 47 / 25 / 48 | A2는 이번에 구현 |
| BTRAN cache 경로 / 빈 행 / level Lᵀ / eta>0 | 7 / 25 / 6 / **0** | |
| 공급 FTRAN `input_after` 미지 칸(비교 제외) | 2,255 | 원본 push 버퍼 잔류값, 21a 계약상 기록만 |

- **trace 변조**(`--self-test` 종료0): 45종 모두 목표 필드의 첫 차이로 거부(연산 누락·순서·머리 해시·오류 주입, 재분해 header·이탈 위치·vstat·spike·버퍼 교환·cache·F1d8, FT 대각·x48·x28 별칭·U 열 시작/경계·행 시작·열 값·행 순서·eta offset/값/정렬·F1a0·x160·t·xe8·x18c·x184·F+0xf0/0xf8·spike 잔류·행 eta 누락/추가/cache 값·진입 spike, FTRAN 출력·별칭·spike·wrapper clear·scratch·재계산 입력 후상태, BTRAN cache·F1d8·level bucket·출력 목록).
- **Rust 소스 변이**(임시 Cargo 사본, 24종, `rust-mutations.json`): 거부 21, 재생 시간 초과 1, **생존 2**. 계약 필수 두 변이 `ft-skipped`(FT 대신 factor 유지, 매번 재분해 대체)·`roweta-always`(구간 판정 무시 무조건 행 eta)는 첫 차이로 거부됐다(interior#80 U 열 6 길이, interior#80 행 eta 추가). 그 밖에 행 eta 생략·local_40 첫 분기·빈 열 승격 생략·local_80 1↔2·3↔0·회전 생략·풀 여유·행 용량·열 풀 기준·tail/prefix 경계·F1a0 덮기·순차 내적·F+0xf8 표시·LU+0xe8 갱신 생략·재번호 o↔n·이탈 위치·vstat·wrapper clear 생략·bucket 비공유가 거부됐다. `cycle-strict`(순환 탈출 `≤`→`<`)는 재생이 끝나지 않아(무한 루프) 240초 제한으로 trace 없이 거부된다(첫 실행은 제한이 없어 20분 멈춰 하네스에 제한을 넣었다). 생존: `local40-last`(`t−2+(np=0)` → `t−2`)는 그 분기가 i35>i52라 r 밖 spike 원소가 있어 np≥1이므로 **동치**, `dot-peel`(cache 내적에 peel 강제)은 이번 8-lane 11회에서 결과 비트가 같아 **판별 불가**라 cache 버퍼 16 정렬 가정은 미검증으로 남는다(순차 변이는 adlittle#512 대각에서 거부).
- **회귀**([요약](../../bench/oracle/m3_push_lu_rust/regression/summary.txt)): 14명령 모두 종료0: 재생 write·`cargo test --test push_lu`·`--compare`·`--self-test`·`--mutate-rust`, 21b `cargo test --test push_ftran`·`push_ftran_rust.py --compare`(A2 추가 뒤 push 50호출·M1 커널 186호출 그대로 통과), 21d `--test push_btran`·`push_btran_rust.py --compare`, 21e `push_lu_check.py --controls --acceptance`, py_compile, `cargo clippy --workspace --all-targets -- -D warnings`(첫 실행이 타입 복잡도·이중 비교 2건으로 실패해 고친 뒤 재생·변조·변이를 다시 실행), `cargo test --release --workspace` **138통과·0실패·ignored12**, `git diff --check`.
- 실행하지 않은 검사: solver·확장·Python 경로를 바꾸지 않아(`lib.rs`에 모듈 등록, 21b adapter에 A2 추가) 확장 설치·pytest·Netlib·P3는 제외했다. 원본 캡처 0회. 독립 검토자는 쓰지 않았다(사용자 지시로 이 세션이 작업자·마스터를 겸함).
- **계약 변경**(결과 확인 중, 이유 기록):
  1. 첫 비교(`4b3af39`)에서 재분해 뒤 FT가 바꾼 순열이 원본에서는 F+0x28/0x38에도 보였다. 재분해 C:2572–2573이 F+0x40/0x48을 F+0x28/0x38 버퍼로 돌리기 때문이다. 이 별칭(21e `lu:leave.buffers.x40_is_x28/x48_is_x38`)을 주입점에 넣었다(분해 뒤 설정의 일부, 비교 규칙 불변).
  2. capri FTRAN이 21b 미구현 `21b9de` 레벨 경로 A2를 탔다(첫 비교에서 capri 62연산 adapter 오류). §4는 이 경우를 실패로 기록하라고 했으나, 범위를 줄이지 않고 C:259–604(+ asm 0x120823: wrapper null이면 F+0x1a8)를 읽어 A2를 21b adapter에 구현했다. 실패 기록은 `4b3af39` 이력과 이 문서에 남긴다.
  3. 공급 FTRAN의 `input_after`는 원본 push 버퍼의 이전 값이 남은 칸을 포함한다. 21a 계약이 이 필드를 "기록만"으로 둔 것을 따라, Rust가 아는 칸만 비교하고 미지 칸 수를 출력한다. 재계산 FTRAN의 `input_after`는 전부 비교한다.

## 7. 판정과 인수인계

- 작업 수행: **구현 완료(계약 범위)**. 동등성: push 자연 7사례의 factor 연산 207개에서, 재분해 지점의 `036ade` 결과만 주입하고 그 사이 FT 갱신 36회·FTRAN 120회·BTRAN 32회와 재분해 재번호 19회가 factor 전체(풀 위치·eta 오프셋 포함)·출력·spike·cache·bucket까지 원본과 **비트 일치(통과)**.
- 미검증: FT cache 재사용(F1d8=r, 자연 0회), FT 실패·재할당 경로(Rust는 오류로 멈춤), pricing 경로, BTRAN eta>0(자연 0회), cache 값 버퍼 정렬(16 정렬 가정, peel 변이 생존으로 판별 안 됨), 위 생존 변이의 경계. F190·work는 비교하지 않았다.
- 범위 밖: `036ade` 분해와 C:2570–2760 분해 뒤 설정(주입점, **M3-04c-21g**), 루프 설치22·품질15a·batch23·x87·P3.
- 다음: **M3-04c-21g**(`036ade` 분해 수치와 분해 뒤 factor 설정을 원본 배열 표현으로 생성해 21f 주입점을 없앰). `python3 tools/session.py claim M3-04c-21g --owner <새-owner>` → 이 문서 §1·§2, `docs/m1/m1_02a2_urow.md`·`m1_02q_work.md`(`036ade` 구조), `tools/dc.py 036ade`. M1 `Lu::factor`를 원본 배열로 옮기는 adapter와 `036ade` 직접 이식 중 무엇을 할지 계약에서 정한다. 기준 재검사 `python3 tools/push_lu_rust.py --compare`.
- 재실행: `python3 tools/push_lu_rust.py --write` → `cargo test -p rsolver-lp --test push_lu` → `python3 tools/push_lu_rust.py --compare` / `--self-test` / `--mutate-rust <새 경로>`.
- branch `work/m3-04c-21f`, worktree `../rsolver-m3-04c-21f`, 선행 미커밋 없음. 최종 commit·병합은 main `Merge M3-04c-21f` 이력을 따른다. main 미추적 `.claude/settings.local.json` 보존. 원격/업로드 없음.
- 교훈: `docs/lessons/original.md`(풀 좌표·F40/F28 별칭·`09e47c`=`21a941`·A2 bucket 공유), `tooling.md`(연쇄 재생의 주입점과 미지 칸).
