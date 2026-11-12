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

(실행 후 작성)

## 7. 판정과 인수인계

(실행 후 작성)
