# M3-04c-21a 독립 검토

- 대상: branch `work/m3-04c-21a`, commit `6133f13..3af1cf7`(검토 중 `3af1cf7` 문서 커밋이 추가됨), 기준 main `7c3a18f`.
- 검토자는 추적 파일을 바꾸지 않았다. 이 디렉터리에 `review.md`, `review_probe.py`(분포 확인·추가 변조), `retry-review.json`(새 캡처)만 만들었다.

## 1. 실행한 명령과 결과

| 명령 | 결과 |
|---|---|
| `python3 tools/push_ftran_check.py --controls --acceptance --self-test` | rc 0. controls arms 578/168, retry 221/60, interior 252/72, control 0. acceptance 50호출. self-test 35종 모두 거부 |
| `python3 bench/oracle/m3_push_ftran/review/review_probe.py` | 분포 확인과 추가 변조 8종(§4) |
| `tools/gurun.sh node tools/push_ftran_capture.js primal bench/oracle/m3_push_retry/retry.lp bench/oracle/m3_push_ftran/review/retry-review.json "$(cat bench/oracle/m3_push_retry/retry.extra.json)"` | rc 0, 사건 221. `retry-1.json`과 비교하면 pid·run_id만 다르고 `events`·`result`·`output`·`environment`·해시·params는 모두 같다. Gurobi는 1프로세스만 실행했다 |

## 2. 원본 대조

1. **반환 주소**: `source.asm`의 call+5와 모두 일치한다. 214344 `0x1208b6`→`0x1208bb`, 21b9de `0x120922`→`0x120927`, 21574d `0x1219a9`→`0x1219ae`, 2126cd `0x121ca1`→`0x121ca6`, 21686f `0x121ccc`→`0x121cd1`, wrapper 09cf9b `0x12066e`→`0x120673`, 09d008 `0x12072b`→`0x120730`. 본체 안에서 각 커널을 호출하는 지점은 하나씩이다.
2. **본체 인자 위치**: push 6개와 `sub 0xa8`, 반환 주소를 더하면 stack 인자는 `0xe0`에서 시작한다. `0xe0`=a[6] spike(r14), `0xe8`=a[7] param_8 vals(r11, null이면 F+0x1b8, `0x12080b`), `0xf0`=a[8] param_9 mark(r9, null이면 F+0x1b0, `0x120817`), `0xf8`=a[9] param_10(rsi, null이면 **F+0x1a8**, `0x120823`), `0x100`=a[10] work다. agent의 `vals_null=a[7]`, `mark_null=a[8]`은 맞다. `tools/push_ftran_agent.js:299` 주석의 `spike,mark,vals` 순서만 반대다(외관).
3. **wrapper 인자**: 09d008은 `(rsp)=rcx` spike, `8=r8`, `0x10=r9`, `0x18`·`0x20`은 자기 stack 인자이고 `ecx=1`이다. spike가 null이면 `ebp=ecx=0`이다. 09cf9b는 `0..0x18`이 0이고 `0x20=r8` work다. 검사기의 save·spike_null 규칙과 맞는다.
4. **scratch 형식**: `21686f` 호출에서는 `r8←r9`(F+0x1b0)가 Ghidra param_6이고 int로 쓰이며, `r9←r11`(F+0x1b8)이 param_7이고 double로 쓰인다(001223e0 C:109–151). `21b9de`는 6번째 레지스터 r9가 F+0x1b0이고 int 계수로 쓰이며(C:90–154), stack0 F+0x1b8은 double로 쓰인다(C:167–201). agent가 x1b0을 i32, x1b8을 f64로 읽는 것은 맞다.
5. **L 판정**(C:113–164, asm `0x12082c–0x120888`): dense 입력이면 항상 214344다. sparse 입력은 F30 치환 뒤 `count·F104 > m`(`imul; cmp; jg`)이면 214344, `F128 ≥ 0.1m`이면 214344(`comisd; jb`), 아니면 21b9de다. `l_kernel`과 같다.
6. **U 판정**(C:687–699, asm `0x121c5d–0x121c8f`): count<0이면 2126cd다. 그 밖에는 `(m−F100)+Fec+count < m`(int)이거나 `(dd·dd)·count < m+m`이면 21686f다. 곱셈 순서도 `l_kernel`/`u_kernel`과 같다. NaN 비교 차이는 `f64()`가 유한값을 강제해 생기지 않는다.
7. **spike**(C:703–778): dense(`count<1`)이면 `F10<|v|`(strict)인 위치를 오름차순으로 저장하고 값은 `v·save`다. sparse이면 count와 목록을 복사하고 행 위치에 `v·save`를 쓴다. save=1이므로 값의 비트가 같다. `spike_from`과 같다.
8. **입력 변환**(C:111–160): dense gather는 `out.val[i]=in.val[F20[i]]`다. 그다음 `out.count=c`, out.idx/val↔in의 원래 idx/val을 서로 바꾸고 `in.count=-1`로 둔다. sparse는 `idx'=F30[idx]`를 제자리에서 바꾼다. 검사기의 규칙3과 맞다(dense는 in 쪽 별칭·값만 확인한다).
9. **factor 불변**: spike를 뺀 전체 factor를 진입·반환에서 비교하므로 맞다.

## 3. 발견 사항(심각도순)

### 병합 차단: 없음

### 비차단(수정 권고)

1. **[중·문서 사실 오류] F+0x1a8을 "본체·FTRAN 커널이 읽지 않는다"고 적었지만 사실이 아니다.** `docs/m3/primal_push_ftran_oracle.md:19`에 해당한다. 본체는 param_10이 null이면 F+0x1a8을 기본값으로 읽는다(asm `0x1207cf`, `0x12081e–0x120823`). 이 값은 `21b9de`의 stack1로 넘어가고(`0x1208f4`), 21b9de는 이를 레벨 bucket용 int scratch로 쓴다(0012a990 C:262–280, 앞부분 `memset [m, m+levels]` 뒤 사용). Ghidra C 본체 서명에서는 이 기본값이 빠져 있다. 자연 자료는 F128=−1뿐이어서 이 레벨 경로가 실행되지 않았다. 따라서 캡처·판정 결과는 영향을 받지 않는다. 21b가 이 문장을 그대로 믿지 않도록 "param_10 기본 F+0x1a8 = 21b9de 레벨 경로 int scratch, 자연 미관측"으로 고칠 것을 권한다. agent의 `p10` 이름도 같은 의미로 기록하면 좋다.
2. **[중·공허 규칙] dense spike 문턱 필터가 자료에서 아무것도 거르지 않았다.** spike dense 8회 모두 U 입력 m개 값이 `|v|>1e-13`이어서 spike가 전체 m 위치였다. "ZERO 초과만 저장"이라는 술어는 자료로 구별되지 않는다. 0 보존 같은 다른 술어도 통과한다. §7의 "±ZERO 정확 경계 미관측"보다 범위가 넓다. "필터에서 제외된 위치 0건"을 미관측 한계에 추가할 것을 권한다. 검사기 논리 자체는 맞다. 협조형 변조 T6/T7로 strict 경계를 정확히 구현한 것을 확인했다(§4).
3. **[저·공허 분기] U 판정식은 sparse U 입력 33회에서 모두 참(21686f)이었다.** sparse이면서 판정이 거짓(→2126cd)인 자연 사례가 0건이므로 수치 부분은 한쪽 방향으로만 관측됐다. 변조 `R4-U-condition`은 거짓 방향을 거부하므로 검사기 코드는 실행된다. 문서 §6 표에는 일부만 적혀 있다("count 1–2, F_ec=0").
4. **[저·미검사 필드] 기록하지만 판정하지 않는 필드가 있다.** 추가 변조에서 다음은 거부되지 않았다(§4 T1–T5, T8).
   - eta=0 호출에서 U 진입 입력 = L 반환 출력. C:167–178은 순수 포인터 교환이며 자료 45/45에서 성립하지만 규칙이 없다.
   - L 반환 출력 값(수치는 21b 범위이므로 합당하다).
   - dense 교환 뒤 out 구조체(별칭·원래 입력 값).
   - 본체 반환 별칭 `alias`. 계약 규칙6은 "버퍼 별칭"을 말하지만 진입 ctx만 확인한다.
   - 공급 호출의 `input_after`(재계산만 wrapper와 대조한다).
   - U 반환 시점의 in 벡터.

   21b의 비트 재생 기준으로 쓰기 전에 최소한 "eta0이면 U in == L out"과 반환 별칭을 규칙으로 고정할 것을 권한다.
5. **[저·미실행 코드] 미실행 경로의 검사가 느슨하다.** `count0`·`L_out0` 경로는 자료 0건이라 실행되지 않았다. 특히 `tools/push_ftran_check.py:227–228`의 L 출력 0 조기 종료는 spike 불변·본체 out count 0·wrapper raw 대조 없이 `return`한다. C(LAB_002219b2)에 맞추려면 spike 불변과 out count 0도 확인해야 한다. `spike_from`은 R 뒤 count가 0이면 C처럼 dense 루프로 처리하지만, captured 벡터에 m개 값이 없어 IndexError가 난다(`:141–146`). 두 경우 모두 미관측이라 현재 결과에는 영향이 없다.
6. **[외관]** `tools/push_ftran_agent.js:299` 주석의 인자 순서 `mark,vals`는 실제 `vals(a[7]),mark(a[8])`와 반대다.

## 4. 추가 변조(review_probe.py, in-memory)

| 변조 | 기대 | 결과 |
|---|---|---|
| T1 eta0 U 진입 입력 1비트(arms save0) | 이상적으로 거부 | **통과**(발견4) |
| T2 L 반환 출력 1비트 | 수치 21b 범위 | 통과 |
| T3 dense 교환 뒤 out 별칭 | 이상적으로 거부 | **통과**(발견4) |
| T4 본체 반환 별칭 | 규칙6 문구상 거부 | **통과**(발견4) |
| T5 공급 입력 후상태 1비트 | "기록" 대상 | 통과 |
| T6 dense spike 위치 값을 정확히 ZERO로 바꾸고 spike에서 제외(협조) | 통과(strict 규칙상 옳음) | 통과 |
| T7 같은 위치를 ZERO+1ulp로 바꾸고 제외(협조) | 거부 | 거부(`interior#3.spike 저장.count`) |
| T8 U 반환 시 in 벡터 1비트(재계산) | 이상적으로 거부 | 통과 |

자료 분포: F128={−1}, F104={2}, dense spike에서 걸러진 값 0건(정확 0도 0건), eta0 호출 45회 모두 U in == L out, sparse U에서 판정 거짓 0건.

## 5. 결론

원본 식별(반환 주소·인자 위치·scratch 형식), C 판정식 이식, 독립3회와 새 재캡처 재현, 선행19 비간섭, 변조35 거부가 모두 확인됐다. 문서 사실 오류(F+0x1a8)와 공허·미검사 규칙은 현재 자료와 판정 결과를 바꾸지 않는다. 문서 정정과 한계 목록 보강은 병합 전에 하기를 권한다.

병합 차단 문제: 없음
