# M2-06h-2 후보 하나의 SB 제어 Rust

## 1. 범위와 주장

구현(L1). 기준 main `01d388b`, 선행 06h-1 병합, 미커밋 선행 없음.
`695bd7` 각 호출의 첫 정수 후보 한 개에서 up/down LP 결과를 명시적 입력으로 받아 분기 bound 요청, flush, LP 호출, 목적 소비, 반복 비용 누적, 상태 reset·저장 basis 복구·bound 복원 요청 순서를 재생한다. 저장·복구 객체의 내용은 06h-1 기준으로 검증한 외부 backend가 맡는다. Rust LP·factor 수치 계산·후속 후보 사전 채움·stall·최종 부모 재풀이·B&B 연결은 제외하고 후속 M2-06h-3으로 넘긴다. 전파 backend는 선택 열만 조이는 정상 경로로 한정하고 추가 전파·모순·오류·연속 열·x87·병렬은 명시적으로 거부한다.

## 2. 원본과 상태 경계

Gurobi 13.0.3 SHA256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
C `00571770_*695bd7.c`: 317–338 저장·해 읽기, 393–442 bound 역스케일·정수 분기, 460–610 up 소비·복구, 870–990 down 소비·복구. `001a45d0_*110b77.c`는 rdi=LP,rsi=열 해 출력,rdx=행 출력(SysV·ELF 진입 확인). 이 호출 반환의 unscaled 해와 env IntFeasTol을 새 읽기 전용 훅으로 기록한다. 첫 후보의 저장 원본 상태는 06h-1과 동일해야 한다.
시작은 저장 객체가 준비되고 부모 ObjVal·unscaled 해를 얻은 뒤, 종료는 down LP 후 basis 복구·UB 복원 요청을 큐에 넣은 직후다. 다음 후보/최종 cleanup의 flush는 이 경계 밖이다.
정정(사후 독립 재검토, 원문 위 유지): pk1 첫 후보 24개는 모두 전파 경로(`local_90≠0`, `6953ff` 0x573d30)를 탔다. 실제 근거 줄은 up bound C:669–679, down bound C:772–781, up 복구 루프 C:592–647, down 복구 루프 C:993–1047, 복구 값은 C:368–369 getter(`115607`/`1157b8`)의 local_a0/a8이다. 이 경로의 복구는 `a0<b0`·`b8<a8` 엄격 부등일 때만 요청하고 `*param_12 += 6·개수`(C:648–650, 1039–1041)가 붙는다. Rust의 무조건 복원 요청은 분수 x에서 분기 bound가 항상 원래 값과 다르다는 전제에 기댄다. 위 393–442·584–585·975–990·비용 490·881(266b6a 기준, 695bd7에서는 484·878)은 비전파 경로 또는 다른 함수 줄이다. 구간 안의 work 카운터 `param_12`는 이식하지 않았다(후속 M2-06h-3).
입력: 후보 열·타입, 해 x, IntFeasTol, 원래 unscaled LB/UB(열 scale·flip), 부모 목적, up/down LP 상태·ObjVal/dual·IterCount. 저장 객체는 backend 불변 참조이며 이 구간은 snapshot 재저장을 명령하지 않는다.

## 3. 비교 계약

| 필드 | 규칙 | 근거 |
|---|---|---|
| 방향·열·명령 순서 | 정확 일치 | 첫 후보 up→down, reset→restore→bound restore |
| 분기값 | `floor(x+IntFeasTol)+1`, `floor(x+IntFeasTol)` u64 비트 | C 433–442 |
| 목적 | 상태2: obj<=base 이면 base; 상태7: base<=dual 이면 dual | C 492–576·884–961 |
| 비용 | up IterCount 뒤 down IterCount 순서로 더함, 비트 | C 490·881 |
| bound 큐·적용과 snapshot | 기존 snapshot 검사 + 새 명령 trace 대조 | setter 직후 모델 bound 보존, flush 때 scale 적용 |

정정: clamp는 asm에서 `maxsd`다(상태2 0x57271c·0x573063 `obj>base?obj:base`, 상태7 0x57267b·0x572fc2 `base>dual?base:dual`). 디컴파일의 `<=` 형태는 NaN에서 다르므로 Rust는 asm 피연산자 순서를 따른다. 1e100 판정은 asm 0x572b37 `comisd; jae`라 NaN이면 down으로 간다(Rust `>=`와 같음).

사례표(§9)

| 범주 | 원본 실행 증거 | 합성 Rust 회귀 |
|---|---|---|
| 상태7 dual==base(비트 동률) | 34방향 | ±0 동률 |
| 상태7 dual>base | 8방향 | — |
| 상태7 base>dual(clamp) | 없음 | `clamp_and_signed_zero` |
| 상태2 obj>base | 5방향 | — |
| 상태2 obj==base | 1방향 | ±0 동률 |
| 상태2 obj<base(clamp) | 없음 | `clamp_and_signed_zero` |
| NaN 목적 | 해당 없음(하네스가 입력 거부) | maxsd 피연산자 선택 |
| up/down ≥1e100 | 없음 | `sentinel_returns_unported_after_restoration`(거부만) |
| x+tol 정확 정수 경계 | 없음 | 없음(허용오차 안 x는 후보가 아님) |
| 상태 3·6·11 등, 연속 열, 추가 전파, 오류 | 없음 | 하네스·타입이 거부 |

오차 0, ±0 보존. NaN·무한 IEEE 값은 입력 경계에서 거부, ±1e100은 내부 sentinel. Threads=1, 시간 측정 없음. 검증되지 않은 상태·전파·오류를 성공으로 바꾸지 않는다. 같은 값끼리 clamp의 동률은 C의 피연산자 선택 순서대로 한다.

## 4. 사례와 원본 자료

자연 pk1, Threads=1 Seed=0 NodeLimit=300 GURO_PAR_SBMAXITERS=10. 첫 후보 24개·방향48, 상태2/7(기존 관측). 새 원본 3회와 무훅 대조, 기존06h-1로 투영 일치. 동점·정확 허용오차 경계·기타 상태는 합성 Rust 회귀와 실제 원본 판정을 구분한다.
manifest는 입력 모델·바이너리·캡처 도구·캡처·재생 도구 해시를 고정한다. 외부 re C와 x86-64 Linux 환경은 기존06h-1을 따른다.

## 5. 재실행과 오류 검출

새 캡처: `tools/mip_sb_control_runs.sh bench/oracle/m2_06h2/capture`.
재생: `python3 tools/mip_sb_control_rust.py --self-test`(Rust harness 실행·원본 명령/결과 비교).
바이너리/모델 해시0, 사건 비우기, 필수 입력 누락, 순서·분기값·목적·비용·복원 누락 변조를 거부. Rust 소스 임시 복사본의 방향 순서·clamp·반복합·복구 명령 변이를 실제 컴파일해 거부한다. 관측상 구별되지 않은 변이는 생존으로 기록한다.
회귀: snapshot/restore 검사기 self-test, cargo test -p rsolver-mip, cargo test --release --workspace, clippy workspace. Python 확장 미변경으로 pytest 제외.

## 6. 실행 결과

- 원본 캡처: `tools/mip_sb_control_runs.sh bench/oracle/m2_06h2/capture` 독립 3회 gz 동일, 무훅 대조 최종 출력 동일(`control.json.gz`). 투영이 06h-1 저장 자료와 같다. 정수 타입을 입력에 넣기 전 캡처는 `capture_v1/`, 첫 탐색은 `explore.json.gz`로 보존.
- 재생: `python3 tools/mip_sb_control_rust.py --self-test` 종료 0. 첫 후보 24개·방향 48(상태7 42, 상태2 6)의 bound 요청·flush·solve·reset·restore·bound 복원 순서, 분기값, 목적값, 반복 비용 누적이 u64 비트로 원본과 같다. 비용 시작값은 원본 `cost0`이고, 후보가 1개인 묶음 7개는 원본 묶음 종료 비용 `cost1`과도 비트 일치한다.
- 오류 검출: 입력·trace 변조 15종, 이 도구의 개수·묶음 비용 검사 직접 변조 2종 거부. Rust 임시 복사본 변이 6종 중 방향 역순·비용 대입·restore 삭제·분기값+2 4종 거부. 상태2·상태7 clamp 제거 2종은 원본 재생에서 생존(원본 자료에 clamp 분기 없음, 사례표 참조)하고 합성 테스트만 거부한다.
- `1e100` 경계: C:660(asm `comisd; jae`)은 up 값이 1e100 미만일 때만 down으로 가고, 아니면 복구 뒤 down을 건너뛴다. down ≥1e100이면 원본은 stall 카운터만 0으로 하고 다음 후보로 간다(C:1066–1071). 둘 다 후속 경로 미이식이라 Rust는 복구 명령 뒤 `Error::Unported`를 반환한다. 원본 자료에는 이 경로가 없고 합성 테스트만 있다.
- 회귀: `python3 tools/mip_sb_snapshot_check.py --self-test`·`python3 tools/mip_sb_restore_check.py --self-test` 종료 0, `cargo test --release --workspace` 127 통과·0 실패·12 무시, `cargo clippy --workspace --all-targets -- -D warnings` 0. Python 확장 미변경으로 pytest 제외.
- 독립 검토: (1) Codex 하위 에이전트, `1e100` 보강 전 코드, 높음·중간 없음, 정수 타입 입력 추가. (2) 병합(`64137a2`) 뒤 Claude 하위 에이전트(다른 모델) 재검토: 높음 없음, 중간 2(clamp 두 분기 원본 미판별·근거 줄 오인용), 낮음 6(work 카운터 미기재, NaN clamp가 asm과 다름, down Unported 표현, 연속 열 전제, 하네스 판별력·사례표, 비용 기대값 시작값). 모두 이 절·§2·§3 정정과 코드·하네스 수정으로 반영했다.

## 7. 판정과 인수인계

구현(L1), 조건부 통과. 명령 순서·분기값·반복 비용과 관측된 목적 소비(동률·dual>base·obj>base)는 원본 비트 일치로 통과. 목적 clamp 두 분기(base 선택)는 원본 미검증(합성만), work 카운터 `param_12`는 미이식. 전체 LP 수치 포함 동등성, `1e100` 이후 경로, 추가 전파·모순·오류·연속 열·x87·병렬은 미검증이다. branch `work/m2-06h-2`.
후속 M2-06h-3: 추가 전파·모순·전체 후보 루프와 부모 재풀이 경계. M2-06i reliability 판단은 이 첫 후보 제어와 구분한다.
교훈은 [docs/lessons/m2_sb_snapshot.md](../lessons/m2_sb_snapshot.md)에 기록했다.
