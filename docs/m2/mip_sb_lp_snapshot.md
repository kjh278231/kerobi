# M2-06h-1 LP 저장·복구 오라클

## 1. 범위와 주장

종류: 읽기 전용 오라클. 기준 main `04bb80e`, 선행 06g-2 병합 확인, 미커밋 선행 없음. `695bd7` double 경로의 factor 저장 배열과 복원 직후 bound를 포착한다. Rust 구현·오류·x87·병렬·전체 LP 풀이 일치는 제외한다.

## 2. 원본과 상태 경계

13.0.3 바이너리 SHA256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
C: `00576220`(68d6ec), `00576a30`(save), `005754e0`(restore), `00571770`(695bd7). asm 진입 push와 SysV rdi/rsi/rdx 확인. 06g-2 캡처기를 확장하며 기존 사건은 투영 비교한다. 원시 포인터는 기록하지 않는다.

## 3. 비교 계약

저장 구조의 포인터 배열을 의미 좌표로 직렬화한다. 행 수 m, 행 U 끝 = 저장+0x90의 int64[m], 열 U 끝 = 저장+0x68의 int64[m], 접두 행 목록 길이 = 저장+0xcc(int32). 각 배열은 복원 helper가 memcpy하는 유효 길이만 비교한다. 정수·수치는 원시 바이트 정확 일치(±0·NaN 포함). 복원 뒤 LU의 대응 배열과 저장 배열이 일치해야 한다. bound setter 반환마다 before/after와 요청 열·방향·값을 기록하고 setter는 변경을 큐에 넣으므로(C `d8d5f` 확인) 적용 경계는 `edd71` 반환이다. setter 직후 기존 모델 bound 보존과 flush 뒤 요청 적용을 구분해 확인한다. 기존 solve 전후·최종 부모 bound 검사도 유지한다.

## 4. 사례

pk1, Threads=1 Seed=0 NodeLimit=300 GURO_PAR_SBMAXITERS=10, 독립 3회 + 무훅 1회. 모델·도구·원본·캡처 SHA는 manifest. 자연 실행 up/down solve와 restore가 필수이며 0회는 거부한다. 기존 06g-2 pk1 iters10 자료로 투영 비간섭 비교.

## 5. 재실행

`tools/mip_sb_snapshot_runs.sh bench/oracle/m2_06h1/새디렉터리` (기존 출력 덮어쓰기 거부; 저장 기준 검사는 아래 명령)
`python3 tools/mip_sb_snapshot_check.py --self-test`
해시 0·빈 사건·factor 누락/값/길이·bound 변조를 거부해야 한다. Rust/Python 패키지 미변경이므로 cargo/pytest 해당 없음.

## 6. 결과

관측 범위 통과. 상세 결과는 아래 실행 결과 보완을 따른다. 기존 `python3 tools/mip_sb_restore_check.py --self-test`도 종료0, 6사례·변조15종 회귀 통과.

## 7. 판정과 인계

오라클 수행 완료, pk1 관측 배열·bound 적용 경계는 통과. Rust 동등성·전체 L/eta·접두 행 목록 비어 있지 않은 경우·오류·x87·병렬은 미검증. 다음 M2-06h-2는 실제 LP 결과를 입력으로 후보 제어를 재생하며 전체 LP 동일성 주장을 제외한다. branch `work/m2-06h-1`, 계약13ad828·훅2d1e4ff·캡처8ffd6e3 뒤 최종 검토 반영. shared 점유가 M1-02v에 있어 main 병합과 docs/11_progress.md·12_roadmap_diagram.md 갱신은 대기한다. 작업 branch 인계만 갱신하며 상세 자료를 main에 복사하지 않는다. 교훈: docs/lessons/m2_sb_snapshot.md.

### Rust 입력 재구성 표(C·캡처)

| 저장 구조 포인터 | 복원 LU 포인터 | 원소 수 × 바이트 |
|---|---|---|
| +0x30 | +0x50 | m × 8 |
| +0x90 | +0xb0 | (m+1) × 8 |
| +0xb0 | +0xd0 | 행 U끝 × 4 |
| +0xa8 | +0xc8 | 행 U끝 × 8 |
| +0x68 | +0x88 | (m+1) × 8 |
| +0x78 | +0x98 | m × 8 |
| +0x88 | +0xa8 | 열 U끝 × 4 |
| +0x80 | +0xa0 | 열 U끝 × 8 |
| +0xd0 | +0xf0 | 접두 행 수 × 4 |
| +0xd8 | +0xf8 | m × 4 |

행 U끝은 저장+0x90 배열의 int64[m], 열 U끝은 저장+0x68 배열의 int64[m], 접두 행 수는 저장+0xcc의 int32다. 실제 캡처 m=47, 접두 행 수=0으로 접두 행 목록 원소 복사는 실행되지 않았다. 이는 유효 배열 접두부이며 할당 용량·미사용 꼬리를 입력으로 삼지 않는다. 기존 06g-2 `state`가 basis/status/flags/vector/basic을 제공한다.

save helper는 LU +0x20에서 0x128바이트, +0x148에서 0x30바이트를 복사하고 +0x188/+0x1e0/+0x1e8/+0x1f4/+0xc를 복사하며 +0x184를 0으로 한다(C). 포인터가 포함된 블록 전체는 ASLR 독립 원시값 비교 대상이 아니다. 복원은 LU +0x40/+0x48에 identity 행 번호, +0x1a0에 2e9, +0x90에 열 U 다음 포인터, +0xb8/+0xc0에 행 U 다음 포인터를 다시 만든다(C). 이 파생 필드·블록의 개별 스칼라 값은 이번 캡처에서 별도 검증하지 않았다. 전체 factor 메모리 동일성이나 LP 재시작 가능성을 주장하지 않는다.

### 실행 결과 보완

최종 판정 자료는 `bench/oracle/m2_06h1/verified/manifest.json`이다. `final/`은 scale 필드 추가 전 탐색 자료이며 그 시점 검사에서 요청 1이 내부 bound 8로 적용돼 실패한 상태를 보존했다. C `edd71`의 bound 처리에서 `abs(value)<1e30`이면 열 scale로 나누는 것을 확인해 scale·flip 입력을 추가하고 **다시** 독립 3회+무훅 캡처했다. 판정 범위를 완화하지 않았다. flip 적용 사례는 이번 입력에서 없으며 발견하면 검사기가 거부한다.

- 최종 캡처 3회는 JSON 전체와 gzip 해시까지 동일. 무훅 출력 동일.
- factor 저장 24·복원 110(up/down 각55), 요청220·flush134 비교. 원시 바이트 일치. 모델 bound는 setter 직후 보존, flush에서 큐의 마지막 방향별 요청을 scale 변환해 적용한 결과와 일치.
- 기존 06g-2 pk1 iters10 캡처에 추가 사건·필드만 제거한 전체 events/counts/output/params 투영 동일.
- `python3 tools/mip_sb_snapshot_check.py --write-manifest --self-test`: 종료0, 변조19종 거부.
- Rust·Python 패키지 미변경. cargo/pytest 미실행(해당 없음). Gurobi 캡처는 gurun 직렬 실행, Herdr를 사용하지 않은 짧은 개별 실행이다.

독립 검토에서 `LU+0x88`은 열 방향 U, `+0xec/+0xf0/+0xf8`은 접두 행 목록임을 기존 M1 원본 필드표로 재확인했다. 첫 탐색의 l/eta 이름을 colEnd/prefixCount로 정정하고 최종 자료를 verified에 새로 캡처했다. L·eta 전체 내용은 이 helper의 배열 복사 대상이 아니며 미검증이다. 첫 double의 bound 요청/flush를 전부 지우는 누락을 검사기가 놓쳐 solve마다 flush와 현재 bounds를 연결하고 변조19번째로 추가했다.
