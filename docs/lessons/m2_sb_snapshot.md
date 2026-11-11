# SB 저장·복원 경계

- **SB bound setter `d8d5f` 반환은 모델 bound 복원 완료가 아니다.** C·캡처: 변경은 큐에 들어가며 `edd71`이 적용한다. 열 scale이 있으면 |값| < 1e30 요청을 scale로 나누므로 요청1이 내부8이 될 수 있다. setter 직후·flush 직후를 분리해 검사한다. (M2-06h-1, [근거](../m2/mip_sb_lp_snapshot.md))
- **`695bd7`은 up 결과가 1e100 이상이면 복구 뒤 down을 풀지 않는다.** C:660 `param_10[j] < 1e+100`이 down 진입 조건이다(asm 0x572b37 `comisd; jae` 기준이라 NaN이면 down으로 간다. 디컴파일 `<` 표기는 NaN에서 반대로 읽힌다). 후보 루프를 옮길 때 이 건너뛰기와 복구 순서(reset→restore→bound 복원 요청 뒤 판정)를 같이 옮긴다. (M2-06h-2, [근거](../m2/mip_sb_control_rust.md#6-실행-결과))
- **pk1 첫 후보 자료로는 목적 clamp의 base 선택을 두 상태 모두 판별할 수 없다.** 48방향 중 상태7은 dual==base 34·dual>base 8, 상태2는 obj>base 5·obj==base 1이다. clamp는 asm `maxsd`(디컴파일 `<=` 형태는 NaN에서 다름)이므로 이식은 asm 피연산자 순서를 따른다. 검증하려면 다른 후보·모델을 캡처한다. (M2-06h-2, [근거](../m2/mip_sb_control_rust.md#6-실행-결과))
- **Rust 변이 검사의 원문 패턴은 소스 수정 때 같이 깨진다.** `mip_sb_control_rust.py`의 mutant `old` 문자열이 소스에 없으면 assert로 멈춘다. 오류 처리 형태를 바꾸면 변이 표와 manifest(`--write-manifest`)를 함께 갱신한다. (M2-06h-2)
- **`695bd7`의 전파 경로(`local_90≠0`)와 비전파 경로는 bound·복구 줄이 다르다.** pk1 첫 후보는 모두 전파 경로(C:669–679, 772–781, 복구 루프 592–647·993–1047)였고, 복구는 엄격 부등일 때만 요청하며 work `param_12`에 6·개수를 더한다. 근거 줄은 실제 훅 경로(`6953ff`)로 확인하고 인용한다. (M2-06h-2 재검토, [근거](../m2/mip_sb_control_rust.md#2-원본과-상태-경계))
- **완료 선언 뒤 바꾼 코드는 독립 검토를 다시 받는다.** 06h-2는 첫 검토 뒤 `Unported`·변조를 추가하고 재검토 없이 병합했다가, 사후 재검토에서 clamp 미판별·근거 줄 오인용이 나왔다. 검토 기준 commit을 기록하고 그 뒤 diff가 있으면 재검토한다. (M2-06h-2)
