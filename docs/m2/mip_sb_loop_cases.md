# M2-06h-4 SB 루프 0회 분기 원본 사례 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 기준 main `d4b0787`(M2-06h-3 병합), 미커밋 선행 변경 없음. 작업자는 이 세션(Claude Code, "다음스프린트 작업해 너가 작업자야" 지시). branch `work/m2-06h-4`, worktree `../rsolver-m2-06h-4`.

## 1. 범위와 주장

- 작업 종류: 조사(자연 사례 탐색) + 발견 사례의 원본 재생 비교(기존 L1 하네스 확장).
- 조사 질문: M2-06h-3에서 원본 0회였던 `695bd7` 분기가 Threads=1 자연 실행에서 나오는가. 대상은 전파 모순(up·down), 타이머 만료, down ≥ 1e100(up < 1e100), 두 방향 ≥ 1e100, up 사전 채움(down만 평가), 상태7 dual < parent clamp, LP 상태 3·11·−1000, 연속 열 후보, 비전파 경로, stride > 1, 복구 루프에서 flag는 섰지만 bound가 원래 값과 같은 항목(`restore strict`), 변경 목록의 중복 열이다.
- 답의 형식(분기마다 하나): (a) 자연 사례를 찾고 원본 재생 비교를 실행했다, (b) 자연 사례를 찾았으나 Rust가 `Unported`라 비교하지 못했다(구현은 후속 번호), (c) C·asm 근거로 계약 범위(Threads=1, 공개 파라미터) 안에서 도달 불가, (d) 탐색한 모델·설정에서 0회이고 도달 조건 미확정.
- 비교 범위(착수 때 고정): Rust `strong_branch::run`이 이미 옮긴 분기(down ≥ 1e100, 두 방향 ≥ 1e100, up 사전 채움, 상태7·11·−1000 dual clamp, 상태 3, restore strict, 중복 열)는 자연 사례가 있으면 06h-3과 같은 비교 규칙으로 원본 재생 비교까지 한다. 전파 모순·타이머 만료는 Rust가 `Unported`를 돌려주므로 사례를 찾아도 이번에는 구현하지 않고 후속 번호로 넘긴다. 연속 열·비전파·stride > 1도 Rust 미이식이면 같다.
- 제외: Threads > 1 결과의 동등성(§8 별도 계약), LP 수치·전파 내부·B&B 연결, 원본 상태를 Frida로 바꾸는 합성 실험.
- 선행: M2-06h-3(`strong_branch::run`, `tools/mip_sb_loop.js`, `tools/mip_sb_loop_rust.py`).

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C `../re/decomp/c/00571770_PRIVATE0000000000695bd7.c`, 호출 `691b56 → 266b6a → 695bd7`. 분기 줄은 [M2-06h-3 §2](mip_sb_loop_rust.md#2-원본과-상태-경계). 상태 switch: 2(최적)·3/6(1e100)·7/11/−1000(`max(parent, LP+0xe0+0x138)`)·기타(로그) = up C:487–565, down C:881–960.
- 캡처 도구는 06h-3의 `tools/mip_sb_loop.js`를 그대로 쓴다(시작·종료 경계, ABI, 기록 필드 동일). 새 모델은 같은 파라미터(Threads=1, Seed=0, NodeLimit=300, GURO_PAR_SBMAXITERS=10)를 기본으로 하고, 다른 공개 파라미터를 쓰면 사례표에 적는다.

## 3. 비교 계약

- 탐색: `tools/mip_sb_census.py`가 캡처의 사건으로 분기 횟수를 센다. 06h-3 기준 3모델에서 모든 목표가 0이어야 한다(재현 확인). 탐색 캡처(1회)는 기준 자료가 아니다.
- 비교: 분기별 최소 모델을 골라 `bench/oracle/m2_06h4/final`에 독립 3회 + 무훅 대조로 캡처한다(§6). 3회 JSON 동일, 무훅 최종 출력 동일. 재생 비교는 06h-3 [§3 비교 계약](mip_sb_loop_rust.md#3-비교-계약)과 같다: 명령 trace(전파·복구·풀이·values·poll·timer 순서와 값)·최종 down/up 배열·반복 비용·Rust work 덧셈이 u64 비트 일치. 오차 0.
- 판별력: 06h-3에서 원본 자료로 판별하지 못한 Rust 변이(`clamp7`, `restore strict`)가 새 원본 자료에서 거부되는지 본다. 새 사례에 대한 변조 검사도 둔다.
- 기존 3모델의 06h-3 비교·변조·변이 결과는 그대로 유지해야 한다(회귀).

## 4. 사례와 원본 자료

탐색은 `../re/bench/data/miplib`의 38모델 전부를 기본 파라미터(Threads=1·Seed=0·NodeLimit=300·SBMAXITERS=10)로 1회씩 캡처했다(`tools/mip_sb_census_runs.sh`, 모델당 300초 한도). 33모델 완료, 5모델(csched007·csched008·ic97_potential·neos-4338804-snowy·neos-860300)은 한도 초과로 **미캡처**다(0회가 아니다). 4모델(enlight_hard·gen-ip054 등)은 SB 묶음이 없었다. 집계 [로그](../../bench/oracle/m2_06h4/explore/census.log). 탐색 캡처 파일은 크기(약 10MB) 때문에 저장소에 넣지 않았다. 같은 명령으로 다시 만들 수 있다.

| 분기 | 탐색 합계(33모델) | 기준 사례(final) | 답 |
|---|---:|---|---|
| 상태 3(down) → down ≥ 1e100 | 상태 3 158, down ≥ 1e100 109 | markshare_4_0 1 | (a) 비교 통과 |
| 상태 −1000(7과 같은 clamp) | 268 | markshare_4_0 6 | (a) 비교 통과 |
| up 사전 채움(down만 평가) | 43 | markshare_4_0 2 | (a) 비교 통과 |
| 상태7 dual < parent clamp | 5 | neos-3627168-kasai 2 | (a) 비교 통과, `clamp7` 변이 이제 거부 |
| 타이머 만료 | 5(ran14x18-disj-8) | ran14x18-disj-8 5 | (b) 원본 3회 재현, Rust `Unported` |
| 비전파 경로(N = NULL) | 0(기본 설정) | — | (b) VarBranch=3에서 실행 증거, 캡처 훅·Rust 미지원 |
| 전파 모순 up·down | 0 | — | (d) 도달 조건은 정적 확인, 사례 미발견 |
| 연속 열 후보 | 0 | — | (d) SOS 멤버 경로로만 가능(추정), 사례 미발견 |
| 상태 11 | 0 | — | (d) |
| restore strict·변경 목록 중복 열 | 0 | — | (d) 합성만 |
| 두 방향 ≥ 1e100 | 0 | — | (c) 함수 안 도달 불가(신뢰 중간) |
| stride > 1 | 0 | — | (c) Threads ≥ 2 전용 |

정적 근거(C 줄은 디컴파일 파일 기준, 하위 에이전트 조사 뒤 핵심 줄 확인):

- stride: `691b56` C:68 순차 호출은 stride 리터럴 1. C:240(worker 0 인라인)과 `693469` worker 본체만 stride = worker 수이고, 이 경로는 worker ≥ 2·n ≥ 6·SBMAXCANDS ≠ −1일 때만 열린다. Threads=1 계약 밖.
- 비전파: C:343은 N(rsi) ≠ 0이고 `N+0x18` ≠ 0일 때 전파한다. `62f909` 경로는 N과 N+0x18을 먼저 역참조하므로 항상 전파한다. `62a05d` C:785(asm 0x53882b `xor %esi,%esi`)만 N = NULL을 넘기며 VarBranch = 3에서만 열린다(예산 1e100, stall 2e9, &iters NULL). VarBranch=3 훅 실행에서 `691b56` 진입 훅이 `Node+0x18`(`tools/mip_sb_snapshot.js:70`)을 읽다 주소 0x18 접근 위반을 냈고, 사건 수에 `batch`만 없고 `double`·`kernel`은 20회였다([로그](../../bench/oracle/m2_06h4/vb3_null_node.log)). 이 실행은 탐색이며 기준 자료가 아니다. 훅을 단 VarBranch=3은 노드마다 전체 SB를 해 NodeLimit=300에서 모델마다 300초를 넘겼다.
- 타이머: `ade736(T, work, 0)`가 T[0] = 1 − work₀, `adf1a6`가 예산 < 1e100일 때만 한도를 둔다. `adf2be`는 (T[0] + work)/예산 ≥ 1, 즉 이 호출의 work 증가 + 1 ≥ 예산이면 만료다(work 기반, 결정적). 두 번째 후보부터 검사한다. 유한 예산은 루트 사전 SB(`64da9f` C:911)뿐이다. ran14x18-disj-8에서 5회 났다.
- 전파 모순: `6953ff`는 직접 교차, SOS, 클리크(`M+0x3370`), 암시 bound 표(`694e78`)에서 모순을 세운다. 선형 행 활동 전파는 없다. 33모델 NodeLimit 300에서는 0회였다.
- 연속 열: 후보 목록 `5d4427` 1군은 `types[j] != 'C'`로 거른다. SOS 2군만 타입 필터가 없다(`M+0x680` 출처 미추적, 추정).
- 두 방향 ≥ 1e100: 진입 때 down/up은 −1e100이다. up ≥ 1e100(상태 3/6/clamp, up 모순)이면 down을 건너뛰고, down 모순은 결과 판정을 거치지 않는다. 그래서 C:1061–1065 가드에 닿는 경로를 찾지 못했다.

기준 자료 `bench/oracle/m2_06h4/final`: 3모델 × 독립 3회 + 무훅 대조. [manifest](../../bench/oracle/m2_06h4/final/manifest.json)가 06h-3 자료·도구·이번 자료의 해시를 함께 고정한다(06h-3 manifest는 그대로 둔다). 캡처 도구는 06h-3과 같은 `tools/mip_sb_loop.js`(해시 동일).

## 5. 재실행 명령

- 탐색: `tools/mip_sb_census_runs.sh <OUT> '<params json>' [모델...]`(Gurobi 직렬, 모델당 300초 한도), 집계 `python3 tools/mip_sb_census.py [--self-test] <OUT>/*.json.gz`.
- 기준 캡처: `tools/mip_sb_loop_runs.sh bench/oracle/m2_06h4/final markshare_4_0 neos-3627168-kasai ran14x18-disj-8`(12회, 약 10분).
- 판정: `python3 tools/mip_sb_loop_rust.py --self-test`(06h-3 3모델 + 06h-4 2모델 재생, 타이머 모델은 거부 확인, 변조·변이).

## 6. 실행 결과

- 실행 commit 기준 main `d4b0787` + 이 branch의 도구 변경. x86-64 Linux(WSL2), Node 22.23.4, 바이너리 해시는 §2.
- 원본: 12회 모두 `ok true`. 모델마다 독립 3회 JSON이 같고 무훅 대조의 최종 출력이 같다(하네스 `controls`). `_1` gz SHA-256 앞자리는 markshare_4_0 `2f512092`, kasai `f39cf2a7`, ran14x18 `a5634ac0`.
- 탐색 재생(기준 판정 아님): 기본 설정 33모델 중 SB 묶음이 있는 31모델 2,058묶음이 Rust 명령 trace·결과 배열·비용·work까지 같았다(상태 3 158·−1000 268·down ≥ 1e100 109·up 사전 채움 43·dual < parent 5 포함). 거부는 ran14x18-disj-8 `double[49]` 타이머 만료 1건뿐이다.
- 판정: `python3 tools/mip_sb_loop_rust.py --self-test` 종료 0([로그](../../bench/oracle/m2_06h4/selftest.log)). 5모델 195묶음 비트 일치, 필수 사례에 상태 3·−1000·dual < parent·down ≥ 1e100·up 사전 채움을 추가했다. 타이머 모델은 `timer expired`로 거부되는지 확인한다. 변조 17·직접 3·trace 4 거부, Rust 변이 18 중 16 거부. 06h-3에서 생존하던 `clamp7`이 이제 kasai 원본 재생에서 거부된다. 남은 생존 2종: `restore strict`(원본 0회, 합성 거부), `score shift`(관측 불가).
- 회귀: `cargo test --release --workspace` 129 통과·0 실패·12 무시, `cargo clippy --workspace --all-targets -- -D warnings` 0. Python 미변경으로 pytest 제외.
- 변경: `crates/mip/tests/strong_branch_l1.rs` 응답 파서가 상태 3·11·−1000을 받는다(Rust 본체 `LpValue` 대응은 06h-3 그대로). 하네스는 자료 디렉터리를 모델별로 두고, 비간섭 검사(06g-2·06h-2)는 기존 3모델에만 적용한다.
- 실행하지 못한 것: 미캡처 5모델, VarBranch=3 캡처(훅 미지원), 깊은 NodeLimit·다른 Seed에서의 모순 탐색.
- 사고: VarBranch=3 탐색을 멈추려고 `pkill -f`를 썼다가 자기 셸이 먼저 죽었다(교훈 `tooling.md`에 이미 있는 실패). PID를 직접 kill해 정리했다.

## 7. 판정과 인수인계

- 작업 수행: **완료(조사)**. 동등성: 상태 3·−1000·dual < parent·down ≥ 1e100·up 사전 채움은 **자연 원본 사례 범위 통과**(5모델 195묶음). 타이머 만료·비전파는 자연 사례가 있으나 Rust 미이식이라 **미검증**이다. 전파 모순·연속 열·상태 11·restore strict·중복 열은 사례 미발견으로 **미검증**이다. stride > 1·두 방향 ≥ 1e100은 정적 근거로 계약 범위 밖 또는 도달 불가다.
- 후속:
  - **M2-06h-5**(구현): 타이머 만료 경로 Rust 이식. ran14x18-disj-8 기준 자료로 재생한다.
  - **M2-06h-6**(오라클·조사): VarBranch=3 N = NULL 경로. 캡처 훅을 null 안전하게 바꾸고(06h-3 자료의 도구 해시와 분리) 비전파 `d8d5f` 직접 bound 요청을 기록한다. 이어서 구현한다.
  - 전파 모순·연속 열: 클리크·SOS 모델에서 NodeLimit을 늘리는 탐색. 06h-6에 묶는다.
- branch `work/m2-06h-4`, worktree `../rsolver-m2-06h-4`. 미커밋 선행 없음.
- 교훈: `docs/lessons/m2_sb_snapshot.md`(분기 도달 조건, 상태 −1000), `docs/lessons/tooling.md`(VarBranch=3 훅 비용·null Node).
