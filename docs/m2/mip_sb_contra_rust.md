# M2-06h-9 SB 전파 모순 이식 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 기준 main `34ea2d3`, 미커밋 선행 변경 없음. 작업자는 이 세션(Claude Code, "다음 스프린트 진행해 너가 작업자야" 지시). branch `work/m2-06h-9`, worktree `../rsolver-m2-06h-9`. 점유 기록의 branch는 main으로 찍혔다(06h-4와 같은 현상, 실제는 위 branch).

## 1. 범위와 주장

- 작업 종류: 구현(L1). 06h-3 재생 하네스를 06h-7 기준 자료로 넓힌다. 새 원본 실행은 없다.
- 완료할 동작: `695bd7` 후보 루프에서 `6953ff`가 `&모순`에 0이 아닌 값을 돌려준 경우.
  - up 모순: `up[k] = 1e100`. 전파가 조인 bound를 되돌려 요청하고 목록 길이 × 6 work를 더한다. LP는 풀지 않고 다음 후보로 간다. 같은 후보의 down은 계산하지 않아 `down[k] = −1e100`으로 남는다.
  - down 모순: `down[k] = 1e100`. 복구 요청과 work는 up과 같다. 그 뒤 **판정 블록(점수·stall·두 방향 무한 종료)을 건너뛰고** 다음 후보로 간다.
- 입력 전제: Threads=1, 전파 경로(Node·`Node+0x18` ≠ 0), 정수 후보, stride 1. 나머지 루프 제어는 06h-3~06h-6과 공유한다.
- 제외: 연속 열 `'C'` 후보(down에서 '>'·'<' 두 번 전파, **M2-06h-10**), 타이머 산술(**M2-06h-8**), Threads > 1, 0x2a31f0(266b6a 쪽 전파 사본) 경로.
- 선행: M2-06h-7 기준 자료 `bench/oracle/m2_06h7/final`(csched007 NL150 up 7회, ic97_potential NL1050 down 1회).

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C `../re/decomp/c/00571770_PRIVATE0000000000695bd7.c`(C 라벨 = asm 주소 + 0x100000).
  - C:669–740 up: `6953ff(floor+1, …, '>', &local_158, …, &local_190)`. `local_190 ≠ 0`이면 `param_10[j] = 1e100`. 목록마다 flag 1이면 `orig_lb < cur_lb`일 때 `'>'` 요청, flag 2면 `cur_ub < orig_ub`일 때 `'<'` 요청, 요청 뒤 현재 값을 원래 값으로 쓰고 flag를 0으로 한다. 목록 길이 × 6.0을 work에 더한다. C:733 `LAB_00672e6c`로 간다.
  - C:772–838 down: 정수 후보는 C:772 `LAB_00672c2c`에서 `6953ff(floor, …, '<', …, &local_198)`. `local_198 ≠ 0`이면 `param_9[j] = 1e100`, 같은 복구 루프, C:834 `LAB_00672e64` → `LAB_00672e6c`.
  - asm 0x572e64–0x572e82: 오류 검사 → `k += stride` → `k < n`이면 0x57352e(poll·timer)로, 아니면 종료(0x57356d). 판정 블록 0x57347d(C:1064–1085)를 지나지 않는다. 정상 down 경로는 C:990·1048에서 0x57346f(C:1061) → 0x57347d 판정으로 간다.
  - 복구 루프는 정상 경로(C:588–633, C:996–1047)와 같은 코드 모양이다. Rust는 한 함수 `undo`로 공유한다.
- 경계와 ABI: 06h-3 [§2](mip_sb_loop_rust.md#2-원본과-상태-경계), 06h-7 [§2](mip_sb_contra_cases.md#2-원본과-상태-경계)와 같다. 전파 결과(`out`, 목록 열·flag·전파 뒤 bound, work)는 backend 응답이다.

## 3. 비교 계약(첫 Rust 재생 전 고정)

06h-3 비교 규칙을 그대로 쓴다. 명령 trace(poll, timer, prop, bound, flush, solve, values, reset, restore, 종료 flush·solve)는 순서와 값이 정확히 같아야 한다. 요청 값·work·최종 down/up 배열·반복 비용은 u64 비트로 비교한다. 오차는 0이고 ±0을 보존한다(csched007 복구 값 −0.0).

- 재생 대상: 06h-7 기준 자료 csched007(150묶음)·ic97_potential(532묶음) 전체를 하네스 기본 모델 집합에 넣는다(`contra/<모델>`). 모순 묶음 5개 안의 비모순 후보도 이번에 처음 비교한다.
- 원본 자료 단계 검사(Rust 입력과 무관): `prop.out ∈ {0,1}`. 모순 prop 뒤에는 `boundwrite`만 오고, 그다음은 `poll` 또는 종료 `flush`다(LP 없음). 방향별 `up contradiction`·`down contradiction`을 세고 둘 다 필수 사례다.
- 통제: 06h-7 manifest의 `_2`·`_3` gz 해시가 `_1`과 같고, `_1` 파일 해시가 그 값과 같아야 한다(반복 파일은 저장소에 없다). 무훅 대조(`_control`)는 사건 0개·출력 동일. 도구 해시·params는 06h-7 manifest의 `base_params`+모델별 params와 같아야 한다.
- 판별력: Rust 변이 4개(`contra ignored`·`contra value`·`contra undo`·`contra judge`)를 넣고 원본 재생 또는 합성 테스트가 잡는지 본다. 기존 변이·변조 목록은 그대로 돌린다.
- 회귀: `python3 tools/mip_sb_loop_rust.py --self-test` 종료 0, `cargo test --release --workspace`, clippy.

## 4. 사례와 원본 자료

| 사례 | 실제/합성 | 분기 | 자료 |
|---|---|---|---|
| csched007 NL150 | 실제(MIPLIB) | up 모순 7회(묶음 1020·1025·1030·1035의 첫 후보 등), 목록 = 후보 자신, 복구 값 −0.0 | `bench/oracle/m2_06h7/final/csched007_1.json.gz` |
| ic97_potential NL1050 | 실제(MIPLIB) | down 모순 1회(묶음 2833, 후보 2 = 열 105, 목록 105·102·104, 복구 요청 3번) | `…/ic97_potential_1.json.gz` |
| `contradiction_skips_lp_and_judgement` | 합성(`strong_branch_l1.rs`) | up 모순 복구·LP 없음, stall 1 상태의 down 모순 뒤 stall 유지 → 다음 후보에서 한도 종료 | C·asm 근거 |

## 5. 재실행 명령

```
python3 tools/mip_sb_loop_rust.py               # 원본 응답 → Rust → 원본 trace 비교
python3 tools/mip_sb_loop_rust.py --self-test   # + 변조·Rust 변이·합성 테스트(약 12분)
cargo test --release -p rsolver-mip --test strong_branch_l1
```

## 6. 실행 결과

- 환경: branch `work/m2-06h-9`, 새 원본 실행 0회. 기준 자료는 06h-7 그대로다(manifest 해시 일치를 하네스가 검사).
- 원본 재생 `python3 tools/mip_sb_loop_rust.py`: **PASS**, 971묶음(전파 871 + 비전파 100) 명령 trace·요청 값·work·배열·비용 비트 일치. 그중 06h-7 두 모델 682묶음(csched007 150, ic97 532)이 새로 들어갔고 모순 묶음 5개(그 안의 비모순 후보 포함)도 일치했다. 필수 사례 `up contradiction` 7, `down contradiction` 1. 첫 실행부터 차이가 없었다.
- `--self-test` 종료 0(약 11분): 기존 변조·trace 변조 모두 거부. Rust 변이 중 `contra ignored`·`contra value`·`contra undo`는 원본 재생에서 거부됐다. `contra judge`(down 모순 뒤 판정 실행)는 원본 재생에서 **살아남고** 합성 테스트 `contradiction_skips_lp_and_judgement`가 잡는다. ic97의 유일한 down 모순에서 stall을 0으로 되돌려도 뒤의 종료가 바뀌지 않기 때문이다. 생존 목록은 기존 5개 + `contra judge`다.
- `cargo test --release --workspace` 166 통과·0 실패·13 무시, `cargo clippy --workspace --all-targets -- -D warnings` 0. pytest는 Python 미변경이라 돌리지 않았다.
- 하네스·소스·테스트 해시가 바뀌어 06h-6 manifest(`bench/oracle/m2_06h6/final/manifest.json`)를 `--write-manifest`로 다시 썼다(06h-7 manifest·자료 파일 해시 추가 포함). 06h-7 manifest의 `tools` 해시는 캡처 당시 값으로 두었다(검사하는 도구 없음).
- 하네스 변경: `controls()`에 `contra/` 분류(도구 `mip_sb_direct.js`, params = 06h-7 manifest), `_2`·`_3`은 `_1`을 다시 쓰고 06h-7 manifest 해시로 반복 동일을 확인(`_2`·`_3` 파일 자체는 다시 대조하지 않는다, 06h-7 기록을 믿음), `prop.out = 1` 허용과 모순 뒤 구조 검사, 필수 사례 2개, Rust 변이 4개, `branches()`에서 모순 후보를 `up >= 1e100`·`down >= 1e100`(LP 불가능 필수 사례)에서 빼고 `up contradiction batch`로 따로 셈(독립 검토 낮음 2). `--skip-contradiction`은 06h-7 재현용으로 남겼다.

## 7. 판정과 인수인계

- 작업 수행 상태: 완료. 동등성: **통과**(오차 0) — 원본 자연 사례 범위(up 7회, down 1회).
- 남은 한계: down 모순 뒤 판정 건너뛰기는 원본 사례로 판별되지 않았다(C·asm 근거 + 합성 테스트). down 모순 원본 사례는 1회뿐이다. 0x2a31f0 경로, Threads > 1, 연속 열 후보(06h-10)는 미검증. `strong_branch::run`은 여전히 `bnb.rs`에 미연결이다.
- 독립 검토(Opus, 읽기 전용): 높음 0·중간 0·낮음 4. 반영: 반복 검사 한계 명시, 모순 후보를 `>= 1e100` 카운터에서 분리, 하네스 주석 C 줄 범위 수정, 인계에 commit 기록. 검토 뒤 하네스를 바꿔 `--self-test`를 다시 돌렸다.
- `Error::Unported` 변형을 지웠다(이 모듈의 유일한 사용처가 모순이었다).
- 후속: **M2-06h-10**(연속 열 후보), M2-06h-8(타이머 산술), M2-06i.
- 교훈: `docs/lessons/m2_sb_snapshot.md`(down 모순의 판정 건너뛰기), `docs/lessons/process.md`(손 변이 뒤 `git checkout`이 미커밋 구현을 지움, 기존 M1-01e 항목에 덧붙임).
