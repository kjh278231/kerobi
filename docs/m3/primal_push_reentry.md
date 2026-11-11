# M3-04c-5 `055f68` 작은 pivot 재진입과 `12dd49` 요청 2

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다. §1–§5는 원본을 실행하기 전에 썼다.

## 1. 범위와 주장

- 작업 종류: 오라클 구축 + 구현. 기준 main `f3ec66a`, branch `work/m3-04c-5`, worktree `../rsolver-m3-04c-5`. 미커밋 선행 변경 없음. 점유·병합은 마스터 세션(`claude-m3-04c5-20261112`)이 맡는다.
- 완료할 동작: feasible ratio `055f68@0xef960`의 작은 pivot 재진입 패스(C `bVar2=false`)와, 그 패스가 후보 없이 또는 상한 I로 끝날 때 `12dd49` 반환 > 16이면 `S+0x294=2`·`S+0x2f8=-1`을 쓰는 요청 2. Rust `feasible_ratio`가 지금 `Err(Reentry)`로 거부하는 조건을 원본대로 복원한다.
- 포함 분기(스프린트 행 완료 조건 포함):
  - 1차 패스 끝 작은 pivot 판정: `0.1 > |pivot|`과 `(tol·max)·coef > |pivot|`. 강화 계수 표 0.1/0.01/1e-3/1e-4/1e-5/1e-6의 경계 1e4·1e7·1e10·1e13, 임계 동률, max의 홀수 꼬리 원소와 음수 alpha.
  - 2차 패스: 새 허용오차 `tol·(max·coef)`(곱 순서는 asm 기준, §2.2), 후보 선택, 유한 상한 flip, 후보 없음과 후보 있는 상한 I에서 `12dd49` 호출·요청 2, 3차 패스 없음.
  - `12dd49`: `S+0x470→+0x18c` 우선, 없으면 `S+0x498→+0x1a4`, 둘 다 없으면 -1. 비교는 signed `> 16`.
  - 1차 패스의 2차 선택 상한 동률(`ratio == bound`)과 후보가 있는데 상한이 I인 반환(04c 생존 변이 두 경계).
  - NaN alpha가 섞인 열: 후보 판정에서 빠지는 방식과 max 계산(`maxsd`)의 NaN 처리. 합성 전용이며 자연 FTRAN 열에서 NaN을 주장하지 않는다.
- 제외:
  - 열길이 가중(`param_7>0`). push 호출점은 `edx=0`이다(04c §2).
  - `054eef`·`056973`의 재진입(Rust는 계속 `Err(Reentry)`). 이번 함수만 바꾼다.
  - push 루프 연결. Rust push 루프가 아직 없으므로 요청 2는 반환값으로만 내보낸다.
  - 강화 계수 경계 10: 후보 pivot은 항상 `> tol`이고 max=10에서 계수가 0.1이든 0.01이든 임계가 `tol` 이하라 결과가 같다(§2.3). 판별 불가로 분류한다.
- 선행: M3-04c(첫 패스 Rust, 합성 29사례 `bench/oracle/m3_primal_push/ratio-{1,2,3}.json`), M3-04c-7(같은 방식의 `056973` 재진입 합성 캡처).

## 2. 원본과 상태 경계

### 2.1 원본·ABI

- Gurobi 13.0.3 `../.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so`, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. `.rodata` 가상주소와 파일 오프셋이 같다(`readelf -S`: `0x2a5d000`/`0x2a5d000`).
- C `../re/decomp/c/000ef960_PRIVATE0000000000055f68.c`(329줄), `../re/decomp/c/001bb6f0_PRIVATE000000000012dd49.c`. asm `objdump -d --start-address=0xef960 --stop-address=0xf0111`.
- ABI는 [04b §3](primal_push_branches.md#3-합성-ratio-오라클-계약과-결과)과 같다. `xmm0`=Harris 여유 h, `xmm1`=pivot 허용 tol, `xmm2`=I, `xmm3`=초기 상한 cap, `rdi`=S, `esi`=m, `edx`=0, `rcx`=열길이, `r8`=header, `r9`=열, stack=basic·폭·표시·후보 scratch·ratio scratch·theta*·pivot*·work*. 재진입 패스도 같은 인자를 쓰며 tol만 `xmm6`에서 바뀐다.
- 시작: `055f68` 직접 호출 직전. 종료: 반환 직후. 호출마다 새 S(0x500바이트)와 factor 객체를 만든다.

### 2.2 asm으로 확인한 재진입 제어 흐름

| 위치 | 내용 | C와 다른 점·비교 방향 |
|---|---|---|
| `0xefc8d–0xefca8` | `dl`(bVar2)=0이면 반환. 아니면 `comisd |pivot|,0.1; jbe 반환` → `0.1 > |pivot|`일 때만 계속 | NaN pivot은 반환 쪽 |
| `0xefcae–0xefd2c`, `0xefea8–0xeff33` | 열 값(sparse는 count개, dense는 m개)의 abs를 둘씩 `maxsd`로, 홀수면 마지막 원소를 한 번 더 `maxsd` | `maxsd`는 `dst > src ? dst : src`라 NaN이 src면 NaN, dst면 src가 남는다 |
| `0xefd31–0xefdb5` | 계수: `10>max`→0.1, `1e4>max`→0.01, `1e7>max`→1e-3, `1e10>max`→1e-4, `1e13>max`→1e-5, 그 밖(NaN 포함)→1e-6. 상수는 `.rodata` `0x2a5dcc0`…`0x2a5dcf8`에서 읽어 확인 | strict `>`. 경계 값 자체는 다음 구간 |
| `0xefdb9–0xefdcc` | `xmm9=(tol·max)·coef`; `comisd |pivot|,xmm9; jbe 반환` → `(tol·max)·coef > |pivot|`일 때만 재진입 | 동률은 반환. NaN은 반환 |
| `0xefdd2–0xefe57` | 계수를 다시 고르고 `xmm0=max·coef`, `xmm6=tol·xmm0`, `dl=0`, `0xef9fb`로 | **새 tol은 `tol·(max·coef)`**. C의 `param_2*dVar14*dVar16`(왼쪽부터)과 반올림이 다를 수 있다 |
| `0xef9fb` | 재진입 시 후보 수·ratio 쓰기 위치·상한(`xmm1=cap`)만 다시 초기화. `ebx`(반환 위치)·theta·pivot은 그대로 | 2차 패스 scan이 후보를 기록할 때마다 `ebx`=그 위치 |
| `0xefb3a–0xefb78`, `0xf00bf–0xf0107` | work 누적은 패스마다 한 번 | 재진입이면 두 번 더한다 |
| `0xefb7d–0xefb89` | `I > cap`이고 `bound >= cap`이면 -1·theta=cap·pivot=0(패스 무관) | |
| `0xefb8f–0xefb9e` | `bound == I`(ucomisd, NaN 제외) 또는 후보 0이면 `0xeff38` | |
| `0xeff4c–0xeff92` | `dl`≠0이면 -1·theta=I·pivot=0. `dl`=0이면 theta·pivot을 쓰지 않고 `12dd49(S)`; `cmp $0x10; jle` 아니면 `S+0x294=2`, `S+0x2f8=-1`. 반환은 `ebx` | 후보 있는 상한 I이면 `ebx`는 2차 패스의 마지막 기록 후보, 후보 0이면 1차 선택 |
| `0xefba4–0xefc89` | 2차 선택 `ratio <= bound`(`jb` 건너뜀), `|a| > best`(`jbe` 건너뜀), 끝에 pivot=`|val[ebx]|` 저장 후 `dl`=0이면 반환 | 3차 패스 없음 |
| `0xefa50`·`0xefaab`, `0xeffdd`·`0xf0039` | 후보 판정 `a > tol`과 `-tol > a`(둘 다 ordered). NaN alpha는 두 판정 모두 거짓이라 후보·dense 시험 수에서 빠진다 | Rust의 `a <= piv_tol`은 NaN에서 반대로 간다(수정 대상) |
| `0x1bb6f0` | `12dd49`: S≠0, `S+0x470`≠0이면 `+0x18c`, 아니면 `S+0x498`≠0이면 `+0x1a4`, 아니면 -1 | int 반환, `jle`는 signed |

### 2.3 판별 불가 경계

- 강화 계수 경계 10: 후보는 `|a| > tol`이고 max<10이면 임계 `(tol·max)·0.1 < tol`, max=10이면 `tol·10·0.01 = 0.1·tol`. `<=`로 바꿔도 임계가 `tol`이라 어느 쪽도 재진입하지 않는다.
- 2차 패스 안의 작은 pivot 판정: `dl=0`이면 바로 반환하므로 판정 자체가 없다. "3차 패스 없음"은 2차 선택 pivot이 `0.1`·새 임계보다 작은 사례로 확인한다.

### 2.4 입력 상태와 보존

- 읽는 상태: 열(희소·dense), basic 값·폭·표시, h·tol·I·cap, `S+0x148`(work 계수 0.25), `S+0x470`·`S+0x498` 포인터와 각 factor의 `+0x18c`·`+0x1a4`.
- 변경 상태: theta*·pivot*·work*, 후보·ratio scratch(비교 안 함), 요청 2일 때만 `S+0x294`·`S+0x2f8`.
- 보존: 열·basic·폭·표시·header·열길이 바이트, S 전체(요청 2의 두 칸 제외), factor 객체 바이트.

## 3. 비교 계약

| 필드·사건 | 의미 | 비교 규칙 | 캡처·재구성 |
|---|---|---|---|
| `chosen`·`row` | 반환 위치(sparse는 희소 위치)·실제 행 | 정수 정확 | ctypes 반환 |
| `theta`·`pivot` | 출력 슬롯. 호출 전 -123.0 | f64 LE hex 비트 비교(±0, I 포함) | 슬롯 값 |
| `work` | 시작 0, 계수 0.25 | 비트 비교. 재진입이면 두 패스 합 | 슬롯 값 |
| `refactor`·`leaving` | `S+0x294`·`S+0x2f8`. 호출 전 0·7 | 정수 정확. 요청 2면 2·-1, 아니면 0·7 | S 버퍼 |
| `workspace_preserved`·`factor_preserved` | 위 두 칸을 뺀 S 바이트, factor 두 객체 바이트 | 참이어야 함 | 호출 전후 bytes |
| `inputs_preserved` | 열·basic·폭·표시·header·열길이 | 참이어야 함 | 호출 전후 bytes |
| 재진입 사건(파생) | 재진입 여부 | 원본은 직접 관측하지 않는다. 2차 패스 결과(위치·theta·work)가 1차 패스로는 나올 수 없는 사례로 판정. 사전 기대 `passes`와 work 단위 수로 대조 | `cases()` 기대 + work |

- 사전 기대값: `cases()`에 사례마다 기대 위치(행)·theta·pivot·요청 여부·패스 수를 고정한다. 값의 최종 기준은 원본 출력이고, 기대와 다르면 캡처 검사가 실패하며 기대를 결과에 맞춰 고치지 않는다(실패로 분류해 기록).
- 재진입 확인: work = 0.25·Σ(패스 단위)이고 단위는 패스마다 정수 합이다. 사전 기대 패스 수로 계산한 work와 원본 work가 비트로 같아야 한다(1패스와 2패스의 work가 다르게 설계).
- NaN: 열 값에만 넣는다(`nan-*` 사례). theta·pivot·work 출력이 NaN이면 실패. ±0, I(1e30)는 비트 그대로 비교한다.
- 시간·work·병렬: 해당 없음(타이머·RNG·스레드 미사용 helper).
- 원본 재현성: 독립 프로세스 3회 캡처, 모든 사건 필드 일치.

## 4. 사례와 원본 자료

모두 **원본 합성** 입력이다(자연 crossover 상태 아님). 기본값 h=1e-6, tol=1e-9, I=1e30. 사례마다 sparse·dense 두 형태.

| 사례군 | 노리는 분기 | 기대 |
|---|---|---|
| `control-*` | 04c 29사례 그대로(같은 S·factor 설정) | 저장 `ratio-1.json` 사건과 9필드 동일(대조군) |
| `reentry-pick` | 1차 pivot 2e-9 < 임계 1e-8 → 2차가 2e-8 행 선택. 2e-8 < 0.1·2차 임계지만 3차 없음 | 행 1, theta≈1000, 요청 없음 |
| `reentry-negmax-pick` | max가 홀수 개 열의 마지막 음수 alpha(폭 I라 후보 아님) | 행 1(0.5), 꼬리를 빼면 1차 반환 |
| `reentry-flip` | 2차 상한 ≥ 유한 cap | -1, theta=cap, pivot 0 |
| `reentry-inf-bound`(갱신 17)·`-keep`(3) | 2차에 후보가 있는데 상한 I | 위치=2차 마지막 후보, theta·pivot=1차 값, 요청은 17만 |
| `reentry-none-*` | 2차 후보 0, factor 변형 9종: 470=17·16·0·-5·2147483647, 498만 17·16, 둘 다 없음, 470=0+498=20 | 1차 선택 유지. 요청은 470=17·2147483647·498만 17 |
| `pivot-at-0.1`·`pivot-below-0.1` | 0.1 경계(임계≈1) | 0.1은 1패스, 직전 값은 재진입+요청 |
| `thr-equal`·`thr-below` | `(tol·max)·coef == |pivot|` 동률 | 동률은 1패스, 직전 값은 재진입+요청 |
| `scale-at-B`·`scale-below-B`(B=1e4·1e7·1e10·1e13) | 계수 표 경계 | max=B는 1패스, 직전 값은 재진입+요청 |
| `mul-order` | 새 tol=`tol·(max·coef)`=1e-8, 왼쪽부터 곱이면 1.0000000000000002e-08. 그 값의 alpha 행 | 2차가 그 행 선택(왼쪽 곱이면 후보 0) |
| `nan-max-last` | 마지막 원소 NaN → max NaN → 계수 1e-6·임계 NaN → 재진입 안 함 | 1패스 |
| `nan-mid` | 가운데 NaN → 다음 원소가 max | 재진입+요청, dense 시험 수에서 NaN 제외 |
| `second-bound-tie` | 1차 패스 2차 선택 `ratio == bound`, 큰 pivot 쪽 | 행 1 |
| `candidate-inf-bound` | 1차 패스 후보 있는 상한 I | -1, theta=I |

- 입력: 새 도구 `tools/primal_push_reentry_oracle.py`의 `cases()`. 04b 도구는 04b·04c 기준 자료가 자기 해시로 고정하므로 고치지 않고, 그 `cases()`를 import해 대조군으로 쓴다.
- 원본 자료: `bench/oracle/m3_reentry/reentry-{1,2,3}.json`(스키마 `M3-04c-5-reentry-v1`), 메타에 ELF·C 두 파일·도구·입력 해시.

## 5. 재실행 명령

worktree 루트에서 실행한다. Gurobi는 `tools/gurun.sh`로 한 프로세스씩.

```bash
D=bench/oracle/m3_reentry
for i in 1 2 3; do tools/gurun.sh ../.venv/bin/python tools/primal_push_reentry_oracle.py --capture $D/reentry-$i.json; done
python3 tools/primal_push_reentry_oracle.py --controls $D/reentry-{1,2,3}.json     # 대조군: 04c 29사례 불변·3회 일치
python3 tools/primal_push_reentry_oracle.py --acceptance $D/reentry-{1,2,3}.json   # 목표: 사전 기대·재진입·요청 2
python3 tools/primal_push_reentry_oracle.py --self-test $D/reentry-{1,2,3}.json    # 변조 거부
python3 tools/primal_push_ratio_rust.py --write && cargo test -p rsolver-lp --test push_ratio && python3 tools/primal_push_ratio_rust.py --compare
python3 tools/primal_push_ratio_rust.py --self-test
python3 tools/primal_push_ratio_rust.py --mutate-rust <새 경로>.json
```

회귀: `cargo test -p rsolver-lp`, 04c·04c-2·04c-7·04c-8 재생 비교(각 기록 §5), `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`.

목표 합격 검사는 다음이면 0이 아닌 값으로 끝나야 한다: 재진입 사례 0, 요청 2 사례 0, 필수 필드 누락, 해시 0, 사건 배열 비움, 3회 미만, 합성 출처가 아닌 기록. 변조 목록은 §6에 실행 결과와 함께 적는다.

## 6. 실행 결과

(실행 후 작성)

## 7. 판정과 인수인계

(실행 후 작성)
