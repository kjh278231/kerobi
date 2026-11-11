# M2-06c-3 검증 계약과 결과: plunge 규칙 4·7·9 점수, 규칙 본문 조상 질의, 규칙 5 뒤 RNG

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 앞선 작업은 [M2-06c](mip_plunge_rule.md)(규칙 번호·RNG·규칙 3/12·꼬리), [M2-06c-2](mip_plunge_score.md)(규칙 1·2·5·6 점수, 규칙 4·7은 asm 식만), [M2-06d](mip_plunge_rust.md)(Rust `plunge`, 규칙 4·7 385호출은 원본 선택을 입력으로 씀)다. 사용자의 "스프린트 이어서 작업해줘 너가 작업자야" 지시로, M2 인수인계의 첫 다음 작업을 맡았다(2026-11-11, 작업자는 이 세션).

## 1. 범위와 주장

- 작업 종류: 조사·오라클 구축(새 원본 캡처 + Python 재생). Rust·Python 패키지는 바꾸지 않는다.
- 조사 질문:
  - Q4: 규칙 4는 [06c-2 §2](mip_plunge_score.md#2-원본과-상태-경계)의 asm 식(`depth − |x[I+0x34] − round(I+0x38)|/100`, `I+0x40 == '<'`면 floor 아니면 ceil, `I+0x34 < 0` 건너뜀)대로 고르는가. x(`5e6f57`)는 어떤 값인가.
  - Q7: 규칙 7은 `(int8 I+0x40 + 10·depth) − (est − minEst)/1e8`의 엄격 최대인가. `I+0x40`에는 어떤 값이 들어 있는가.
  - Q9: BranchDir = ±1에서 규칙 9는 `10·depth − q ∓ int8 I+0x40`(r10d = −1이면 −)의 엄격 최대인가.
  - QA: 규칙 1·6 본문의 `5fb13c` 질의(첫 반복의 새 최댓값마다 1회, 둘째 반복의 sc > best 항목마다 1회)의 순서·인자가 06c-2 재생과 같은가. 질의 결과를 직접 쓰면 06c-2의 재생 불가 34호출이 모두 재생되는가.
  - Q5: 규칙 5의 `a7b4d6` 뒤 슬롯 `0x70(%rsp)`은 LCG(`rule.st`) 한 단계이고 반환값은 그 tempering 31비트 정수인가.
- 완료 후 주장할 것: 위 질문에 캡처 재생으로 답한다(첨자·질의 순서·RNG 상태 정확 일치). 판별되지 않은 절은 변종 생존으로 따로 적는다. 이 작업은 Rust 일치를 주장하지 않는다.
- 제외 범위와 후속:
  - Rust `plunge::pick`에 규칙 4·7·9를 더하는 것: 새 캡처를 하네스 입력으로 쓰는 별도 구현 작업 **M2-06d-3**(가칭).
  - 규칙 11(목적 간격 −1e100, 전부 정수 열), 규칙 8·10, hint 형제 교환(r8b), MIPFocus = 4, 제외 항목(`I+0x41`): 이번 사례에서 닿지 않으면 asm만으로 남긴다.
  - x의 열 공간(원래·축약): 탐색에서 저절로 드러나지 않으면 미확인으로 남긴다.
- 선행: main `b5c118a`(M2-06d 병합). 미커밋 선행 변경 없음(main의 `.env`는 추적 밖, 무관).

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`, 주소는 file offset. `6e8eb3` = `0x5b29d0`(본체는 `0x5b4c50`까지, [교훈](../lessons/tooling.md)).
- 규칙 4 본문 `0x5b3e1e`–`0x5b3fa6`(objdump 직접 읽음): `call 5e6f57`(`0x5b3e35`, 반환 `0x5b3e3a`, rax = x) 뒤 단일 루프. 항목마다 `I+0x41 ≠ 0` 또는 `movslq I+0x34 < 0`이면 건너뜀. `I+0x40 == 0x3c`이면 `floor(I+0x38)`(`0x5b4a68`), 아니면 `ceil`. `sc = (double)I+0x2c − (|x[I+0x34] − b| & 절댓값 마스크)/100`, `comisd`·`cmova`(sc > best일 때만 첨자), best는 `maxsd`. work += 7.0·n.
- `5e6f57` = `0x509b60`(36바이트 잎 함수): `(M+0x3220)+0x50 < 0`이면 null, 아니면 `*(((M+0x3220)+0x18)[idx·0x20] + 0x18)`(해 풀 최선해 x).
- 규칙 7 본문 `0x5b39fc`–`0x5b3bbc`: 2개씩 풀어 쓴 루프. `sc = cvtsi2sd(movsbl I+0x40 + 10·I+0x2c) − (est − m)/1e8`, 엄격 최대, work += 5.0·n.
- 규칙 9 본문 `0x5b3910`–`0x5b39f7`: [06c-2 §2](mip_plunge_score.md#2-원본과-상태-경계) 식, r10d = env+0x27e0(BranchDir).
- 규칙 1·6 본문의 `5fb13c`(`0x5101a0`) 호출 반환 지점: `0x5b3c2c`(규칙 6 첫 반복), `0x5b3d1f`(규칙 6 둘째), `0x5b42e6`(규칙 1 첫), `0x5b4402`(규칙 1 둘째). 꼬리 호출(`0x5b465b`·`0x5b4766`)은 06c 도구가 이미 잡는다.
- 규칙 5: `call a7b4d6`(`0x88a870`, `0x5b3dc0`, rdi = `&0x70(%rsp)`, 반환 `0x5b3dc5`, eax = 정수).
- 시작 경계·종료 경계: 06c와 같다(`rule` probe `0x5b361d`, `pick` probe `0x5b44a9`). 상태는 모두 읽기만 한다.
- 호출 문맥: 최상위 `6e8eb3` 호출만(06c `cur`). Threads=1, Seed=0, NodeLimit 2000.

## 3. 비교 계약(판정 규칙, 첫 캡처 전 고정)

### 3.1 캡처 도구

`tools/mip_plunge_score.js` = `tools/mip_plunge_rule.js`(06c, manifest 해시 고정이라 바꾸지 않는다)를 복사하고 **새 사건만 더한다**. 06c 사건(`pick.list` 행 포함)은 한 바이트도 바꾸지 않는다.

| 새 사건 | 시점 | 내용 |
|---|---|---|
| `ents` | `pick` probe(`0x5b44a9`) 직전, 같은 함수형 probe 안 | 목록 항목마다 `[I+0x34 (s32), I+0x38 (hex), I+0x40 (u8)]` |
| `x4` | `5e6f57` 반환, 호출 지점 `0x5b3e3a`만 | `null`(x 포인터가 null인지), 그리고 `I+0x41 = 0`이고 `I+0x34 ≥ 0`인 항목마다 `[첨자, I+0x34, x[I+0x34] hex]` |
| `ranc` | `5fb13c` 반환, 위 네 본문 지점 | 지점, 시작 노드 id, d, 시작 노드 depth, 결과 노드 id(null) |
| `r5` | `a7b4d6` 반환, 호출 지점 `0x5b3dc5`만 | 반환 eax(s32), 슬롯 `0x70(%rsp)` hex |

- `0x5101a0`에는 기존 훅 하나만 둔다(사이트 목록에 본문 네 지점을 더하고, 꼬리 지점은 기존 `anc` 사건 그대로).
- 06c 투영 = 새 사건 네 종류를 뺀 사건 목록과 `counts`.

### 3.2 사례

| 사례 | 실제/합성 | 노리는 분기 | 파라미터(공통 Threads=1, Seed=0, LogToConsole=0, NodeLimit=2000) |
|---|---|---|---|
| markshare_4_0, pk1, neos5, tr12-30, pk1-pool | 실제 | 06c 5사례 그대로: 규칙 4 347·7 38·1 2,346·6 2,000·5 239(06c 표) | 06c와 같음(pk1-pool은 PoolSearchMode=2) |
| pk1-bd1 | 실제 | BranchDir = 1 → 규칙 9(06c pk1은 `M+0x848` = 0이라 hint·6 강제가 없다) | BranchDir=1 |
| pk1-bd-1 | 실제 | BranchDir = −1 → 규칙 9의 부호 반대 | BranchDir=−1 |

모델은 `../re/bench/data/miplib/<model>.mps.gz`. 각 사례를 독립 프로세스로 3회 캡처한다. BranchDir 두 사례는 06c 기준 자료가 없으므로 고정된 06c 도구 `tools/mip_plunge_rule.js`로 같은 파라미터의 대조 실행을 1회씩 더 한다(pk1-pool 선례).

### 3.3 검사기 `tools/mip_plunge_r47_check.py`

- **C0(대조)**: manifest(바이너리·도구·모델·파일 해시), 사례마다 3회 사건·`counts` 일치, 출력 줄(`DONE <status> <nodes>`, 3회 같음). 06c 5사례는 투영이 `bench/oracle/m2_06c` run 1과 같다. BranchDir 두 사례는 투영이 06c 도구 대조 실행과 같다.
- **C1(기존 검사기 회귀)**: 7사례 run 1의 투영에 06c 검사기 `check_capture`(R/G/S/T)와 06c-2 검사기 `check_capture`(S2/W)를 그대로 돌려 통과. 규칙 9 호출에서 기존 검사기가 범위 밖으로 멈추면 그 이유를 결과에 적고 해당 검사만 규칙 9 호출을 건너뛴다(규칙 9 선택은 아래 S9가 본다).
- **S4/S7/S9**: 규칙 4·7·9 호출마다 재생 첨자 = `pick.idx`(−1이면 0). 재생은 §2의 식, 입력은 `pick.list`(est·depth·`I+0x41`), `ents`, `x4`, `rule.r10d`, minEst(06c-2 `min_est`).
  - 규칙 4 호출에는 `x4`가 정확히 1개, `null` = 거짓이어야 한다(null이면 실패). `x4`의 항목 집합은 재생이 x를 읽는 항목 집합과 같아야 한다.
  - 규칙 4가 아닌 호출에 `x4`가 있으면 실패. 규칙 5가 아닌 호출에 `r5`가 있으면 실패.
- **SA(규칙 1·6 본문 조상)**: 재생이 부르는 `5fb13c` 질의 순서(지점, 시작 id, d)가 `ranc` 사건 순서와 정확히 같다. 결과는 `ranc`의 것을 쓴다. 시작 depth는 `pick.list`와 같아야 한다. 이 재생의 첨자 = `pick.idx`가 규칙 1·6 **모든** 호출에서 성립해야 한다(재생 불가 0). 꼬리 사슬(06c `anc`)로 같은 질의를 계산할 수 있는 경우 그 결과도 `ranc`와 같아야 한다.
- **S5(RNG)**: 규칙 5 호출마다 `r5.st` = LCG(`rule.st`), `r5.eax` = tempering 31비트 정수(06c-2 `tint`). 그 호출의 첨자도 06c-2 식으로 다시 확인한다.
- **합격 조건**: 위 검사 전부 일치, 그리고 사례 합계에서 모두 1회 이상: 규칙 4 재생(목록 2개 이상), 규칙 7 재생(목록 2개 이상), 규칙 9 재생(r10d = 1, r10d = −1 각각, 목록 2개 이상), 규칙 4에서 floor와 ceil 경로 각각(x를 읽는 항목 기준), 규칙 1·6 재생에서 06c-2 재생 불가였던 호출(꼬리 조상 null, maxd ≥ 2), `r5`.
- **변종(결과 전 고정, 재생에 적용해 거부 사례 보고. 생존 = 그 절은 이 실행으로 판별되지 않음)**:
  X1 규칙 4 floor/ceil 바꿈, X2 규칙 4 반올림 없이 `I+0x38`, X3 규칙 4 나눗수 10, X4 규칙 4 depth 항 없음, X5 규칙 4 동률 `>=`(뒤 항목), X6 규칙 4 `I+0x34 < 0` 항목을 x[0]으로 계산, X7 규칙 7 방향 바이트 무시, X8 규칙 7 방향 바이트 부호 −, X9 규칙 7 depth 계수 1, X10 규칙 9 부호 반대(r10d = −1이면 +), X11 규칙 9 방향 바이트 무시, X12 규칙 1·6 첫 반복 마지막 최심 항목(06c-2 W2), X13 규칙 1·6 둘째 반복 동률 `>=`, X14 규칙 1·6 둘째 반복이 모든 비제외 항목에 질의(게으른 평가 아님), X15 규칙 5 뒤 슬롯 = 시드(LCG 없음).
- **self-test(변조, 하나라도 거부되지 않으면 실패)**: 규칙 4·7·9 각각 `pick.idx`(선택을 바꾸는 호출), 규칙 4 선택 항목의 x 값(재생 선택을 바꾸는 호출), 규칙 4 `x4` 삭제, 규칙 4 `x4.null` = 참, 규칙 7 선택 항목의 `I+0x40`(선택을 바꾸는 호출), 규칙 9 `rule.r10d`, `ranc` 결과 id(선택을 바꾸는 호출), `ranc` 사건 하나 삭제, `r5.st`, 규칙 4 호출 전부 삭제(합격 0회), 투영 필드 하나(`pick.list` est), 회차 2 사건 하나, 바이너리 해시 0.
- 수치: 모든 double은 비트 비교(재생은 Python float = IEEE double, `math.floor`·`math.ceil`은 정확).

## 4. 사례와 원본 자료

§3.2. 기준 자료는 `bench/oracle/m2_06c3/`(캡처 21개, BranchDir 대조 2개, manifest). 06c 자료 `bench/oracle/m2_06c`는 그대로 읽기만 한다.

## 5. 재실행 명령

- 캡처(main worktree 아닌 작업 worktree에서도 됨, Gurobi 한 프로세스씩): `tools/hrun.sh --timeout 580000 -- 'tools/mip_plunge_score_runs.sh bench/oracle/m2_06c3'`(124면 `--wait <pane>`).
- 판정: `python3 tools/mip_plunge_r47_check.py`(기대: C0·C1 PASS, 사례별 S4/S7/S9/SA/S5 PASS, `ACCEPTANCE PASS`, 종료 0).
- 판별력: `python3 tools/mip_plunge_r47_check.py --self-test`(`ACCEPTANCE PASS (+ self-test)`, 종료 0).
- 회귀: `python3 tools/mip_plunge_rule_check.py --self-test`, `python3 tools/mip_plunge_score_check.py --self-test`, `python3 tools/mip_plunge_rust.py --self-test`(06c 자료, 바뀌지 않아야 함). cargo·pytest는 Rust·Python 미변경이라 해당 없음.

## 6. 실행 결과

(실행 후 작성)

## 7. 판정과 인수인계

(실행 후 작성)
