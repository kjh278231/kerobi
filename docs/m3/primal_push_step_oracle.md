# M3-04b-1 정상 primal push 한 단계 오라클

## 1. 착수 검증 계약

- 종류: 정상 실행 오라클 구축. 기준선 main `297d44b`, branch `work/m3-04b-1`, 미커밋 선행 변경 없음.
- 목표: 읽기 전용 Frida 훅으로 double `134a87`의 진입→후보 공급→ratio→pivot 또는 bound 이동→다음 후보 공급과 반환을 연결한다. primal-first와 dual-first를 각각 독립 프로세스 3회 실행하며, 각 순서에서 실제 pivot·flip을 최소 한 건씩 요구한다.
- 원본: Gurobi 13.0.3 ELF SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. C `001c0070_PRIVATE0000000000134a87.c`, `001c5310_PRIVATE000000000025615c.c`, `000ef960_PRIVATE0000000000055f68.c`, `000eefa0_PRIVATE0000000000054eef.c`, `000f15c0_PRIVATE000000000005b257.c`, 호출자 `001bc5d0_PRIVATE000000000013f716.c`를 사용한다. 경로는 저장소 옆 `re/decomp/c/` 기준이다.
- 시작·종료: push 함수 진입·반환, 후보 공급자 전후, ratio 전후, ELF `0x1c2994` bound 이동 직전·`0x1c2ab5` pivot 승인 직전부터 다음 공급자 진입까지. 같은 push 호출·후보 순번으로 사건을 연결한다. 내부 훅의 S는 r14이며 ELF로 확인했다.
- 입력: workspace 차원·CSC·목적·rhs·bound·vstat·basis header, x/slack 연속 배열, basic 값·폭·표시, FTRAN 희소 벡터, 후보 순서·소비 위치·실패 횟수·지원 집합, factor 요청·반복·허용오차를 의미 기반으로 캡처한다. 주소를 비교하지 않는다. 초기화 전 scratch·출력값은 읽지 않고 해당 수명 경계에서만 기록한다.
- 반환: C 반환·workspace 상태·retry·signed remaining·condition과 x/slack을 각각 저장한다. ratio 선택은 sparse 위치와 행을 구분하고 theta·pivot을 저장한다.
- 비교: 정수·후보 순서·사건 순서·상태·별칭은 정확 비교, double은 little-endian 8바이트 hex로 ±0까지 비트 비교한다. 초기화되지 않은 scratch나 메모리 padding은 대상이 아니다. NaN은 허용하지 않는다. 내부 sentinel은 비트 비교한다.
- 환경: Threads=1, Seed=0, Presolve=0, Method=2, Crossover=1 또는 3. 다른 파라미터와 시작값은 캡처에 고정한다. 원본 allocator·factor 전체와 RNG 내부 상태의 독립 재생, x87·실패·중단·batch·다른 모델·Rust 구현·시간 및 전체 work 패리티는 제외한다. factor 내부를 복원하지 못해도 캡처를 factor 재생 오라클로 보고하지 않는다.
- 사례: 선행 소형 LP와 필요한 실제 LP를 사용해 목표 도달을 확인한다. 입력 탐색의 실패·목표 0건도 history에 보존한다. Crossover=0 대조군은 목표 0건을 별도 판정한다. 동점·퇴화·허용오차 경계의 강제 입력은 이번 정상 실행 범위에 포함하지 않는다.
- 합격: 같은 설정 원본 독립 3회에서 모든 계약 필드가 일치하고 각 순서에 pivot·flip이 존재한다. 목표 0건·핵심 상태 누락·합성 출처·바이너리/입력 해시 변조를 CLI에서 거부한다. 완성된 검사기는 실제 캡처 변조로 검사한다.
- 회귀: 기존 ratio·crossover entry 저장 검사를 실행한다. 솔버·Python API 변경이 없으므로 Rust·확장 재빌드는 대상이 아니다. 명령·입력·도구·소스·캡처 해시와 실제 결과는 아래에 추가한다.

## 2. 탐색 이력

목표 미달 탐색은 `history/`에 보존했으며 합격 근거로 쓰지 않는다. `small-primal.json.failed`는 다른 세션의 Gurobi 잠금으로 실패한 첫 실행이다. `small-primal-2.json`은 정상 push 반환이지만 후보가 0건이다. `face-primal-1.json`은 pivot 1건·flip 0건이다. `interior-primal-1.json`과 `interior-dual-1.json`은 이전 훅(`step:applied` 없음)으로 얻은 pivot 7건·flip 2건 캡처다. 현재 manifest의 agent 해시와 다르므로 검사기가 거부한다.

## 3. 캡처 경계 보완

`interior-primal-1.json`의 5번째 후보에서 pivot 후 factor 갱신이 header `[4,14,18,16,15,19,6,8]`을 `[14,18,16,15,19,8,4,6]`으로 재배열했다. 다음 후보 직전의 header에 교환 직후 행 위치를 그대로 요구한 초안 검사는 이 사례를 거부했다.

C `134a87` 1592–1614줄과 2084–2224줄, ELF `0x1c2994`–`0x1c3004`를 확인했다. flip 경로는 `LAB_002c2994`에서 `LAB_002c3004`로 이어지고, pivot 경로는 header 교환(`local_f0[row]=enter`)·`S+0x2f8=row`·vstat 갱신 뒤 `goto LAB_002c3004`로 합류한다. objdump상 `0x1c3004`(`cmp 0x1f20(%rsp),%rbx`)로 가는 jmp 9개는 모두 `0x1c2994`–`0x1c3004` 안에 있고, 이 구간에서 r14에 쓰는 명령은 없다. 그래서 `0x1c3004`에 교환 직후·rhs 보정 전 사건 `step:applied`를 추가했다. 검사기는 이 경계에서 header 한 위치의 교환, `selected.leave`의 sparse 위치→행 변환, vstat와 반복 카운터 +1을 검사한다. 다음 공급자 직전 `step:leave`에서는 factor 처리 뒤 전체 순서를 그대로 비트 비교하고 basis 집합 보존을 검사한다. 순서 차이를 정규화해 지우지 않는다.

새 훅의 비간섭: `primal-1.json`·`dual-1.json`에서 `step:applied`만 빼면 이전 훅의 `interior-primal-1.json`(78건)·`interior-dual-1.json`(76건)과 사건 JSON 전체가 같다.

## 4. 재실행 명령

worktree 루트에서 실행한다. Gurobi는 한 번에 한 프로세스만 실행한다.

```bash
D=bench/oracle/m3_primal_push_step
for k in primal dual control; do for i in 1 2 3; do
  tools/gurun.sh node tools/primal_push_step_capture.js $k $D/interior.lp $D/$k-$i.json
done; done
python3 tools/primal_push_step_check.py $D/primal-{1,2,3}.json
python3 tools/primal_push_step_check.py $D/dual-{1,2,3}.json
python3 tools/primal_push_step_check.py --control $D/control-{1,2,3}.json
python3 tools/primal_push_step_check.py $D/primal-{1,2,3}.json --mutation $D/primal-mutation.json
python3 tools/primal_push_step_check.py $D/dual-{1,2,3}.json --mutation $D/dual-mutation.json
```

캡처 도구와 `--mutation`은 기존 파일을 덮어쓰지 않으므로 재실행할 때는 새 경로를 쓴다.

## 5. 실행 결과

- 입력: `interior.lp`(8행×12열, 목적 0) SHA-256 `e62f130897e3149d77148ecf0799838a056004608c61c9ef58baadc9609009d0`. 원본 ELF는 §1의 해시다.
- 도구: agent `9dd25ce04d140527b10dfc97a35ed961e7569565e9de01eb830e3c7b8ba8b71e`, capture `3e2ed3c80bd971df64de582b7e0adf29591e239f027ec0abd66b762aeb12c458`, check `21546d10a4de9a703f9181ad28356ea745b2530683da0813748e033884cedab9`. 원본 C 6개 해시와 사례별 사건 해시는 [manifest](../../bench/oracle/m3_primal_push_step/manifest.json)에 있다.
- primal-first(Crossover=3): 독립 3회 모두 성공, 사건 87건, push 1회·pivot 7건·flip 2건. 세 실행의 사건 해시가 같다. 검사기 종료 0.
- dual-first(Crossover=1): 독립 3회 모두 성공, 사건 85건, push 1회·pivot 7건·flip 2건. 사건 해시가 같다. 검사기 종료 0.
- 대조군(Crossover=0): 독립 3회 모두 push 0건, VBasis·CBasis 없음, status 2. `--control` 종료 0.
- 9단계 모두 반복 카운터가 정확히 1 증가했다. ratio 9건은 모두 feasible 경로이고 batch는 0이다.
- 변조: 실제 primal·dual 캡처 각각에 대해 15종(해시 4종, 사건 비움, 합성 표시, 상태 누락, pivot 0건, flip 0건, ratio theta, basis header, remaining, `step:applied` 누락, 순서 종류, 중복 실행)이 모두 종료 1로 거부됐다. 기록은 `primal-mutation.json`, `dual-mutation.json`이다. 일부 거부 사유는 메시지 없는 assert이며 거부된 검사 위치는 출력하지 않는다.
- 교차 거부: primal 캡처를 `--control`로 검사하면 종료 1, control 캡처를 목표 검사하면 "목표 0회"로 종료 1, 이전 agent의 history 캡처를 섞으면 "agent_sha256"으로 종료 1이다.
- 회귀: `python3 tools/primal_push_ratio_oracle.py --check bench/oracle/m3_primal_push/ratio-{1,2,3}.json --selftest` 종료 0. `crossover_entry_check.py` pdhg·barrier·warm 각 3회와 `--control` 종료 0. `node --check` 두 JS, `py_compile`, `git diff --check` 통과. 솔버·Python API 변경이 없어 Cargo·확장 재빌드는 실행하지 않았다.

## 6. 판정과 한계

오라클 구축 계약을 충족했다. 같은 설정 원본 독립 3회에서 계약 필드가 비트 단위로 일치했고 두 순서 모두 pivot·flip이 있다. 목표 0건·상태 누락·합성 출처·해시 변조는 CLI에서 거부된다.

한계:

- 생성한 LP 하나(8×12, 목적 0)의 지정 설정만 확인했다. 다른 모델·파라미터는 미검증이다.
- primal-first와 dual-first의 primal push는 후보·pivot·flip 순서가 같다. 차이는 반복 카운터 오프셋, ratio work, 진입 factor 요청, 최종 x·slack이다. 서로 다른 단계 순서의 궤적은 아니다. dual push(`M3-04d`) 내부는 훅하지 않았다.
- infeasible ratio(`054eef`)·batch·x87 경로는 실행되지 않았다.
- factor 내부는 재생하지 않으며 이 캡처를 factor 재생 오라클로 쓰지 않는다.
- Rust 구현·동등성은 이번 범위가 아니다. 시간·work 패리티도 미검증이다.
- 검사기 실패 메시지가 첫 차이의 필드·값을 모두 출력하지는 않는다.

다음은 M3-04c다. 이 캡처의 pivot 한 건 또는 flip 한 건 중 한 경계를 골라 Rust와 비교한다.

## 7. 교훈

`docs/lessons/`의 original·tooling에 `0x1c3004` 합류 지점과 factor 재배열 경계, 잠금 재시도·사용량 중단 인계를 기록했다.
