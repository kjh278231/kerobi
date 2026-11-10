# M2-03d-8 검증 계약과 결과: DynamicRows 다이브 행 선택 규칙

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 배경은 [M2-03d-7](mip_blank_rows.md) §7이다. 03d-7은 B&B 노드 LP의 빈 행이 DynamicRows 다이브 행 비활성(`5fb8ec` C:1856 → `10e5fc` → `edd71`)임을 확인했지만, 어떤 행을 고르는지는 C만 읽었다. 03d-7 인수인계의 제안 항목이고, 사용자의 "다음 스프린트 진행" 지시로 착수했다(2026-11-10).

## 1. 범위와 주장

- 작업 종류: 오라클 구축(원본 실행 훅으로 선택 입력과 결과를 캡처)과 그 입력으로 C 규칙을 재구성한 Python 판정. Rust 솔버 코드는 바꾸지 않는다.
- 완료 후 주장할 것: 계약 사례에서 `5fb8ec` 선택 블록(C:1581–1856)이 `10e5fc`에 넘기는 행 목록(개수·순서)이 캡처한 입력과 §2 규칙으로 정확히 재현된다. 규칙의 각 절이 실행에서 결과를 바꾸는지(판별력)를 절별로 보고한다.
- 포함 분기:
  - 선택 블록 관문(C:1581–1610): DynamicRows≠0, 노드 정보 `I+0x2c`=`param_2+0x34`, `I+0x28`=2, `I+0xa0`·`I+8`≠0, `a7f0ad(lp)`=0, `a7f923(lp)`=0, `*(param+0x18)`<2.
  - 모델 행 절: 플래그 배열, `7889a5` 비트, 최대 활동값 ≤ rhs+0.1·FeasTol, '=' 행의 최소 활동값 ≥ rhs−0.1·FeasTol, 그렇지 않은 행의 `66661c` cutoff 검사(DualReductions≠0, cutoff′<1e100, sense≠'=').
  - cut 행 절: 플래그, cut `+0x2c`≠0x14, `+0x40`=0, slack>1e-5.
  - 선택 0행(`10e5fc` 호출 없음).
  - DynamicRows=0 대조군: 공개 파라미터 `GURO_PAR_DYNAMICROWS`=0(범위 −1..1, 기본 −1)으로 선택 블록과 `10e5fc`·`10e662`가 0회인지.
- 제외 범위와 후속:
  - C:526–529 다이브 시작 재활성(`10e662`)의 관문 의미, DynamicRows=−1의 자동 판정(어떤 모델에서 꺼지는지), `edd71`의 빼기 순서와 행 삭제 재번호는 이번 범위 밖이다.
  - Rust 이식은 M2-03d-4(또는 새 하위 번호)다.
  - `66661c` 내부: 호출 단위(행별 반환값)로 Python 포팅을 맞춘다. 맞지 않으면 캡처 반환값을 규칙에 넣지 않고 `66661c`를 미검증으로 분리한다.
- 선행: main `c383966`. 미커밋 선행 변경 없음.
- 계약 변경(탐색 probe 뒤, 고정 캡처 전): §6.1 probe에서 nidda H 설정은 실행 중 DualReductions(env+0x2430)가 0이라 `66661c` 경로가 꺼지고 모든 행이 '<'였다. 그래서 기본 설정의 두 사례(gmu-35-40: `66661c` 1 반환, assign1-5-8: '=' 행 선택)를 §4에 더했다. `7889a5`·cut `+0x40`·cut 종류 0x14는 probe 37모델에서 한 번도 참이 아니었으므로 V에서 C만으로 남을 것으로 예상한다. 검사 목록(§3)의 규칙은 바꾸지 않았고, N·D의 03d-7 대조는 nidda에만 적용한다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 file offset(= objdump 주소, ghidra 주소 −0x100000)이다.
- 원본(C 읽기, asm 대조):

| 위치 | 내용 |
|---|---|
| `5fb8ec` `0x512468` | C:1581 관문. env=r12, I=`*(0x130(%rsp))+0x18`, rbx=`param_2`(진입 rsi, `0x110(%rsp)`에 저장·복원). `511b22`·`511b93`이 이 명령으로 직접 점프하므로 I는 스택 칸으로 읽는다 |
| `0x513078` | 관문 통과 직후. jmp 출처 없음(직전 `je` fallthrough). 8바이트 `mov` |
| `0x513198`–`0x5131bc` | `a7f0ad(node lp)`·`a7f923(node lp)`가 0이 아니거나 `*(r15+0x18)`≥2면 선택 없이 끝난다(`a7f0ad`·`a7f923`은 모델 종류 검사, C) |
| `0x5131f3` | cutoff′ = `5e7204(mip)`(=`*(*(mip+0x620)+0x3220)+0x40`, 없으면 1e100). 1e100 미만이면 `I+0x10`을 뺀다 |
| `0x513273`–`0x513413` | 모델 행 i=0..nrows−1(nrows=`(mip+8)->D+8`). 플래그 `I[0x14][i]`≠0 또는 `7889a5(mip+0x32e8, i)`(배열 `+0x10`의 값 ≥0, i<`+0xc`)이면 건너뛴다. maxact=−`a7cec0(−1, 행)`. rhs+0.1·FeasTol < maxact이면 cutoff 경로. 아니면 '='은 minact=`a7cec0(+1)` < rhs−0.1·FeasTol일 때 cutoff 경로, 그 밖에는 선택. cutoff 경로는 DualReductions(env+0x2430)≠0, cutoff′<1e100, sense≠'=', `66661c`≠0일 때만 선택 |
| `0x5133df` | `66661c(rhs, cutoff′, len, ind, val, vtype=mip+0x650, lb=I+0x80, ub=I+0x88, rc=(I+8)->+0x28, work)`. 유일한 호출자 |
| `0x51346b`–`0x513921` | cut j=0..ncuts−1(`mip+0x3240` 객체 `+0x60` 개수, `+0x78` 포인터 배열). 플래그 `I[0x14][nrows+j]`=0, cut `+0x2c`≠0x14, `+0x40`=0, slack `(I+8)->+0x20`[nrows+j] > 1e-5이면 nrows+j를 선택 |
| `0x513951` | 선택 끝. `0x30(%rsp)` 개수, `0x48(%rsp)` 목록. 개수>0이면 `0x513994` `10e5fc(lp, 개수, 목록)` |

  `a7cec0(s, n, ind, val, lb, ub, work)`는 s·a≤0이면 ub, 아니면 lb를 곱해 차례로 더하고(곱 `s·a`를 먼저 계산, FMA 없음) 무한 bound(±1e30)를 만나면 −1e100을 돌려준다. 상수 1e100·0.1·−1·1·1e-5는 rodata에서 확인했다.
- 시작·끝 경계: top-level·중첩 MIP의 `0x513078`부터 `0x513951`까지 한 선택 사건이다. 계측은 메모리 읽기만 한다.
- 입력(사건마다 `0x513951`에서 읽는다): 스택 칸 `0x58·0x60·0x88·0xb8·0x90`이 I에서 유도한 포인터와 같은지 대조하고, 모델 CSR(`D+0x118/0x120/0x130/0x138`), rhs(`+0x380`), sense(`+0x388`), vtype은 내용 해시로 식별해 처음 볼 때만 저장한다. 노드 lb·ub는 같은 모델의 직전 사건과 다른 칸만 저장하고 전체 해시를 남긴다. 플래그(0 아닌 칸), `7889a5` 배열(0 이상인 칸), cut 속성·slack, rc(cutoff 경로가 가능할 때), FeasTol·DualReductions·DynamicRows·cutoff′ 비트를 저장한다. `0x48`·`0x30`의 목록과 `66661c` 호출별 (행, 반환값)도 저장한다.
- 관문 기록: `0x512468`을 지날 때마다 조건 값을 읽어, 통과 예측이면 사건(노드, `I+0x2c`, `param_2+0x34`)을 남기고 실패는 첫 실패 조건별로 센다. `5fb8ec` 진입(`0x5101d0`)에서 `param_2`를 저장해 rbx와 같은지 확인한다.
- 설정: 03d-7과 같다. `{"Threads":1,"Seed":1,"LogToConsole":0,"NodeLimit":32898,"Presolve":0,"ScaleFlag":0,"Quad":0,"NumericFocus":0,"MarkowitzTol":0.0001,"Heuristics":0,"RINS":0}`. 대조군은 여기에 `"GURO_PAR_DYNAMICROWS":0`을 더한다.
- 비간섭: 새 도구는 03d-7 `tools/mip_blank_rows.js`의 훅을 그대로 포함한다. 03d-7 사건 종류만 골라낸 목록이 03d-7 캡처와 바이트 단위로 같아야 한다. 03d-7 도구 파일은 고치지 않는다.

## 3. 비교 계약

`tools/mip_dive_rows_check.py`가 판정한다. 하나라도 실패하면 종료코드 1이다.

| 검사 | 규칙 |
|---|---|
| M 입력 | 각 회차가 정상 종료했고(nidda `DONE 8 32898`, gmu·assign `DONE 8 3000`, 오류 없음) 바이너리·모델·도구 해시, 파라미터가 맞다. manifest 파일 해시가 같다 |
| B 재현 | 사례마다 독립 3회의 사건 전체가 정확히 같다 |
| N 비간섭 | nidda 3회 각각의 03d-7 사건 종류 투영이 03d-7 캡처(`bench/oracle/mip_blank_rows/nidda_bnb_H_s1_1.json`)의 사건 목록과 같다 |
| G 관문 | 선택 사건마다 직전 관문 기록이 통과 예측이고 노드가 같다. 통과 예측 수 = `0x513078` 수. `0x513078` 수와 `0x513951` 수의 차이는 `a7f0ad`·`a7f923`·`+0x18` 절이 막은 횟수로 보고한다. rbx=`param_2`. 스택 포인터 칸이 I 유도 포인터와 같다 |
| D 결과 | top-level 선택 사건의 목록이 같은 호출의 `10e5fc` 인자(`b:op` deact의 rows, 반환 `5fb8ec+0x37c9`)와 같다(모든 사례). 목록이 비면 그 선택 뒤 `10e5fc` 호출이 없다. nidda top-level deact 수는 03d-7 통계와 같다(재시작 전 10, 뒤 1135) |
| A `a7cec0`·`66661c` | Python 포팅으로 cutoff 경로에 들어가는 행 집합이 캡처한 `66661c` 호출 행 집합과 순서까지 같고, 호출별 반환값이 같다 |
| R 규칙 | 모든 선택 사건에서 §2 규칙의 Python 재구성 목록이 캡처 목록과 개수·순서까지 정확히 같다 |
| C 대조군 | DynamicRows=0 3회: env+0x2b48=0, 모든 관문 기록의 첫 실패 절이 DynamicRows, 선택 사건 0, `10e5fc` 0회, X 목록 생성 0회. `10e662`는 재시작 경로 `6a6563+0x211`(C:123–124, DynamicRows 관문 없음)에서 X 없이 부른 호출만 허용한다(처음 계약은 `10e662` 0회였다. 계약 변경 §6.3). 정상 종료 |
| V 판별력 | 규칙 절을 하나씩 바꾼 변종(모델 행·cut 플래그 무시, `7889a5` 무시, 0.1 계수 0, maxact 비교 방향, '=' 반대쪽 생략, DualReductions 무시, cutoff′<1e100 무시, `66661c` 항상 0·1, cut 0x14 무시, `+0x40` 무시, slack 1e-5 → 0, 순서 cut 먼저 등)마다 R(목록)과 A(`66661c` 호출 행·반환값)가 실패하는지 본다. 실패하지 않은 절은 "실행에서 결과를 바꾸지 않음 → C만"으로 보고하며 이 경우도 판정 실패로 치지 않는다. 절별 실행 횟수(플래그≠0, `7889a5` 참, '=' 행, cutoff 경로·`66661c` 1 반환, cut 0x14, `+0x40`≠0, slack 경계)를 함께 낸다 |

- 정수·목록은 순서까지 정확 비교, 부동소수는 u64 비트로 저장하고 Python에서 같은 연산 순서로 계산한 값을 그대로 비교한다(허용오차 없음).
- 시간·work: Threads 1, NodeLimit만 쓴다. 타이머 분기는 없다.
- 자기검사(`--self-test`): 바이너리·도구 해시, 파라미터, 회차 차이, 빈 사건, 비간섭 투영, 관문 노드, 선택 목록 한 행, 목록 순서, 입력 bound 한 칸, rhs 한 칸, slack, cut 종류, `66661c` 반환값, 대조군 선택 사건 추가 변조를 모두 거부해야 한다.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 | 원본 증거 |
|---|---|---|---|---|
| nidda H seed 1, NodeLimit 32898 | 원본 실제 실행(읽기 전용 훅) | 03d-7의 deact 전부(재시작 전·후, 중첩 MIP 포함) | `neos-3754480-nidda.mps.gz`(03d-6과 같은 해시) | `bench/oracle/mip_dive_rows/nidda_H_s1_{1,2,3}.json.gz` |
| gmu-35-40 기본 설정(Threads 1, Seed 0), NodeLimit 3000 | 원본 실제 실행 | '=' 없음, DualReductions 1, `66661c` 호출·1 반환, 중첩 MIP(깊이 2·3) | `gmu-35-40.mps.gz` | `bench/oracle/mip_dive_rows/gmu-35-40_default_{1,2,3}.json.gz` |
| assign1-5-8 기본 설정, NodeLimit 3000 | 원본 실제 실행 | '=' 행의 최소 활동값 절로 선택, '=' cutoff 경로 건너뜀 | `assign1-5-8.mps.gz` | `bench/oracle/mip_dive_rows/assign1-5-8_default_{1,2,3}.json.gz` |
| nidda H 설정 + DynamicRows=0 | 원본 실제 실행(공개 숨은 파라미터 `GURO_PAR_DYNAMICROWS`) | 관문 첫 절 거짓 | 같음 | `bench/oracle/mip_dive_rows/nidda_H_s1_dr0_{1,2,3}.json.gz` |

- 적용하지 않은 범주: 허용오차 경계는 실제 실행에서 나온 만큼만 본다(합성 없음). 실행에서 결과를 바꾸지 않는 절은 V에서 C만으로 표시한다.

## 5. 재실행 명령

- 원본 캡처(작업 worktree 루트, Gurobi 한 프로세스씩): `tools/hrun.sh --timeout 580000 -- 'tools/mip_dive_rows_runs.sh bench/oracle/mip_dive_rows'`.
- 판정: `python3 tools/mip_dive_rows_check.py --self-test`. manifest 재작성은 `--write-manifest`.
- 탐색 sweep(판정 아님, 1회씩): `--sweep DIR`가 DIR의 모든 캡처에 R·A만 적용해 표로 낸다. probe 명령은 §6.1에 적는다.
- 관련 회귀: `python3 tools/mip_blank_rows_check.py --self-test`(03d-7 자료 불변).

## 6. 실행 결과

### 6.1 탐색 probe(계약 확정 전, 판정 근거 아님)

- smoke: nidda H, NodeLimit 3000, 1회. 훅이 모두 실행됐고 스택 칸 대조(`bad_slots`)가 비었다. 실행 중 env+0x2430(DualReductions, `ac81d7` C:146에서 이름 확인)이 0이고 모든 행이 '<'이었다. 그래서 nidda H에서는 `66661c` 경로와 '=' 절이 실행되지 않는다.
- sweep: 커밋 `df81c27`의 도구(sha256 앞 16자리 `b2830515833fedec`)로 `../re/bench/data/miplib`의 38모델을 `{"NodeLimit":3000,"TimeLimit":30}`(Threads 1, Seed 0)로 1회씩 돌렸다. 명령은 `for f in ../re/bench/data/miplib/*.mps.gz; do ./tools/gurun.sh timeout 120s node tools/mip_dive_rows.js $f OUT/$(basename $f .mps.gz).json.gz '{"NodeLimit":3000,"TimeLimit":30}' 0; done`이고 판정은 `python3 tools/mip_dive_rows_check.py --sweep OUT`이다. 결과 파일은 저장소에 넣지 않았다(15MB, 1회 실행, TimeLimit 사용).
  - 36모델에서 선택이 일어났다(enlight_hard·sp150x300d는 0회). 선택 8,365건(행 629,391개)과 `66661c` 호출 661,664건(1 반환 1,593건)이 R·A로 모두 정확히 맞았다(`SWEEP MATCH`).
  - 절 실행 횟수: 플래그≠0 모델 행 511,491·cut 173,049, cutoff 경로 736,649, '=' cutoff 경로 417, maxact 절을 통과한 '=' 행 17, maxact가 (rhs, rhs+0.1·tol]에 든 행 2,825, slack∈(0, 1e-5]인 cut 5,629. `7889a5` 참·cut `+0x40`≠0·cut 종류 0x14는 0회다.
  - 이 결과로 기본 설정 두 사례를 §4에 더했다(§1 계약 변경).

### 6.2 고정 캡처

- 실행 도구는 manifest의 `tools` 해시로 고정한 현재 파일이다. 위치는 worktree `../rsolver-m2-03d-8`이다. 첫 고정 캡처는 rc를 DualReductions≠0일 때만 저장해서 V의 `no_dual` 변종을 nidda에서 평가할 수 없었다. 도구가 rc를 항상 저장하도록 고치고 12회를 모두 다시 돌렸다(판정 전, 판정 규칙 변경 없음).
- 네 사례 모두 3회가 바이트 단위로 같다(gz 파일 SHA-256 동일). nidda·대조군은 각 회차 `DONE 8 32898`, gmu·assign은 `DONE 8 3000`이고 오류는 없다.
- 판정 출력(`--self-test`, 종료 0):

```text
M: 4 cases x 3 runs, binary/model/tool/params/output
B: 3 runs identical: nidda_H_s1 7344, nidda_H_s1_dr0 9, gmu-35-40_default 1905, assign1-5-8_default 533
N: nidda b:* projection = M2-03d-7 capture (4700 events, 3 runs)
G: every selection follows its gate (rbx = param_2, stack slots = I/mip/D pointers, cutoff): nidda_H_s1 1320 gates, blocked 0, gmu-35-40_default 689 gates, blocked 0, assign1-5-8_default 84 gates, blocked 0
D: top-level selection list = 10e5fc rows (5fb8ec+0x37c9): nidda_H_s1 1145, gmu-35-40_default 74, assign1-5-8_default 64
A/R: 2093 selections, 172135 rows, 26278 66661c calls reproduced exactly (list order, 66661c row and result)
C: DynamicRows=0 x3: env+0x2b48 = 0, gate first clause false at every pass, 0 selections, 0 deact, 1 restart react_all (6a6563+0x211, no X)
clause counts: {"66661c": 26278, "66661c_1": 328, "cut_flag": 38671, "cutoff_path": 387462, "cutoff_path_eq": 73, "eq_redundant_side": 11, "maxact_in_tol": 19507, "model_flag": 140906, "slack_le_1e-5": 928}
```

- V(절 변종, 첫 차이). 처음 판정 출력은 목록(R)만 비교해서 DualReductions·'=' cutoff 제외 절을 생존으로 보고했다. 독립 검토 뒤 `66661c` 호출 집합(A)도 비교하도록 고쳤다.

| 변종 | 결과 | 첫 차이·이유 |
|---|---|---|
| 모델 행 플래그 무시 | 거부(R·A) | nidda call 808: 행 235 추가 |
| 0.1·FeasTol → 0 | 거부(R·A) | nidda call 186: 238·388 누락 |
| maxact 비교를 rhs−t로 | 거부(R·A) | nidda call 74 |
| maxact 절 제거 | 거부(R·A) | nidda call 11(루트): 306행 추가 |
| '=' 행도 `66661c` 호출 | 거부(A) | assign call 76: 호출 72 vs 71(반환은 0이라 목록은 같음) |
| DualReductions 무시 | 거부(A) | nidda call 74: 호출 302 vs 0(nidda 전체 360,492회 모두 0 반환) |
| `66661c` 항상 0 / 항상 1 | 거부(R) | gmu call 117 / call 72 |
| `66661c` 정수 반올림 제거 / 폭 상한 제거 | 거부(R·A) | gmu call 117 |
| slack 임계 1e-5 → 0 | 거부(R) | nidda call 2173: 행 435 추가 |
| cut을 모델 행보다 먼저 | 거부(R) | nidda call 74(순서) |
| cut 플래그 무시 | 생존 | 플래그≠0 cut 38,671개가 모두 다른 절(slack 등)로도 빠짐 |
| `7889a5` 무시 | 생존 | `7889a5` 객체가 모든 사건에서 null |
| '=' 최소 활동값 절 제거 | 생존 | maxact 절을 통과한 '=' 11행이 모두 최소 활동값 절도 통과 |
| cutoff′<1e100 무시 | 생존 | cutoff′≥1e100인 사건에서 cutoff 경로 행이 없거나 DualReductions 0 |
| cut 0x14 무시 / `+0x40` 무시 | 생존 | 해당 cut 0개 |

- 자기검사 변조 21종을 모두 거부했다. 대상은 바이너리·도구·모델 해시, 파라미터, 회차 차이, 빈 사건, 03d-7 투영, gmu `10e5fc` 인자 순서(D만 비교), 관문 노드, 선택을 blocked로 대체, 선택의 DynamicRows 0, cutoff 칸, 목록 한 행·순서(순서는 중첩 MIP 사건이라 D가 아닌 R이 거부), 선택된 행의 bound·rhs, 선택된 cut의 slack을 1e-5로, cut 종류 0x14, `66661c` 반환값, 대조군 관문 사건·op 지점이다. bound·rhs·slack 변조는 처음에 결과와 무관한 칸을 바꿔 통과했다. 선택된 행을 직접 노리도록 고친 뒤 거부됐다. 디스크의 캡처 1바이트 변조 시 종료 1이다.
- 관련 회귀: `python3 tools/mip_blank_rows_check.py --self-test` 종료 0(03d-7 자료 불변).

### 6.3 관찰과 계약 변경

- 관문 노드 조건(`I+0x2c`=`param_2+0x34`)이 참인 top-level 횟수는 C:526 `10e662`(`5fb8ec+0x79e`) 호출 수와 맞는다. nidda 1146=1146, gmu 74=74이고, assign은 통과 66 + `I+0x28`≠2 실패 3 = 69다. 두 지점이 같은 노드 조건을 쓰고 선택은 그중 상태 2인 노드에서만 일어난다. 횟수만 맞췄고 노드별 대조는 하지 않았다.
- 선택 2,093건 중 빈 목록은 nidda 2, gmu 98, assign 4건이다. 빈 목록 뒤에는 `10e5fc`가 없다(D). `a7f0ad`·`a7f923`·`+0x18` 절로 막힌 선택은 0건이고, 관문의 `I+0xa0`·`I+8` null 절이 첫 실패인 경우도 0건이다(`gate_counts`에는 pass·depth·state만 있다).
- 모델 행 자료 `(mip+8)->D`는 노드 LP가 아니라 cut이 없는 MIP 모델 LP다(`mlp_is_node_lp` 거짓, nidda 402행, cut은 최대 177개). cut 행 번호는 nrows+j다.
- 계약 변경(결과 확인 뒤): 대조군에서 `10e662`가 1회 나왔다. 반환 주소 `0x5807b1`은 재시작 `6a6563+0x211`이고, C:123–124는 DynamicRows를 보지 않고 부른다. X가 없어 목록은 바뀌지 않는다. 처음 계약의 "`10e662` 0회"는 이 C를 읽지 않은 기대였다. 판정을 "`5fb8ec` 지점 0회, 재시작 지점은 X 없이만 허용"으로 고쳤고 다른 규칙은 바꾸지 않았다.

## 7. 판정과 인수인계

- 작업 수행 상태: **완료(오라클 구축)**.
- 동등성: 해당 없음(Rust 미변경). Python 재구성은 원본 대조 도구이고 Rust 구현이 아니다.
- 통과 범위: 네 사례(실제 실행 3회씩)에서 선택 블록의 관문, 입력, 결과 목록(개수·순서), `66661c` 호출 행과 반환값이 정확히 재현됐다. 실행으로 판별된 절은 모델 행 플래그, maxact ≤ rhs+0.1·FeasTol, DualReductions, '=' 행의 cutoff 경로 제외, `66661c`(정수 반올림·폭 상한 포함), cut slack>1e-5, 모델 행→cut 순서다. 탐색 sweep 38모델(1회씩)도 모두 맞았다.
- 비간섭 근거: nidda는 03d-7 사건 투영이 바이트 단위로 같다(N). gmu·assign은 계측 없는 기준 실행이 없어 읽기 전용 훅과 3회 일치만이 근거다.
- 남은 한계:
  - C만(실행에서 판별되지 않음): cut 플래그 절, `7889a5` 비트, '=' 행 최소 활동값 절, cutoff′<1e100 절, cut 종류 0x14·`+0x40` 절, 관문의 `I+0xa0`·`I+8` null 절, `a7f0ad`·`a7f923`·`+0x18` 절.
  - 미확인: `param_2+0x34`의 의미(다이브 시작 깊이로 보이지만 쓰는 쪽을 읽지 않았다), C:526 재활성의 노드별 대조, DynamicRows=−1의 자동 판정, `edd71` 빼기 순서와 행 삭제 재번호, nidda H 설정에서 DualReductions가 0인 이유.
- M2-03d-4에 주는 뜻: 다이브 행 선택은 노드 bound, 행 플래그, cut 속성·slack, rc, cutoff로 정해진다. Python 재구성이 원본 입력에서 같은 목록을 낸다. Rust 실제 경로에는 플래그 배열(`I+0xa0`), cut 속성, 노드 LP 해의 slack·rc가 필요하다.
- 후속(제안, 번호 미배정): Rust 이식. `d:sel` 입력을 그대로 재생하면 L1 수준 대조가 가능하다. M2-03d-4의 선행이다.
- 작업 branch: `work/m2-03d-8`. 검사 통과 뒤 main에 `--no-ff`로 병합한다.
- 교훈: [원본 사실](../lessons/original.md)(선택 규칙, nidda H의 DualReductions 0, 재시작 경로 `10e662`), [도구](../lessons/tooling.md)(변조는 결과를 바꾸는 칸을 노릴 것)에 추가했다.
