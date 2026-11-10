# M3-04a-1 crossover 정상 진입 원본 오라클

지정한 소형 LP에서 PDHG·barrier mode 0과 시작 벡터의 mode 1을 각각 원본 독립 프로세스 3회 실행했다. 경로별 상태·벡터가 비트 단위로 일치했고, 검토자가 경로마다 1회 재캡처해 같은 결과를 확인했다. Crossover=0 대조군은 3회 모두 목표 진입 0건이다. 이번 완료 범위는 정상 진입 오라클이며 Rust 구현 동등성은 미검증이다.

## 1. 범위와 선행 상태

- 종류: 원본 정상 실행 캡처 오라클 구축. `797e58d`에서 시작했고 선행 미커밋 변경은 없다. [M3-04a 정적 조사](crossover_entry_contract.md)의 `4a8c2ab`를 상속한 임시 의존 branch에서 작업했다.
- 포함: Presolve=0의 실제 LP 하나에서 PDHG의 `34f033 → 2baaa4 → 13f64d → 13f716(mode 0)`, barrier의 `2bb770 → 13f64d → 13f716(mode 0)`, 두 시작 벡터의 `0c3d0d → 13f716(mode 1)`.
- 제외: push 알고리즘 전체·factor·norm·RNG 내부 상태, 실패·중단·고급 presolve 매핑, 다른 모델, 시간·work, Rust 비교. 실패·중단·매핑은 M3-04a-2, push 조사는 M3-04b·04d, cleanup 연결은 M3-04f로 남긴다.
- 기준 범위는 착수 전에 고정했다. warm의 Method=1이 목표를 밟지 않아 원본 분기 근거를 확인하고 Method=2로 변경했다. 실패 자료를 지우거나 목표 0건을 합격으로 바꾸지 않았다.

## 2. 원본과 캡처 경계

Gurobi 13.0.3 바이너리 SHA-256은 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`다. 최종 캡처 도구는 `c461a9c6edf02db7f86cd2813c689a7dd5465a196f74f547a12a781aeb99746a`, 입력은 `c14a6db31c1ec2ed206ddf2a585f9467ced4e1dbdff11202685f61dd5c478a8d`다. [manifest](../../bench/oracle/m3_crossover_entry/manifest.json)에 스키마 버전, 도구·원본 C·기준 파일 해시와 실행 환경을 연결했다.

각 함수 진입 직전과 반환 직후를 읽기 훅으로 기록한다. Linux x86-64 ABI와 C 선언의 차이, R/S 필드의 근거는 선행 정적 조사를 따른다. 준비 벡터는 `13f716`의 ELF `0x1bd281`에서 읽는다. `rbp`는 모델, `rbx`·`r12`는 x·reduced cost, `rsp+0x238`·`rsp+0x240`은 slack·y 포인터다. mode 0의 할당·저장은 ELF `0x1bcb31`, `0x1bcb55`, `0x1bcb7d–0x1bcb8d`, mode 1은 `0x1bcd77–0x1bcdc3`에서 확인했다.

모델·결과 R·시작 구조 S·배열 포인터는 실행 내 안정 ID와 null로 기록한다. 같은 주소가 해제 뒤 재사용된 경우의 객체 수명은 별도 추적하지 않았다. 원시 주소가 같은지로 서로 다른 실행을 비교하지 않는다. mode 1은 R이 없는 것이 정상일 수 있으므로 시작 벡터와 준비 배열을 필수로 요구한다.

warm은 진입 시 스케일 포인터가 null이지만 함수 내부 `07d3ce` 후 준비 시점에는 스케일이 생성된다. 준비 사건에서 스케일·sense·flip을 다시 읽어 종료값으로 추정하지 않는다. 이 입력의 준비 시점 열 스케일은 `[2,4,2]`, 행 스케일은 `[0.25,0.25,0.25]`다.

## 3. 필드별 비교 계약

| 대상 | 비교와 필수 조건 |
|---|---|
| 출처 | 실제 실행 표시, 고정 바이너리 해시, 실제 입력·캡처 도구 해시와 연결, 고정 설정·시작값 |
| 사건 | 부모·자식 호출 중첩과 순서, 같은 모델 ID, mode, 준비 사건, 반환값과 상태 |
| R·S·준비 벡터 | 경로별 필수 포인터·차원·길이·원시 8바이트. mode 0의 R→S 스케일·부호 식과 R→준비 복사·y 부호 반전도 검사 |
| 스케일 | 열·행 배열, 목적 스케일, 행 sense, 열 flip와 활성 플래그. 명시적 null과 누락 구분 |
| 복원 | 환경 `+0x2478`·`+0x24a0`의 원시 double과 `+0x26fc` 정수가 본체 반환 후 진입값과 같음 |
| basis | workspace 자체 행·열 차원의 내부 vstat, API의 VBasis·CBasis를 별도로 기록 |

정수와 사건 순서는 정확히 비교하고, 수치는 little-endian IEEE 754 원시 비트로 비교한다. ±0도 구분한다. 이 실제 입력의 유한 값에 대한 계약이며 NaN·무한대 입력을 포괄하지 않는다. 공개 basis를 API가 제공하지 않으면 두 배열을 명시적 null로 남긴다. 이 사례의 목표 실행에서는 열 `[0,-1,-1]`, 행 `[-1,0,0]`을 관측했고 대조군에서는 null이었다.

원본 3회와 검토자 재캡처의 모든 저장 필드를 비교한다. `output`은 진단 로그라 제외하되 구조화 status·basis·훅 오류·반환값·환경·배열은 제외하지 않는다. 타이머와 work는 캡처하지 않아 그 재현성을 주장하지 않는다. Seed는 0으로 고정했으나 내부 RNG 전체 상태를 재구성한 것은 아니다.

## 4. 실제 사례와 실패 이력

입력은 [small.lp](../../bench/oracle/m3_crossover_entry/small.lp)로, 열·행 각각 3개이며 `>` 행과 비자명한 스케일을 포함한다. 공통 설정은 Threads=1, Seed=0, OutputFlag=0, Presolve=0, LPWarmStart=2다. 기타 파라미터는 이 바이너리의 기본값이다.

| 사례 | 설정 | 원본 3회 | 검토자 추가 1회 |
|---|---|---|---|
| PDHG | Method=6, Crossover=1 | mode 0 각 1건, 비트 일치 | 일치 |
| barrier | Method=2, Crossover=1 | mode 0 각 1건, 비트 일치 | 일치 |
| warm | Method=2, Crossover=1, PStart `[2,1,1]`, DStart `[1,0,0]` | mode 1 각 1건, 비트 일치 | 일치 |
| 대조군 | Method=2, Crossover=0 | 목표 0건, 별도 비교 일치 | 추가 실행하지 않음 |

현행 기준은 `*-{1,2,3}-final.json`, 검토자 캡처는 `*-review.json`이다. [history](../../bench/oracle/m3_crossover_entry/history/lock-collision-initial.json) 아래에 초기 잠금 충돌과 이전 도구의 모든 캡처를 보존했다. 이전 성공 자료는 현행 도구 해시와 다르므로 현재 합격 입력이 아니다.

초기 warm Method=1의 정상 status 2 실행은 세 번 모두 목표 0건이었다. [실패 기록](../../bench/oracle/m3_crossover_entry/history/warm-method1-1.json)과 나머지 두 기록을 보존했다. 원본 `0014acb0_PRIVATE00000000000c3d0d.c` 990–1009줄은 `env+0x2700 > 0`에서 Method=1일 때 primal start를 버리고 시작 상태를 4로 바꾼다. 1018–1024줄의 `11603b`는 상태 2를 요구하므로 mode 1에 도달하지 않는다. 두 벡터를 유지하는 Method=2로 변경한 뒤 목표를 관측했다. 숨은 파라미터나 상태 주입은 사용하지 않았다.

## 5. 재실행 명령

저장소 루트에서 실행한다. 캡처에는 Node와 이 환경에 설치된 Frida 모듈, `../.venv`의 Gurobi 13.0.3과 라이선스가 필요하다. Frida 모듈 경로는 캡처 스크립트에 고정돼 있다. 다른 컴퓨터에서 도구·환경이 바뀌면 새 해시와 기준을 생성해야 한다. `tools/gurun.sh`와 `gurobi_guard.pth`의 단일 프로세스 잠금을 우회하지 않는다.

```bash
for kind in warm barrier pdhg control; do
  for i in 1 2 3; do
    tools/gurun.sh node tools/crossover_entry_capture.js "$kind" bench/oracle/m3_crossover_entry/small.lp "bench/oracle/m3_crossover_entry/$kind-$i-final.json"
  done
done
for kind in warm barrier pdhg; do
  python3 tools/crossover_entry_check.py bench/oracle/m3_crossover_entry/$kind-{1,2,3}-final.json
done
python3 tools/crossover_entry_check.py --control bench/oracle/m3_crossover_entry/control-{1,2,3}-final.json
python3 tools/crossover_entry_selftest.py
python3 tools/crossover_entry_mutation.py
node --check tools/crossover_entry_capture.js
```

manifest의 자료·도구·원본 C 해시는 다음 명령으로 확인한다. 원본 C 전체 확인에는 저장소 옆 `re/`가 필요하다. 저장 기준 비교기만 실행할 때는 Gurobi 라이브러리를 로드하지 않는다.

```bash
python3 - <<'PY'
from pathlib import Path
import hashlib, json
root = Path.cwd()
base = root / 'bench/oracle/m3_crossover_entry'
m = json.loads((base / 'manifest.json').read_text())
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(base / m['input']['path']) == m['input']['sha256']
for name, expected in {**m['tools'], **m['sources']}.items():
    assert sha(root / name) == expected, name
for item in m['captures'] + m['history']:
    assert sha(base / item['path']) == item['sha256'], item['path']
print('manifest 해시 일치')
PY
```

검토자 재캡처·비교는 다음 명령을 실제 실행했다.

```bash
for kind in warm barrier pdhg; do
  tools/gurun.sh node tools/crossover_entry_capture.js "$kind" bench/oracle/m3_crossover_entry/small.lp "bench/oracle/m3_crossover_entry/$kind-review.json"
  python3 tools/crossover_entry_check.py "bench/oracle/m3_crossover_entry/$kind-1-final.json" "bench/oracle/m3_crossover_entry/$kind-2-final.json" "bench/oracle/m3_crossover_entry/$kind-review.json"
done
```

## 6. 실행·독립 검토 결과

- Terra의 최종 기준·도구 commit은 `e1f841c`다. 원본 목표 9회와 대조군 3회, 검토자 원본 재캡처 3회를 수행했다. 모든 해당 비교 명령은 종료 0이다. 저장 기준만 재생한 결과와 실제 원본 재캡처를 구분했다.
- `crossover_entry_mutation.py`에서 실제 barrier 기준 복사본의 바이너리·입력 해시 0, 사건 비움, 목표 0건, 필수 x 누락, 준비 벡터 비트 변조, 합성 출처 7개가 각각 CLI 종료 1로 거부됐다. 수치 변조는 첫 사건·인덱스·기준 및 실제 비트를 출력했다. 검토자도 직접 재실행했다.
- 검토자는 실제 캡처에 도구 해시 0·임의 입력 해시·스케일 누락·순서 반전·고정 파라미터 변경·S 비트 변조까지 포함한 12개 변조를 별도로 검사해 거부를 확인했다. 위 7개와 겹치므로 합산하지 않는다.
- 합성 self-test는 오류 주입 15개, 비균일 스케일·행/열 반전·±0을 확인했다. 합성 입력은 실제 목표 실행 수에 포함하지 않는다. Node 문법 검사, 문서 링크·자료 해시와 `git diff --check`도 확인했다.
- 통합 전 manifest의 현행 캡처 15개·이력 26개·원본 C 11개·도구 4개 해시와 입력 해시를 확인했고, 관련 문서 7개의 상대 링크 122개도 통과했다. 작업 worktree의 진행·로드맵을 최신 main과 비교해 이번 결과를 반영했다.
- 초기 guard 충돌과 Method=1 warm 미도달 3회는 실패 이력으로 보존한다. 대상 범위를 합성 실행으로 대체하거나 누락 상태를 허용해 합격시킨 사례는 없다.
- 솔버 코드는 변경하지 않아 Cargo·확장 통합 회귀는 실행하지 않았다. Rust와 원본의 동등성 비교도 수행하지 않았다.

## 7. 판정과 다음 작업

오라클 구축은 지정한 정상 입력·캡처 필드에서 완료했다. 원본 자체의 반복 재현성과 원본 복사·변환 검사는 통과했고 Rust 동등성은 미검증이다. 다른 모델과 내부 push·basis 전체 수명·실패·중단·presolve 매핑의 동등성까지 검증한 것은 아니다.

선행 정적 조사는 main `057ab2f`에 병합했다. 이번 결과와 공유 진행·로드맵·스프린트 목록을 작업 branch에서 갱신해 main에 병합하고, 같은 단계에서 작업 worktree·branch 및 얻었던 점유를 정리한다. [M3 인수인계](../handoff/M3.md)의 다음 후보는 M3-04b primal push 분기 조사이며, M3-04d와 M3-04f가 이어진다.
