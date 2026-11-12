# M3-04c-21a 한 RHS FTRAN 수치 오라클 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 선행 조사는 [M3-04c-21](primal_push_lu_survey.md) §2.1·4.2·4.3이다.

## 1. 범위와 주장

- 작업 종류: **오라클 구축**. 기준 main `7c3a18f`(M3-04c-21 병합), branch `work/m3-04c-21a`, worktree `../rsolver-m3-04c-21a`. 선행 미커밋 변경 없음. main의 미추적 `.claude/settings.local.json`은 작업과 무관하며 보존한다. 첫 커밋 `6133f13`은 `tools/session.py`의 작업 번호 정규식이 `21a`처럼 끝 문자가 붙은 하위 번호를 거부해 점유가 불가능했던 문제를 고쳤다(shared 점유).
- 완료할 동작: crossover push 안의 한 RHS FTRAN 본체 `2207de`를 공급자 wrapper `09d008`(save=1/spike)와 재계산 wrapper `09cf9b`(save=0)에서 부를 때, **같은 호출의 초기 factor 상태·입력 벡터·L 커널 경계·R 뒤 U 입력·spike 저장·U 커널 경계·raw 출력·입력 후상태·버퍼 별칭·scratch**를 읽기 전용 훅으로 수집해 원본 독립3회·비간섭·판정 규칙·변조 거부로 고정한다.
- 포함 분기: 자연 arms/retry/interior의 공급 FTRAN 42회(sparse 33·dense 9)와 재계산 FTRAN 8회. sparse/dense 입력, L sparse(`21b9de`)/dense(`214344`), R eta 0/양수, spike 저장 sparse 복사/dense 문턱 필터, save0, U sparse(`21686f`)/dense(`2126cd`), 입력 count0 조기 종료가 자연 실행에서 나오는 만큼 센다. 미관측 분기는 미관측으로 기록한다.
- 제외 범위: Rust 구현과 수치 비트 재생(21b), BTRAN(21c·21d), 지연 LU·FT 갱신·재분해 연쇄(21e·21f), 두 번째 RHS 경로 `21574d`의 수치(자연 발생 시 사건만 기록), 품질15a·루프22·batch23·x87·work 산술의 동등성·전체 P3. 원본 수치 명령을 실행하는 합성 경계는 이번 범위에서 만들지 않는다. 자연 미관측 분기(±ZERO 정확 경계·동률·희소 U 등)는 21b 착수 전 보강 후보로 남긴다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 ELF file offset(로드 base + offset)이다.
- C: `../re/decomp/c/00120740_PRIVATE00000000002207de.c`(본체), wrapper `001205d0`(09cf9b)·`00120680`(09d008), 커널 `0012a990`(21b9de)·`0012bd30`(214344)·`001223e0`(21686f)·`00121ef0`(2126cd)·`00123690`(21574d).
- 본체 호출 지점(`tools/dc.py 0x120740 --asm`): `0x1208b6` 214344, `0x120922` 21b9de, `0x1211ee` memcpy(sparse spike 복사), `0x1219a9` 21574d, `0x121ca1` 2126cd, `0x121ccc` 21686f. 커널 훅은 반환 주소(call+5)로 이 지점에서 온 호출만 센다. wrapper에서 본체로 오는 호출은 `09cf9b` `0x12066e`(반환 `0x120673`)와 `09d008` `0x12072b`(반환 `0x120730`)뿐이다.
- 시작 경계: wrapper가 spike를 지운 뒤의 `2207de` 진입. 종료: `2207de` 반환 직후(wrapper 반환 전, wrapper는 본체 뒤 상태를 바꾸지 않음을 선행 `supplier:ftran:leave`/`recompute:ftran:leave`와 대조). 기존 wrapper 진입의 spike(지우기 전)는 선행 agent 사건이 이미 담는다.
- 본체 인자(C 서명): `(F, in, out, save, k5, p6, spike, vals, mark, p10, work)`. `in/out`은 `{i32 count; i32* idx @8; f64* val @16}`이며 일반 sparse val은 목록 위치별 packed, spike val은 행 인덱스로 접근하는 dense 저장소다. vals(param_8)/mark(param_9)가 null이면 F+0x1b8/F+0x1b0(C:103–108)을 쓴다. spike가 null이고 save≠0이면 F+0x1c8(C:705–707).
- 읽는 factor 상태(C·커널의 `param + off`로 확인, 본체 `plVar1 = F+0x20`을 커널에 넘김): m F+8, ZERO F+0x10, 행/기저 순열 F+0x20/0x28/0x30/0x38, U 위치 F+0x40, U 순서 F+0x48, 대각 F+0x50, L 열 시작 F+0x58(long m+1)·값 F+0x60·행 F+0x68, L 레벨 F+0x118/0x120·레벨 수 F+0x128, U 열 시작/끝/경계 F+0x88/0x90/0x98(long)·값 F+0xa0·행 F+0xa8, U 행 시작/끝 F+0xb0/0xb8(long)·값 F+0xc8·열 F+0xd0, F+0xd8·F+0xdc·F+0xe4·F+0xe8, 접두 목록 F+0xec(개수)/0xf0/0xf8, 접두 t F+0x100, F+0x104/0x108/0x110, eta 개수 F+0x180·길이 합 F+0x184·레코드 F+0x198(24byte `{r,len,idx*,val*}`)·F+0x1a0, 풀 F+0x148/0x150/0x158/0x160/0x168, 갱신 F+0x188/0x18c, spike F+0x1c8, work 배율 F+0x1f8, scratch F+0x1b0(i32 표시 m)/F+0x1b8(f64 값 m; `21686f` param_6/param_7, 탐색 뒤 C로 정정). L 행 방향 F+0x70/0x78/0x80과 F+0x1a8은 본체·FTRAN 커널이 읽지 않으므로 이번 상태에서 제외하고 그 사실을 기록한다(BTRAN 21c 범위).
- 경로 판정(원본): L은 입력 count>0이고 `count·F104 <= m`이며 `F128 < 0.1m`이면 `21b9de`, 아니면 `214344`(C:113–164). R은 F+0x180개 eta를 생성순으로(C:167–685). spike는 U 전에 저장한다: R 뒤 count<1(dense)이면 `F10 < |v|`인 위치를 오름차순으로, 아니면 목록을 그대로 복사하고 값은 `v·save`를 행 위치에 쓴다(C:703–778). U는 두 번째 RHS가 없을 때 count>=0이고 `(m−F100)+F_ec+count < m` 또는 `(F108+F110)²·count < 2m`이면 `21686f`, 아니면 `2126cd`(C:687–699, asm `0x121c61–0x121c8f`).
- 모델·설정: 선행19와 같은 arms/retry/interior LP·extra, Threads=1·Seed=0·Presolve=0·Method=2·Crossover=3·LPWarmStart=2, 대조군 arms Crossover=0. RNG·타이머 분기는 주장하지 않는다.

## 3. 비교 계약

| 필드·사건 | 의미·좌표 | 규칙 | 캡처 |
|---|---|---|---|
| `ftran:enter` | 호출 식별(push/공급자 id 또는 재계산 id, wrapper·save·spike null·factor 갱신수), 위 factor 상태 전체, in/out/spike 벡터, scratch | 정수·목록 정확, f64 ±0 포함 비트. 포인터는 저장하지 않고 의미상 별칭(in=S+0x220 등)과 eta 레코드의 풀 offset·mod16만 저장 | `2207de` onEnter |
| `ftran:kernel` | L·U 커널의 진입/반환에서 본체 in/out 두 벡터, spike | 같은 호출의 사건 순서·커널 이름·호출 지점 정확, 벡터 비트 | 커널 onEnter/onLeave(반환 주소 필터) |
| `ftran:leave` | raw out, 입력 후상태, spike, scratch, factor 전체 | factor는 spike 외 진입과 동일해야 함. scratch는 진입·반환 모두 0 | `2207de` onLeave |
| 선행 사건 | 선행19 agent의 전체 사건·최종 결과 | 새 사건을 빼면 저장된 선행19 캡처와 JSON 전체 일치(pid/run_id 제외) | 같은 agent에 훅 추가 |

판정 규칙(결과를 보기 전 고정, 모두 원본 C의 위 식에서 재계산):

1. **식별**: 공급 FTRAN마다 본체 1회, 재계산 FTRAN마다 본체 1회, 수는 선행 `supplier:ftran`/`recompute:ftran` 사건 수와 같다. 본체 반환 out은 선행 wrapper 반환 raw와 비트 동일.
2. **factor 무결성**: F+8=m, F+0x10=1e-13 비트(`3d3c25c268497682`), F20↔F30·F28↔F38·F40↔F48이 서로 역순열, L·U 시작/끝 단조·범위 안, eta 레코드 수=F180·길이 합=F184. 진입과 반환의 factor 필드는 spike 외 동일.
3. **입력 변환**: sparse 입력은 첫 커널 진입에서 `idx' = F30[idx]` 제자리 치환·값 불변. dense 입력은 `out[i] = in[F20[i]]` gather 뒤 두 벡터의 idx/val 포인터 교환(의미상 별칭으로 확인).
4. **경로 선택**: 실제 L/U 커널이 위 판정식과 일치. 입력 count0이면 커널 없음·out count0.
5. **spike**: save=1이면 U 진입 시점의 spike가 R 뒤 벡터에서 위 규칙으로 얻은 목록·값과 비트 동일, save=0 또는 count0 조기 종료면 spike는 본체 진입 상태 그대로. 본체 진입 spike count=0이고 지우기 전 목록 위치 값은 +0이다(wrapper clear).
6. **출력·별칭**: 본체 반환 out = U 커널 반환 시점 out, wrapper가 넘긴 out/in 구조체의 의미상 위치(S+0x210/0x220, 재계산 frame) 유지, 입력 후상태 기록.
7. **scratch**: F1b0 표시 m개 모두 0, F1b8 값 m개 모두 +0을 진입·반환에서 확인.
8. **재현·비간섭**: 바이너리·모델·도구 해시, 독립 pid/run_id 3개의 사건 동일(pid/run_id 제외), 대조군 사건0, 선행19 투영 일치.

- 수치: 모든 f64는 ±0 포함 비트 비교다. NaN/IEEE무한대가 나오면 실패로 보고한다. 수치 L/R/U 값의 독립 재계산은 21b다.
- work·시간: 본체 work 누적값은 기록만 하고 판정하지 않는다.
- 재현성: 자연 3모델·대조군 각 독립3회. 첫 탐색 1회는 history에 보존하고 정식 판정에 쓰지 않는다.

## 4. 사례와 원본 자료

| 사례 | 실제/합성 | 노리는 분기 | 입력 | 증거 |
|---|---|---|---|---|
| arms | 실제 | sparse 공급 25·재계산 3, 지원 생략 반복 | `bench/oracle/m3_push_arms/arms.lp`+extra | 정식3회 |
| retry | 실제 | sparse 공급 8(spike 저장1)·재계산 2 | `bench/oracle/m3_push_retry/retry.lp`+extra | 정식3회 |
| interior | 실제 | dense 공급 9(spike 저장8)·재계산 3, 요청0 갱신 뒤 eta | `bench/oracle/m3_primal_push_step/interior.lp` | 정식3회 |
| control | 실제 | Crossover=0, push 사건0 | arms | 정식3회 |

- 도구: `tools/push_ftran_{agent.js,capture.js,runs.py,check.py}`(선행19 agent 복사 + 훅 추가). 자료 `bench/oracle/m3_push_ftran/`.
- 미관측 분기는 사건 수와 함께 표로 남기고 통과로 세지 않는다.

## 5. 재실행 명령

- 캡처(작업 worktree 루트, Gurobi 한 프로세스): `tools/hrun.sh --timeout 580000 -- 'python3 tools/push_ftran_runs.py <새-출력디렉터리>'`. 기존 출력 덮어쓰기 거부.
- 판정: `python3 tools/push_ftran_check.py --controls`, `--acceptance`, `--self-test`. self-test는 해시0·빈 사건·회차 차이·투영 차이·null-null·규칙 1–7 각각의 대상 필드 변조를 모두 거부해야 종료0이다.
- 회귀: 선행 `push_round_check.py --controls --acceptance --self-test`, `push_lu_survey.py --check`, `push_chain_oracle.py --check`·`--self-test`, `cargo test -p rsolver-lp --test push_chain`과 `push_step_chain_rust.py --compare`. Rust 변경이 없으면 workspace 전체·clippy·확장·pytest·Netlib·P3는 제외한다.

## 6. 실행 결과

- 환경: WSL2 x86-64 Linux 6.6.87.2, Node v22.22.1, Python 3.10. 바이너리 해시는 매 캡처에서 위 값과 일치했다. 정식 캡처 도구 해시: agent `da123ebe…b035ff`, capture `56b06ddf…93d9af`(전체 값은 [manifest](../../bench/oracle/m3_push_ftran/manifest.json)). 계약 `913146c`, 도구·캡처·검사기 `f62034b`.
- 원본 실행 총 20회: 탐색 1차 4회(`history/explore/`, scratch 형식을 반대로 읽은 첫 agent), 탐색 2차 4회(`history/explore2/`), 정식 12회(4사례×3). 탐색 자료는 판정에 쓰지 않았고 실패한 해석을 보존한다.
- **재현성·비간섭**(`--controls` 종료0): arms 578사건(새 168)·retry 221(60)·interior 252(72)·대조군 0사건이 각각 독립 pid/run_id 3회에서 동일하다. 새 `ftran:*` 사건을 빼면 선행19 [`m3_push_round`](../../bench/oracle/m3_push_round/) `*-1.json`의 전체 사건·최종 결과·출력과 JSON 전체가 같다.
- **판정 규칙**(`--acceptance` 종료0, [집계](../../bench/oracle/m3_push_ftran/verification-acceptance.json)): FTRAN **50호출**(공급 42=25/8/9, 재계산 8=3/2/3) 전부 규칙 1–7 통과.

| 분기 | 관측 수 | 비고 |
|---|---:|---|
| save0 / save1 | 41 / 9 | save1은 retry 1·interior 8. 비어 있지 않은 목록의 wrapper clear 1회(interior, 이전 spike 8개 → 모두 +0) |
| 입력 sparse / dense / count0 | 42 / 8 / 0 | dense는 재계산 8회 |
| L `21b9de` / `214344` | 33 / 17 | interior 공급 sparse 입력 9회도 `count·F104 > m`이라 dense L |
| U `21686f` / `2126cd` | 33 / 17 | 희소 U는 U 입력 count 1–2, 접두 목록 0개(F_ec=0)뿐 |
| spike dense 필터 / sparse 복사 | 8 / 1 | sparse 복사는 count1만 |
| eta>0 | 5 | 모두 interior, R 뒤 dense·dense U |
| 비identity 순열 | 50 | F20/F28/F40 모두 실제 순열 |

- **변조 거부**(`--self-test` 종료0): 35종. 바이너리·manifest 해시, 빈 사건, 회차 차이, 선행 투영 차이, 기준 결과, 새 사건0, null-null, 본체 leave 누락, wrapper 밖 사건, 규칙2(ZERO·역순열·eta 길이합·scratch·factor 변경·U 경계), 규칙3(sparse 치환·dense gather·포인터 교환), 규칙4(L/U 경로 이름, x104·U 판정 입력 협조 변조), 규칙5(spike 순서·값·누락·U 진입 시점·clear·save0 spike·save flag), 규칙6(출력 비트·순서·재계산 입력 후상태·별칭·wrapper raw). 모든 거부 메시지가 해당 규칙의 첫 차이를 가리키는지 확인했다. 처음 작성한 R4 두 변조는 쌍/불변 검사에서 먼저 거부돼 경로 규칙을 시험하지 못했으므로 협조형으로 고쳤다.
- **회귀**([요약](../../bench/oracle/m3_push_ftran/regression/summary.txt), 13명령 모두 종료0): 선행 `push_round_check.py --controls --acceptance --self-test`, `push_lu_survey.py --check`·`--check --self-test`, `push_chain_oracle.py --check`·`--self-test`(8종), `cargo test -p rsolver-lp --test push_chain` 1통과, `push_step_chain_rust.py --compare` 236단계·5,144필드(42구간·재분해 제외6 유지), py_compile·`node --check`. `git diff --check` 통과.
- 실행하지 않은 검사: Rust·확장 코드 변경이 없어 workspace 전체·clippy·확장 설치·pytest·Netlib·P3는 제외했다.
- 계약 변경: 탐색 1차 뒤 scratch 형식을 C(`21686f` param_6/param_7)로 다시 읽어 F+0x1b0=i32 표시, F+0x1b8=f64 값으로 정정했다(정식 캡처 전). wrapper 반환 주소를 asm으로 확정해 기록했다. 판정 규칙 자체는 바꾸지 않았다.

## 7. 판정과 인수인계

- 작업 수행: **오라클 구축 완료**(계약 범위). 원본 식별·독립3회·선행19 비간섭·C 규칙 판정·변조 35종 거부를 확인했다. 독립 검토: REVIEW_PLACEHOLDER
- 동등성: 이번 작업은 Rust 구현을 포함하지 않는다. 수치 L/R/U 값의 독립 재계산과 Rust 비트 비교는 **미검증**(21b).
- 남은 한계(자연 미관측): 입력 count0 조기 종료, L 출력 0, `21574d`, L 레벨 경로(F128≥0, 모두 −1), eta>0에서의 sparse R/희소 U, 접두 목록이 있는 희소 U, sparse spike count≥2, ±ZERO 정확 경계·동률. 21b는 이 범위에서 L/R/U 비트 재생을 시작하고, 필요한 분기는 원본 수치 명령을 실행하는 별도 오라클로 보강한다. work 누적값은 기록만 했다.
- 다음: **M3-04c-21b**. `python3 tools/session.py status` → `python3 tools/session.py claim M3-04c-21b --owner <새-owner>` → 이 문서 §2·§6과 `docs/m1/m1_02j_lsolve.md`·`m1_02l_eta_peel.md`·`m1_02m_usolve.md`를 읽고, `bench/oracle/m3_push_ftran/{arms,retry,interior}-1.json`의 `ftran:enter` factor를 초기 상태로 Rust 한 RHS FTRAN의 커널 경계·spike·raw·입력 후상태를 비트 비교한다. 기준 재검사는 `python3 tools/push_ftran_check.py --controls --acceptance --self-test`.
- branch `work/m3-04c-21a`, worktree `../rsolver-m3-04c-21a`, 선행 미커밋 없음. 최종 commit·병합은 main `Merge M3-04c-21a` 이력을 따른다. main 미추적 `.claude/settings.local.json` 보존. 원격/업로드 없음.
- 교훈: `docs/lessons/original.md`(FTRAN scratch 형식, 자연 dense L 경로), `tooling.md`(변조 거부 이유 확인, scratch 비영 목록, session.py 번호 정규식).
