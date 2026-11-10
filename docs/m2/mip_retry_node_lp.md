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

- 원본 캡처: `tools/mip_retry_node_lp_runs.sh bench/oracle/mip_retry_node_lp`. Gurobi를 6회, 한 프로세스씩 실행한다.
- 저장 검사: `python3 tools/mip_retry_node_lp_check.py --self-test`. 기대 종료코드는 0이다.
- 회귀: 솔버 코드를 바꾸지 않는다. `python3 tools/mip_retry_basis_check.py --self-test`와 `python3 tools/mip_retry_rust_expect.py --self-test`가 계속 0이어야 한다.

## 6. 실행 결과

(실행 후 작성)

## 7. 판정과 인수인계

(실행 후 작성)
