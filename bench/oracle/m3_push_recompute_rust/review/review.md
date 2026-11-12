# M3-04c-18 독립 검토

판정: **계약 범위 통과**. 구현의 기능 불일치나 합격을 막는 검사기 문제를 발견하지 않았다. 검토자는 `/root/m3_04c18_review`이며 구현·도구·계약은 수정하지 않았다. 작업자의 Cargo 실행과 겹치지 않도록 검토자는 Cargo 빌드·소스 변이를 새로 실행하지 않았다.

원본 C `001c0070_PRIVATE0000000000134a87.c`의 1897–1980·2329–2418행과 보존 asm의 두 구간을 확인했다. FTRAN 반환 뒤 별칭에 따른 basic 복사, 새 header의 열 좌표, 유한 하한 이동, 폭 sentinel, F/B/X/L/N/A/U의 비교 순서와 동률 처리가 Rust와 같다. 원본의 B는 `PInf -= v`, A는 `PInf += (v-w)`이며 basis 위치 순차 가감이다. retry는 status=2·attempt=0·PInf>1e10이며 정상 경계에서 request를 지우지 않는다. round의 요청0 쓰기 `0x1c400e`는 정지 경계 `0x1c400a` 다음이다.

Clippy 대응으로 최종 Rust 폭 계산은 `lower <= -inf || upper >= inf`일 때 I로 결합됐다. 이는 유한 입력 계약에서 원본과 같은 폭이며 lower shift는 여전히 `lower > -inf`일 때만 수행한다. 원본이 나중에 버리는 `upper-lower`의 중간 계산이나 FP 예외 플래그까지 동일하다고 주장하지 않는다.

`RecomputeValues`의 안전한 enum이 별도 raw 복사와 실제 basic 제자리 처리를 구분한다. Rust 입력에는 header/bound/raw와 frame 입력만 있고 원본 marks/basic/PInf 정답은 없다. Python 비교기는 유효성을 확인한 원본 leave/result에서 기대값을 직접 읽으며 계산 모델의 출력으로 Rust 정답을 만들지 않는다. 필드 타입·배열 순서·키 집합·f64 비트·입력 해시와 중복 JSON 키를 검사한다.

독립 실행과 검증 결과는 다음과 같다.

- `tools/gurun.sh ../.venv/bin/python tools/push_recompute_extra.py --capture bench/oracle/m3_push_recompute_rust/review/extra.json`: 종료0, 원본 두 경계 추가10사례·18행 재캡처. 정식 `extra-1.json`과 pid/run_id를 제외한 모든 필드·환경·사건·바이너리/도구/입력 식별값이 일치하고 새 pid/run_id다.
- 검토 스크립트 `python3 bench/oracle/m3_push_recompute_rust/review/check_review.py`: 정상 trace 원복 뒤 종료0. 자연8·합성130, 총138입력·2,958필드가 정확/비트 일치한다.
- 원본 선행17 검사기의 controls/acceptance와82종 self-test, 추가 캡처21종 self-test, Rust trace84종 self-test를 직접 호출해 통과했다.
- 검토자가 별도로 작성한16종 오류 주입은 전부 거부했다. 바이너리/입력 해시0, 빈 사건·빈 Rust 목표, null, manifest 변조, PInf 한 비트·A 누적을4로 변경, 잘못된 분류·request0·별칭·정수 대신 bool 등을 포함한다. 최초 차이를 `review.json`에 보존했다.
- 작업자의 최종 Rust33종 변이 기록은 고정 이름/순서가 일치하고 모두 실제 출력 차이로 거부됐음을 확인했다. 검토자가 해당33종을 새로 빌드한 것은 아니다. 첫 검토가 의도적 request-cleared 변이 trace를 읽어 종료1이었던 사실과 정상 원복 후 재실행 결과를 `review.json`의 execution_history에 구분했다.

낮은 우선순위의 `RecomputeInput` 배열 길이 주석 문제는 부모에게 전달했고, 최종 주석이 열 좌표 bounds와 basis 길이를 구분하도록 수정됐음을 확인했다. 작업 계약의 초기128사례·추가 원본 불필요·4종 생존·독립검토 미사용 문구는 최종 결과 정리가 필요해 부모에게 전달했다. 이는 진행 중 문서이며 기능 판정의 실패로 계산하지 않았다.

검증은 유한값·유효 header/bounds·retry 초기0의 현재 corpus 범위다. 별칭true·free/fixed·retry·정확 경계는 합성 증거이며 자연 도달성 전체를 입증하지 않는다. rhs/FTRAN 호출과 수치 backend, 실제 LU21·루프22·batch23·x87·전체 work/P3는 제외한다. 원본 라이브러리·원본 C/asm·도구·입력·Rust trace·정식/독립 캡처의 해시는 `review.json`에 있다. 독립 캡처 `extra.json`과 재실행 가능한 검토 스크립트도 같은 디렉터리에 보존한다.
