# M2-03c-1 검증 계약과 결과

## 1. 범위와 주장

- 조사·읽기 전용 정상 실행 오라클 구축. 선형 호출자 상태 5→12, 즉시 재풀이, 상태 3 대체 분기와 새 자식 상태 10을 관찰한다. Rust 구현은 제외한다.
- 선행 M2-03b-1, 기준 main `6c8ea6a`, 선행 미커밋 없음. 실제 자식은 부모 포인터·깊이·루트 비동일성을 함께 확인한다. 합성 주입과 루트 로그는 실제 자식 증거가 아니다.
- 성공은 각 새 경계의 실입력·판단·출력·보존 상태 및 사건 순서를 동일 조건의 독립 원본 프로세스 3회에서 비교하고 비교기의 의미 있는 오류 검출까지 확인해야 한다. 재현하지 못한 분기는 실패 사례를 보존하고 미완료로 남긴다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 Linux x86-64, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- `../re/decomp/c/004f0e10_PRIVATE00000000005ce67b.c`, `005320c0_PRIVATE0000000000627f3e.c`, `005177f0_PRIVATE00000000005ff358.c`, `00518170_PRIVATE0000000000600d25.c`, `00517690_PRIVATE00000000005ff14d.c`, `00519b10_PRIVATE000000000060145b.c`; 03b-1의 ELF `0x4f1a98`·`0x4f1aca`, `0x5320d1`–`0x5320f3`, `0x532b62`–`0x532bb7`의 어셈블리를 확인한다.
- 시작은 `5ce67b` 진입, 끝은 분기·자식 생성 후 후속 노드 처리. 입력은 모델과 파라미터, 노드 부모 `N+0x10`, `I=N+0x18` 간접 포인터의 깊이 `I+0x2c`, 루트 동일성, 상태·도메인·basis·캐시·카운터·재풀이 설정과 호출 문맥이다. 출력은 상태 변경, 반환, 자식 관계, 재풀이 및 분기 사건 순서와 보존 상태다. 원시 주소는 관계를 나타낼 때만 사용한다.

## 3. 비교 계약

| 필드·사건 | 의미·순서 | 규칙 | 방법 |
|---|---|---|---|
| 모델·설정·원본 식별 | 동일 입력과 ABI | SHA-256 정확 비교 | manifest |
| 노드·부모·깊이·루트 | 실제 자식 식별 | 관계·정수 정확 비교 | 정상 실행 읽기 훅 |
| 상태 5→12·재풀이·3·분기·새 10 | 사건 전후와 순서 | 상태·순서 정확 비교 | 전후 읽기 훅 |
| domain·basis·카운터·설정 | 입력 및 보존 상태 | 정수 정확·실수 비트 비교 | 사건 전후 캡처 |

NaN은 유효 사례로 가정하지 않는다. 무한대와 ±0은 비트로 구분한다. 타이머·work·병렬 순서는 동등성 대상이 아니며 Threads=1과 벽시계 제한 없는 조건을 기본으로 한다. 독립 실행 3회에서 주소 자체가 아닌 관계와 값 비교. 관측 불가능한 필드를 생략하고 완료로 판정하지 않는다.

## 4. 사례와 원본 자료

실제 선형 MIP의 루트 무계, 유한 루트·자식, 목표인 자식 LP 무계 및 상태 3→새 상태 10을 구분한다. 모델·설정과 각 실행의 해시와 실행 증거를 manifest에 남긴다. 동점·수치 경계는 목표 경로를 얻은 뒤 별도 관측하며 미실행일 때 명시한다. 합성 상태 주입은 정상 실행과 별개로 기록한다. 외부 바이너리·Node Frida·Python gurobipy와 라이선스가 필요하다.

## 5. 재실행 명령

작업 worktree 루트에서 순차 실행한다. 저장 파일을 덮어쓰므로 검토 시에는 출력 경로를 별도 임시 디렉터리로 바꿀 수 있다.

```bash
for model in fractional_branch root_unbounded; do
  for run in 1 2 3; do
    ./tools/gurun.sh node tools/mip_linear_child_trace.js "bench/mip_linear_child/$model.lp" "bench/oracle/mip_linear_child_trace/${model}_${run}.json" || exit
  done
done
python3 tools/mip_linear_child_check.py --self-test
python3 tools/mip_deferred_gate_oracle.py --check bench/oracle/mip_deferred_gate.json
node --check tools/mip_linear_child_trace.js
python3 -m py_compile tools/mip_linear_child_check.py
git diff --check
```

저장 비교는 원본 재캡처가 아니다. 솔버 변경이 없으므로 Rust·Python 확장 통합 검사는 제외한다.

## 6. 실행 결과

- 환경: Node 22.23.4, Node Frida 17.23.2, Python 3.13.12, GNU objdump 2.42. 바이너리·C 6개·기존 어셈블리·도구·입력·캡처 해시는 [manifest](../../bench/oracle/mip_linear_child_manifest.json)이다. `bench/oracle/mip_unbounded_reclassification.s`의 명령 경계를 확인했다. 모델별 Threads=1, Presolve=0, Cuts=0, Heuristics=0, Seed=0, NodeLimit=100이며 두 입력 모두 제한 전에 종료했다. 주입 없이 매번 별도 원본 프로세스 하나를 실행했다.
- `fractional_branch.lp`: 세 독립 실행 모두 종료 상태 2, NodeCount 3, 각 90개 사건이 정확히 일치. 루트 0의 자식 1·2는 부모 포인터가 루트, 깊이 1, 루트 비동일이다. 7회 solve 중 루트 자식 2회이며 세 열의 domain 길이는 각각 3이다. 자식 2의 두 번째 하한이 루트의 -0에서 1로 바뀌었고 자식 1은 두 번째 상한이 1에서 +0으로 바뀌었다. 이 차이에는 분기 후 전파가 포함될 수 있다. 상태 2 일반 분기이며 무계 자식이 아니다.
- `root_unbounded.lp`: 세 독립 실행 모두 종료 상태 5, NodeCount 1, 각 12개 사건 일치. 정규화 전 루트 LP 상태 5·캐시 -1, 후 상태 5·캐시 5다. 자식 사건은 없다. 루트 무계를 자식 무계로 계산하지 않는다.
- 두 사례에서 목표 5→12, 즉시 수치 재풀이, 상태 3의 대체 분기, 새 자식 상태 10은 **0회** 관측했다. 두 입력의 실패 사례를 그대로 보존한다. 자연 재현을 보장하는 입력은 찾지 못했다. 정확한 수학에서 유한한 루트 LP의 자식은 더 좁은 feasible set이므로 그 자체로 무계가 될 수 없다. 실제 원본에서 이 경로가 나타나려면 수치 판정이나 relaxation 변경 등 별도 원인과 그 실행 증거가 필요하다. 합성 주입은 하지 않았다.
- 입력·출력 추적: 함수별 call 번호, node ID·부모 ID·깊이·루트 비교, domain 64비트 표현, 상태·캐시·MIP 상태·top+0xd4, LP 상태·Presolve·Method·ConcurrentMethod·Threads, workspace vstat와 일부 basis 포인터 존재를 저장한다. `solve`·`branch` 반환은 이전 N을 역참조하지 않고 유효한 holder를 다시 읽으며, holder 교체 여부를 저장한다. gate 반환은 노드를 역참조하지 않는다. 정규화 전후와 함수 입출력의 사건 순서를 보존한다.
- **계약 미충족**: basis 포인터의 존재와 LP workspace vstat만으로 저장 basis·factor·norm 전체 내용을 비교하거나 보존을 입증할 수 없다. 재풀이의 선택 입력 전체, 노드별 RNG·카운터 전체, 분기 변수·분할값, holder가 교체된 경로의 수명, 상태 3·10·재등록 전후와 이후 자식은 캡처·검증되지 않았다. 각 목표 사건의 입력·출력·보존 상태와 세 원본 반복은 성립하지 않는다. 이번 두 사례의 정확 재현은 목표 경계 재현이 아니다.
- 저장 비교기는 모든 필드와 사건 순서를 비교하고 일반 자식의 부모·깊이·루트 관계 및 세 열 domain·bound 변화를 검사한다. 배열 길이·bound 비트·부모 ID·사건 순서 변조 4개를 각각 거부했다. 첫 차이의 사례·JSON 경로(사건 인덱스 포함)·기대·실제·정확 비교 규칙을 출력했다. 이 오류 검출은 관측된 기준 기록에만 해당한다.
- 이전 합성 게이트의 저장 재생은 통과했으나 새 원본 캡처는 아니다. Rust·Python 확장 통합 검사는 솔버·공개 API 구현을 변경하지 않아 실행하지 않았다. 타이머·work·성능·동점·오류 분기 및 LP 수치 동등성은 실행하지 않았다.

## 7. 판정과 인수인계

- 작업 수행: **미완료**. 정상 실행 읽기 훅과 두 대조 입력, 3회 원본 반복 및 저장 비교기는 확보했다. 목표 사건 재현과 전 상태 캡처는 실패·미검증으로 남는다. Rust 동등성은 미검증이다.
- 다음: M2-03c-1을 계속한다. 원본의 수치 상태 5 발생 조건과 relaxation 변경 여부부터 조사하고 이를 자연 실행으로 관찰할 수 있는 입력을 탐색한다. 합성 주입이 필요하면 별도 실험으로 분리한다. 상태 3→새 10의 입력·출력, 저장 basis·factor·norm·도메인·후속 게이트를 전후 캡처한 뒤 3회 반복과 오류 검출을 수행한다.
- branch `work/m2-03c-1`, main 미병합. 선행 미커밋 변경 없음. 사용자 인계 지시에 따라 M2-03c-1·shared 점유와 worktree를 유지한다. 이 파일이 포함된 작업 커밋은 `git log -1 --oneline`으로 확인한다.

## 8. 수정 착수 전 추가 계약 (2026-11-10)

기준은 기존 commit `3e9fc00`이며 main `1247ee4`에는 검토 인계 문서만 추가됐다. 기존 두 입력과 여섯 캡처는 바꾸지 않는다. 자연 실행과 합성 주입은 별도 자료로 저장하고 합성 결과는 자연 목표 달성에 포함하지 않는다.

- 자연 실험: 기존 세 정수 변수 모델에 범위가 있는 연속 변수를 더해 같은 단일 스레드 설정으로 루트와 자식의 LP 상태·분기를 관찰한다. 상태 5가 나오지 않으면 실패 결과를 보존한다. 유한한 루트의 자식은 수학적으로 유한하지만 미고정 연속 변수는 대체 분기 후보가 된다.
- 합성 실험 A: 실제 비루트 자식의 `5ce67b` 내부 ELF `0x4f1a98`에서 `r13+0x38` 결과 상태가 2이고 루트 캐시가 2일 때만 결과 상태를 2→5로 바꾼다. 원래 LP와 basis는 변경하지 않는다. `0x4f1aca`에서 재분류 결과와 재풀이의 입출력·파라미터 복구를 읽는다. LP 상태와 실제 모델·basis의 불일치를 의도적으로 만들므로 실제 자식 무계의 증거로 쓰지 않는다.
- 합성 실험 B: 연속 변수를 추가한 모델의 루트 `627f3e` 진입에서 노드 상태를 2→3으로 바꾸고 정수 변수 domain의 상한을 현재 하한으로 맞춘다. 연속 변수 domain은 바꾸지 않는다. 변경 전후 값을 열별 비트로 기록한다. 정수 후보를 인위적으로 없애므로 실제 자연 분기가 아니며 LP 해와 domain이 모순될 수 있다. 새 자식 상태 10과 gate가 발생하면 부모·깊이·루트·순서로 구분한다. 두 실험은 독립 프로세스로 격리해 실패와 비정상 종료도 보존한다.
- 각 실험의 바이너리·모델·도구 해시와 파라미터를 검증하고 동일 조건의 독립 원본 프로세스 3회에서 새 경계를 확인한다. 노드·domain·LP 상태·설정·확인 가능한 basis 값과 사건 순서를 비교한다. 미확인 factor·norm·분기값은 누락을 인정하고 목표 합격 조건에서 제외하지 않는다.
- 비교기는 `controls`와 `acceptance`를 구분한다. 대조군은 모델·필수 사건·식별자를 검증한 다음 반복을 비교한다. 목표 검사는 자연 자식 5→12·재풀이·상태 3 분기·새 자식 10·후속 gate와 필수 상태가 빠지면 종료 코드 1을 반환한다. 정상·합성·미검증 수를 출력한다. 모든 캡처의 바이너리 해시를 0으로 바꾸고 루트 무계 세 캡처의 사건 배열을 비우는 독립 검토자의 재현을 반드시 거부한다. 해시·빈 사건·값·순서·필수 필드 변조도 각각 검사한다.
- Rust·Python 확장을 바꾸지 않으므로 통합 빌드는 대상이 아니다. 자연 목표 사건이 없거나 필요한 basis·factor·norm·분기 입력이 빠지면 스프린트 미완료, 구현 동등성 미검증을 유지한다.

## 9. 수정 결과 (2026-11-10, 작업자 중단 후 인계 커밋)

Terra 작업자는 §8 실험 도중 사용 한도로 중단했다. 결과 정리 없이 남은 미커밋 변경을 owner `claude-m2-03c1-takeover-20261110`이 점유를 인계받아 그대로 커밋했다. 코드·캡처 내용은 바꾸지 않았고 원본 재캡처도 하지 않았다.

- 자연 실험 `continuous_candidate.lp`: 독립 원본 3회 일치. 자식 LP 상태 5, 상태 3 분기, 새 자식 상태 10은 모두 0회다.
- 합성 A(재분류 주입): 3회 모두 자식 결과 2→주입 5→원본 12→simplex 재풀이 순서이고, 관측한 Presolve·Method·ConcurrentMethod·Threads가 같은 solve 반환에서 복구됐다. 합성 경계 관찰일 뿐 자연 무계 증거가 아니다.
- 합성 B(상태 3·domain 주입): 3회 모두 주입 직후 또는 자식 생성 진입 뒤 프로세스가 비정상 종료했다. 주입 상태가 원본 불변식을 깨뜨린 것으로 보며 상태 10·gate 증거로 쓰지 않는다. `controls`는 주입 시점까지의 사건 일치만 확인한다. 이는 **실패 재현**이지 대조군 통과가 아니다.
- `earlier_*`·`branch_*`·`reclass_1` 캡처는 도구 수정 전 탐색 기록이다. 비교 기준이 아니며 실패 이력으로만 보존한다.
- factor·norm 전체, holder 교체 경로의 수명, 동일 노드에서 재분류·재풀이·자식·gate의 연결은 미캡처다.

검증 명령(작업 worktree 루트, Gurobi 미실행):

```bash
python3 tools/mip_linear_child_accept.py controls          # 기대 종료코드 0
python3 tools/mip_linear_child_accept.py --self-test controls  # 0, 변조 7종 거부
python3 tools/mip_linear_child_accept.py acceptance        # 기대 종료코드 1 (자연 목표 0회)
python3 tools/mip_linear_child_check.py --self-test        # 0
```

원본 재캡처: `for mode in natural inject_reclass inject_branch; do for run in 1 2 3; do ./tools/gurun.sh timeout 30s node tools/mip_linear_child_experiment.js "$mode" bench/mip_linear_child/continuous_candidate.lp "<출력>/${mode}_${run}.json"; done; done`. 한 번에 한 프로세스만 실행한다.

판정: **미완료**. 정확한 산술에서 유한 루트의 자식 LP는 무계가 될 수 없으므로, 다음 작업은 자연 입력 탐색을 반복하기 전에 원본에서 자식 LP 상태 5가 나오는 수치 경로(ray 판정·허용오차·relaxation 변경)가 있는지부터 C·asm으로 답한다. 경로가 없으면 M2-03c-1 목표를 합성 경계 오라클로 재정의할지 사용자에게 확인한다.

## 10. 도달 가능성 판정과 자연 경로 스캔 (2026-11-10, owner `claude-m2-03c1-takeover-20261110`)

[위임 절차](../handoff/README.md) §5에 따라 자연 입력을 더 찾기 전에 목표에 도달할 수 있는지부터 판정했다.

**정적 판정 (`5ce67b` C:735–780)**
- 5→12 재분류는 루트 캐시 `M+0xa8==2`일 때만 일어난다. 캐시는 현재 노드가 루트일 때만 갱신된다. 첫 루트 LP가 상태 5이면 캐시 5가 되고, D+0x18·0x1c·0x258·0x1a8가 모두 0이면 `M+0=5`로 종료한다. 이 네 값은 2차·쌍선형·비선형 요소 개수라서 선형 MIP에서는 모두 0이다.
- 선형 MIP의 자식 LP는 루트 LP 행에 bound를 더한 부분 문제다. 따라서 루트 LP가 최적(2)이면 정확한 산술에서 자식 LP는 무계가 될 수 없다. 자식 상태 5와 캐시 2의 조합은 수치 오류일 때만 생기고, 원본이 이를 12(수치)로 바꾸는 것은 그 판단과 맞는다. **자연 5→12 재분류는 정상(정확) 산술에서는 불가능하고 이번 스캔에서도 관측되지 않았다.** 다만 수치 오류로 발생할 가능성은 배제하지 않는다. 그래서 acceptance에서는 이 경계를 합성 시험으로 검증한다.
- 재풀이 블록(`0x4f53a8`, C:767–773)은 재분류와 별개로 LP가 원래 반환한 상태 12에서도 들어간다. 조건은 캐시 2이거나 현재 노드가 루트인 경우다. 즉시 상태 3으로 가는 조건 `GURO_PAR_FEASHELPER==-2`는 공개 파라미터 범위(-1..1, `docs/m2/m2_param_offsets.tsv:274`) 밖이라 사용자 설정으로 만들 수 없다.

**자연 스캔 (읽기 전용, 주입 없음)**
- 도구: `tools/mip_linear_child_scan.js`. 정규화 전후(`0x4f1a98`·`0x4f1aca`)의 (루트 여부, 캐시, LP 상태), 재풀이 진입 `0x4f53a8`, 노드 상태 setter `0x5180f0`의 값을 센다. 대조군 `fractional_branch`·`root_unbounded`에서 기존 캡처와 같은 상태 흐름을 확인했다.
- 입력: `../re/bench/data/miplib` 38개 × 2개 설정(`tools/mip_linear_child_scan.sh`). default는 `Threads=1 Seed=0 TimeLimit=20 NodeLimit=20000`이고, stress는 여기에 `Presolve=0 ScaleFlag=0 Quad=0 NumericFocus=0 MarkowitzTol=1e-4`를 더했다. 76회 모두 정상 종료했다. 결과는 `bench/oracle/mip_linear_child_scan/{default,stress}/`, 해시는 `bench/oracle/mip_linear_child_scan_manifest.json`이다.
- 비루트 + 캐시 2 + LP 상태 5: **0회**(76회 실행, 약 160만 노드 LP). 상태 10 setter도 0회다.
- 재현 3회(`tools/mip_linear_child_scan_repro.sh`, `.../repro/`):

| 입력·설정 | 관측 | 스캔 1회 + 독립 재실행 3회 |
|---|---|---|
| `neos-3754480-nidda` stress | **비루트 자식 LP가 원래부터 상태 12**(캐시 2), 정규화 뒤 12, 재풀이 1회. 상태 3 setter 없음 | 4회 모두 전체 카운트 동일(NodeLimit 종료) |
| `gmu-35-50` default | 루트 첫 LP 상태 12(캐시 -1→12), 재풀이 1회 | 4회 모두 전체 카운트 동일(NodeLimit 종료) |
| `neos-4338804-snowy` default | 노드 상태 3 setter 1회, 재풀이와 상태 12는 없음 | 4회 모두 1회. TimeLimit 종료라 노드 수는 다름. 3을 쓰는 호출자는 미확인 |

**판정과 남은 일**
- 자연 경로로 재풀이 블록 진입은 성립한다(자식 1건, 루트 1건, 각 4회 재현). 재분류 5→12와 상태 3→새 자식 10은 자연 실행에서 미관측이다. 재분류는 정상 산술에서 불가능하고 이번 스캔에서 미관측이다.
- 이 스캔은 횟수만 센다. 재풀이 입출력·파라미터 복구·basis 내용은 캡처하지 않았다. 따라서 `mip_linear_child_accept.py acceptance`는 그대로 종료코드 1이다.
- 다음: (1) nidda/stress 자식 사례에 상세 훅(`mip_linear_child_experiment.js`의 natural 모드를 파라미터 인자로 확장)을 붙여 재풀이 입력·출력·복구를 3회 캡처한다. (2) snowy의 상태 3 setter 호출자를 반환 주소로 특정한다. (3) acceptance 계약의 "자연 5→12"를 "자연 12 재풀이 + 합성 5→12 경계"로 바꿀지 사용자가 결정한다.

## 11. 수정 acceptance 결과 (2026-11-10, 사용자 결정 반영)

사용자 결정: 자연 경로는 상태 12에서 재풀이로 들어가는 사례로 검증한다. 5→12 재분류는 합성 경계로 검증한다. 자연 5→12는 "정상 산술에서 불가능, 이번 스캔에서 미관측"으로 기록하고 절대 발생하지 않는다고 단정하지 않는다. 재풀이 전후 입출력·basis·파라미터 복구를 3회 검증하고 상태 3의 호출자를 추적한다.

**도구와 실행**
- 캡처: `tools/mip_linear_child_retry.js` (읽기 전용, 주입 없음)
  - `retry` 모드: `0x4f53a8`에 도달한 `5ce67b` solve 하나의 사건을 순서대로 저장한다. solve 진입·반환, 정규화 전후, 재풀이 블록 안의 `0c3d0d`·`cda5b`·`1cb7e0` 입출력, finish, setter다. 반환 시 노드는 holder를 다시 읽어 얻는다.
  - `set3` 모드: `5ff754(node,3)` 호출이 일어난 `682037` 블록 전체를 저장한다. 나머지 `682037` 호출은 (노드 상태, LP 상태) 횟수만 센다. 표시한 노드에서 상태 3인 첫 `627f3e` 분기와 그 자식 생성을 저장하고, 상태 10 setter도 센다. 노드 id는 재사용되므로 id 일치만으로 같은 노드라고 보지 않는다.
- 재실행: `tools/mip_linear_child_retry_runs.sh bench/oracle/mip_linear_child_retry`. Gurobi를 한 프로세스씩 9회 실행하고 약 20분 걸린다.
- 판정: `python3 tools/mip_linear_child_retry_check.py --self-test`. 기대 종료코드는 0이다. manifest는 `bench/oracle/mip_linear_child_retry_manifest.json`(파일 14개, 외부 모델 3개)이다.
- 설정: Threads=1, Seed=0. **TimeLimit은 쓰지 않는다.** TimeLimit을 걸면 훅 부하에 따라 궤적이 바뀌어 snowy의 상태 3 위치가 달라졌다. 이 실패 캡처는 비교 기준이 아니다. 원본 3개(합계 약 640MB)는 저장소에 넣지 않고 `.../timelimit_tainted/summary.json`에 요약만 보존했다. 요약에는 원본 SHA-256·크기·사건 수·상태 3 위치(1회차만 노드 4238, TimeLimit 직전)·첫 분기 사건 11780이 들어 있다.

**결과 (각 3회 독립 원본 프로세스, 사건 전체 일치)**

| 사례 | 경로 | 입출력 | 파라미터 | basis |
|---|---|---|---|---|
| `neos-3754480-nidda` stress, NodeLimit 6000 | 비루트 노드 8287(캐시 2)의 LP가 원래부터 12 → 정규화 뒤 12 → 재풀이. `cda5b` 직접 호출, `0c3d0d` 없음 | 재풀이 LP 상태 12→6, 노드 상태 0→5 | Presolve 0·Method 1·ConcurrentMethod -1·Threads 1, 진입·재풀이·반환 동일 | 입력 vbasis 253·cbasis 464, basic 464개(행 수와 같음). 실패 시점 header와 다른 basis로 출발한다. header는 전후 모두 유효하고 basic 집합 40칸이 바뀌었다 |
| `gmu-35-50` default, NodeLimit 20000 | 루트 첫 LP 12(캐시 -1→12) → 재풀이. workspace 해제 후 `0c3d0d` | 첫 `0c3d0d`(Presolve 1, Threads 1) 안에서 `cda5b` 2회, 상태 2. 둘째 `0c3d0d`(Presolve -1, Method 0) 상태 2. 노드 상태 2 | 반환 시 Presolve -1·Method 1로 복구됨 | 재풀이 진입 시 workspace가 없어 입력 basis 배열이 없다(C와 일치). header는 전후 유효하고 basic 집합 38칸이 바뀌었다 |

- 노드 8287은 루트 포인터(`M+0x790`)와 다르지만 부모 포인터가 null이고 깊이가 0이다. 실제 부모 관계는 미확인이므로 "비루트 노드"로만 기록한다.
- 합성 5→12: 기존 `inject_reclass` 3회(2→주입 5→원본 12→재풀이, 파라미터 복구)가 `mip_linear_child_accept.py controls`에서 통과했다. 자연 실행에서는 미관측이다.

**상태 3 호출자 (`neos-4338804-snowy` default, NodeLimit 20000, 3회 일치)**
- 호출자는 `682037`(file `0x566030`)의 `0x566c48`이다. 근거는 C 567행이다. 호출 경로는 `5ce67b` C:1435/1440이다. 진입 조건은 `5eda00(...)==1`, `local_60==0`, `pdVar23+0x10>0`, `a7f0ad==0`, `115e51!=0`, `ws+0x3f0!=1`이며 각 조건의 의미는 미해석이다.
- `682037`은 env+0x2960을 `2*local_120+100`으로 설정하고 `cda5b`를 부른다. 결과가 최적이 아니면 100으로 바꿔 다시 부른다. 그래도 최적이 아니면 LP 결과 +0x38과 LP +0x64를 12로 쓰고 노드를 3으로 설정한다(C:540–567).
- 관측: 노드 268(부모 211, 깊이 15)은 한도 106→100, 노드 373(부모 21, 깊이 10)은 126→100이었다. 두 번 모두 LP 상태 9이고, 이어서 상태 3이 설정됐다. 노트는 env+0x2960을 IterationLimit로 적지만 관측 상태는 9다. 의미는 단정하지 않는다.
- 이후: 두 노드 모두 상태 3으로 `627f3e`에 들어가 상태 0인 자식 2개를 만들었다(268→270·271, 373→375·376). 상태 10 setter는 0회다.
- 2만 노드 동안 `682037` 호출 횟수: 노드 0·LP 없음 10221, 노드 0·LP 2 20417, 노드 2·LP 2 799, 노드 2·LP 없음 771, 노드 0·LP 7→2 1.

**검사기 오류 검출**: 메모리 변조 14종을 모두 의도한 검사에서 거부했다. 대상은 바이너리 해시, 빈 사건, 파라미터 미복구, 재풀이 후 12 잔존, 비루트→루트, 재풀이 제거, header 무효, workspace 미해제, basic 수, basis 길이, 실행 간 불일치, 파라미터, 상태 3 호출자, 상태 3 제거다. 디스크 캡처 1바이트 변조도 manifest 해시 불일치로 종료코드 1이 됐고, 복원하면 0이다.

**판정**: 수정 계약의 필수 검사는 통과했다(오라클 부분 완료). 다음은 미검증으로 남는다.
- 재풀이 후 basis의 VBasis/CBasis 형식 값. simplex 인자 버퍼는 반환 뒤에도 그대로라서 출력이 아니다. `ws+0x168` header는 음수 코드의 의미를 해석하지 않았다.
- 재풀이 입력 basis의 출처. 노드 저장 basis로 추정하지만 읽지 않았다.
- factor·norm 내용.
- `682037` 진입 predicate의 의미.
- 상태 10 생성: 자연 실행에서 0회다.
- 자연 5→12: 정상 산술에서 불가능하고 이번 스캔에서 미관측이다.

Rust 동등성은 범위 밖이며 미검증이다. 기존 `mip_linear_child_accept.py acceptance`(옛 계약)는 종료코드 1로 남고 이번 계약으로 대체된다.

## 12. 최종 판정과 범위 구분 (병합 시점)

**C 원본 관찰·검증인지 Rust 구현인지**
- 이번 작업의 결과는 모두 **Gurobi 13.0.3 원본 바이너리의 관찰과 검증**이다. 근거는 디컴파일 C(`5ce67b`, `682037`, `5ff754`, `627f3e`)와 Frida 읽기 훅으로 얻은 원본 캡처다. 합성 실험만 원본 메모리에 상태를 주입했다.
- **Rust 솔버(`crates/mip` 등)는 한 줄도 바꾸지 않았다.** Rust의 재풀이·상태 3 처리는 원본과 대조하지 않았고 동등성은 미검증이다. 현재 Rust는 자식 Numeric·Unbounded를 Numeric으로 처리하며 이번 작업으로 바뀌지 않았다. Rust·Python 통합 테스트는 대상이 아니어서 실행하지 않았다.
- 추가한 코드는 원본 관찰용 도구(`tools/mip_linear_child_*.js|py|sh`)와 기준 자료(`bench/`)뿐이다.

**검증 완료 (원본 3회 이상, 검사기 종료코드 0)**
- 자연 상태 12 재풀이: 비루트 노드(nidda)와 루트(gmu)에서 사건 순서, LP·노드 상태 입출력, Presolve·Method·ConcurrentMethod·Threads 복구, 재풀이 입력 basis의 크기와 basic 수, 전후 basis header의 유효성과 변화량을 확인했다. 루트 경로에서는 `0c3d0d` 진입 전 workspace 해제와 Presolve 1·Threads 1 설정을 확인했다.
- 합성 5→12 재분류 경계: 3회.
- 상태 3 호출자: `682037@0x566c48`. 반복 제한 LP 두 번이 비최적이면 상태 3이 되고, 이어지는 `627f3e` 분기와 자식 상태(0)를 확인했다(노드 2개, 3회).

**미검증 (검증 완료로 간주하지 않음)**
- M2-03c-2로 분리: 재풀이 후 VBasis/CBasis 형식의 basis, 재풀이 입력 basis의 출처, factor·norm 동작.
- M2-03c 범위에 남김: 상태 10 생성(자연 0회), `682037` 진입 predicate의 의미, env+0x2960 한도의 의미(노트는 IterationLimit, 관측 LP 상태는 9), 노드 8287의 부모 관계.
- 자연 5→12: 정상 산술에서 불가능하고 이번 스캔(76회)에서 미관측이다. 수치 오류로 발생할 가능성은 배제하지 않는다.
- 옛 계약의 `mip_linear_child_accept.py acceptance`는 종료코드 1로 남는다(대체됨).

**병합 전 전체 재실행**: `tools/mip_linear_child_premerge.sh target/m2-03c-1-premerge` 후 `python3 tools/mip_linear_child_rerun_compare.py target/m2-03c-1-premerge`. 결과는 아래 §13에 기록한다.

## 13. 병합 전 전체 재실행 결과 (2026-11-10)

- 명령: `tools/mip_linear_child_premerge.sh target/m2-03c-1-premerge`를 실행했다. 원본 Gurobi 13.0.3 프로세스 33회를 한 번에 하나씩 돌렸고, 결과는 git이 무시하는 `target/`에 두었다. 이어서 `python3 tools/mip_linear_child_rerun_compare.py target/m2-03c-1-premerge`로 비교했고 **종료코드 0**이다.
- 비교 30건 PASS, FAIL 0건:
  - 대조군 trace 6회: 저장 기록과 JSON 전체 일치
  - 자연·합성 재분류 실험 6회: 전체 일치
  - 합성 상태 3 분기 3회: 비정상 종료와 주입 시점까지의 사건 일치
  - 스캔 재현 nidda·gmu 6회: 카운트 일치
  - 재풀이·상태 3 캡처 9회: 사건·출력·파라미터·해시·`fallback_counts` 일치
- INFO 3건: snowy TimeLimit 스캔에서 상태 3 setter가 2·1·1회였고 노드 수도 달랐다. TimeLimit에 따라 결과가 달라진다는 앞선 판단을 다시 확인했으며 합격 근거로 쓰지 않는다.
- 저장 자료 검사:
  - 종료코드 0: `mip_linear_child_retry_check.py --self-test`(변조 14종 거부), `mip_linear_child_accept.py controls`와 `--self-test controls`, `mip_linear_child_check.py --self-test`, `mip_deferred_gate_oracle.py --check`
  - 종료코드 1(예상): `mip_linear_child_accept.py acceptance`. 대체된 옛 계약이다.
  - scan(88)·retry(12) manifest와 외부 모델 해시가 모두 일치했다. `node --check`, `py_compile`, `bash -n`, `git diff --check`도 통과했다.
- 검증 범위: 원본 동작의 재현성과 저장 기준의 일치만 확인했다. Rust는 변경하지 않았다(`git diff 6c8ea6a -- crates python` 0파일). Rust 동등성, Rust·Python 통합 테스트는 대상이 아니어서 실행하지 않았다.
- 판정: **M2-03c-1 부분 완료**. §12의 미검증 항목은 검증 완료로 간주하지 않는다.
