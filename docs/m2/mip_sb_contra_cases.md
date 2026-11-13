# M2-06h-7 SB 전파 모순·연속 열 후보 원본 사례 탐색 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 기준 main `d09db91`, 미커밋 선행 변경 없음. 작업자는 이 세션(Claude Code, "다음 스프린트 진행해 너가 작업자야" 지시). branch `work/m2-06h-7`, worktree `../rsolver-m2-06h-7`.

## 1. 범위와 주장

- 작업 종류: 조사(자연 사례 탐색). 발견 사례는 기준 캡처와 기존 재생 하네스의 판정까지 한다.
- 조사 질문: `695bd7` 후보 루프에서 06h-4·06h-6까지 원본 0회였던 두 분기가 Threads=1 자연 실행에서 나오는가.
  1. 전파 모순(up·down): `6953ff`가 `&모순`(스택 +32)에 0이 아닌 값을 쓰는 경우.
  2. 연속 열 후보: 커널 후보 목록에 타입 `'C'` 열이 들어가는 경우.
- 답의 형식(분기마다 하나, 06h-4와 같음): (a) 자연 사례를 찾고 기준 캡처와 재생 판정을 했다, (b) 자연 사례를 찾았으나 Rust가 `Unported`이거나 하네스가 거부해 비교하지 못했다(구현은 후속 번호), (c) C·asm 근거로 계약 범위(Threads=1, 공개 파라미터) 안에서 도달 불가, (d) 탐색한 모델·설정에서 0회이고 도달 조건 미확정.
- "자연 실행"은 원본 바이너리를 공개 API로 실행하고 훅은 읽기만 하는 실행이다. 입력 모델은 MIPLIB·jsplib 파일과 이번에 만든 합성 **모델**(gurobipy로 쓴 MPS, 해시 고정)을 모두 허용한다. 합성 모델은 실제 입력이므로 그 실행은 자연 실행이다. Frida로 원본 상태를 바꾸는 실험은 제외한다.
- Rust 범위: 모순은 `strong_branch::run`이 `Unported`를 돌려준다(`crates/mip/src/strong_branch.rs:122`). 연속 열 후보는 하네스가 거부한다(06h-6 §1). 이번 번호는 구현하지 않는다. 사례를 찾으면 기준 자료를 고정하고 구현은 후속 번호로 넘긴다.
- 제외: Threads > 1, 원본 상태 변경 실험, 06h-4가 정적 근거로 닫은 분기(두 방향 ≥ 1e100, stride > 1), 타이머 산술(06h-8).
- 선행: M2-06h-4 탐색(33모델 NodeLimit 300 0회)과 06h-6 도구 `tools/mip_sb_direct.js`.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- 커널 `695bd7`(0x571770, C `../re/decomp/c/00571770_PRIVATE0000000000695bd7.c`), 전파 `6953ff`(0x573d30, C `../re/decomp/c/00573d30_PRIVATE00000000006953ff.c`)와 callee `694e78`. 266b6a(0x29d980)와 그 전파 사본 0x2a31f0도 같이 센다.
- 경계: 커널 진입(후보 목록·타입 배열 `a[5]`·Node·`Node+0x18`)과 전파 반환(`&모순` 값, 열, 방향 r9, 호출 지점). ABI는 06h-3 [§2](mip_sb_loop_rust.md#2-원본과-상태-경계)와 같다.

## 3. 비교 계약(탐색 전 고정)

- 탐색 도구: 새 `tools/mip_sb_scan.js`(커널 진입과 전파 반환만 훅, 깊은 NodeLimit용)와 `tools/mip_sb_scan_runs.sh`. 탐색 결과는 기준 자료가 아니다.
- 스캐너 대조(착수 확인): 같은 모델·파라미터에서 스캐너의 커널 수·호출 지점별 전파 수가 저장된 전체 캡처와 같아야 한다. markshare_4_0 NodeLimit 300(`bench/oracle/m2_06h4/final/markshare_4_0_1.json.gz`): 커널 23·`double` 23·전파 up 31·down 31로 같았다.
- 발견 사례의 기준 자료: 분기별 최소 사례를 `bench/oracle/m2_06h7/final`에 `tools/mip_sb_direct.js`로 독립 3회 + 무훅 대조 캡처한다. 3회 JSON 동일, 무훅 최종 출력 동일. `tools/mip_sb_census.py`가 그 자료에서 해당 분기를 1 이상 세야 하고, 스캐너 수와 같아야 한다.
- 재생 판정: `tools/mip_sb_loop_rust.py`가 발견 사례에서 그 분기 묶음은 `Unported`(모순) 또는 연속 열 거부로 **거부**하고, 같은 모델의 나머지 묶음은 06h-3 비교 규칙(명령 trace·배열·비용·work u64 비트)으로 일치하는지 본다. 나머지 묶음 불일치는 실패로 기록한다.
- 판별력: 스캐너가 모순을 놓치지 않는다는 증거로, 발견 사례에서 스캐너 수와 전체 캡처 census 수가 같아야 한다. 발견하지 못하면 스캐너의 모순 검출은 미검증으로 남기고(양성 대조 없음) 결론은 (d)다.
- 회귀: 기존 하네스 `python3 tools/mip_sb_loop_rust.py --self-test`가 종료 0을 유지한다. Rust를 바꾸면 `cargo test --release --workspace`와 clippy.

## 4. 사례와 원본 자료

탐색(스캐너, `bench/oracle/m2_06h7/scan_*`):

- MIPLIB 38모델 NodeLimit 5000(`scan_nl5000`, 전부 정상 종료, 시간 한도 0): 전파 모순 up 46회(csched007 41·csched008 1·neos-860300 2·neos-911970 2), down 1회(ic97_potential, 커널 serial 989, 열 105). 연속 열 후보 0회(SOS 절이 있는 모델 없음).
- 첫 사건의 커널 serial은 NodeLimit에 관계없다(csched007 k=223, neos-860300 k=235·243은 NL150에 있음, ic97은 NL1050에 있고 NL1000에 없음). 그래서 최소 사례를 csched007 NL150, ic97_potential NL1050으로 정했다.
- 정적 근거(`695bd7`·`6953ff`·`694e78` C 읽기): 모순은 정수 후보와 정수 bound에서는 직접 교차로 생기지 않고 SOS(P:189/194/277/285), 클리크 `M+0x3370` 한 단계(P:413/451), 암시 bound 표 `M+0x3298`의 BFS(Q:79/89/181/190)에서 생긴다. 연속 후보는 `5d4427` 2군(presolve 뒤 남은 SOS의 연속 멤버, `M+0x680`)에서만 온다. `PreSOS1BigM=0`·`PreSOS2BigM=0`이면 big-M 변환(`57b2bf` C:160)을 건너뛴다. 클리크 표 내용과 후보 3·4군의 쓰임은 확인하지 않았다.
- SOS 합성 모델 `tools/mip_sos_models.py`(6개, `bench/oracle/m2_06h7/models`, 다시 만들면 바이트 동일): 그룹마다 연속 x[g,k]가 SOS1/SOS2 하나, 수요 등식, 용량 `≤ int(slack·Σd/K)·y[k]`(이진 y). slack 1.6 모델은 SB 없이 루트에서 끝났다. 빡빡한 모델 중 sos2_g40k5t는 기본 설정에서 2묶음에 연속 후보 2개씩, SOS1 두 모델은 `PreSOS1BigM=0`에서만 후보가 났다(`scan_sos_default`·`scan_sos_nobigm`). SOS 모델에서 모순은 0회였다.

기준 자료 `bench/oracle/m2_06h7/final`(`tools/mip_sb_direct.js`, Threads=1 Seed=0, `GURO_PAR_SBMAXITERS=10`):

| 사례 | 모델 | NodeLimit | 분기 | 묶음 | 위치 |
|---|---|---|---|---|---|
| up 모순 | csched007(MIPLIB) | 150 | 7회 | 150 | 묶음 111–114, 열 487·562·549·498·574·602·617(모두 B) |
| down 모순 | ic97_potential(MIPLIB) | 1050 | 1회 | 532 | 묶음 494, 열 105, 목록 105·102·104(B·C·C) |
| 연속 후보 | sos2_g40k5t(합성) | 5000(루트 1노드에서 끝남) | 2묶음×2열 | 2 | 열 20·24, 147·149 |

관측된 원본 순서:

- up 모순(csched007): `propEnter`(ra 572375, '>' 1) → bound 요청 → `prop` out=1, 목록 = 후보 열 자신 하나(lower=upper=1) → 복구 요청(값 −0.0) → poll → timer → **다음 후보의 up**. 같은 후보의 down은 계산하지 않아 a = −1e100(초기값)으로 남고 z = 1e100이다. LP 풀이 없음.
- down 모순(ic97): 목록 3개(B·C·C, 암시 bound 연쇄), 복구 요청 3번 → poll → timer. a = 1e100, z = 3949.0000000000005.
- 연속 후보(sos2): up은 `6953ff`를 ra 573b67에서 0x3e로, down은 572ca6에서 0x3c로 부른다(정적 근거의 '>'·'<' 두 번). 네 후보 모두 a = z = 부모 값(6443.816, 6517.0).

## 5. 재실행 명령

```
# 탐색(스캐너, 결과는 기준 아님)
FRIDA_NODE=/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida \
  tools/mip_sb_scan_runs.sh bench/oracle/m2_06h7/scan_nl5000 '{"NodeLimit":5000,"GURO_PAR_SBMAXITERS":10}'
tools/gurun.sh ../.venv/bin/python tools/mip_sos_models.py bench/oracle/m2_06h7/models
# 기준 캡처(무훅 대조 + 3회 × 3사례, 약 100분)
tools/hrun.sh --timeout 14400000 -- 'tools/mip_sb_contra_runs.sh bench/oracle/m2_06h7/final'
# 판정
python3 tools/mip_sb_census.py bench/oracle/m2_06h7/final/*_1.json.gz
python3 tools/mip_sb_loop_rust.py --explore DIR [--skip-contradiction]   # DIR = 위 _1 파일들
python3 tools/mip_sb_loop_rust.py --self-test
```

csched007·ic97의 2·3회는 1회와 gz SHA-256까지 같아 저장소에 넣지 않고 `final/manifest.json`의 `files`·`not_committed`에 해시만 남겼다.

## 6. 실행 결과

- 재현: csched007·ic97은 3회 gz 바이트 동일. sos2_g40k5t는 3회가 **한 필드만** 다르다: 첫 묶음(ev[18]) save/restore의 workspace `+0x418` flags 요약(250바이트). 원시 바이트를 찍는 일회성 probe(save `0x576a30`·restore `0x5754e0` 진입)를 3번 돌리니 첫 묶음에서 바이트 2–5·9–11(7개)만 실행마다 달랐고 둘째 묶음은 같았다. 값 모양으로 보아 해제된 힙 청크에 남은 포인터·키(ASLR 의존)로 추정한다. 이 배열은 save/restore가 그대로 복사할 뿐이고 Rust 하네스는 비교하지 않는다. 나머지 필드(명령 trace·배열·비용·work·LP 상태)는 3회 같다. 계약의 "3회 JSON 동일"은 sos2에서 이 한 필드 때문에 깨졌고 이유를 위와 같이 기록한다.
- 무훅 대조: 세 사례 모두 최종 출력(Status, NodeCount, IterCount, ObjBound, SolCount, ObjVal 16진)이 훅 실행과 같다. 스캐너 출력과도 같다.
- census vs 스캐너: up 모순 7 = 7(`contradiction:572375:62`), down 모순 1 = 1(`contradiction:572ca6:60`), 연속 후보 2묶음 열 {20,24}·{147,149} = 스캐너 hits와 같다(스캐너의 `continuous candidate` 4는 kernel·double 진입을 같이 센 묶음 수, census 4는 열 수다).
- 재생(`--explore`, 건너뛰기 없음): csched007 `double[1020].prop[2]: expected=0 actual=1`, ic97 `double[2833].prop[2]: expected=0 actual=1`로 모순 묶음에서 거부, sos2 `double[13].noninteger candidate` 거부.
- 재생(`--skip-contradiction`): csched007 146묶음, ic97 531묶음 MATCH(모순 묶음 4+1 건너뜀), sos2 연속 후보 거부. 합계 677묶음, 상태 7 2,366, prop 4,542.
- 회귀: `--skip-contradiction` 추가로 하네스 자기 해시가 바뀌어 `--self-test`가 `manifest.tools/mip_sb_loop_rust.py` 불일치로 처음 실패했다. 06h-6 검토 때처럼 `--write-manifest`로 다시 썼고 바뀐 줄은 그 해시 한 줄이다. 다시 돌린 `--self-test` 종료 0. Rust 미변경이라 cargo는 돌리지 않았다.
- `tools/mip_sb_scan_runs.sh`의 `timeout`은 node만 죽여 Frida가 띄운 Gurobi 자식이 남을 수 있다. 이번 탐색은 rc=124가 0회였다. 다시 쓸 때 시간 한도에 걸리면 남은 python 프로세스를 확인한다.
- 스캐너의 0x2a31f0(266b6a 쪽 전파 사본) 훅은 탐색 전체에서 호출 0회라 그 쪽 모순 검출은 양성 대조가 없다.

## 7. 판정과 인수인계

- 전파 모순 up·down: **(b)**. 자연 사례(csched007 NL150, ic97 NL1050)를 찾고 기준 자료를 고정했다. Rust는 `Unported`라 모순 묶음은 비교하지 못했고, 같은 모델의 나머지 677묶음은 비트까지 같다. 건너뛰기는 묶음 단위라 모순 묶음 5개 안의 **비모순 후보와 그 LP 풀이도 비교하지 않았고**, 그 묶음은 stride·prefix 같은 구조 검사도 받지 않았다. 06h-9에서 처음 비교되므로 불일치가 모순 처리 탓이 아닐 수 있다. 구현은 **M2-06h-9**: up 모순 뒤 같은 후보의 down을 건너뛰고 다음 후보로 가는 흐름, 복구 요청 −0.0, a = −1e100 유지, down 모순 a = 1e100.
- 연속 열 후보: **(b)**. 합성 SOS2 모델 sos2_g40k5t(기본 설정)에서 자연 사례를 찾았다. 하네스가 `noninteger candidate`로 거부한다. 구현은 **M2-06h-10**: up을 부모 값으로 채우고, down에서 0.0으로 `6953ff`를 '>'(573b67)·'<'(572ca6) 두 번 부르는 경로.
- 남은 한계: 클리크 표 `M+0x3370` 내용과 후보 3·4군 미확인, 0x2a31f0 훅 양성 대조 없음, 266b6a(0x29d980) 진입이 695bd7과 같은 인자 배치라는 가정은 같은 묶음의 double 진입 값과 같다는 간접 확인뿐, Threads > 1 제외, sos2 flags 앞 바이트의 정체는 추정(해제된 청크 메타데이터).
- 교훈: `docs/lessons/m2_sb_snapshot.md`, `docs/lessons/tooling.md`.
