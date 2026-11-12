# M3-04c-15 직렬 factor 제어 오라클

## 1. 범위와 주장

- 종류: 오라클 구축. 승인 pivot 뒤 요청 선택 → 직렬 factor 호출·alpha 복원 → 오류/retry 게이트 → 재계산 진입 또는 품질 주기 제어를 검증한다.
- 기준 main `79a0893`, branch `work/m3-04c-15`, worktree `../rsolver-m3-04c-15`. 선행 미커밋 없음. 담당 `codex-m3-04c15-20261112`, M3-04c-15·shared 점유.
- 자연 arms·retry·interior와 Crossover=0 대조군은 기존 13 오라클과 사건 전체 투영·최종 출력이 같아야 한다. factor 판단 필드·호출 인자·복원·품질 주기와 support 생략을 추가 관측한다.
- 합성 경계는 실제 원본 명령을 격리 프로세스에서 호출한다. 요청 선택은 원본 `0x1c336d`부터 호출/생략 분기까지, wrapper는 `0x1c3465`부터 정상 종료·retry/오류 또는 **재계산 직전**까지다. 합성 wrapper의 factor·품질 계산·두 문맥 전환 helper는 명시적 backend 대체다. 합성 helper 결과를 자연 LU 증거로 세지 않는다.
- 제외: 재계산 본문17·실제 LU21·batch23·전체 push22·x87·전체 work/P3. 품질 helper `089f61`의 수치 오차 계산은 이번 caller 계약 밖이며 별도 후속15a로 등록한다. 16 구현의 품질 계산은 외부 backend다.

## 2. 원본과 상태 경계

- Gurobi13.0.3, SHA256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`, x86-64 Linux, ELF file offset 기준.
- C: `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c`1725–1997; `00115ed0_PRIVATE0000000000089f61.c`238–272(수치 helper는 정적 참고).
- asm `0x1c336d–0x1c345a`: r14=S, rsp+0x2048=F, +0x1e78=1.5m, +0x1e80=5m², +0x1e98=m, +0x209c=support. 원본 branch target 3465(call),3b6e(skip)을 합성 전용 반환 stub으로 교체한다. 내부 산술·비교 명령은 바꾸지 않는다.
- wrapper는 위 frame 및 +0x2090=alpha, +0x1fa0=attempt, +0x1fe0=초기 numeric+10, +0x1fd8=품질 주기, +0x20e8=retry pointer를 사용한다. batch capacity/count/position=0. 별도 fake stack에 충분한 하위 호출 공간을 두고 SysV callee-saved 레지스터를 보존하는 trampoline을 쓴다. 원본 주소를 절대값으로 기록하지 않는다.
- 자연 훅은 함수 진입 훅 외에는 함수형 probe를 쓴다. 명령 경계·incoming branch를 asm에서 확인한다. `S+0x210` 원래 포인터와 현재 alpha는 주소 대신 별칭 boolean·벡터 내용으로 기록한다. batch=0에서 F+0x1c8 보존도 확인한다.

## 3. 비교 계약

| 필드/사건 | 규칙 | 확보 방법 |
|---|---|---|
| 요청 우선·k==limit·k<q/==q/>q·N·support | 정수·호출 유무 정확 일치 | 자연 판단 전후 + 원본 선택 slice 합성 |
| m·L/U·C/E·준비 상수 | 정수 정확, f64 ±0 포함 LE hex 비트 | 자연 frame 및 합성 입력 |
| 비용식 | ((f64(C)+f64(E))*2)*m-min(q*((m*5)*m),(f64(L+U)-m*1.5)^2), >0만 요청10 | asm 결합 순서, 유한 입력만 |
| factor 호출·반환1 반복·12·오류 | 순서·인자·요청·반환 정확 | 자연 + 명시적 합성 backend |
| alpha 교체/복원·spike 보존 | 별칭과 내용 정확, null-null 거부 | 자연 전후·합성 sentinel 두 벡터 |
| retry·status·attempt·failure·numeric 게이트 | 상태·쓰기·요청 보존 정확 | 원본 wrapper slice, cleanup 전에 종료 |
| quality 호출·increment·request6·주기 | 호출 순서·local_100·최종 request 정확 | wrapper 원본, 품질 수치는 외부 입력 |

- 입력은 유한 수·유효 int32/int64 합(overflow 없음), NaN/Inf 비용식은 제외. helper의 수치 오차는 외부 backend라 오차식을 적용하지 않는다.
- Threads1·Seed0, 타이머 분기와 work 누적 비교 제외. 원본 캡처는 독립3회 사건/결과 동일 및 기존13 사건 투영 동일을 요구한다.

## 4. 사례와 원본 자료

- 자연: 13의 arms·retry·interior 입력/starts/params를 그대로 사용. Crossover0은 사건0이어야 하고 목표 자연 자료는 판단·factor 사건이 비어 있으면 실패한다.
- 선택 합성: 기존 요청2/6/10, k==limit 및 양옆·limit 초과, k=q 전후, 비용0/양/음, q*5m²와 a²의 min 양쪽, support0/1/-1, N0/비영. 모델 가능한 자연 상태로 주장하지 않는다.
- wrapper 합성: 반환0/1→0/1→1→0/12/>1, status -999/-1000, attempt0/1, failure 및 numeric 임계값 양옆, request0/비영 재계산 진입, 품질 주기 k==주기·k0·주기>50 clamp, increment5/10/20/50, request6 재호출·status12.
- 출력 `bench/oracle/m3_push_factor/`, manifest에 바이너리·모델·도구·입력 명세·캡처·출처 C 해시를 연결한다. 동일 ABI/실행 설정을 새 독립3회로 검증한다.

## 5. 재실행과 완료 기준

예정 도구: `push_factor_capture.js`/`agent.js`, `push_factor_synthetic.py`, `push_factor_runs.py`, `push_factor_check.py`.

1. 탐색1회 → 자연/대조/합성 독립3회.
2. `python3 tools/push_factor_check.py --controls`(출처·재현·비간섭)와 `--acceptance`(자연 사건 존재·제어 전이 재생)를 분리한다.
3. `--self-test`는 해시0·빈 사건·목표0회·누락/null·회차 차이·요청/호출/별칭/게이트/주기 변조를 전부 거부해야 한다.
4. 선행 supplier 검사3종, push chain 검사·self-test, Rust `push_chain` 및236단계/5,144필드 비교를 실행한다. Rust/확장 미변경이므로 workspace 전체/clippy/확장 설치/Netlib/P3는 제외한다.
5. oracle-capture 스킬이 요구하는 독립 검토를 받고, 결과·한계·다음 명령·교훈·공유 문서를 갱신해 --no-ff 병합한다.

## 6. 실행 결과

미실행. 계약을 먼저 커밋하고 도구 첫 동작·캡처·검사 단계마다 기록한다.

## 7. 판정과 인수인계

착수. 원본 구현 동등성 미검증. 다음은 도구 작성과 탐색 캡처다. 품질 helper 수치15a를 완료해도 caller16·LU21와 별개다.
