# M3-04c-13 독립 검토

- 검토자: `/root/m3_04c13_review`. 작업자와 별도로 C/asm·훅·직접 호출·검사기를 검토하고 저장 검사·변조·원본 재캡처를 실행했다.
- 검토 기준: branch `work/m3-04c-13`, 검사기 단계 commit `ed0843c`와 같은 도구 내용. 소스 수정 없음. 최종 문서 갱신은 작업자가 수행한다.
- 판정: **통과**. 이번 오라클 계약 범위에서 병합을 막는 결함 없음. Rust 공급자 구현과 FTRAN/LU 수치 동등성은 이 판정에 포함하지 않는다.

## 원본과 캡처 경계

- `001c5310_PRIVATE000000000025615c.c` 526–688 및 objdump `0x1c5310–0x1c5364`, `0x1c5d5e–0x1c5e55`, `0x1c60d3–0x1c610f`, `0x1c6335–0x1c6420`을 직접 확인했다. 5포인터 SysV ABI, support=max(support,0), cursor 선증가, failure>2 보류, CSC/slack 지원 충돌, prefix 보존, 압축 전 count 합·성공 수 갱신과 strict `abs(v)>tol`의 행 오름차순 in-place 압축이 검사기 재생과 일치한다.
- `09d008@0x120680` 진입/끝 훅과 원본 C·objdump `0x120680–0x120740`를 확인했다. 자연 훅은 원본 메모리·호출을 대체하지 않고 input=S+0x220, output=S+0x210, 지원0일 때 spike=F+0x1c8·양수일 때 null을 기록한다. spike 값은 dense backing 배열에서 indices가 가리키는 좌표를 읽는다.
- 합성 하네스는 별도 프로세스에서 `09d008`만 callback으로 대체하고 원본 supplier를 직접 호출한 뒤 코드 바이트를 복원한다. 따라서 dense 정확 경계·support -1·skip 제어의 증거이며 실제 수치 FTRAN 증거가 아니다. schema/origin/substitution과 계약이 이 구분을 명시한다.
- `history/synthetic-m9-error-{1,2,3}.json`은 짝수 m 인자가 before 내부로 잘못 전달된 이전 실패 이력이다. 검사기가 raw 길이 기준9/실제8을 거부한 뒤 수정·재캡처했다. 이 파일들은 history 해시 보존용이며 최종18사례나 합격3회에 포함되지 않는다.

## 직접 실행한 검사

```bash
python3 tools/push_supplier_check.py --controls
python3 tools/push_supplier_check.py --acceptance
python3 tools/push_supplier_check.py --self-test
```

- 모두 종료0. controls: arms237·retry96·interior105·control0사건, 각각 독립3회 일치와 이전 agent 사건 전체 투영·출력·결과·설정 일치. 자연 run_id와 PID의 독립성 및 입력/바이너리/도구 해시를 확인했다.
- acceptance: 대표 자연51공급(성공42·고갈9), 구조40·slack2, 지원 양수39·0은12, 지원 충돌13·실패 한도 skip1·실패2회 후보2, 내부 raw dense9·sparse33. 합성18사례(성공14·고갈4·dense4·sparse10)도 전후 제어 재생 통과.
- self-test31종 모두 거부. 동작 변조는3회 모두에 동일하게 적용하므로 단순 실행 간 차이만으로 거부한 것이 아니다.

검토자가 별도로 작성한 in-memory 변조도 `controls`/`acceptance`를 직접 호출해 거부를 확인했다. 기준 파일은 수정하지 않았다.

| 변조 | 실제 첫 차이 |
|---|---|
| arms 바이너리 SHA 전부0 | `arms-1.binary_sha256` |
| arms 모델 SHA 전부0 | `arms-1.model_sha256` |
| arms 사건 비움 | `arms: 목표 사건0회` |
| retry에서 supply/supplier 사건만 제거 | `retry: 목표 사건0회` |
| 합성 structural-sparse의 alpha/input/aliases/raw를 null | `FTRAN original column`의 dict/None 타입 차이 |
| dense-boundaries 출력의 첫 값 부호 반전 | `alpha compression/order.values[0]` 비트 차이 |
| arms 첫 공급 후 support 1→2 | `after.support` |

## 검토자 원본 재캡처

```bash
 tools/gurun.sh node tools/push_supplier_capture.js primal bench/oracle/m3_push_arms/arms.lp bench/oracle/m3_push_supplier/review/arms-recapture.json "$(cat bench/oracle/m3_push_arms/arms.extra.json)"
 tools/gurun.sh ../.venv/bin/python tools/push_supplier_synthetic.py --capture bench/oracle/m3_push_supplier/review/synthetic-recapture.json
```

- 두 명령을 순차 실행했고 각각 종료0. `review/arms-recapture.json`은 arms-1의237사건·output·result·params·starts·binary/model/tool/agent SHA와 일치한다.
- `review/synthetic-recapture.json`은 synthetic-1의18사건·schema·origin·binary/tool SHA와 일치한다. 원본3회 비교와 별개로 검토자가 각각1회 더 실행한 결과다.

## 검토한 파일 식별값

| 파일 | SHA-256 |
|---|---|
| `tools/push_supplier_agent.js` | `9369c0c1b55a3efab8877e4c6b68cf9bd7fdd28422c34ff83f2f2afbc3c9b329` |
| `tools/push_supplier_capture.js` | `0a93dfbe43563c361800b2c72bbde8609f8778710193dbe14363d0bc01a60fb7` |
| `tools/push_supplier_synthetic.py` | `35dde0a2f26afa06c917f8b910c6da3d961acf890f8004e056002539bfea6fbe` |
| `tools/push_supplier_check.py` | `2111b9192c5724e0cee21f0360a0cd0b75dfad0bac57036191367b4f5653df5e` |
| `tools/push_supplier_runs.py` | `c1cc394a87618fcb4e29231c588c14679c63e4c8f39dbf49e6141bfc5f5d1da9` |
| `bench/oracle/m3_push_supplier/review/arms-recapture.json` | `67f54a33c67f339232dd74e8548298b25768cc3d0792be4360f3a3f4b4dfe50a` |
| `bench/oracle/m3_push_supplier/review/synthetic-recapture.json` | `deac5b7b8c3962db031233cf7d023ad0ad62f670b9d79b9f738d436c6844f806` |

## 남은 한계

- 자연 support=-1, 정확 drop 임계값의 양쪽·동률, 역순 sparse 보존은 합성 backend 사례로 보강했다. 자연 관측과 구분한다.
- batch·x87·전체 crossover·work/시간 패리티·Rust 공급자 제어·실제 FTRAN/LU 구현은 검증하지 않았다. Rust 구현은 M3-04c-14, 실제 수치 연결은 M3-04c-21 후속이다.
- 새 Rust 소스 변경이 없는 오라클 작업이라 검토자가 workspace 전체 빌드·clippy·Python 확장 설치를 수행하지 않았다.
