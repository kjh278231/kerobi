# SB 저장·복원 경계

- **SB bound setter `d8d5f` 반환은 모델 bound 복원 완료가 아니다.** C·캡처: 변경은 큐에 들어가며 `edd71`이 적용한다. 열 scale이 있으면 |값| < 1e30 요청을 scale로 나누므로 요청1이 내부8이 될 수 있다. setter 직후·flush 직후를 분리해 검사한다. (M2-06h-1, [근거](../m2/mip_sb_lp_snapshot.md))
