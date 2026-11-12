# M3-04c-13 단일 공급자 오라클

## 1. 범위와 주장

- 종류: 오라클 구축. 선행 main `cb0c701`, M3-04c-12. branch `work/m3-04c-13`, worktree `../rsolver-m3-04c-13`, 선행 미커밋 없음(main의 `.claude/settings.local.json`은 별개로 보존).
- 목표: batch 용량·채움·소비가 모두 0인 `25615c@0x1c5310`의 후보 소비·skip·보류·통계·dense 압축을 같은 입력에서 재생할 수 있도록 고정한다.
- 자연 사례: arms·retry·interior의 primal-first, Crossover=0 대조군. 각 독립 3회. 기존 agent 사건 전체에 투영해 비간섭을 확인한다. 구조 열·slack·support 0/1·실패 2/3·충돌·고갈은 자연 캡처에서 집계한다.
- 정확 경계: 별도 프로세스의 합성 직접 호출. 5포인터 ABI와 C/asm의 읽기 필드를 확인한 후 최소 S/P/F와 벡터를 구성한다. 수치 factor를 위조하지 않고 `09d008`만 명시적 합성 backend로 대체해 정한 raw 출력(sparse 역순, dense ±tol/그 안팎/±0)을 공급한다. support -1 정규화·failure 2/3·충돌과 미충돌·고갈·빈 열도 검사한다. 이 실험은 자연 FTRAN 수치 증거와 구분한다.
- 제외: Rust(14), FTRAN 수치 구현·실제 LU 연결(21), batch(23), x87·전체 crossover·work 패리티. backend 호출 전후 work는 수집하되 합격은 통계·제어와 alpha 비트에 한정한다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C: `../re/decomp/c/001c5310_PRIVATE000000000025615c.c` 71–80·526–688; `00120680_PRIVATE000000000009d008.c`.
- ELF offset=파일 이름 주소(Ghidra +0x100000). SysV x86-64 `int supplier(P*,S*,alpha**,support*,work*)`; `09d008(F*,input*,output*,spike*,0,0,0,work*)`.
- 시작/종료: 공급자 함수 진입/반환. 중첩 `09d008@0x120680` 진입/반환에서 원본 CSC/slack, raw alpha와 spike를 수집한다. 포인터 대신 S+0x210·S+0x220·F+0x1c8과의 별칭 참거짓을 기록한다.
- P: m,total,candidates,failures,marks,cursor,deferred,count 합·성공 수,batch 세 필드. S: m,n,CSC,drop tolerance. 사용 중인 prefix가 덮어써져도 순서를 보존한다. 고갈에서는 기존 alpha 내용을 유효 출력으로 해석하지 않는다.

## 3. 비교 계약

| 필드 | 규칙 | 방법 |
|---|---|---|
| 후보·cursor·deferred·failures·지원 표시·support·반환 j | 정수·순서 정확 일치; 실패 횟수와 지원 표시는 공급자 동안 보존 | 전 상태에서 제어 재생 후 후 상태와 비교 |
| 성공 수·count 합 | 압축 전 raw count 누적; f64 비트 일치 | raw FTRAN 출력으로 예측 |
| 원본 열 | CSC 저장 순서 또는 slack +1의 비트 일치 | push 진입 모델과 FTRAN 입력 비교 |
| raw/output alpha | sparse 순서 유지; dense는 abs(v)>tol만 행 오름차순, ±0 포함 비트 비교 | FTRAN 반환과 공급 반환 비교 |
| 별칭·호출 순서 | 지원0일 때 spike=F+0x1c8, 양수일 때 null; output=S+0x210,input=S+0x220 | 진입/반환 훅 |
| 출처·재현·비간섭 | 바이너리/입력/도구 해시·설정 고정, 독립3회 사건·출력 일치, 이전 사건 전체 투영 일치 | manifest + controls |

유한한 허용오차와 유한 raw 값이 전제다. NaN·무한대 수치 factor는 제외한다. 후속 분기에 쓰는 f64는 오차 허용 없이 LE hex 비트 비교한다. 시간 분기·병렬은 제외(Threads=1), work 수치 동등성은 제외다. 필수 상태 null·사건0회·합성만 존재하면 목표 합격 거부.

## 4. 사례와 원본 자료

`bench/oracle/m3_push_supplier/`에 자연3사례×3회·대조군3회·합성3회·manifest·검증 결과를 둔다. 탐색은 history에 보존한다. 기존 입력은 arms(`m3_push_arms`), retry(`m3_push_retry`), interior(`m3_crossover_entry`의 실제 manifest 경로 확인 후 사용)를 재사용한다. 자연 자료에는 실제 FTRAN 결과와 spike를, 합성에는 backend 대체를 명시한다. 최종 입력·기준 경로/해시는 manifest로 고정한다.

## 5. 재실행 명령

- 자연 캡처: `tools/gurun.sh node tools/push_supplier_capture.js primal <model> <output> '<extra>'`.
- 합성: `tools/gurun.sh ../.venv/bin/python tools/push_supplier_synthetic.py --capture <output>`.
- 검사: `python3 tools/push_supplier_check.py --controls`, `--acceptance`, `--self-test`, `--write-manifest`.
- 회귀: `python3 tools/push_chain_oracle.py --check`, `--self-test`; `python3 tools/push_step_chain_rust.py --compare`。Rust 변경이 없으므로 전체 workspace·clippy·확장 재설치는 제외.
- 오류 주입: 바이너리/입력 해시0·빈 사건·목표0회·필드 null·3회 차이·후보/보류/통계/별칭/raw/압축 순서 변조를 거부해야 한다. 동일 변조를3회 모두 적용해 재현성 검사만으로 거부되는 것을 방지한다.

## 6. 실행 결과

착수 계약. 아직 원본 실행·검사 미실행.

## 7. 판정과 인수인계

진행 중. 다음 M3-04c-14 L1 제어 구현. 완료 전에 독립 검토·교훈·진행·로드맵·인계 갱신과 점유 해제를 수행한다.
