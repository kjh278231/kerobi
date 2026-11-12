# M3-04c-1c 조건수 추정 `116fd7` 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 선행: [1b 초기화 오라클](primal_push_init_oracle.md), [1d 외부 값 경로 비교](primal_push_init_ext_rust.md).

## 1. 범위와 주장

- 작업 종류: 오라클 확장(원본 내부 경계 캡처) + 구현(Rust 이식) + 검증.
- 완료할 동작: push 초기화 품질 gate가 부르는 `12f282` → double 경로 `116fd7`(Hager식 1-노름 조건수 추정)을 Rust로 옮겨, 1d 하네스의 조건수 tape(원본 출력 주입)를 Rust 계산으로 바꾼다. 원본과 같은 factor·같은 입력 상태에서 ‖B‖₁, 반복마다 FTRAN 입력·출력, 부호 벡터(BTRAN 입력)·BTRAN 출력, 추정값, 선택 index, 반복 수와 종료 사유, 출력 condition이 비트/정확 일치함을 보인다. 이어 1d의 기존 기대 줄(gate·retry·루프 직전 상태) 전체가 주입 없이 일치함을 보인다.
- 포함 분기와 입력 전제: double(S+0x28 = 0), m > 0, F+0xc = 0(재분해 직후, `05b257` C:1977). 1b 14사례(first=1)·19 호출에서 실제로 지나는 종료 사유(추정값 비증가, index 반복, 5회 한도)와 FTRAN 출력 표현(dense/sparse)은 결과 §6에 관측 여부로 적는다.
- 제외와 후속: x87 경로 `23e604`(precision ≠ 0), m = 0(→ 0.0)·F+0xc ≠ 0(→ 1.0) 조기 반환(C 근거로만 구현, 자연 미관측 예상), work 누적(`9e288` param_4·`09cf9b` work 인자, P2), NaN 입력의 SIMD max/argmax 동작, `12f282`의 다른 caller 10곳(simplex·MIP 등), first=0(조건수 호출 없음, 1d 범위), 해 설치(04f).
- 선행 commit: main `e698f5e`(1d 병합). 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 ELF file offset(Ghidra = +0x100000).
- 디컴파일 `../re/decomp/c/001a92c0_PRIVATE0000000000116fd7.c`(659줄), `001bbf80_…12f282.c`, asm `python3 tools/dc.py 116fd7 --asm --max 2000`(0x1a92c0–0x1a9e8b). 호출: `09cf9b@0x1205d0`(dense/sparse FTRAN, param_4 = 0 → spike 없음, `2207de`), `9e288@0x12de10`(BTRAN, F+0x200 = 0 뒤 `221173(F, in, out, cache=0)`, in.count = 0이면 out.count = 0만).
- ABI(asm 확인): `116fd7(rdi = 모델 객체, rsi = &condition, rdx = work 포인터)`. `12f282`가 레지스터를 그대로 넘기는 tail call이므로 116fd7의 return address는 caller `0x1c17b8`이다. S = *(rdi+0xe0), F = S+0x470, m = S+0x64, n = S+0x68, A 열 시작 S+0x70(i64)·길이 S+0x78(i32)·값 S+0x88, header F+0x18. x = S+0x220 구조, y = S+0x228 구조(+0 count, +8 idx*, +0x10 val*). FTRAN 호출 `0x1a966a`(→ `0x1a966f`): `09cf9b(F, x, y, 0, work)`. BTRAN 호출 `0x1a9985`(→ `0x1a998a`): `9e288(F, y, x, work)`.
- C·asm으로 정한 동작(이번에 원본 캡처와 대조할 규칙):
  1. m = 0 → 0.0, F+0xc ≠ 0 → 1.0 반환(0x1a9a8d·0x1a9a71).
  2. ‖B‖₁: 위치 k = 0..m에서 j = header[k]. j ≥ n이면 1.0, 아니면 열 j 값의 |a| 합. 합산 순서(asm 0x1a9387–0x1a948b): 길이 < 8 또는 (주소 & 7 ≠ 0) 또는 길이 < peel + 8이면 순차, 아니면 주소 & 0xf = 8일 때 첫 항을 lane 0에 먼저(peel), 8항 묶음을 lane (0,1)(2,3)(4,5)(6,7)에 더한 뒤 ((0+2)+(4+6)) + ((1+3)+(5+7)), 나머지 순차. 최댓값은 `maxsd new, old`(new > old면 new), 초기 0.
  3. x = 1/m dense(count = −1). 반복: FTRAN(x → y), x.count = y.count, 값(sparse면 idx도) 복사(포인터가 다를 때만). est_new = ‖x‖₁(x.val 앞 count개, dense면 m개; 합산 순서는 2와 같고 주소는 x.val). 반복 > 0이고 est_old ≥ est_new면 종료. y = dense(count = −1) 모두 1.0, x 값 < 0인 위치(sparse면 idx 위치)는 −1.0. BTRAN(y → x). 반복 += 1, 5이면 종료. j = argmax |x|(초기 최대 0·index −1, 엄격 >, sparse는 packed 순서의 idx) — SIMD 경로(0x1a9b64–0x1a9dcf)는 lane별 엄격 > 뒤 최댓값과 같은 lane 중 최소 index라 NaN이 없으면 순차 첫 최대와 같다. j = 직전 j(처음 −1)면 종료. x = e_j(count = 1, idx[0] = j, val[0] = 1.0).
  4. 종료 시 condition = ‖B‖₁ · est_new(`mulsd`, 0x1a9b4d). 반환값 0.
- 시작 경계: `116fd7` 진입(return address `0x1c17b8`, 초기화 중). 종료 경계: 반환 직후 *rsi.
- 입력 상태: A(push:enter model), header F+0x18, F+0xc, 초기 재분해된 factor(1d에서 Rust가 같은 factor를 만든다는 것이 이미 검증됨: FTRAN·잔차·basic 비트 일치), 버퍼 주소의 16바이트 정렬(A 값 배열 기준, x.val·y.val). 정렬은 합산 순서를 바꾸므로 캡처한 mod 16을 Rust 입력으로 준다.
- 별칭·수명: FTRAN/BTRAN이 헤더(count·idx·val 포인터)를 교환할 수 있어 x.val이 가리키는 물리 버퍼가 바뀐다. 진입 시 x.val·y.val·x.idx·y.idx 포인터를 이름(X0·Y0·X0i·Y0i)으로 두고 각 호출 전후 어느 버퍼인지 기록한다.

## 3. 비교 계약

| 필드·사건 | 의미·좌표·순서 | 비교 규칙·근거 | 캡처·재구성 방법 |
|---|---|---|---|
| `cond:enter` | m, n, F+0xc, precision, header, A 값 기준 정렬, x/y 버퍼 이름·정렬, condition 진입값 | 정확 | 새 agent 116fd7 진입 훅 |
| `cond:ftran` 입력·출력 | 반복 k, x 헤더(count, idx, val) → y 헤더, 호출 전후 버퍼 이름 | count·idx 정확, 값 비트 | `09cf9b` 진입·반환(caller 0x1a966f) |
| `cond:btran` 입력·출력 | 부호 벡터 y(dense ±1) → x 헤더, 버퍼 이름 | 정확·비트 | `9e288` 진입·반환(caller 0x1a998a) |
| `cond:leave` | condition 출력, 반환값 | 비트·정확 | 반환 훅 |
| C 재계산(검사기) | ‖B‖₁, 반복별 est, 종료 사유, j, FTRAN 입력 규칙, 부호 규칙, 출력 = ‖B‖₁·est | 비트·정확 | Python이 §2 규칙으로 캡처만으로 재계산 |
| Rust 내부 줄 | `cnorm`(‖B‖₁), `cft`(입력·출력), `cbt`(입력·출력), `cest`(반복·est), `cres`(종료 사유·반복·condition) | 비트·정확 | Rust `crossover_condition`이 1d 하네스 안에서 `LuState` FTRAN/BTRAN으로 계산 |
| 1d 기대 줄 전체 | `lu`·`ftin`·`ftout`·`res`·`mk`·`cond`(before = 직전 Rust 조건수)·`gate`·`start`/`retry`(cond) | 1d와 같은 규칙 | 조건수 tape 0개(주입 없음) |

- 수치 오차 허용 없음(모두 비트). NaN·±0: 원본 자료에 NaN이 나오면 실패로 보고 범위 밖으로 기록한다. −0.0은 부호 규칙에서 +1(엄격 < 0)로 C와 같게 다룬다.
- 시간·work·병렬: Threads = 1. work 포인터 값은 기록만 하고 비교하지 않는다(P2).
- 재현성: 1b 15사례 설정 그대로 새 agent로 독립 3회. 3회 사건이 같고, 새 사건(`cond:*`)을 뺀 투영이 1b run 1과 같아야 한다(비간섭).

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 전제·노리는 분기 | 입력 경로 | 원본 출력·분기 실행 증거 |
|---|---|---|---|---|
| arms·retry·interior·adlittle·scsd1·capri | 실제 | 자연 조건수(1b: 1.0·1.0·1.0·313.6·54.6·6196) | 1b와 같음 | 결과 §6 |
| adjust·mk_low·mk_limit·mk_retry 등 합성 8 | 합성(1b 편집 그대로) | 조정 2라운드(조건수 2회)·retry 재호출(attempt = 1)·다른 factor(Markowitz 강화 뒤) | `bench/oracle/m3_push_init/synthetic.json` | 결과 §6 |
| adlittle_x1 | 실제(first = 0) | `116fd7` 호출 없음 대조 | 1b와 같음 | 사건 0 |

- 자료: `bench/oracle/m3_push_condition/`(15사례 × 3, manifest), Netlib 모델은 저장소 밖(`../re/bench/data/netlib`, 해시는 캡처에 기록).
- 적용하지 않은 범주: x87, 다른 caller, m = 0, F+0xc ≠ 0, NaN(자연 사례에서 생기지 않는다고 보고 생기면 기록).

## 5. 재실행 명령

- 원본 캡처: `tools/hrun.sh --timeout 580000 -- 'python3 tools/push_cond_runs.py'`(main worktree가 아닌 작업 worktree에서, Gurobi 한 프로세스).
- 검사기: `python3 tools/push_cond_check.py --controls --acceptance`, `--self-test`(변조 거부), `--write-manifest`.
- Rust 재생·비교: `python3 tools/push_cond_rust.py --write && cargo test -p rsolver-lp --test push_init_ext && python3 tools/push_cond_rust.py --compare`(기대: 1d 17호출·132줄 + 조건수 내부 줄), `--self-test`, `--mutate-rust OUT`.
- 회귀: `python3 tools/push_init_rust.py --write && cargo test -p rsolver-lp --test push_init_ext && python3 tools/push_init_rust.py --compare`(1d, 주입 경로 불변), 1a `push_loop_rust.py --init`, 24 `--lu`, `push_init_check.py --controls --acceptance`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`.

## 6. 실행 결과

(실행 후 작성)

## 7. 판정과 인수인계

(실행 후 작성)
