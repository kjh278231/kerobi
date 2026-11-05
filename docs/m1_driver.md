# M1 LP 드라이버 작업 노트 (`9828a1` → `c23877` → `0c3d0d`, 파라미터 `ac81d7`)

G5([11 §4](11_progress.md)) 작업 중에 알아낸 사실. 확정된 내용은 [02](02_driver.md)로 옮긴다. 브랜치 `m1-lp-driver`(worktree `rsolver-presolve`).

## 0. 현재 상태 (2026-11-06)

| G5 기준 | 상태 | 근거 |
|---|---|---|
| (1) 파라미터 표 548개 = gurobipy | **충족** | §2, `tests/test_params.py` 548/548 |
| (2) Optimize 헤더 로그 102개 | 미착수 | |
| (3) 분류·Method 확정·자동 조정·상태·요약 로그 | 미착수 | |
| (4) Presolve 켠 연속 경로 (Method 2·6 반복 수, Method 1 상태·목적) | 미착수 | |
| (5) clippy 0, 테스트, 문서 | 진행 중 | |

## 1. 호출 범위 (2026-11-06)

### 1.1 정적 호출 그래프로는 갈리지 않는다

`re/decomp/callgraph.tsv`에서 `9828a1`·`c23877`·`0c3d0d`는 모두 같은 강연결 덩어리(닿는 함수 2,767개: common 804, cut·휴리스틱 377, presolve 365, simplex 313, B&B 264, genconstr 234, barrier·PDHG 223, driver 149)에 들어간다. sub-MIP·다목적·휴리스틱이 `98b671` → `c23877`로 되돌아오기 때문이다. 그래서 범위는 **실제로 도는 경로**를 Frida로 떠서 정했다.

방법: `c23877`·`9828a1`·`0c3d0d`·`172cc6`·`2bb770`·`34f033`·`9b029d`와 그 직접 callee 181개에 진입·복귀 훅을 걸고(`tools/fn_trace.js`, `leave: []`), afiro를 Presolve=-1, OutputFlag=1, Threads=1로 Method 기본·1·2(Crossover=0)·6(Crossover=0)에서 풀었다. 스펙 생성기와 트리 출력기는 세션 스크래치에 있다(재현: callgraph에서 위 7개의 callee를 뽑아 `{"name","off","leave":[]}` 목록을 만든다).

### 1.2 연속 LP에서 도는 경로 (관찰)

```
9828a1
  a841df(→a7f063)  edd71(update)  bb8b1f(batchmode 검사 →a7f821)  edd71  9a8287  a7f063 ×2
  c23877(model, 1, timer)
    a7f063 ×2 (isMIP)  115e51 (웜 basis)  115faf (시작 벡터)
    a8c184 (지문)  a8aa6a ("Model has …")  a7f063  a9a78e (계수 범위 통계)
    19febe (다목적 수)  a7f0ad/a7fb80/a7f923 반복 (QP·2차·QCP 판정, Method 확정)
    86113 (이전 결과 해제)  75a3b0 (Lazy 배열 정리)
    ├ Method 기본·1: 0c3d0d                                   → a6a8da (LP 결과 속성: IterCount 등)
    ├ Method 2, Crossover 0: 2bb770                            → a6ad23 (비기저 해 속성), a7f923
    └ Method 6, Crossover 0: 34f033                            → a6ad23
  9a8287  a84513(→a7f063)
```

`0c3d0d` 안 (Method 1, afiro):

```
ac7d4e (스레드 수)  10f448  88c65 (이전 presolved 모델 해제)  3b88f5
88237 (웜스타트 모드: 10a423, a7f0ad, 11603b, 115faf)  10a423  115faf  1160a1  88068 (웜스타트 구조체 해제)
f7dc1(→83696)  a7f0ad
c31c5 → f7dc1, ★390989 (presolve), a7f0ad          ← PreDual 판정 겸 presolve 래퍼 (노트 13 §1의 "c379f = presolve"는 틀림)
b9b82 (PreDual 판정)  a7f0ad ×2
cda5b (presolved 모델 simplex: 7d3ce 스케일, a83b2f 한도, a6a8da)
d10b1  a7aa8f ×2
c379f (SolutionTarget < 1이면 곧바로 반환. 아니면 36578f)
★363daa (LP uncrush)  12ef7f  a8ff5e
88c65 (presolved 모델 해제)  3b88f5  a7f0ad
cda5b (원래 모델, uncrush 기저로 웜스타트: "Extra simplex iterations after uncrush")
d10b1  a7aa8f  c9457 (상태 메시지 "Optimal objective …")  d10b1 ×2  10f5ae
```

`2bb770`(barrier 진입): a7f0ad, a7f923, 86113, 83696, 2d0596, 2ee55d, f7dc1, ★390989(→2baa47), ★2bd9ab(barrier 본체), ★36578f(LP uncrush 드라이버), 88068, 2ce5c6, 88c65 ×2.
`34f033`(PDHG 진입): a7f0ad, a7f923, 86113, 83696, 88237, 1160a1, 2baa47, f7dc1, ★390989, ade552 ×8, adec59, ★34c004(PDHG 본체), ★36578f, 10b63c ×2, 7a2ee.

### 1.3 범위와 이미 포팅된 부분

| 묶음 | 함수 | 상태 |
|---|---|---|
| 파라미터 표 | `ac81d7`(이름 → env 오프셋 등록, `accc50`/`accde4`/`accefe`) + `.data` 정의 표 | §2. `ac81d7`은 값의 저장 위치만 정하고, 이름·타입·최소·최대·기본값은 `.data`의 40바이트 레코드 표에 있다 |
| 진입 | `9828a1`: update, 최대화 부호 반전, 비볼록 재시도 | 연속 LP에서는 `c23877` 한 번 |
| 드라이버 | `c23877` A·C·F·H·I·K·L 단계([02 §2](02_driver.md)), 분류 `a7f063`·`a7f0ad`·`a7fb80`·`a7f923`·`a7fbf4`, 결과 속성 `a6a8da`·`a6ad23`, 상태 메시지 `c9457` | 미포팅 |
| 헤더 로그 | `a8c184` 지문(포팅됨, `fingerprint.rs`), `a8aa6a`, `a9a78e` | 지문만 |
| LP 드라이버 | `0c3d0d`: 웜스타트 판정, `c31c5`(presolve), PreDual `b9b82`, `cda5b` 두 번, uncrush `363daa` | presolve `390989`·uncrush `363daa`는 포팅됨(G2). `cda5b`는 `simplex.rs` `solve_model`(main의 simplex 에이전트 소유, 이 브랜치에서 고치지 않음). 웜스타트 입력이 없어 uncrush 뒤 재풀이는 아직 못 한다 |
| barrier·PDHG | `2bb770`/`34f033`: presolve → 본체 → `36578f` | 본체는 포팅됨(G3·G4, Presolve=0 경로). `36578f`(비기저 해 uncrush, "big uncrush violation")는 미포팅 |

범위 밖: MIP 경로(`3db15a`, `5bbf7f`, 휴리스틱), 다목적 `1b39db`, 다중 시나리오 `7a0103`, concurrent(`172cc6`, `9b029d`), network `170d99`, sifting, QP `14abfb`, crossover, 원격·라이선스.

## 2. 파라미터 표 (`ac81d7` + `.data` 정의 표)

- `ac81d7`(env 생성 때 `GRBemptyenvadvinternal`·`ad0297`·`bb32bc`가 부름)은 이름마다 `accefe`/`accc50`/`accde4`로 런타임 표(env+0x23e0, 0x40B 항목)를 찾아 env 오프셋을 적고 기본값(+0x28)을 env에 복사할 뿐이다. 이름·타입·범위·기본값의 원본은 `.data`의 **40바이트 레코드 표**다: +0 이름 포인터(R_X86_64_RELATIVE 재배치), +8 플래그(int, 의미 미확인), +12 타입(1 int, 2 double, 3 string, -1 표식), +16/+24/+32 min/max/default(double). `Cutoff`에서 시작해 공개 208개, 표식 `EndPublic`, 숨은 `GURO_PAR_*` 340개, 끝 표식 `End` 순이다.
- 포팅: `tools/param_table.py <libgurobi130.so>`가 이 표를 그대로 `crates/model/src/param_table.rs`(548개, 원래 순서)로 생성한다. `params.rs`는 `Def`, 대소문자 무시 `find`, 문자열 기본값 `default_str`만 둔다. 값 저장(env 오프셋 대응)은 드라이버가 쓸 때 붙인다.
- 기준값: `tools/param_oracle.py`가 gurobipy `getParamInfo`를 548개 이름(숨은 것 포함, 모두 받아 줌)에 대해 떠서 `bench/gurobi_params.tsv`에 둔다. Python `Model.getParamInfo`와 비교해 이름·타입·최소·최대·기본값 548/548 같다.
- gurobipy 규약: double의 ±1e100(GRB_INFINITY)은 ±inf로 보고한다(예: Cutoff -inf..inf, 기본 inf; GURO_PAR_INFBOUND 최대 inf). 문자열의 min/max는 `''`.
- 표 밖 기본값은 `NodefileDir = "."` 하나다(표에는 빈 문자열). 이 값을 넣는 코드는 찾지 않았다.
- 노트 11 §5.1의 "562개 항목"은 이 표(550 레코드, 표식 2개 포함)와 다르다. 다시 센 값은 548개다.
- 생략: docs/01 §1.2의 정적 필드 구조체(빌드 스크립트 생성)와 `setParam` 범위 검사·오류 10007/10008. 드라이버는 이름으로 값을 읽는다.
