# M3-04a-1 crossover 진입 원본 오라클

## 착수 검증 계약

- 종류: 원본 정상 실행 캡처 오라클 구축. 기준은 `work/m3-04a-1`의 `797e58d`이며 선행 정적 조사 `4a8c2ab`를 포함한다. 선행 미커밋 변경은 없다.
- 원본: Gurobi 13.0.3 `libgurobi130.so`, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. ELF 파일 오프셋과 ABI는 [정적 조사](crossover_entry_contract.md)를 따른다.
- 실제 소형 LP와 고정 단일 스레드 설정에서 PDHG `34f033 → 2baaa4 → 13f64d → 13f716` mode 0, barrier `2bb770 → 13f64d → 13f716` mode 0, 시작 벡터가 있는 `0c3d0d → 13f716` mode 1을 관찰한다. `Crossover=0` 등 미도달 대조군은 목표 합격과 분리한다.
- 시작은 각 호출 직전, 종료는 반환 직후다. 사건 순서와 모델 문맥을 식별하고 n/m, R의 x/slack/reduced cost/y, S의 복사·변환 결과, 스케일 C/R/O, row sense·flip, 환경 복원 필드, 반환값·모델 상태·내부 vstat 및 가능한 공개 basis를 저장한다. R, S와 작업 상태를 구분한다.
- 포인터는 존재·안정 식별·별칭으로 정규화한다. 정수와 사건 순서는 정확 비교, 부동소수는 IEEE 원시 비트 비교한다. 시간은 별도 측정량이며 판단에 쓰이는 값의 차이를 무시하지 않는다. 입력·바이너리·도구의 유효한 SHA-256 및 실제 입력 출처를 요구한다.
- 각 경로는 독립 원본 프로세스 3회로 비교한다. 목표 사건 0회, 필수 상태 누락, 합성만 존재하거나 해시가 유효하지 않으면 목표 검사는 실패해야 한다. 비교기는 최초 차이의 사례·필드·기준·실제를 보여주고 실패한다. 오류 주입으로 검사 신뢰성을 확인한다.
- 메모리 실패·중단·고급 presolve 매핑 전체는 M3-04a-2에 남긴다. Rust 구현 동등성은 주장하지 않는다. 원본 필드를 해석할 수 없거나 자연 진입이 없으면 실패 입력과 근거를 보존하고 미완료로 남긴다.

## 현재 상태

착수. 원본 캡처, 재현성, 비교기와 오류 주입은 아직 미실행이다.

## 훅 초안과 잠금 대기

`tools/crossover_entry_capture.js`는 실제 Python 프로세스를 Frida로 시작하고 읽기 훅으로 호출과 상태를 수집하는 초안이다. `small.lp`는 실제 LP다. 첫 barrier 실행은 다른 M2 원본 프로세스의 잠금 때문에 `tools/gurun.sh`에서 대기 중이며 아직 훅 동작과 필드 안전성은 검증되지 않았다. `mode 1`은 R이 없는 것이 정상일 수 있으므로 시작 벡터와 후속 작업 배열을 필수로 하고 R은 선택적으로 기록해야 한다. 스케일 포인터 null은 원본 변환 생략과 구분해서 기록한다. 이 초안은 합격 검사기가 아니다.

## 검사기 초안

`python3 tools/crossover_entry_selftest.py`는 합성 형식 fixture에 해시·사건·배열·순서 변조 10건을 주입해 거부를 확인한다. 이 fixture는 실제 실행이 아니다. `tools/crossover_entry_check.py`는 아직 복사·변환 식과 경로별 중첩을 충분히 확인하지 못하므로 명령은 의도적으로 항상 실패한다. 목표 합격으로 사용할 수 없다. 입력 파일 해시를 실제 `small.lp`에 연결하고 환경·스케일·준비 배열의 길이와 원시 비트 형식을 검사한다.

준비 훅 `0x1bd281`의 배열 위치는 ELF `0x1bcb31`, `0x1bcb55`, `0x1bcb7d–0x1bcb8d` 및 mode 1의 `0x1bcd77–0x1bcdc3`에서 확인했다. 두 경로의 `rbx`와 `r12`가 열 x와 reduced cost이고 `rsp+0x238`, `rsp+0x240`이 행 slack과 y다. 합류 지점의 `rbp`는 모델이다. 실제 훅 실행과 필드 내용은 아직 확인하지 않았다.

현재 재실행 명령은 다음과 같다. 캡처는 잠금을 기다리는 동안 실패 파일에 진단을 남기며 성공 파일을 덮어쓰지 않는다. 검사기 명령은 미완성 표시를 출력하고 종료 1이다.

```bash
tools/gurun.sh node tools/crossover_entry_capture.js barrier bench/oracle/m3_crossover_entry/small.lp bench/oracle/m3_crossover_entry/barrier-probe.json
python3 tools/crossover_entry_selftest.py
python3 tools/crossover_entry_check.py bench/oracle/m3_crossover_entry/barrier-probe.json
```

도구는 프로세스 시작 전에 입력·도구·바이너리 해시를 고정하고 종료 시 다시 대조한다. warm 시작값은 PStart `[2,1,1]`, DStart `[1,0,0]`으로 메타데이터에도 기록한다. 결과 구조의 `output`은 진단 로그로 보관하되 상태 비교에서 제외한다. 결과 status·훅 오류·반환값·환경과 배열은 제외하지 않는다. 시간 측정값을 캡처하지 않았고 시간이나 work의 동등성은 주장하지 않는다. 실제 자료로 변환 식을 아직 검증하지 않았으며 CLI를 합격용으로 활성화하지 않았다.
