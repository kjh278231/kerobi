# M3-04c primal push feasible ratio Rust 구현

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다.

## 1. 범위와 주장

- 작업 종류: 구현. 기준선 main `98fe1ce`(M3-04b-1 병합), branch `work/m3-04c`, 미커밋 선행 변경 없음.
- 완료할 동작: primal push가 basic 위반 0일 때 부르는 double feasible ratio `055f68@0xef960`의 첫 패스를 Rust로 복원한다. 입력은 FTRAN 열(희소·dense)·basic 값·폭·표시·허용오차·초기 상한이고, 출력은 반환 위치·theta·abs(pivot)·work 누적이다.
- 경계 선택 이유: 인계 문구는 "04b-1 캡처의 pivot 한 건 또는 flip 한 건"이었다. 그런데 ratio 뒤 거부 판정(`0x1c294c–0x1c2989`)은 `0x1ec8(%rsp)`(C의 `S+0x1f8` double 해석)를, pivot 적용의 이탈 상태 선택은 C에서 `CONCAT`으로만 보이는 `local_250`을 읽는다. 둘 다 04b-1 캡처에 없고 의미도 미확정이다. 방향식은 9단계 모두 결과가 0이다(목적 0인 LP). 입력 상태를 확보할 수 없는 구간은 착수할 수 없으므로(기준 §1), 같은 원본 함수를 자연 9건과 합성 29사례로 모두 관측한 `055f68` 하나로 좁혔다.
- 포함 분기: 희소·dense 열, 양수 alpha의 하한 ratio, 음수 alpha의 상한 ratio(폭<I), `F` 표시의 양수 alpha 제외, Harris 1차 상한, 유한 초기 상한에서의 조기 반환(flip), 선택 없음(theta=I·pivot=0), 2차 선택의 최대 abs(pivot)과 동률 시 첫 후보 유지, -0 보존, 작은 pivot 검사의 통과 쪽, work 누적식.
- 제외(구현하지 않고 명시적으로 거부):
  - 작은 pivot 재진입 패스(`bVar2=false`)와 그 안의 `12dd49`>16 때 `S+0x294=2`·`S+0x2f8=-1`. 04b·04b-1 어느 캡처에서도 실행되지 않았다. Rust는 이 조건에서 `Err(Reentry)`를 반환한다. 후속 M3-04c-5.
  - 열길이 가중(`param_7>0`). push 두 호출점은 `edx=0`(ELF `0x1c284c` `xor %edx,%edx`)이다. 다른 호출자 `22da66`·`b44ce`는 범위 밖이다.
  - 방향식·부호 반전·theta=I 반대 방향 재호출, ratio 뒤 거부·flip·pivot 판정, pivot 적용, infeasible ratio `054eef`·`056973`, x87 `1ef3c1`. 각각 M3-04c-1·04c-2 후보와 새 오라클(아래 §7)로 넘긴다.
  - push 전체·Rust crossover 연결. 이번 함수는 아직 호출자가 없다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C `../re/decomp/c/000ef960_PRIVATE0000000000055f68.c` 전체(329줄). 호출점 `001c0070_PRIVATE0000000000134a87.c` 1309–1312줄, 1543–1546줄.
- ABI는 [04b 조사 §3](primal_push_branches.md#3-합성-ratio-오라클-계약과-결과)을 따른다. `xmm0`=Harris 여유, `xmm1`=pivot 허용, `xmm2`=I, `xmm3`=초기 상한. 반환은 행이 아니라 희소 위치다(dense면 행).
- 자연 호출의 xmm 값은 훅이 읽지 않았으므로 asm으로 재구성한다. push 진입 직후 `0x1c021e–0x1c028d`에서 `S+0x370→0x1fc8(%rsp)`, `S+0x380→0x2070(%rsp)`, `S+0x398→0x2078(%rsp)`를 한 번 저장한다. 함수 전체(`0x1c0070–0x1c5310`)에서 세 슬롯에 쓰는 명령은 이 셋뿐이다. 호출점 `0x1c284e–0x1c289f`는 `rbp=rsp+0x1eb8`에서 `xmm0=0x2070`, `xmm1=0x2078`, `xmm2=xmm3=0x1fc8`을 넣는다. 따라서 xmm0=`S+0x380`, xmm1=`S+0x398`, xmm2=xmm3=`S+0x370`이며, 투영 도구는 push 진입과 모든 ratio 진입에서 세 값이 같음을 검사한다.
- 시작: `055f68` 진입. 입력 희소 열은 방향 부호 반전 뒤 값이다(훅이 인자 포인터에서 읽음). 종료: 반환 직후. theta 포인터의 호출 전 값은 자연 캡처에 없다. 원본이 theta를 쓰지 않는 경로라면 Rust 출력이 NaN 초기값으로 남아 비교에서 실패한다.

## 3. 비교 계약

| 필드 | 비교 규칙 | 출처 |
|---|---|---|
| 반환 위치·행 | 정수 정확 비교 | 자연 `ratio:leave.chosen/row`, 합성 `chosen/row` |
| theta·abs(pivot) | little-endian f64 hex, ±0 포함 비트 비교 | 같은 사건 |
| work | 비트 비교. 합성만(계수 `S+0x148=0.25`, 시작 0) | 합성 `work`. 자연은 호출 전 값과 계수가 없어 제외 |
| 재진입 | Rust `Err(Reentry)` 수. 기대 0 | 비교기가 출력 |
| 입력 보존·`S+0x294/2f8` | Rust는 불변 참조와 workspace 미사용이라 판별 필드로 세지 않음 | 합성 기록은 원본 보존을 확인 |

NaN 입력은 없다. 내부 I(`1e30`) 비트는 그대로 비교한다. 시간·병렬 분기는 없다.

## 4. 사례와 원본 자료

| 사례 | 실제/합성 | 분기 | 원본 자료 |
|---|---|---|---|
| 자연 primal-first 9건×3회, dual-first 9건×3회 | 실제 | 희소, 하한·상한 ratio, Harris, 2차 선택, pivot≥0.1 | `bench/oracle/m3_primal_push_step/{primal,dual}-{1,2,3}.json` |
| 합성 29사례×3회 | 합성 | §1 포함 분기 전체(dense 14·희소 15) | `bench/oracle/m3_primal_push/ratio-{1,2,3}.json`, 입력 `tools/primal_push_ratio_oracle.py` `cases()` |

자연 9건의 입력은 두 순서에서 같을 수 있다. 사례 수를 셀 때 중복을 따로 표시한다. Gurobi는 새로 실행하지 않는다. 판정은 "저장 기준 재생"이다.

## 5. 재실행 명령

```bash
python3 tools/primal_push_ratio_rust.py --write          # 원본 캡처 → Rust 입력 텍스트
cargo test -p rsolver-lp --test push_ratio && python3 tools/primal_push_ratio_rust.py --compare
python3 tools/primal_push_ratio_rust.py --self-test       # 비교기 변조 거부
```

비교기는 원본 캡처를 직접 읽어 Rust trace와 대조한다. trace에 입력 텍스트 해시를 넣고 다르면 거부한다. Rust 구현의 판별력은 임시 소스 변조(비교 `<=`/`<`, Harris 부호, 2차 동률, 폭<I, `F` 제외, 조기 반환, 임계값, work식)로 확인하고 결과를 §6에 적는다. 변조는 최종 코드에 남기지 않는다.

## 6. 실행 결과

(실행 후 작성)

## 7. 판정과 인수인계

(실행 후 작성)
