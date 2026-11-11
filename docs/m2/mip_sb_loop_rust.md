# M2-06h-3 SB 후보 루프 Rust(L1)

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 기준 main `f3ec66a`(M2-06h-2r 병합), 미커밋 선행 변경 없음. 작업자는 이 세션(Claude Code, "다음 스프린트 진행해 너가 작업자야" 지시).

## 1. 범위와 주장

- 작업 종류: 오라클 확장 + 구현(L1).
- 완료할 동작: `695bd7` 한 호출의 후보 루프 전체. 후보마다 up/down 전파 요청 값, flush·LP 호출·결과 소비, 최적 해로 뒤 후보 사전 채움(값 쓰기와 down/up 추정치 min 갱신), 전파 변경 목록의 복구 요청과 work `+= 6·개수`, 추정치 게이트에 의한 후보 통째 건너뛰기·down 건너뛰기, up ≥ 1e100 건너뛰기, 점수(`628772`)·stall 카운터·조기 종료, poll·타이머 호출 순서, 종료 flush와 부모 재풀이 호출, 최종 down/up 배열과 반복 비용.
- 포함 분기와 전제: 정수(B/I) 후보, stride 1, 전파 경로(`local_90 ≠ 0`), 전파 무모순(`local_190 = 0`), LP 상태 2·6·7, 타이머 0. 추가 전파(변경 목록 > 1)는 포함한다. LP 수치·전파 내부·저장/복원 객체·타이머는 외부 backend가 원본 기록을 돌려준다.
- 제외 범위(자연 실행 0회): 전파 모순(up·down)과 타이머 만료는 Rust가 `Unported`를 돌려준다. 연속 열 `'C'`, 비전파 경로, stride > 1, 상태 3·11·−1000·기타는 하네스가 입력을 거부한다(Rust 타입은 C대로 3을 6과, 11·−1000을 7과 같이 받는다). down ≥ 1e100(stall 0)과 두 방향 ≥ 1e100 종료는 일반 판정 코드로 옮겼으나 원본 0회라 미검증, 상태7 base > dual clamp도 원본 0회(합성만). 원본 사례 탐색은 후속 **M2-06h-4**. LP 수치·B&B 연결은 이번 범위가 아니다.
- 선행: M2-06h-2(첫 후보 제어). 이번 변경은 `strong_branch::evaluate`를 루프 함수로 대체한다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C `../re/decomp/c/00571770_PRIVATE0000000000695bd7.c`, asm `python3 tools/dc.py 695bd7 --asm --max 5000`. 점수 `00545530_*628772.c`(asm 0x545530: `maxsd 1e-10`, `(a+1e-5)(b+1e-5)` sqrt, `−1e-5`).
- 루프 줄(C): 진입 poll 386, 후보 시작 388–455, 통째 건너뛰기 게이트 447–452·1050–1059(asm 0x572203–0x5722a4, `best >= score`면 건너뜀), up 전파 669–681, up flush·풀이·소비 463–575, 사전 채움 499–560, up 복구 루프 584–655(work 649), up 판정 660(`< 1e100`이면 down, 아니면 다음 후보), up 모순 682–727(work 725), down 추정 게이트 741–752, down 전파 761–783, down 소비 868–963(사전 채움 893), down 복구 993–1047(work 1040), 결과 판정·stall 1061–1083(asm 0x573485–0x57350d), 다음 후보 1084–1098(poll 1094 → 타이머 1096), 종료 1100–1102·1157–1164.
- 사전 채움 asm 0x5727dd–0x5728d1: 뒤 후보 j′의 LP 해 v(`12ee1a` 출력), 부모 해 x[j′]에 대해 `v <= floor(x)+1e-10`이면 down 쪽, `ceil(x)−1e-10 <= v`이면 up 쪽. 각 쪽에서 `eps > obj−parent`면 결과 칸 = parent, 추정치 = `minsd(추정치, obj)`. eps = `(|parent|+1)·1e-15`(asm 0x571e7e–0x571ea7). 상수는 rodata에서 확인(1e-10, 1e-5, 1e-15, ±1e100).
- ABI: `6953ff`(0x573d30) rdi=LP, rdx/rcx=현재 LB/UB 배열, r8=열, r9=방향(0x3e/0x3c), 스택 +8 &개수, +16 목록, +24 flag, +32 &모순, +40 work, xmm0=분기값(호출자 슬롯 up `[rsp+0x210]`, down `[rsp+0xd8]`를 진입 시 읽음; asm 0x572352·0x572c86). `12ee1a`(0x1bbdb0) (LP, 0, 개수, 열 목록, 출력). `adf2be`(0x8eb340) 타이머, `ad8869`(0x8e70a0) poll. 원래 bound getter `115607`/`1157b8`(0x1a7fc0/0x1a8070).
- 시작: 저장 객체·부모 ObjVal·해·원래 bound 배열을 얻은 뒤(C:395). 종료: 커널 반환. 종료 cleanup의 임시 파라미터 복구·해제는 06g-2 계약(비교 대상 아님).
- 입력: 후보 열 순서, 부모 해 x, IntFeasTol, 부모 ObjVal, stall 한도(`param_8`), 원래 LB/UB 배열, 시작 반복 비용. backend 출력: 전파 결과(모순 flag·변경 목록·flag·현재 bound), LP 상태·ObjVal/dual·IterCount, 뒤 후보 해 값, 타이머 반환, callee 안의 work 증가.

## 3. 비교 계약

| 필드·사건 | 규칙 | 근거 |
|---|---|---|
| 명령 순서(poll, timer, prop, flush, solve, values, reset, restore, bound, 종료 flush·solve) | 정확 일치 | 원본 훅 순서 |
| 전파 요청 열·방향·값 | 정확·u64 비트 | `propEnter` |
| 복구 bound 요청(열·방향·값) | 정확·비트, 엄격 부등일 때만 | C:592–647, 993–1047 |
| Rust work 덧셈(6·개수) 직후 값 | 다음 기록 사건의 원본 work와 비트 일치 | C:648–650, 1039–1041 |
| 최종 down/up 배열 | 후보별 u64 비트 | 커널 반환 `a`·`z` |
| 반복 비용 | 커널 반환 `cost1` 비트 | C:483, 877 |
| 건너뛰기·stall 판정 | 명령 순서로 판별(건너뛴 후보는 전파 요청 없음) | 위 asm |

오차 0, ±0 보존. NaN·무한 입력은 하네스가 거부. Threads=1, 시간 측정 없음. 타이머 반환은 원본 기록 값을 입력으로 쓰며 0 아닌 값은 거부한다. work는 Rust가 더하는 항목만 판정하고 callee 내부 증가는 원본 값을 backend가 돌려준다.

원본 반복: 같은 설정 독립 3회 gz 동일 + 무훅 대조 최종 출력 동일. 비간섭: 새 사건·필드를 뺀 투영이 pk1은 `bench/oracle/m2_06h2/capture/1.json.gz`(factor 배열은 요약값 비교), 세 모델은 `bench/oracle/m2_06g2/final/*_iters10_1.json.gz`의 커널 결과·풀이 상태·IterCount와 같다.

## 4. 사례와 원본 자료

| 사례 | 실제/합성 | 노리는 분기 | 탐색 증거(`bench/oracle/m2_06h3/explore`) |
|---|---|---|---|
| pk1 iters10 | 실제 | stall 조기 종료, 사전 채움, 상태2/7 | 24묶음, 전파 110(목록 1), values 2, 조기 종료 9 |
| neos5 iters10 | 실제 | up 상태6 → down 생략, 상태2 obj<parent clamp, 루프 밖 사전 채움 값 | 14묶음, up만 2, values 11 |
| tr12-30 iters10 | 실제 | 추가 전파(목록 > 1), 통째 건너뛰기, down 추정 게이트 | 111묶음, 목록>1 459, 통째 3, up만 20, values 134 |
| 0회 분기(§1 제외) | 합성 | 거부 확인만 | Rust 단위 테스트 |

설정: Threads=1, Seed=0, LogToConsole=0, NodeLimit=300, GURO_PAR_SBMAXITERS=10. 모델 `../re/bench/data/miplib/{pk1,neos5,tr12-30}.mps.gz`. 캡처 도구 `tools/mip_sb_loop.js`(06h-2 `mip_sb_control.js`의 치환 + 새 훅). 크기 때문에 factor 배열, 커널 진입/반환 밖의 bound 배열, 저장·복원 상태 배열은 `길이:FNV-1a32` 요약으로 저장한다(06h-1에서 원시 값 검증). 비간섭 비교는 기존 자료에 같은 요약을 적용한다. 탐색 v0(`explore_v0/`)은 factor 원시 배열 때문에 tr12-30 JSON 길이 초과로 실패해 보존한다.

## 5. 재실행 명령

- 캡처: `tools/hrun.sh --timeout 580000 -- 'tools/mip_sb_loop_runs.sh bench/oracle/m2_06h3/final'`.
- 재생·비교: `python3 tools/mip_sb_loop_rust.py`(`--self-test`는 변조·Rust 변이 포함).
- 변조: 바이너리/모델 해시 0, 사건 비우기, 목표 0회(전파·values·통째 건너뛰기·목록>1 각각), 입력 x·tol·원래 bound·values·work 변조, trace 순서·값 변조. Rust 변이: 사전 채움 방향·eps·min, 게이트 비교, stall 비교·초기화, 6·개수, 복구 엄격 부등, up 1e100 판정, 점수 상수.
- 회귀: 06h-2 첫 후보 하네스 `tools/mip_sb_control_rust.py`는 `evaluate` API와 함께 삭제했다. 06h-2 캡처는 이 하네스가 pk1 비간섭 투영으로 다시 비교하고, 첫 후보 명령은 루프 trace에 포함된다. snapshot/restore self-test, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`. Python 확장 미변경이면 pytest 제외.

## 6. 실행 결과

- 실행 commit `94954dd`(도구 해시는 `bench/oracle/m2_06h3/final/manifest.json`). x86-64 Linux(WSL2), Node 22, Frida 17.23.2.
- 원본: `tools/mip_sb_loop_runs.sh bench/oracle/m2_06h3/final` 종료 0. 모델마다 독립 3회 JSON 동일, 무훅 대조 최종 출력 동일(Status·NodeCount·IterCount·ObjBound·SolCount·ObjVal). 첫 시도(`final_v0/`)는 void getter의 rax를 기록해 반복 실행이 그 필드만 달랐다. 기록을 빼고 다시 캡처했다(다른 필드는 v0 반복 간에도 같았다).
- 비간섭: 새 사건·필드를 뺀 투영이 pk1 06h-2 캡처 전체(요약 적용)와 같고, 세 모델의 커널 결과·풀이 상태·dual·IterCount/ObjVal·저장/복원 상태·solve 전후 bound 요약이 06g-2 `*_iters10_1`과 같다.
- 재생: `python3 tools/mip_sb_loop_rust.py` 종료 0. 149묶음(pk1 24·neos5 14·tr12-30 111) 명령 trace 7,750줄이 원본과 같다. 전파 742회(목록 > 1 459), 복구 요청 2,664, values 147, 상태 2/6/7 = 224/2/516, 평가 후보 382, 후보 통째 건너뛰기 3, down 추정 게이트 20, up ≥ 1e100 2, stall 조기 종료 66, 루프 밖 사전 채움 값 1, 상태2 obj < parent 1. 최종 down/up 배열·반복 비용·Rust work 덧셈 직후 값이 비트로 같다.
- 0회 분기(미검증): 상태7 dual < parent, 전파 모순, 타이머 만료, down ≥ 1e100, 두 방향 ≥ 1e100, up 사전 채움(down만 평가), 연속 열·비전파·stride > 1.
- 오류 검출: `--self-test` 종료 0([로그](../../bench/oracle/m2_06h3/selftest.log)). 변조 22종 거부(해시 0, 빈 사건, 커널 없음, 목표 0회 3종, 해·tol·원래 bound·values·work·stall·reset 누락·타이머·모순·상태, 직접 3종, trace 4종). Rust 변이 18종 중 15종이 원본 재생에서 거부됐다. 생존 3종: `clamp7`(원본 0회)과 `restore strict`(flag는 있으나 bound가 같은 사례 0회)는 합성 테스트가 거부하고, `score shift`(점수 전체 −1e-5 제거)는 best 초기값 0과의 비교를 포함해 모든 판정이 같아 관측할 수 없다.
- 회귀: `cargo test --release --workspace` 129 통과·0 실패·12 무시, `cargo clippy --workspace --all-targets -- -D warnings` 0, `python3 tools/mip_sb_snapshot_check.py --self-test`·`python3 tools/mip_sb_restore_check.py --self-test` `ACCEPTANCE PASS`. Python 확장 미변경으로 pytest 제외.
- 독립 검토(Sonnet 하위 에이전트, 기준 `94954dd`): 높음·중간 0, 낮음 5. 제어 흐름을 C/asm 점프 방향까지 대조했고, 판별력 변조 5종을 거부했으며, pk1 원본 재캡처가 `final/pk1_1`과 바이트까지 같았다. 반영: (4) 변경 목록에 같은 열이 다시 나오면 C처럼 두 번째는 요청하지 않는다(flag 해제와 같은 효과, work는 개수대로). (1) up 사전 채움(nil)을 합성 테스트로 추가했다(원본 0회). C 줄 번호 인용 2곳을 고쳤다. 그대로 둔 것: (2) 사전 채움에서 floor·ceil 두 조건이 동시에 성립하려면 후보 x가 정수여야 한다. 관측된 후보 x는 모두 분수였다(pk1 후보 121개 중 정수 0개). 코드로 보장되는 성질이 아니므로 상위 후보 선정과 연결할 때 다시 확인한다. (3) 점수 상수 이동은 관측할 수 없다. (5) 한쪽만 ≥ 1e100인 판정은 down ≥ 1e100 원본 0회라 미검증이다. 반영분 재검토(`74b591e`)에서 높음·중간 0, 낮음 1(위 (2) 문구)이 나왔고, 새 합성 테스트가 up nil 제거·중복 처리 제거·work 개수 변이 3종을 모두 잡았다. 반영 뒤 `--self-test`를 다시 실행해 결과가 같았고, `cargo test --release --workspace` 129 통과, clippy 0이었다.
- 계약 변경: 06h-2 하네스 삭제(위 §5). 0회 분기 처리 문구를 구현에 맞게 고쳤다(§1).

- M2-06h-4 뒤: 이 자료의 `manifest.json`은 당시 기록으로 남기고, 현재 판정은 06h-3·06h-4 자료를 함께 묶은 `bench/oracle/m2_06h4/final/manifest.json`을 쓴다(하네스·테스트 파서·runs.sh가 바뀌어 옛 manifest 해시는 맞지 않는다).

## 7. 판정과 인수인계

- 작업 수행: 완료. 동등성: 계약 범위(정수 후보·전파 경로·무모순·상태 2/6/7·타이머 0)의 관측 분기에서 통과. §6의 0회 분기는 미검증.
- 남은 한계: LP 수치·전파 내부·저장 객체는 backend가 원본 값을 준다(Rust LP와 미연결). work는 Rust가 더하는 6·개수만 판정한다. `bnb.rs` 미연결.
- 후속: **M2-06h-4**(0회 분기 원본 사례 탐색), M2-06i(reliability 후보 판정).
- branch `work/m2-06h-3`, worktree `../rsolver-m2-06h-3`.
- 교훈: `docs/lessons/m2_sb_snapshot.md`(사전 채움 게이트, 전파 복구·void getter), `docs/lessons/tooling.md`(`pkill -f` 자기 셸, pane 재사용 표시, 큰 배열 요약).
