# M2-06h-1 LP 저장·복구 오라클

## 1. 범위와 주장

종류: 읽기 전용 오라클. 기준 main `04bb80e`, 선행 06g-2 병합 확인, 미커밋 선행 없음. `695bd7` double 경로의 factor 저장 배열과 복원 직후 bound를 포착한다. Rust 구현·오류·x87·병렬·전체 LP 풀이 일치는 제외한다.

## 2. 원본과 상태 경계

13.0.3 바이너리 SHA256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
C: `00576220`(68d6ec), `00576a30`(save), `005754e0`(restore), `00571770`(695bd7). asm 진입 push와 SysV rdi/rsi/rdx 확인. 06g-2 캡처기를 확장하며 기존 사건은 투영 비교한다. 원시 포인터는 기록하지 않는다.

## 3. 비교 계약

저장 구조의 포인터 배열을 의미 좌표로 직렬화한다. 행 수 m, U 끝 = 저장+0x90의 int64[m], L 끝 = 저장+0x68의 int64[m], eta 길이 = 저장+0xcc(int32). 각 배열은 복원 helper가 memcpy하는 유효 길이만 비교한다. 정수·수치는 원시 바이트 정확 일치(±0·NaN 포함). 복원 뒤 LU의 대응 배열과 저장 배열이 일치해야 한다. bound setter 반환마다 before/after와 요청 열·방향·값을 기록하고 setter는 변경을 큐에 넣으므로(C `d8d5f` 확인) 적용 경계는 `edd71` 반환이다. setter 직후 기존 모델 bound 보존과 flush 뒤 요청 적용을 구분해 확인한다. 기존 solve 전후·최종 부모 bound 검사도 유지한다.

## 4. 사례

pk1, Threads=1 Seed=0 NodeLimit=300 GURO_PAR_SBMAXITERS=10, 독립 3회 + 무훅 1회. 모델·도구·원본·캡처 SHA는 manifest. 자연 실행 up/down solve와 restore가 필수이며 0회는 거부한다. 기존 06g-2 pk1 iters10 자료로 투영 비간섭 비교.

## 5. 재실행

`tools/mip_sb_snapshot_runs.sh bench/oracle/m2_06h1`
`python3 tools/mip_sb_snapshot_check.py --self-test`
해시 0·빈 사건·factor 누락/값/길이·bound 변조를 거부해야 한다. Rust/Python 패키지 미변경이므로 cargo/pytest 해당 없음.

## 6. 결과

미실행. 첫 탐색에서 주소·유효 길이 확인 뒤 결과를 추가한다. 계약 완화로 실패를 합격시키지 않는다.

## 7. 판정과 인계

진행 중. 동등성 미검증. 다음: 탐색 캡처, 3회 재현, 검사기, 독립 검토. branch `work/m2-06h-1`. 공유 문서는 다른 세션 점유 중으로 아직 미수정.
