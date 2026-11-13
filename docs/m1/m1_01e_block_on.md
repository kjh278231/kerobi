# M1-01e M1 재분해 2×2 블록 단계 켜기 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 기준 main `1b91bd5`, 미커밋 선행 변경 없음. 작업자는 이 세션이다("다음 스프린트 진행해 너가 작업자야" 지시. M2-06h-6은 다른 세션이 점유 중이라 M1 인수인계의 "다음"을 맡았다). branch `work/m1-01e`, worktree `../rsolver-m1-01e`.

## 1. 범위와 주장

- 작업 종류: 구현(원본 상태 캡처 오라클 포함).
- 완료할 동작: M1 simplex(`simplex.rs` `refactor`)가 `1da0b2` 2×2 블록 단계를 원본처럼 켠다. S(simplex 구조체)의 S+0x2c0(재분해 수)·S+0x4e0(블록 단계 실행 수)·S+0x4e4(LU2X2BLOCK 기본 → 0, 20단계 이상 피벗한 실행 수, S+0x2c0 > 20에서 −1 또는 2e9로 고정)의 수명을 원본과 같게 둔다. 블록 단계 본체는 M3-04c-21h `Lu::factor_block`을 그대로 쓴다.
- 둘째 단계(같은 작업, 별도 커밋·별도 회귀): M1 t(LU+0x100) = s3 정정. M1 `Lu::finish`는 t를 "앞쪽 빈 U 열 수"로 세는데 원본은 `1da0b2`가 둔 열 singleton 단계 3의 끝(s3)이다([교훈](../lessons/original.md), M3-04c-21g). 이 단계의 회귀 규칙(§5)을 통과하지 못하면 블록 단계만 병합하고 t = s3는 **M1-01f**로 분리한다.
- 포함 분기: 블록 단계 실행(활성 ≥ 50)과 건너뜀, S+0x4e0 증가, 자연 사례에 있으면 S+0x4e4 증가(블록 피벗 ≥ 20)·S+0x2c0 > 20의 −1/2e9 고정·2e9 뒤 활성 < 50 실행, 특이 기저 재시도(`05b257` `LAB_001f3392` 루프는 S+0x2c0을 다시 올리지 않음).
- 제외: 블록 단계 내부의 원본 비교(M3-04c-21h에서 push 19재분해로 검증, 이번에는 factor 결과를 기존 M1 검사기로 간접 확인), 블록 단계 풀 부족 −1, 반복 수 차이의 다른 원인(M1-02w·02y-2·02t·02u).
- 선행: M3-04c-21h(`Lu::factor_block`, `BlockState`), M1-02z(원인 특정, `tools/m1_urow_cmp.py`, `RSOLVER_UROW_PROBE`).

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- `036ade@0xd4190`의 호출자는 `05b257@0xf15c0` 하나다(`tools/dc.py 036ade`). 호출 인자: param_1 ZERO·param_2 Markowitz tol은 xmm, S = args[0](= `05b257` param_1), nb = args[2], n = args[3], 기저 head = args[8]. LU = S+0x470(`05b257` C:113), `1da0b2`의 param_21 = LU+0x20이라 C:1514–1534가 LU+0xd8·0xdc·0x100(+0xe0)·0xe4(+0xc4)를 쓴다.
- 상태 갱신 위치: `02f7e1` C:270–281(LU2X2BLOCK → S+0x4e4: 0→−1, 1→2e9, 기본 0), `05b257` C:1399(S+0x2c0 +1, `LAB_001f3392` 재시도 루프 밖), `1da0b2` C:681–684(실행 조건)·C:1494–1510(갱신). 다른 쓰기 위치는 캡처의 연쇄 검사로 확인한다(§3).
- 시작 경계: `036ade` 진입(S+0x2c0/0x4e0/0x4e4, head). 종료 경계: `036ade` 반환(S+0x4e0/0x4e4, 반환값, LU+0xd8/0xdc/0xe0/0xe4/0x100).
- 설정: Method=1, Presolve=0, Threads=1, 면허 크기 Netlib. 캡처 도구 `tools/m1_block_state.js`(읽기 전용, 피벗 번호 k는 M1-02q와 같은 `8b473`@0xc58d5).

## 3. 비교 계약

| 필드 | 규칙 | 방법 |
|---|---|---|
| 원본 재현성 | 정식 사례 × 독립 3회의 사건 목록 JSON 정확 일치 | `tools/m1_block_check.py --repro` |
| 원본 연쇄(다른 writer 없음) | 같은 S의 연속 호출에서 enter[i+1] = (enter[i].x2c0 + 1, leave[i]) | 같은 검사기 |
| Rust 블록 상태 | 원본과 Rust의 재분해를 순서대로 맞추되 **head가 같은 재분해까지** 진입 (x2c0, x4e0, x4e4)·반환 (x4e0, x4e4) 정확 일치. 첫 head 차이 위치를 보고한다 | Rust 탐침 `RSOLVER_BLOCK_PROBE=<file>`, 같은 검사기 |
| LU 경계(둘째 단계) | 같은 범위에서 LU+0xd8·0xe4·0x100(t) 정확 일치 | 같음 |
| factor 결과 | bnl1 e_r 126·129·130 행 U·대각 비트 일치(M1-02y 원본 덤프) | `tools/m1_urow_cmp.py` |

- 판별력: Rust 변이(임시 사본) "재분해마다 상태 `{0,0,0}` 재설정", "x2c0 미증가", "x2c0 시작값 1", "블록 끔(OFF)"을 반드시 거부해야 한다. 검사기 변조(바이너리 해시 0, 사건 비우기, 연쇄 필드 변경)도 거부해야 한다.

## 4. 사례와 원본 자료

- 탐색: 면허 크기 Netlib 64개를 1회씩 캡처해 분기(블록 실행, x4e4 증가, x2c0 > 20 결정 −1/2e9, 2e9 뒤 실행, 특이 재시도 rc ≠ 0)를 덮는 모델을 고른다(`bench/oracle/m1_01e/explore/`).
- 정식: 고른 모델 × 3회(`bench/oracle/m1_01e/formal/`). 결과는 §6에 적는다.

탐색 결과(64모델 × 1회, `tools/m1_block_state.js` SHA-256 `f97e4b40…`, 정식과 같은 판): 모든 실행에서 `036ade` 호출자 반환 주소는 0xf34f2 하나, S는 실행당 하나, 첫 진입 (1,0,0), 연쇄 위반 0. 블록 단계는 35모델에서 돈다. 자연 분기: x4e4 증가(12모델), x2c0 > 20 결정 −1(25fv47·degen3·etamacro·grow15·grow22·perold·scagr25·scsd6), 2e9(bnl1·maros, 이후 매 재분해 실행), 반환 −1 재호출(grow15·grow22, S+0x2c0 그대로). 반환 > 0(특이)은 0회.

| 정식 사례 | 재분해 | 노리는 분기 |
|---|---|---|
| bnl1 | 30 | 2e9 고정·M1-02z U 위치 |
| maros | 37 | x4e4 증가 14회·2e9 고정 |
| 25fv47 | 50 | x4e4 증가·−1 고정 |
| grow15, grow22 | 32, 34 | −1 고정·풀 부족 재호출 |
| scfxm2 | 12 | x4e4 증가 7회 |
| perold | 27 | x4e4 증가·−1 고정 |
| stair, degen2 | 13, 16 | 블록 실행(증가 없음), 기존 M1 검사기 모델 |
| kb2 | 6 | 블록 미실행(활성 < 50) 대조군 |

## 5. 회귀 규칙(착수 때 고정)

- 코드 수정 전 기준선 `bench/oracle/m1_01e/regress.sh base`를 먼저 돌린다(cargo lp·workspace·clippy, pytest, M1 검사기, pathcmp, lp_check, urow 3건).
- `lp_check` Method 1·0·−1 정답 수가 줄지 않는다. 줄면 병합하지 않고 사용자에게 올린다.
- `pathcmp` 완전 일치 수(현재 27/64)가 줄지 않는다. 바뀐 모델 목록과 첫 차이 위치 전후를 남기고, 첫 차이가 앞당겨진 모델은 원인을 적는다.
- `m1_lt_check.py`의 bnl1 고정 7호출(`KNOWN_IDX`)은 블록 단계로 설명되므로 이 작업에서 고정 목록을 고친다. 다른 M1 검사기는 base와 같은 결과여야 하며 바뀌면 원인을 적는다.
- 블록 켜기와 t = s3는 커밋과 회귀를 나눈다.


## 6. 실행 결과

- 환경: WSL2 x86-64 Linux 6.6.87.2, Python 3.13 venv, Node 22, Rust stable. 원본 실행 94회(afiro 시험 1 + 탐색 63(afiro는 시험분 재사용) + 정식 30), Gurobi 한 프로세스씩(`tools/gurun.sh`).
- **원본 재현성**(`python3 tools/m1_block_check.py --repro --self-test`): 10모델 × 3회 사건 목록·출력 정확 일치, 연쇄 검사 통과. 변조 9종 거부(바이너리 해시, 빈 사건, x2c0/x4e0/x4e4 변경, 시작값, 두 번째 S, 실행 실패, 회차 차이).
- **구현**: `simplex.rs` `Engine.block`(시작 {0,0,0}), `refactor()`가 루프 밖에서 x2c0 +1 하고 `Lu::factor_block(…, &mut self.block)`. 2e9ae 무척도 재확인 엔진(`simplex.rs` `e2`, `driver` `check`)은 같은 S이므로 상태를 넘겨받는다. 탐침 `RSOLVER_BLOCK_PROBE`, `Lu::block_fields`. 둘째 단계: `Lu::finish`의 t = `s3`(이전: 앞쪽 빈 U 열 수).
- **주입 재생**(`--replay`, `crates/lp/tests/m1_block_replay.rs`): 원본 재분해마다 원본 head·진입 상태로 Rust `factor_block`을 불러 반환 상태와 LU+0xd8·0xe4를 비교. **255/255 일치**(−1 고정 4건, 2e9 고정 2건, x4e4 증가 43건, 블록 실행 152건, 재호출 2건 포함). t(LU+0x100)는 1단계에서 grow15·grow22·kb2의 6재분해가 달랐고(Rust가 큼, 예: grow22 k 1089 원본 3 대 Rust 9), t = s3 뒤 `--replay --lu` 255/255.
- **자연 궤적 비교**(`--rust --lu`): head가 같은 범위(모델당 5–11재분해)에서 진입·반환 상태와 LU+0xd8·0xe4·0x100 전부 일치. 범위 끝은 기존 궤적 차이(예: bnl1 257에서 기저 열 하나 다름)이고 블록 상태 차이가 아니다. kb2만 마지막 재분해의 head 순서 차이(집합 같음)에서 끝난다. 검사기는 모델별 최소 비교 수(`NEED`, 위 값)를 요구해 탐침·빌드가 없으면 실패한다(PYTHONPATH를 탐침 없는 빌드로 돌려 거부 확인).
- **Rust 변이**(`bench/oracle/m1_01e/mutate.sh`, [결과](../../bench/oracle/m1_01e/mutations.txt)): 6/6 거부 — 재분해마다 {0,0,0}(25fv47 factor 5), x2c0 미증가, x2c0 시작 1, 블록 끔(`factor_tol`), x2c0 > 21 결정, −1/2e9 결정 반전(주입 재생에서 25fv47 k 1259).
- **회귀**(`bench/oracle/m1_01e/regress.sh`, 로그 `base/`·`stage1/`·`stage2/`):

| 항목 | base | 1단계(블록) | 2단계(+ t = s3) |
|---|---|---|---|
| cargo lp·workspace·clippy·pytest(657) | 통과 | 통과 | 통과 |
| M1 검사기 16종 | 통과 | btran·lt 고정 차이 "not seen" 실패 → 고정 목록 정리 뒤 통과 | 통과 |
| bnl1 urow 126·129·130 | 차이(대각 20·행 32) | **일치** | 일치 |
| pathcmp 완전 일치 / 중앙 첫 차이 | 27/64, 0.752 | 27/64, **0.886** | 27/64, 0.886 |
| lp_check 정답(M1·M0·M−1) | 90·90·90 | 90·90·90 | 90·90·90 |
| IterCount 일치(Method 1) | 32/64 | **34/64** | 34/64 |

- pathcmp 바뀐 모델 13개, 첫 차이가 앞당겨진 모델 없음(규칙 충족). 늦어진 모델: 25fv47 306→446, bnl1 220→236, boeing1 368→420, forplan 128→138, maros 475→852, perold 165→365, scfxm2 366→456, scfxm3 488→821, scrs8 107→478. grow15·grow22·sc205·scagr25는 첫 차이 그대로이고 피벗 수만 바뀜(grow15 1066→1170, grow22 1946→2046으로 원본과 더 멀어짐). 2단계는 pathcmp 출력 변화 0, lp_check는 d6cube Method 0(면허 밖, 정답 유지) 한 행만 바뀜.
- 고정 차이 정리: `m1_btran_check.py` `KNOWN_D`에서 forplan 82(이제 150/150 끝까지)·bnl1 174(이제 299/1050, r 차이에서 끝) 제거, stair 185는 남음(M1-02w). `m1_lt_check.py` `KNOWN_IDX` bnl1 7호출 제거와 그 고정 전용 변조 1종 삭제.
- **M3 push 저장 기준 회귀**(t = s3가 `factor_dense`와 공유하는 `Lu::finish`를 바꿈, `bench/oracle/m1_01e/push_regress.sh`, [로그](../../bench/oracle/m1_01e/push_head/push.txt)): 19명령 종료 0. `--write` 입력 불변, 21f 207/207, 21g 211/213(기존 예상 실패 2 그대로), 21h `--block` 219/219, 21j mode70 31/31, 21i dense, 04f-1 설치 59/59, push loop `--lu`·`--init` 6사례 일치.
- **debug_assert 빌드**(`CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS=true`, FT 갱신의 접두 목록·t 불변식 포함): `pathcmp.py all` 64모델 panic 0, 출력이 일반 빌드와 같다([로그](../../bench/oracle/m1_01e/stage2/pathcmp_debug_assert.txt)).
- **재분해 수 대조**(pathcmp 완전 일치 27모델, 탐색 캡처 대 Rust 탐침, head 집합·상태): 7모델(beaconfd·etamacro·grow7·kb2·sc105·scsd6·sctap2) 전부 일치. 19모델은 Rust가 마지막에 재분해를 한 번 더 한다(원본 수 + 1, 그 앞은 일치). adlittle은 Rust가 iter 18에서 재분해를 두 번 해(원본 1번) 이후 x2c0이 1 앞선다. 둘 다 블록 단계와 무관한 기존 재분해 일정 차이이며, 정식 10모델의 비교 범위 안에는 없다. 그래서 2e9ae 무척도 재확인 엔진의 상태 인계(`e2.block = e.block`)는 원본 대조가 없다(미검증).
- **특이·재시도 경로 C**: `05b257`은 특이 기저 수리(C:1520–1598, `local_120` ≠ 0)와 내부 Markowitz tol 재시도(C:1955–1963)에서 `LAB_001f3392`(C:1427)로 돌아가므로 S+0x2c0을 다시 올리지 않는다. Rust `refactor()`의 루프 밖 증가와 같다. 반환 > 0은 C:1468에서 오류 출구(LAB_001f377d)다. Rust의 피벗 불일치 tol 상향(`simplex.rs` 0.5 뒤 `reinvert`)은 새 `05b257` 호출이라 증가가 맞다(C:1399 직후 0x390 ≥ 0.5 설정). 자연 사례에는 특이 재분해가 0회다.
- 독립 검토자 없음(이 세션이 작업자·마스터 겸임). `Engine.pt_rc`(48d5f 시작 재분해 수)도 S+0x2c0의 사본이라 같은 필드가 둘이다(후속 정리 대상).

## 7. 판정과 인수인계

- 작업 수행: **구현 완료(계약 범위 통과)**. 동등성: 원본 10모델 255재분해에서 블록 상태 수명(S+0x2c0/0x4e0/0x4e4)과 LU+0xd8·0xe4·0x100(t)이 주입 재생으로 전부 일치하고, 자연 궤적에서도 head가 같은 범위에서 일치한다. 회귀 규칙(정답 수 유지, 완전 일치 수 유지, 첫 차이 앞당김 없음) 통과.
- 새로 확정한 원본 사실: M1 dual(Method=1, Presolve=0)은 실행당 S 하나로 모든 `036ade`를 부르며 상태는 (1,0,0)에서 시작한다(LU2X2BLOCK 기본 → S+0x4e4 = 0). 풀 부족 −1 재호출은 S+0x2c0을 다시 올리지 않는다. `05b257`은 재분해 뒤 LU+0x100을 읽기만 한다(C:2595–2733) — t = s3.
- 미검증: 특이 기저 재분해 뒤 상태(C 근거만, 자연 0회), 2e9ae 무척도 엔진 인계, Rust 추가 재분해(19모델 끝 +1, adlittle iter 18)로 생기는 x2c0 차이, 블록 단계가 도는 상태에서의 풀 부족 재호출(Rust M1은 풀을 모델링하지 않아 한 번만 분해한다; 자연 사례 2건은 재호출 때 x4e4 = −1이라 차이 없음), Presolve가 켜진 경로·reoptimize 엔진의 S 수명, Markowitz tol 사다리(spx+0x390)가 바뀐 재분해의 주입 재생(주입은 기본 tol만 씀).
- 후속: M1-02w(stair e_r 185 LU+0x108), M1-02y-2, M1-02t, M1-02u. grow15·grow22는 첫 차이(365, 256)가 그대로이고 반복 수가 늘었다.
