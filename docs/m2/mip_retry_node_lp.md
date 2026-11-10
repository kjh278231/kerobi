# M2-03d-3 검증 계약과 결과: 재풀이 노드 LP 원본 캡처

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 경계와 사례는 [M2-03d-1](mip_retry_rust_boundary.md) §3·§6 질문 2의 L2를 따른다.

## 1. 범위와 주장

- 작업 종류: 오라클 구축과 조사. Rust 솔버 코드는 바꾸지 않는다.
- 완료할 동작:
  1. 재풀이가 일어나는 노드 LP를 원본에서 캡처한다. 대상은 행렬·sense·rhs·bound·목적·목적 상수·ModelSense·vtype과 LP env 파라미터(Cutoff 포함)다. Rust가 읽을 수 있는 JSON으로 저장한다.
  2. 캡처한 LP가 simplex가 실제로 푼 LP인지 확인한다. 두 가지를 본다.
     - 첫 LP와 재풀이의 원본 모델 자료 `D`가 같은가.
     - 재풀이 simplex 작업 구조(`ws`)의 배열이 캡처한 값과 같은가.
  3. 원본 독립 3회에서 같은 결과를 얻는다.
- 조사 질문:
  - Q1. nidda 노드 LP의 추가 62행은 무엇인가? 모델 402행과 열은 그대로인가?
  - Q2. gmu 53×110 LP는 sub-MIP인가?
  - Q3. 재풀이 LP 6의 cutoff 값은 어디서 오는가? 재풀이 결과와 모순이 없는가?
- 포함 사례: nidda stress 노드 8287(`node_basis_direct`), gmu default solve 26476(`0c3d0d` 경로). 둘 다 M2-03c-2와 같은 설정의 자연 실행이다.
- 제외 범위와 후속:
  - Rust LP 포함 재생(L2)과 실제 경로 연결은 M2-03d-4다.
  - 62행의 생성 알고리즘(cut 종류·생성 순서)과 sub-MIP 생성 휴리스틱의 판단은 이번 범위가 아니다. 판별 근거까지만 다룬다.
  - LU·pricing·반복 궤적은 다루지 않는다.
- 선행: main `5476fe3`. 미커밋 선행 변경은 없다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 file offset이다.
- 근거(`../re/decomp/c/`):

| 함수 | 줄 | 내용 |
|---|---|---|
| `5ce67b`(`0x4f0e10`) | 924–950 | 재풀이 `cda5b(lp, method, I+0x98, I+0xa0, …)` 직후 같은 `lp`에 `GRBgetintattr(lp,"Status")`, `GRBgetdblattr(lp,"ObjVal"/"ObjBound")`를 부른다. 노드 LP가 공개 API를 받는 모델 객체라는 근거다 |
| `GRBXgetvars`(`0x7a13a0`) | 전체 | 열 우선 `D+0xf8/+0x100/+0x108/+0x110`을 읽는다. 행 scale `D+0x3d8`, 열 scale `D+0x3e0`, 열 부호 플래그 `D+0x408`, '>' 행 부호 `D+0x388`을 풀어 사용자 형식으로 돌려준다. 읽기 전용이다 |
| `GRBcheckmodel`(`0x88d010`) → `a77e9f` | 전체 | env 오류 메시지 버퍼(`env+0x23d8`, `env+0x23d0`)를 지운다. 풀이 상태는 바꾸지 않는다 |
| `02f7e1`(`0xd06f0`) | 247–259 | `ws+0x3b8 = (lp+0xf0==0) ? env Cutoff(env+0x24a8) : 1e100`, `ws+0x3c0 = max(ws+0x3b8, env+0x24b8)` |

- 계측 영향: 공개 API(`GRBgetvars`·`GRBgetconstrs`·attr getter)는 재풀이 `cda5b` 진입에서 한 번만 부른다. 위처럼 env 오류 버퍼를 지우고 임시 메모리를 할당하므로 **계측 사건**으로 표시한다. 나머지 지점은 메모리를 읽기만 한다. 영향이 없다는 판정은 §3 검사 A로 한다.
- 시작 경계: 재풀이가 일어나는 `5ce67b` 호출의 첫 `cda5b` 진입. 끝 경계: 같은 호출의 재풀이 `cda5b` 반환.
- 설정: M2-03c-2와 같다. nidda는 NodeLimit 6000, Presolve 0, ScaleFlag 0, Quad 0, NumericFocus 0, MarkowitzTol 1e-4이고 gmu는 NodeLimit 20000이다. 공통으로 Threads 1, Seed 0이며 TimeLimit은 두지 않는다.

## 3. 비교 계약

새 사건은 `y:` 접두어를 쓴다. 기존 03c-2 사건(`x:` 포함)은 그대로 둔다.

| 사건 | 시점 | 내용 |
|---|---|---|
| `y:dims` | 모든 `5ce67b` 진입. (MIP 구조체, 행 수, 열 수)가 바뀔 때만 보낸다 | solve 번호, MIP 구조체의 등장 순번, 행·열 수, 루트 여부, env+0x2728 |
| `y:dhash` | 재풀이 solve의 각 `cda5b` 진입 | `D` 원시 배열(열 우선 행렬, obj, lb, ub, rhs, sense, scale·부호 플래그 유무)의 FNV. 메모리만 읽는다 |
| `y:lp` | 재풀이 `cda5b` 진입(계측) | 공개 API로 읽은 LP 전체와 LP env 파라미터 전부(`../re/notes/env_param_offsets.tsv`의 int·double, 그리고 `env+0x24b8`), `lp+0xf0` null 여부 |
| `y:ws` | 재풀이 `cda5b` 안의 `02f7e1` 반환 | `ws`의 m·n, CSC, sense, c, b, l, u, 목적 scale·행렬 scale 유무, `ws+0x3b8/0x3c0`, ModelSense |
| `y:ws_leave` | 재풀이 `cda5b` 반환 | `ws+0xf0`(상태), `ws+0x118`(목적값) |
| `y:where` | `retry` 사건 | libgurobi 안 호출 스택(file offset), MIP 구조체 순번, 그 MIP의 LP 모델과 노드 LP가 같은 객체인지 |

- 부동소수는 모두 비트 패턴의 16자리 16진수(u64 big-endian 표기)로 저장한다. JSON 숫자로는 −0과 NaN을 구분할 수 없다.
- 실행 후 같은 Python 프로세스가 `optimize()` 반환 뒤 원래 모델(행·열·bound·목적)을 같은 형식으로 덤프한다. 풀이가 끝난 뒤이므로 풀이 사건에 영향이 없다.

**검사** (`tools/mip_retry_node_lp_check.py`). 하나라도 실패하면 종료코드 1이다.

| 검사 | 규칙 | 근거 |
|---|---|---|
| A 무영향 | `y:` 사건을 뺀 재풀이 사건 전체와 `output`(DONE 줄)이 M2-03c-2 저장 natural 캡처와 정확히 같다 | 계측이 풀이를 바꾸지 않았다 |
| B 재현 | 3회의 `y:` 사건이 정확히 같다(포인터 값은 저장하지 않는다) | 기준 §6 |
| C1 LP 불변 | 재풀이 solve에서 `cda5b` 진입은 정확히 2회이고 두 `y:dhash`가 같다 | 첫 LP와 재풀이의 입력 LP가 같다 |
| C2 ws 일치 | ScaleFlag 0 사례(nidda)는 scale이 없어야 한다. 그때 `ws`의 열 n개 c·l·u, 행 b·sense, CSC가 `y:lp`와 비트 단위로 같다. '>' 행 부호 등 표현 차이는 `GRBXgetvars`의 변환 규칙으로만 맞춘다 | 캡처가 simplex가 푼 LP다 |
| C3 cutoff | `lp+0xf0`가 null이면 `ws+0x3b8 == env Cutoff`, `ws+0x3c0 == max(Cutoff, env+0x24b8)` | `02f7e1` C:247–259 |
| C4 상태 | 재풀이 `ws_leave`의 상태가 03c-2와 같고(nidda 6, gmu 2), 6이면 목적값을 기록한다 | 캡처 |
| E 모델 대조 | nidda: LP 열 수와 모델 열 수가 같다. LP 앞 402행의 행렬·sense·rhs와 열의 obj·vtype이 모델과 비트 단위로 같다. 차이는 bound와 추가 행에만 있을 수 있으며 개수를 보고한다 | Q1 |
| 거부 | 재풀이 사건 0회, `y:lp`·`y:ws` 누락, 해시 불일치, ws≠API, 모델 대조 실패 | 기준 §9 |

- 자기검사(`--self-test`): 저장 자료를 메모리에서 변조해 모든 검사가 거부하는지 확인한다. 대상은 비영 계수 1개, bound 1개, Cutoff, 재풀이 사건 제거, 실행 간 차이, `y:` 이외 사건 변경, dhash, ws 배열이다.
- 수치 비교는 모두 비트 비교다. 오차 허용은 없다.
- 시간·work·병렬: Threads 1, TimeLimit 없음. 해당 없음.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 분기 | 입력 | 원본 증거 |
|---|---|---|---|---|
| nidda stress 노드 8287 | 원본 실제 실행(계측 1지점) | `node_basis_direct`, 재풀이 LP 6 | `neos-3754480-nidda.mps.gz` `63f322dd…a8bf7f` | `bench/oracle/mip_retry_node_lp/nidda_stress_{1,2,3}.json` |
| gmu default solve 26476 | 원본 실제 실행(계측 1지점) | `0c3d0d`, 재풀이 LP 2 | `gmu-35-50.mps.gz` `5652e6ad…adb8d1` | `bench/oracle/mip_retry_node_lp/gmu-35-50_default_{1,2,3}.json` |

- 캡처 도구: `tools/mip_retry_node_lp.js`. `tools/mip_retry_basis.js`(natural 모드)를 복사하고 `y:` 사건을 더했다. 원래 도구는 M2-03c-2 manifest에 해시가 고정돼 있어 고치지 않는다.
- 적용하지 않은 범주: 동률·허용오차 경계는 해당 없다(입력 캡처). 재풀이 rc≠0과 12 잔존은 자연 사례가 없다.

## 5. 재실행 명령

- 원본 캡처: `tools/mip_retry_node_lp_runs.sh bench/oracle/mip_retry_node_lp`. Gurobi를 6회, 한 프로세스씩 실행한다(약 50분).
- 기준 파일·manifest 생성: `python3 tools/mip_retry_node_lp_check.py --write-manifest`. 1회차 캡처에서 `*.node_lp.json`을 만들고, 캡처·기준 파일·도구·모델의 해시를 기록한다.
- 저장 검사: `python3 tools/mip_retry_node_lp_check.py --self-test`. 기대 종료코드는 0이다.
- 회귀: 솔버 코드를 바꾸지 않는다. `python3 tools/mip_retry_basis_check.py --self-test`와 `python3 tools/mip_retry_rust_expect.py --self-test`가 계속 0이어야 한다.

## 6. 실행 결과

- 실행: worktree `../rsolver-m2-03d-3`, 캡처 도구 commit `77e1cf2`. 검사기·기준 파일의 최종본은 이 문서와 같은 commit이다(main `7038dd5` 병합 뒤 재검사). x86-64 WSL2 Linux, Gurobi 13.0.3(위 SHA-256), Threads 1.
- 원본 캡처: `tools/mip_retry_node_lp_runs.sh bench/oracle/mip_retry_node_lp`. Gurobi를 한 프로세스씩 6회 돌렸고 약 50분 걸렸다. 6회 모두 `ok`이고 출력 줄은 03c-2와 같다(nidda `DONE 8 6000`, gmu `DONE 8 20000`).
- 검사: `python3 tools/mip_retry_node_lp_check.py --self-test` 종료코드 0.
  - manifest(`bench/oracle/mip_retry_node_lp_manifest.json`, 03c-2 manifest 포함)를 통과했다.
  - 두 사례 모두 A·B·C1·C3·C4를 통과했다. nidda는 C2·E도 통과했다(gmu는 ws가 scale돼 있고 모델이 sub-MIP라 계약대로 제외).
  - 노드 LP 기준 파일 2개가 캡처에서 다시 만든 것과 같다.
  - 메모리 변조 12종을 모두 거부했다(nidda 10, gmu 2).
- A의 범위: 저장되는 사건은 재풀이 solve 하나의 사건뿐이다. `DONE` 줄은 NodeLimit에 걸려 끝난 값이라 그 뒤 궤적을 판별하지 못한다. 그래서 A가 보이는 것은 "재풀이 solve가 반환할 때까지 계측 영향 없음"이다. API 호출은 그 solve 안에서 한 번 일어난다.
  - 디스크에서 캡처 1바이트(Cutoff 16진수 한 자리)를 바꾸면 종료코드 1(`hash mismatch`)이고, 되돌리면 0이다.
- 회귀: `python3 tools/mip_retry_basis_check.py --self-test`와 `python3 tools/mip_retry_rust_expect.py --self-test`는 0이다. 솔버 코드를 바꾸지 않아 Rust·Python 검사는 실행하지 않았다.
- 실행하지 않은 것: Rust LP 재생(L2, M2-03d-4).

**Rust용 기준 파일** `bench/oracle/mip_retry_node_lp/<case>.node_lp.json`(schema `rsolver.node_lp.v1`)

- 내용: 노드 LP(열 우선 행렬·obj·lb·ub·vtype·rhs·sense·ModelSense·ObjCon), LP env 파라미터 184개, 노드 basis(nidda만), 재풀이 solve의 top-level LP 호출 목록, branch.
- LP 호출 항목은 호출 위치, 노드 LP 여부, retry 여부, method, basis 종류(node·none·given), `lp+0x238` warm 여부, 상태다. given이면 시작 basis 배열도 담는다.
- env 파라미터는 `../re/notes/env_param_offsets.tsv` 오프셋에서 그대로 읽은 값이다. 오프셋을 실행으로 대조한 것은 Cutoff(C3)뿐이다.
- 부동소수는 u64 비트 패턴의 16진수다. nidda bound에는 −0.0(`8000000000000000`)이 31개 있다.
- nidda(`node_basis_direct`):

| k | 호출 위치 | LP | method | basis | 상태 |
|---|---|---|---|---|---|
| 1 | `5ccea5+0x356` | 노드 | 1 | 노드 | 12 |
| 2 | `5ce67b+0x4cf8` | 노드 | 0 | 노드 | 6 |

- gmu(`0c3d0d`):

| k | 호출 위치 | LP | method | basis | 상태 |
|---|---|---|---|---|---|
| 1 | `5ccea5+0x356` | 노드 | 1 | 없음 | 12 |
| 2 | `0c3d0d+0x14e2` | 복사본(원시 자료 해시 같음) | 1 | 없음 | 2 |
| 3 | `0c3d0d+0x51bd` | 노드 | 1 | 호출자가 준 basis(given) | 2 |
| 4 | `0c3d0d+0x5626` | 다른 LP(비영 645) | 0 | `lp+0x238` warm(`68341b` setbasis 뒤) | 2 |

- gmu 재풀이는 LP 하나가 아니다. k=3의 시작 basis 내용은 캡처했지만, `0c3d0d`가 k=2 결과에서 그 basis를 만드는 과정은 추적하지 않았다.

**Q1. nidda 노드 LP와 62행** (캡처, 해석은 표시)

| 항목 | 결과 |
|---|---|
| 크기 | 253열 × 464행, 비영 4827 |
| 모델 402행 | 비영 패턴이 같다. 각 행이 모델 행의 정확한 유리수 배수 λ다(rhs 포함). λ=−1인 행이 100개('>'를 '<'로 바꿈), λ=1/5인 행이 1개, 나머지는 λ=1이다 |
| 추가 62행 | 모두 '<'이고 모델에 없다 |
| 열 | obj·ObjCon·ModelSense는 모델과 같다. 정수 50열은 모델에서 'I'([0,1])이고 LP에서 'B'다. bound는 253열 모두 모델과 다르다. 정수 50열은 모두 고정(31개 −0.0, 19개 1.0)이고, 연속 203열은 lb 0과 유한 ub다 |
| 호출 문맥 | 최상위 `5bbf7f` 안(깊이 1). 노드의 MIP 구조체는 root MIP가 아니고(`M+0x620≠M`), root의 재시작 횟수는 1이다. 재시작 `6a6563`은 solve 1169–1205에서 1회 일어났다. LP env+0x2728=2다 |
| 호출 스택 | FUZZY backtrace(libgurobi 안 프레임만)는 `5bf638`→`5c239f`→`8ad3a3`(노드 휴리스틱 디스패처)→`8bc185`(RINS)→`8d1e5d`→`8e3397`(sub-MIP 엔진)→`8e4682`→`5ce67b`다. 인접한 쌍이 모두 정적 호출 그래프(`../re/decomp/callgraph.tsv`)의 간선이다 |

- 해석:
  - 노드 8287은 B&B 트리 노드가 아니다. 재시작 뒤 본 탐색에서 노드 휴리스틱 RINS가 `8e3397`→`8e4682`로 만든 LP다.
  - 정수를 모두 고정한 연속부 LP다. `8e3397`의 "이산변수가 모두 고정되면 연속부 LP"([휴리스틱 노트](../../../re/notes/16_cuts_and_heuristics.md) 3.6.3)와 모순이 없다.
  - 62행은 그 시점 본 탐색 LP에 붙어 있던 행으로 보인다. 본 탐색(깊이 1, 253열) 노드 LP의 행 수는 402–468 사이에서 바뀐다(`y:dims` 48회). 재풀이 직전 본 탐색 LP(solve 8856, env+0x2728=0)도 464행이다. 개수만 맞췄고 행 내용은 비교하지 않았다. cut의 종류와 생성 순서도 확인하지 않았다(추론).
- 행 정규화: λ=−1·1/5는 모델 행을 MIP 내부 LP로 옮길 때 생긴다. Rust가 같은 LP를 다시 만들려면 모델 파일이 아니라 이 캡처를 써야 한다.

**Q2. gmu 53×110 LP** (캡처)

- 재풀이 solve는 깊이 2의 `5bbf7f` 안에서 일어났다. 그 MIP는 모델 200×81이고 presolve 뒤 110×53이며 env+0x2728=2다. 바깥은 최상위 `5bbf7f`(모델 435×1919, presolve 뒤 358×953)다. 노드의 MIP 구조체는 자기 자신이 root다(`M+0x620==M`, 재시작 0).
- 따라서 sub-MIP의 루트 LP다. 실행 중 `5bbf7f` 진입은 깊이별로 1·275·558·5회이고, 본 탐색은 재시작을 3회 했다.
- 재풀이 판단 시점(`0x4f53a8`)에는 노드 LP의 `D`가 제자리에서 scale돼 있다(`D+0x3d8/0x3e0` 있음, 해시가 다름). k=1과 k=3 진입에서는 scale 배열이 없고 해시가 같다. 그래서 API 덤프는 k=3에서 했다.
- 어떤 휴리스틱이 이 sub-MIP를 띄웠는지는 확인하지 못했다. 깊이 2 `5bbf7f`의 반환 주소는 `c23877`(optimize 드라이버)이고, 재풀이 시점 스택은 `5daa82`→…→`64da9f`→`654502`까지만 정적 간선으로 이어진다(`ac661f`·`6f2e4a`는 간접 호출).

**Q3. cutoff** (캡처·C)

- nidda:
  - LP env Cutoff는 13137.241487924155(`40c9a89ee9138850`)이고 env+0x24b8도 같다. `lp+0xf0`는 null이다. 재풀이 `ws+0x3b8`·`ws+0x3c0`이 env Cutoff와 비트 단위로 같다(`02f7e1` C:247–259).
  - 재풀이(primal) 종료 상태는 6이고 `ws+0x118`은 16444.881908654846 ≥ cutoff로 모순이 없다.
  - 노트상 이 Cutoff는 sub-MIP 구성 `8e9f93`이 min(부모 cutoff, incumbent+1e-6)로 넣는다. 이번 실행에서 그 계산은 확인하지 않았다.
- gmu: Cutoff는 −2607449.2346607447이고 k=3 ws와 같다. 재풀이 상태는 2이고 목적값은 −2608045.40287894다.

**C2 ws** (nidda, 캡처)

- ScaleFlag 0이고 scale 배열은 없다. 목적 scale은 1이다.
- 재풀이 `02f7e1` 반환 시 ws의 열 c·l·u, CSC, 행 b·sense가 API 덤프와 비트 단위로 같다. 이번 LP에는 '>' 행이 없어 부호 변환 규칙은 실제로 쓰이지 않았다.
- slack은 l=0, u=INFBOUND(1e30)이고 c=0이다.
- 첫 LP(k=1)의 ws 해시가 재풀이(k=2)의 ws 해시와 같다. 재풀이는 첫 LP와 같은 ws 배열에서 시작한다.

**계약에서 바꾼 것** (결과를 보고 바꾼 이유와 함께 기록)

1. `y:lp` 덤프 지점을 "재풀이 `cda5b` 진입"에서 "재풀이 판단 뒤 노드 LP 자체를 받는 첫 top-level `cda5b` 진입"으로 바꿨다. nidda에서는 같은 지점(k=2)이다.
   - 이유: gmu 재풀이는 `0c3d0d`가 복사 LP를 푼다. 또 판단 시점의 `D`는 scale돼 있어 API 값이 역scale 계산을 거친다.
   - 대신 덤프 시 `D`의 scale 배열이 없어야 한다는 조건을 C1에 넣었다.
   - `GRBXgetvars`는 `D+0x3c≠0`이면 플래그 열의 부호를 뒤집는다. nidda 덤프 시점은 `D+0x3c=−1`이지만 C2·E가 열 값을 대조한다. 대조가 없는 gmu는 `D+0x3c=0`을 요구한다(실제 0).
2. C1의 "정확히 2회"는 nidda에만 둔다. gmu는 노드 LP 진입이 k=1·3이다. 공통 규칙은 노드 LP 진입의 `D` 해시가 모두 같고 덤프 시 해시와도 같다는 것이다.
3. C3를 노드 LP의 모든 ws에 적용했다(nidda k=2, gmu k=3).
   - C4는 처음에 nidda에서만 검사했으나 검토에서 지적받아 두 사례로 넓혔다. 모든 재풀이 LP 호출의 ws 상태가 `x:cda5b:leave` 상태와 같아야 한다. 노드 LP 호출은 "목적값≥cutoff이면 6, 아니면 2"를 만족해야 한다.
4. E의 "앞 402행이 비트 단위로 같다"는 실패했다. 100행은 부호 반전, 1행은 1/5배였다. 규칙을 "같은 비영 패턴, 정확한 유리수 배수 λ(rhs 포함), λ<0이면 '<'↔'>'"로 바꿨다.
   - vtype은 "같거나, 모델 'I'이고 [0,1]이면 'B'"로 바꿨다.
   - 실패한 원래 규칙의 첫 차이는 행 0 열 203이다(모델 15, LP 3, λ=1/5).
5. MIP 구조체 번호를 포인터 라벨에서 결정적 값으로 바꿨다. 결정적 값은 `5bbf7f` 진입 순번·깊이, `M+0x620==M`, `M+0x628==M`, 재시작 횟수다.
   - 이유: 첫 전체 실행에서 gmu 1·2회차의 top-level 사건 수가 5215와 5214로 달랐다. heap 재사용으로 라벨이 바뀐 것이다. 그 실행은 중단하고 지웠다.
   - `y:mipmain`·`y:restart`·`y:decision_dhash` 사건을 더했고, backtrace는 libgurobi 안 프레임만 남긴다.
6. 자기검사 변조에 모델 행·ws 누락·gmu Cutoff·gmu dhash를 더했다(8종 → 12종).
7. 노드 LP 파일에 호출 위치, basis 종류, 시작 basis, `lp+0x238` warm 여부, branch를 더했다. 처음 파일은 gmu k=3을 "basis 없음"으로 적어 cold 재풀이처럼 읽혔다.
8. runner의 timeout을 900초에서 3600초로 늘렸다. gmu 1회가 약 15분이어서 여유가 없었다. 캡처는 900초 판으로 했고 모두 시간 안에 끝났다.

- pilot 실패:
  - `y:mipmain` 훅에서 `this.depth`에 쓰다가 Frida 오류가 났다. `depth`는 Frida InvocationContext의 읽기 전용 속성이다.
  - 출력 폴더가 없어 모델 덤프가 실패했다.
  - pilot 출력은 ignore된 `target/m2-03d-3-pilot/`에만 있고 기준 자료가 아니다.

## 7. 판정과 인수인계

- 작업 수행 상태: **완료(오라클·조사)**. 두 사례의 노드 LP를 원본 3회 재현으로 캡처했고, 계측 무영향(A)과 변조 검출을 확인했다. Rust가 읽을 기준 파일도 만들었다.
- 동등성: 해당 없음(Rust 미변경). Rust LP 포함 재생은 **미검증**이다.
- 조사 답:
  - Q1: nidda 노드 8287은 재시작 뒤 본 탐색의 RINS sub-MIP 엔진(`8e3397`→`8e4682`)이 정수를 모두 고정해 푼 연속부 LP다. 62행은 모델에 없는 '<' 행이다(본 탐색 LP의 cut으로 추정).
  - Q2: gmu는 sub-MIP(깊이 2, 200×81→110×53) 루트 LP다.
  - Q3: cutoff는 LP env Cutoff에서 `ws+0x3b8`로 들어가고, 재풀이 결과와 모순이 없다.
- M2-03d-4에 주는 영향: 두 원본 사례 모두 일반 B&B 노드가 아니다(RINS 고정 LP, sub-MIP 루트). Rust에는 RINS와 sub-MIP가 없다.
  - 따라서 `bnb.rs` 실제 경로 연결은 이 두 사례로 검증할 수 없다.
  - 가능한 L2는 LP 재생이다.
    - nidda: 캡처한 LP, 노드 basis, method 0, cutoff를 Rust LP에 넣고 상태 6을 비교한다. 첫 LP 12의 재현은 별도 문제다.
    - gmu: 재풀이가 `0c3d0d`의 여러 LP 호출이다. 단일 LP로 비교할 수 있는 것은 k=3(노드 LP, given basis, method 1 → 2)뿐이다.
  - 실제 경로를 검증하려면 일반 B&B 노드에서 일어나는 자연 재풀이 사례가 따로 필요하다. 03c-1 스캔에서는 이 두 사례만 찾았다.
- 남은 한계:
  - 62행의 cut 종류·생성 순서.
  - `8e9f93`의 Cutoff 계산 실측.
  - gmu sub-MIP를 띄운 휴리스틱.
  - FUZZY backtrace의 간접 호출 구간. nidda 연쇄는 정적 간선과 일치한다.
  - 첫 LP 상태 12의 원인(LU·pricing).
  - `5bbf7f` 순번은 이 설정의 실행 순서 라벨이다.
- 작업 위치: branch `work/m2-03d-3`. 결과는 위 commit들과 문서 갱신 commit에 있다. 미커밋 변경은 없다. 검사 통과 뒤 main에 `--no-ff`로 병합한다.
- 교훈: [원본 사실](../lessons/original.md)(nidda 문맥, gmu sub-MIP, 노드 LP 행 정규화, 판단 시점 `D` scale), [도구](../lessons/tooling.md)(`this.depth`, 포인터 라벨, 공개 API 덤프)에 추가했다. 미검증 가설 두 항목은 확인 결과로 바꿨다.
