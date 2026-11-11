# SB 저장·복원 경계

- **SB bound setter `d8d5f` 반환은 모델 bound 복원 완료가 아니다.** C·캡처: 변경은 큐에 들어가며 `edd71`이 적용한다. 열 scale이 있으면 |값| < 1e30 요청을 scale로 나누므로 요청1이 내부8이 될 수 있다. setter 직후·flush 직후를 분리해 검사한다. (M2-06h-1, [근거](../m2/mip_sb_lp_snapshot.md))
- **`695bd7`은 up 결과가 1e100 이상이면 복구 뒤 down을 풀지 않는다.** C:660 `param_10[j] < 1e+100`이 down 진입 조건이다. 후보 루프를 옮길 때 이 건너뛰기와 복구 순서(reset→restore→bound 복원 요청 뒤 판정)를 같이 옮긴다. (M2-06h-2, [근거](../m2/mip_sb_control_rust.md#6-실행-결과))
- **첫 후보 자료로는 최적 상태 목적 clamp(`obj<=base`면 base)를 판별할 수 없다.** pk1 첫 후보 48방향에서 상태2는 6개뿐이고 모두 base 초과였다. 이 clamp를 검증하려면 다른 후보·모델을 캡처한다. (M2-06h-2, [근거](../m2/mip_sb_control_rust.md#6-실행-결과))
- **Rust 변이 검사의 원문 패턴은 소스 수정 때 같이 깨진다.** `mip_sb_control_rust.py`의 mutant `old` 문자열이 소스에 없으면 assert로 멈춘다. 오류 처리 형태를 바꾸면 변이 표와 manifest(`--write-manifest`)를 함께 갱신한다. (M2-06h-2)
