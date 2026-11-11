# M2-06h-2 후보 하나의 SB 제어 Rust

## 1. 범위와 주장

구현(L1). 기준 main `01d388b`, 선행 06h-1 병합, 미커밋 선행 없음.
`695bd7` 각 호출의 첫 정수 후보 한 개에서 up/down LP 결과를 명시적 입력으로 받아 분기 bound 요청, flush, LP 호출, 목적 소비, 반복 비용 누적, 상태 reset·저장 basis 복구·bound 복원 요청 순서를 재생한다. 저장·복구 객체의 내용은 06h-1 기준으로 검증한 외부 backend가 맡는다. Rust LP·factor 수치 계산·후속 후보 사전 채움·stall·최종 부모 재풀이·B&B 연결은 제외하고 후속 M2-06h-3으로 넘긴다. 전파 backend는 선택 열만 조이는 정상 경로로 한정하고 추가 전파·모순·오류·연속 열·x87·병렬은 명시적으로 거부한다.

## 2. 원본과 상태 경계

Gurobi 13.0.3 SHA256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
C `00571770_*695bd7.c`: 317–338 저장·해 읽기, 393–442 bound 역스케일·정수 분기, 460–610 up 소비·복구, 870–990 down 소비·복구. `001a45d0_*110b77.c`는 rdi=LP,rsi=열 해 출력,rdx=행 출력(SysV·ELF 진입 확인). 이 호출 반환의 unscaled 해와 env IntFeasTol을 새 읽기 전용 훅으로 기록한다. 첫 후보의 저장 원본 상태는 06h-1과 동일해야 한다.
시작은 저장 객체가 준비되고 부모 ObjVal·unscaled 해를 얻은 뒤, 종료는 down LP 후 basis 복구·UB 복원 요청을 큐에 넣은 직후다. 다음 후보/최종 cleanup의 flush는 이 경계 밖이다.
입력: 후보 열·타입, 해 x, IntFeasTol, 원래 unscaled LB/UB(열 scale·flip), 부모 목적, up/down LP 상태·ObjVal/dual·IterCount. 저장 객체는 backend 불변 참조이며 이 구간은 snapshot 재저장을 명령하지 않는다.

## 3. 비교 계약

| 필드 | 규칙 | 근거 |
|---|---|---|
| 방향·열·명령 순서 | 정확 일치 | 첫 후보 up→down, reset→restore→bound restore |
| 분기값 | `floor(x+IntFeasTol)+1`, `floor(x+IntFeasTol)` u64 비트 | C 433–442 |
| 목적 | 상태2: obj<=base 이면 base; 상태7: base<=dual 이면 dual | C 492–576·884–961 |
| 비용 | up IterCount 뒤 down IterCount 순서로 더함, 비트 | C 490·881 |
| bound 큐·적용과 snapshot | 기존 snapshot 검사 + 새 명령 trace 대조 | setter 직후 모델 bound 보존, flush 때 scale 적용 |

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

실행 후 기록한다.

## 7. 판정과 인수인계

착수. 동등성 미검증. branch `work/m2-06h-2`, worktree `../rsolver-m2-06h-2`.
후속 M2-06h-3: 추가 전파·모순·전체 후보 루프와 부모 재풀이 경계. M2-06i reliability 판단은 이 첫 후보 제어와 구분한다.
교훈은 종료 시 docs/lessons/m2_sb_snapshot.md에 기록한다.
