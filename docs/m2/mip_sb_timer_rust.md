# M2-06h-5 SB 타이머 만료 Rust 이식 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 기준 main `f45157e`(M1-02x 병합), 미커밋 선행 변경 없음. 작업자는 이 세션(Claude Code, "다음스프린트 작업해 너가 작업자야" 지시). branch `work/m2-06h-5`, worktree `../rsolver-m2-06h-5`.

## 1. 범위와 주장

- 작업 종류: 구현(L1). 기존 06h-3 재생 하네스를 확장한다.
- 완료할 동작: `695bd7` 후보 루프에서 타이머 검사(`adf2be`)가 만료를 돌려줄 때의 루프 반응. Rust `strong_branch::run`은 지금 `Error::Unported`를 돌려준다. 원본처럼 루프를 끝내고 마지막 flush·부모 재풀이를 한 뒤 그때까지의 down/up 배열을 돌려주게 한다.
- 입력 전제: 06h-3과 같다(Threads=1, 전파 경로, 정수 후보, stride 1). 만료 여부는 backend가 돌려주는 값이다. 타이머 산술(`ade736` 시작값·`adf1a6` 예산·`adf2be` 판정 식)은 **이식하지 않는다**. 모듈 설계상 backend가 타이머를 소유하며 캡처(`tools/mip_sb_loop.js`)도 반환값만 기록한다. 그 식의 Rust 구현과 원본 비교는 미검증으로 남기고 후속 번호(가칭 M2-06h-7, B&B 연결 때 타이머 소유자와 함께)로 넘긴다.
- 제외: 전파 모순(`Unported` 유지, 원본 사례 미발견), 비전파 경로·연속 열(M2-06h-6), Threads > 1, LP 수치·B&B 연결.
- 선행: M2-06h-4 기준 자료 `bench/oracle/m2_06h4/final`(ran14x18-disj-8 3회 + 무훅, 만료 5묶음). 새 Gurobi 실행은 하지 않는다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C `../re/decomp/c/00571770_PRIVATE0000000000695bd7.c` C:1086–1107. 후보 인덱스 증가 뒤 `k ≥ n`이면 `LAB_00673582`, 아니면 `ad8869`(poll) → `k ≥ 1`이면 `adf2be(local_138)`, 0이면 루프 본문(`LAB_006720d6`)으로, 0이 아니면 루프를 빠져 `LAB_00673582`(stall 종료와 같은 곳) → `edd71`(flush) → `cda5b`/`14abfb`(부모 재풀이).
- asm: `0x573560 call adf2be; test %eax,%eax; je 5720d6`, 만료면 다음 주소 `0x57356d`로 떨어진다. stall·k ≥ n 종료도 `jge 57356d`(0x57350d·0x57352c)로 같은 주소에 간다. 만료 전용 쓰기·플래그는 없다.
- 시작·종료 경계, 입력·출력 필드는 [M2-06h-3 §2](mip_sb_loop_rust.md#2-원본과-상태-경계)와 같다.

## 3. 비교 계약

- 06h-3 [§3](mip_sb_loop_rust.md#3-비교-계약)과 같다: 명령 trace(전파·복구·풀이·values·poll·timer 순서와 work 값)·최종 down/up 배열·반복 비용·Rust work 덧셈이 u64 비트 일치. 오차 0.
- 만료 묶음은 원본 trace에서 `timer r=1` 뒤 남은 단계가 정확히 `flush, solve`여야 한다(하네스가 원본 자료에서 먼저 검사).
- 기준 판정 범위(착수 때 고정): ran14x18-disj-8 94묶음 전체(만료 5 포함), 6모델 합계 289묶음. 만료 5회 모두 재생 비교에 들어가야 한다(`timer expired` 필수 사례).
- 판별력: 기존 변조 `timer`(0 → 1)는 만료 뒤 단계 검사로, 새 변조 `timer clear`(1 → 0)는 필수 사례(`timer expired` 0회) 검사로 원본 자료 단계에서 거부돼야 한다. Rust 변이 `timer ignored`(만료 무시)·`timer return`(flush·재풀이 없이 반환)를 추가하고 원본 자료로 거부돼야 한다. 생존 변이 목록은 기존 `restore strict`·`score shift` 그대로여야 한다.
- 시간: 만료는 work 기반 판정 결과를 원본에서 받아 쓴다. 벽시계 비교 없음.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 | 원본 증거 |
|---|---|---|---|---|
| ran14x18-disj-8 double[49]·[59]·[64]·[69]·[74] | 실제 | 타이머 만료 → flush·재풀이 | `bench/oracle/m2_06h4/final/ran14x18-disj-8_{1,2,3,control}.json.gz` | 3회 동일(06h-4), `timer r=1 → flush → solve` |
| 나머지 284묶음(06h-4 기준) | 실제 | 회귀 | 06h-3·06h-4 manifest | 06h-4 |
| 합성 L1 테스트 | 합성 | 만료 뒤 후보 미평가·배열 유지 | `crates/mip/tests/strong_branch_l1.rs` | 해당 없음 |

## 5. 재실행 명령

- 원본 캡처(재실행 불필요): `tools/mip_sb_loop_runs.sh bench/oracle/m2_06h4/final markshare_4_0 neos-3627168-kasai ran14x18-disj-8`.
- 재생·비교·판별: `python3 tools/mip_sb_loop_rust.py --self-test`(manifest 갱신은 `--write-manifest`, 바뀌는 것은 도구·소스 해시뿐이어야 함).
- 회귀: `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`. Python 미변경이라 pytest 제외.

## 6. 실행 결과

- 변경: `crates/mip/src/strong_branch.rs` `run`에서 `expired()`가 참이면 `return Err(Error::Unported)` 대신 `break`(마지막 flush·재풀이 뒤 그때까지 배열 반환). 합성 테스트 `contradiction_unported_timer_stops`가 만료 뒤 `flush`·`solve`, 뒤 후보 −1e100 유지, work 반영을 확인한다. 하네스 `tools/mip_sb_loop_rust.py`는 ran14x18-disj-8을 일반 모델로 넣고 만료 묶음을 거르지 않는다. 원본 자료에서 `timer r=1` 뒤 단계가 `flush, solve`뿐인지 먼저 검사하고 `timer expired`를 필수 사례로 센다.
- 새 Gurobi 실행 0회. 기준 자료는 06h-4 그대로이고 manifest에서 바뀐 것은 하네스·소스·테스트 해시 3개뿐이다(캡처 해시 불변).
- `python3 tools/mip_sb_loop_rust.py --self-test` 종료 0([로그](../../bench/oracle/m2_06h5/selftest.log)). 6모델 **289묶음** 명령 trace·결과 배열·비용·work 비트 일치, `timer expired` 5(ran14x18 double[49]·[59]·[64]·[69]·[74]). 거부 43: 변조 18(`timer`는 만료 뒤 단계 검사 `after expiry`에서, 새 `timer clear`는 `no original case for timer expired`에서 Rust 실행 전에 거부. Rust 쪽 판별은 자연 만료 5묶음과 아래 두 변이가 맡는다)·직접 3·trace 4·Rust 변이 20 중 18(새 `timer ignored`·`timer return` 거부). 생존 2는 기존과 같다(`restore strict` 합성 거부, `score shift` 관측 불가).
- `cargo test --release --workspace` 129 통과·12 무시·실패 0([로그](../../bench/oracle/m2_06h5/cargo.log)), clippy 종료 0. pytest는 Python 미변경이라 실행하지 않았다.
- 실행하지 못한 것: 타이머 산술(`ade736`·`adf1a6`·`adf2be`)의 Rust 구현·원본 비교(범위 밖, §1).

## 7. 판정과 인수인계

- 작업 수행: **완료(구현 L1)**. 동등성: 타이머 만료 뒤 루프 반응은 **원본 자연 사례 범위 통과**(ran14x18-disj-8 만료 5묶음, 같은 모델 94묶음 전체, 6모델 289묶음 비트 일치, 오차 0).
- 남은 한계: 만료 판정 자체(예산·work 식)는 backend 몫이고 미이식·미검증. 전파 모순은 `Unported` 유지(원본 사례 없음). 비전파·연속 열은 M2-06h-6. B&B 미연결.
- 후속: M2-06h-6(VarBranch=3 비전파 오라클·이식), 타이머 산술은 B&B 연결 때 타이머 소유자와 함께(가칭 M2-06h-7). 회귀 `python3 tools/mip_sb_loop_rust.py --self-test`.
- branch `work/m2-06h-5`, worktree `../rsolver-m2-06h-5`. 인수인계 `docs/handoff/M2.md`, 진행 `docs/11_progress.md`·`docs/12_roadmap_diagram.md`·`docs/14_sprint_plan.md`.
- 교훈: `docs/lessons/m2_sb_snapshot.md`에 만료 = stall 종료와 같은 출구 항목.
