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
- **범위 확장(탐색 뒤, Rust 비교 전에 결정)**: 1b 15사례에서 비자명 factor가 3개뿐이고 5회 한도·3회 이상 반복이 없어 Netlib 소형 45(1b 탐색 목록 `push_init_runs.SURVEY45`, Crossover=3 자연)를 정식 사례로 더했다. 이를 위해 agent가 초기 재분해 입력(`cond:lu`: F+0x10·0x158·풀 정렬·0x188·0x1f8, `036ade` 진입 S+0x2c0/0x4e0/0x4e4, S+0x46c·0x2fc·0x2f8)도 기록하고, 1b 자연 사례에서는 이것이 1a 입력과 같음을 생성기가 확인한다. 축소한 범위는 없다.

## 5. 재실행 명령

- 원본 캡처: `tools/hrun.sh --timeout 580000 -- 'python3 tools/push_cond_runs.py'`(main worktree가 아닌 작업 worktree에서, Gurobi 한 프로세스).
- 검사기: `python3 tools/push_cond_check.py --controls --acceptance`, `--self-test`(변조 거부), `--write-manifest`.
- Rust 재생·비교: `python3 tools/push_cond_rust.py --write && cargo test -p rsolver-lp --test push_init_ext && python3 tools/push_cond_rust.py --compare`(기대: 1d 17호출·132줄 + 조건수 내부 줄), `--self-test`, `--mutate-rust OUT`.
- 회귀: `python3 tools/push_init_rust.py --write && cargo test -p rsolver-lp --test push_init_ext && python3 tools/push_init_rust.py --compare`(1d, 주입 경로 불변), 1a `push_loop_rust.py --init`, 24 `--lu`, `push_init_check.py --controls --acceptance`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`.

## 6. 실행 결과

- 실행 환경: worktree `../rsolver-m3-04c-1c`, branch `work/m3-04c-1c`(기준 main `e698f5e`). 계약 `9d42a9d`, 도구 `5493957`, 캡처·검사기 `6d6f3a9`, Rust `8fbe738`, 기록 `4c5e651`. 최종 commit/병합은 main `Merge M3-04c-1c` 이력을 따른다. 바이너리 SHA는 §2와 같고 캡처마다 기록.
- **탐색**: 첫 캡처에서 116fd7 훅이 발화하지 않았다. 1b agent가 `0x1c17b3`(call 12f282) 자체에 훅을 붙여 Frida가 call을 trampoline으로 옮기므로 116fd7 진입의 return address가 `0x1c17b8`이 아니라 trampoline이었다(디버그 사건으로 확인). `0x1c17b3` 직전~`0x1c17b8` 직후 플래그로 식별하도록 고쳤다. 같은 디버그에서 116fd7이 초기화 밖 push 안(`0x1c4276`·`0x1c8a0e` 근처 caller)에서도 불림을 보았다(범위 밖, 후속 04f).
- **원본 캡처**(`bench/oracle/m3_push_condition/`, manifest 180파일): 1b 15사례 + Netlib 45, 각 독립 3회. `python3 tools/push_cond_check.py --controls --acceptance`([출력](../../bench/validation/m3_push_condition/check.txt)): 60사례 × 3회 사건 동일, 1b 15사례 cond:* 제외 투영이 1b run 1과 전체 동일(비간섭), x87 경로 0회, **116fd7 64호출 전부 C·asm 규칙 재계산과 비트/정확 일치**(첫 실행부터): ‖B‖₁, FTRAN 입력(1/m dense, e_j), est, 종료, 부호 벡터, argmax, condition = ‖B‖₁·est.
  - 관측 분기: 종료 2회 반복 index 반복 45, 1회 추정 비증가 14(전부 슬랙 기저 ‖B‖₁ = 1), 3회 반복 4(israel·scagr7·scagr25·sc205), 5회 한도 1(sc105). e_j FTRAN의 dense 출력 6호출. 정렬은 64호출 모두 A 값·x·y 버퍼 mod 16 = 0이지만 A 열 시작이 홀수인 열(peel 경로)이 31호출·409열. F+0xc = 0, m ≤ 687. sparse 입력 FTRAN과 dense 입력 BTRAN은 매번 헤더를 교환해 x 값 버퍼가 X↔Y로 번갈아 바뀐다.
  - `--self-test`([출력](../../bench/validation/m3_push_condition/self_test.txt)): 변조 16종 거부. 규칙 변이 필수 2(합산 순차·슬랙 0) 거부, 그 외 거부 5(부호 ≤ 0·한도 4·엄격 >·argmax 마지막·peel 무시), 생존 2(‖B‖₁ max ≥: 동률 동치, 합산 버퍼 = y: 모든 버퍼 정렬 0). 처음 변조 4종(순서 무관 header 교환, 정렬이 합에 영향 없는 사례, 최하위 비트 반올림 흡수)이 생존해 판별 가능한 변조로 바꿨다.
- **Rust 비교**: `crossover_condition::condition`을 1d 하네스(`push_init_ext` `init_cond_trace`)에 넣고 조건수 tape를 비웠다. 첫 비교 61/62호출 일치, modszk1에서 `21686f` H 경로가 힙 미지칸을 표시하며 이미 쓴 출력 칸까지 지워 `read of an unknown cell` 오류. 힙(sec2−1부터 아래)과 출력(0부터 위)은 서로 다른 비접두 행이라 겹치지 않으므로 위치 ≥ out인 칸만 미지로 두게 고쳤고, 이후 **62호출·966줄·4,378필드 전부 일치**([출력](../../bench/validation/m3_push_condition/rust_compare.txt)): 116fd7 64호출의 cnorm 64·cft 135·cest 135·cbt 121·cres 64 + 1d 형식 줄(gate·start 53·retry 2·exit End 7 등). modszk1 cft 출력 index가 원본과 같아 수정이 맞음을 확인했다. Netlib 45 중 7개는 후보 0으로 초기화 직후 해 설치로 가는 `End` 종료이며 1d에 없던 경로라 생성기에 기대 줄 `exit;End`를 더했다.
  - `--self-test` 변조 3,618종 전부 거부([출력](../../bench/validation/m3_push_condition/rust_self_test.txt)). `--mutate-rust` 17종 거부 15·생존 2([출력](../../bench/validation/m3_push_condition/rust_mutations.txt)): 첫 반복에서도 비증가 검사(est > 0이라 항상 거짓, 동치), sparse BTRAN 출력의 packed 위치(BTRAN 입력이 dense라 출력이 항상 dense, 미관측).
- **회귀**([요약](../../bench/validation/m3_push_condition/regression.txt)) 모두 종료 0: 1d `--write`(입력 불변)·`push_init_ext`·`--compare` 17호출·132줄·727필드, 1b `push_init_check.py --controls --acceptance`, 1a `--init` 731줄, 24 `--lu` 725줄, 21b `push_ftran` 50호출, 21f `push_lu` 207연산, 21g `--generate`(예상 실패 2연산 유지), 21h `--block`, py_compile·node check·diff check, clippy(첫 실행 1건 실패 → 수정 → 0), `cargo test --release --workspace` 148통과·0실패·ignored 12.
- 원본 실행 277회: 탐색 7회(디버그 1 포함), Netlib 탐색 45회, `cond:lu` 추가 전 첫 정식 45회(커밋 전 폐기, 같은 사례를 새 agent로 재캡처), 정식 180회. 독립 검토자 미사용.

## 7. 판정과 인수인계

- 작업 수행 **완료**. 동등성 **통과**(비트): 1b 15사례·Netlib 45 자연의 push 초기화 116fd7 64호출에서 Rust가 원본과 같은 factor·입력으로 ‖B‖₁·반복별 FTRAN/BTRAN 입출력·버퍼 교환·추정값·종료·condition을 비트 일치로 재현하고, 조건수 주입 없이 1d 초기화 기대 줄(62 push 호출)이 일치한다.
- 남은 한계: 정렬 mod 16 = 8 버퍼(x·y)는 자연 0회(합산 peel은 A 열에서만 검증), sparse BTRAN 출력·est = 0·NaN·m = 0·F+0xc ≠ 0(1.0 조기 반환)·x87 `23e604` 미관측, work 누적 미비교, 116fd7의 push 루프·다른 caller 호출 미비교. `SerialStart`의 기본 `condition`은 여전히 값 유지이며 하네스만 Rust 계산을 쓴다(solver 경로 연결은 04f에서 `CondEntry::ALIGNED`로).
- 후속: **M3-04f**(해 설치, 116fd7 초기화 밖 호출 포함) 또는 23(batch). 재검사 `python3 tools/push_cond_check.py --controls --acceptance && python3 tools/push_cond_rust.py --write && cargo test -p rsolver-lp --test push_init_ext && python3 tools/push_cond_rust.py --compare`.
- 갱신: `docs/11_progress.md`·`12_roadmap_diagram.md`·`14_sprint_plan.md`·`m3/sprint_design.md`·`handoff/M3.md`. 교훈 `docs/lessons/tooling.md`(call 지점 훅 아래 tail call 대상의 return address, 미지값 표시가 확정 출력을 지운 사례), `original.md`(116fd7 동작·버퍼 교환·호출 지점).
