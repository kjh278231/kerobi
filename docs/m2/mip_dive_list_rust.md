# M2-03d-10 검증 계약과 결과: DynamicRows 비활성 목록 유지와 D 교체 Rust 이식(L1)

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. [M2-03d-9](mip_dive_rows_rust.md) §7의 후속 제안("목록 유지·관문 이식")이고, 사용자의 "다음 스프린트 진행" 지시로 착수했다(2026-11-11).

## 1. 범위와 주장

- 작업 종류: 구현(L1, 저장 기준 재생). 원본을 다시 캡처하지 않는다. 기준은 03d-6·03d-7·03d-8의 고정 캡처다.
- 완료 후 주장할 것:
  - P 목록: `crates/mip/src/dive_rows.rs`의 `RowList`가 `10e5fc`(비활성 표시), `10e662`(전부 재활성), `edd71` 목록 블록(C:8020–8417)을 옮겼다. 03d-8 run 1 세 사례의 top-level LP 전부(lp 1·2)에서 원본 연산 열만 넣으면, 원본이 기록한 모든 X 목록 변경(`b:xlist`)을 개수·순서까지 재현한다.
  - S 교체: `10f448`/`10f5ae`를 옮겼다. 03d-7 대상(nidda 노드 16948, call 19901)에서 Rust가 재생으로 만든 목록과 03d-6 API LP를 넣으면, 교체 뒤 열 앞부분(길이·ind·val)과 rhs가 03d-6 `y:ws` 덤프와 비트 단위로 같다. 복원 뒤에는 API 값과 같다.
- 포함 분기: 빈 목록에 붙이기, 비어 있지 않은 목록에 붙이기(이미 있는 행 무시), react_all로 비우기, react_all의 X 없음·빈 목록 조기 반환, 행 수 증가(비어 있지 않은 목록에서도), 행 수 감소(빈 목록에서만), X 사본 처음 만들기, 이미 만든 사본 재사용(k=2), 참조수 0→1→0.
- 제외 범위와 후속:
  - 관문(C:1581–1610)과 C:526–529의 재활성 조건: 03d-8 `d:gate` 사건은 통과한 호출만 입력을 기록하고(사례별 `d:gate` 수 = `d:sel` 수), 실패는 `d:gate_counts` 집계뿐이다. `I+0x28`·`I+0x2c`·`a7f0ad`·`a7f923` 입력도 없다. 음성 사례 입력이 없어 L1 대조가 불가능하므로 이번 범위에서 뺀다. 관문 입력 오라클(번호 미배정, 제안)로 넘긴다.
  - `edd71`의 "마지막 원소를 옮겨 빼기"로 목록이 비지 않게 남는 경우: 세 사례 모두 실행 0회다(비어 있지 않음→비어 있지 않음 76단계 중 행이 빠진 단계 0). C만으로 옮기고 미검증으로 남긴다.
  - 이미 목록에 있는 행의 비활성 요청(위치≥0이면 무시): 세 사례 모두 실행 0회다. C만이다.
  - `10f448` 두 포인터 교환의 열 내 순서: 03d-6 API LP는 이미 분할된 상태(R 행이 있는 169열 모두 비목록 행이 앞)다. 재생에서 교환이 한 번도 일어나지 않는다. C만으로 옮기고 미검증으로 남긴다. 판별하려면 `X+0x18`=0으로 들어오는 `10f448`의 전후 ind·val 캡처가 필요하다(후속 제안).
  - 비어 있지 않은 목록에서의 행 삭제와 재번호: 실행 0회이고 원본 위치도 찾지 않았다. Rust는 이 경우를 지원하지 않는다(panic, `shortcut:` 주석).
  - `10e62f`(행별 재활성, 호출자 0)와 `10e5fc`의 두 번째 호출자 `68341b`(실행 0회), 중첩 MIP의 목록, `10f973`·`5d4427`의 목록 읽기, `bnb.rs` 연결은 하지 않는다.
- 선행: main `32db6a4`(M2-03d-9 병합). 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`(03d-7·03d-8 manifest).
- 원본 근거(C를 직접 읽음):

| 위치 | 이식 내용 |
|---|---|
| `10e5fc` `0x1a2c20` | 대기 객체(`lp+0x230`)의 행 플래그(`+0xb0`, D 행 수)를 처음이면 0으로 만든다. 행마다 `& ~0x2000 \| 0x1000`. 행이 [0, D 행 수) 밖이면 그 자리에서 `0x2716`을 돌려준다(앞 행은 표시된 채) |
| `10e662` `0x1a2fc0` | X 없음 또는 X+0 개수 < 1이면 0을 돌려준다. 아니면 `edd71`을 먼저 부르고, 목록의 모든 행에 `& ~0x1000 \| 0x2000`(범위 밖이면 `0x2716`) |
| `edd71` C:8020–8417 | 대기 행 플래그가 있으면 X가 없을 때 만들고(위치 X+0x10은 −1), 행 번호 오름차순으로 `0x1000`이면 위치<0일 때 목록 끝에 붙이고, `0x1000`이 아니고 `0x2000`이면 위치≥0일 때 마지막 원소를 그 자리로 옮기고 위치를 −1로 둔다. 끝에 X+0 = 개수, X+0x18(만듦 표시) = 0 |
| `edd71` C:3433–3456, `dcd29`, `db102`, `fbf68` | X+0x18 = 0. `edd71`은 rhs 변경(행 플래그 4) 블록 끝, `dcd29`·`db102`는 `D+0x3a4`·`D+0x3a0`을 갱신하는 함수(행·열 용량 변경으로 추정)로 `dcd29`는 새 행 위치를 −1로 늘린다. `fbf68`은 대기 객체 `+0x38` 개수≠0일 때(의미 미확인) |
| `10f448` `0x1a31c0` | 참조수 X+0x1c를 올려 0→1일 때만 동작한다. X+0x18 = 0이면 열마다 [beg, beg+len)에서 앞 포인터는 비목록 행(위치<0)을 건너뛰고, 목록 행을 만나면 뒤 포인터를 비목록 행까지 내려 ind·val을 맞바꾼다. 앞부분 개수를 X+0x28에 쓴다. D rhs를 X+0x20에 복사하고 목록 행을 +0으로 둔다. X+0x18 = 1. 그 뒤 D+0x100↔X+0x28, D+0x380↔X+0x20을 맞바꾼다 |
| `10f5ae` `0x1a33d0` | 참조수를 내려 0이 되면 두 쌍을 다시 맞바꾼다 |

- 시작 경계(P): top-level LP가 처음 보일 때(`b:lpnew`, X 없음). 입력은 그 LP의 `b:op`(op, rows, D 행 수) 열과 `b:xlist` 위치(갱신 시점), 사건마다의 D 행 수다.
- 종료 경계(P): 각 `b:xlist`(그 LP의 X 목록이 바뀐 `edd71` 반환). 출력은 X 목록(개수·순서).
- 시작·종료 경계(S): 03d-7 `b:t:cda5b` k=1 진입(교체 전)부터 `b:t:swap_out` k=2까지. 입력은 P 재생으로 얻은 Rust 목록(같은 시점의 캡처 `x_rows`와 비교), 03d-6 API LP(`nidda_bnb_H_s1.node_lp.json`의 cbeg·ind·val·rhs, `cda5b` 진입 시 D와 같은 해시). 출력은 교체 뒤 D 길이·ind·val·rhs와 복원 뒤 길이·rhs, 참조수.

## 3. 비교 계약

`tools/mip_dive_list_rust.py`가 판정한다. 하나라도 실패하면 종료코드 1이다.

| 검사 | 규칙 |
|---|---|
| M 입력 | 03d-8 run 1 캡처 세 개가 `bench/oracle/mip_dive_rows_manifest.json`과, 03d-7 run 1·03d-6 API LP·03d-6 run 1이 `bench/oracle/mip_blank_rows_manifest.json`과 SHA-256이 같다 |
| X 재생 입력 | 생성기는 `b:lpnew`·`b:op`(op, rows, D_rows)와 `b:xlist`의 위치·D_rows만 쓴다. `b:xlist`의 rows·count, `x_rows`, `y:ws` 내용은 넣지 않는다. 입력 머리에 사례 이름과 캡처 SHA-256을 적는다 |
| T trace | Rust trace 머리의 사례·SHA-256이 입력과 같다. 실행 전에 이전 trace를 지운다 |
| P 목록 | 모든 `b:xlist`에서 Rust 목록이 캡처 rows와 개수·순서까지 같다. 건너뛴 단계는 0이어야 한다 |
| K 개수 | 사례별 (lp 1 단계, lp 2 단계, 비어 있지 않음→비어 있지 않음, 그중 행 감소) 고정값: nidda_H_s1 (20, 2377, 36, 0), gmu-35-40_default (32, 283, 46, 0), assign1-5-8_default (17, 200, 30, 0). 03d-7 nidda_bnb_H_s1도 같은 연산 열이며(03d-8 nidda의 투영과 같음) (20, 2377, 36, 0)이다 |
| S 교체 | 03d-7 대상 k=1·k=2(03d-7 캡처와 03d-8 nidda 캡처 둘 다 대상 사건이 있다. 노드 16948, D 해시 `4051b704-7e653419`, 노드 LP임을 먼저 확인): 교체 전 Rust 목록 = 캡처 `x_rows`(96행), 참조수 0→1→0. 교체 뒤 길이 합 1840, 열 앞부분 ind·val·열 시작이 `y:ws` k=2의 cbeg·ind·val과, rhs가 `y:ws` b와 비트 단위로 같다. 바뀐 rhs 행은 캡처 `rhs_diff`(58행)와 같다. 복원 뒤 길이·rhs가 API 값과 같다. k=1에 사본을 만들고 k=2는 다시 만들지 않는다 |

- 정수·목록은 순서까지 정확 비교. 부동소수(val, rhs)는 u64 비트 비교. 허용오차 없음.
- 시간·work: 이 함수들에는 work 증가가 없다(C 확인). 비교하지 않는다.
- 재현성: 03d-7이 3회, 03d-8이 사례마다 3회 바이트 동일을 확인했다(manifest). run 1만 재생한다.
- 자기검사(`--self-test`):
  - trace 변조 거부: 목록 한 행 삭제, 목록 순서 교환, 갱신 한 줄 삭제, 캡처 SHA-256 변경, 교체 뒤 ind 한 칸 변경.
  - 코드 변이 거부(임시 복사본): 행 번호 내림차순 반영, react_all 무시, rhs +0 생략, 길이 사본에 원래 길이.
  - 판별되지 않음을 확인하는 변이(거부되지 않아야 한다. 거부되면 §1 미검증 항목을 다시 쓴다): 이미 있는 행 중복 붙이기, 빼기를 순서 보존 삭제로, 두 포인터 교환을 안정 분할로.
  - 계약 변경(첫 자기검사 뒤, 판정 전): "이미 있는 행 중복 붙이기"를 거부 대상에서 판별되지 않음으로 옮겼다. 세 사례에서 이미 목록에 있는 행을 다시 비활성하는 요청이 0회임을 캡처로 셌기 때문이다(§1).

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 | 원본 증거 |
|---|---|---|---|---|
| nidda_H_s1 (03d-8) | 원본 실제 실행 캡처 | 재시작 앞뒤 두 LP, deact 1145·react_all 1147, 행 수 증가 | `bench/oracle/mip_dive_rows/nidda_H_s1_1.json.gz` `6defab9c…` | 03d-8 §6.2 |
| gmu-35-40_default (03d-8) | 원본 실제 실행 캡처 | 비어 있지 않은 목록의 행 수 증가 46, 행 수 감소(빈 목록) | `gmu-35-40_default_1.json.gz` `e1e4ccee…` | 03d-8 §6.2 |
| assign1-5-8_default (03d-8) | 원본 실제 실행 캡처 | react_all 수가 deact보다 많음(빈 목록 조기 반환) | `assign1-5-8_default_1.json.gz` `bc45f038…` | 03d-8 §6.2 |
| nidda_bnb_H_s1 (03d-7 대상) | 원본 실제 실행 캡처 | 교체·복원, 사본 재사용 | `bench/oracle/mip_blank_rows/nidda_bnb_H_s1_1.json` `43f768a4…`, API LP `bench/oracle/mip_retry_node_lp/nidda_bnb_H_s1.node_lp.json` `1c0b56ac…`, `y:ws` `nidda_bnb_H_s1_1.json` `9ef4c0ec…` | 03d-7 §6.2, 03d-6 |

- 적용하지 않은 범주: 비어 있지 않게 남는 빼기, 분할 교환 순서, 비어 있지 않은 목록의 행 삭제, `0x2716` 오류(실행에 없음). 합성 입력으로 이 분기를 "검증"하지 않는다. 단위 테스트의 손으로 만든 사례는 이식 코드의 자기점검일 뿐이다.

## 5. 재실행 명령

- 원본 캡처: 없음(고정 자료 재사용).
- 재생과 비교(작업 worktree 루트): `python3 tools/mip_dive_list_rust.py --self-test`. 입력 생성 → `cargo test --release -p rsolver-mip --test dive_list_l1 -- --ignored` → 비교 → 변조·변이 순서다. 비교만은 인자 없이 실행한다.
- 관련 회귀: `cargo test -p rsolver-mip`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `python3 tools/mip_dive_rows_rust.py`(03d-9 재생 불변).

## 6. 실행 결과

- 실행: worktree `../rsolver-m2-03d-10`, 최종 트리 = 이식 `f05b596` + 행 삭제 가드 수정 `0c44b26` + main `a5d68f4`(M3-04c-7) 병합 `416d913` + 드라이버 변이 처리(이 기록과 같은 commit)(계약 `8147630`), x86-64 Linux(WSL2), `cargo test --release`. 입력은 03d-8 run 1 세 개, 03d-7 run 1, 03d-6 API LP·run 1(SHA-256 §4, manifest 일치)이다. 원본은 다시 실행하지 않았다(저장 기준 재생).
- `python3 tools/mip_dive_list_rust.py --self-test` 종료 0(최종 트리, 파이프 없이 종료 코드 확인):

```text
M: captures match the M2-03d-7/03d-8 manifests (4 cases, API LP, y:ws)
X: replay inputs in target/mip_dive_list_rust/in
P/K: nidda_H_s1 updates lp1 20 lp2 2377, non-empty -> non-empty 36 (row left 0)
P/K: gmu-35-40_default updates lp1 32 lp2 283, non-empty -> non-empty 46 (row left 0)
P/K: assign1-5-8_default updates lp1 17 lp2 200, non-empty -> non-empty 30 (row left 0)
P/K: nidda_bnb_H_s1 updates lp1 20 lp2 2377, non-empty -> non-empty 36 (row left 0)
S: nidda target (call 19901 node 16948) k=1,2 in both nidda captures: rust list = x_rows, reduced D = y:ws (cbeg/ind/val/b bits), restore = API, refs 0-1-0
rejected: list row dropped: P gmu-35-40_default: lp 1 call 952 node 5: rust 3 rows != capture 4; ...
rejected: list order: P assign1-5-8_default: lp 1 call 352 node 7: rust 8 rows != capture 8; first diff at 0: ...
rejected: update deleted: K nidda_H_s1: 2396 list updates in the trace, capture 2397
rejected: capture sha256: T gmu-35-40_default: trace is for gmu-35-40_default e1e4…7d00, capture e1e4…7d0c
rejected: swapped ind: S k=1: reduced ind != y:ws, first diff at 0
mutant rejected: rows in decreasing order: P nidda_H_s1: lp 1 call 74 node 1: rust 7 rows != capture 7; ...
mutant rejected: react_all ignored: replay panicked: thread 'dive_list_replay' … panicked at crates/mip/src/dive_rows.rs:181:13: row deletion with a non-empty dive list
mutant rejected: rhs of listed rows kept: S k=1: rhs != y:ws b, first diff row 125
mutant rejected: reduced length = full length: S k=1: reduced nnz 4955
mutant passes (not discriminated by the captures, section 1): already listed row appended again
mutant passes (not discriminated by the captures, section 1): order-preserving removal
mutant passes (not discriminated by the captures, section 1): stable partition
PASS
```

- P: 03d-8 세 사례의 모든 `b:xlist` 2,929건에서 Rust 목록이 개수·순서까지 캡처와 같다. 03d-7 캡처(2,397건)는 03d-8 nidda와 같은 연산 열이라 독립 증거가 아니고, S 입력 목록을 잇는 데 쓴다. 건너뛴 단계는 없다. 행 수 감소는 모두 빈 목록일 때였다(03d-7 P는 감소 뒤 15단계를 건너뛰었지만 이번 재생은 전부 비교했다). skip·xfail 없음.
- S: 대상 k=1에서 Rust가 사본을 만들고(만듦 표시 0→1) k=2는 재사용했다. 원본은 k=1 진입 때 이미 사본이 있었다(`built_before` 1, call 19865 이후 만들어짐). 교체 뒤 D는 03d-6 `y:ws`와 비트 단위로 같다. 이 비교는 사본을 만든 시점과 무관하다. 대상까지 D 계수가 바뀌지 않았다는 전제는 03d-7 L(진입 D 해시 = 03d-6)이 받친다.
- 판별되지 않은 변이 3종은 계약대로 통과했다(§1 미검증 항목의 근거).
- 행 삭제 가드: 처음 작성은 "목록 행 번호 ≥ 새 행 수"일 때만 panic이라, 앞 번호 행이 지워지는 경우를 놓쳤다(검토에서 지적). 직전 행 수보다 줄면 목록이 비어 있어야 하도록 고쳤고 재생에서 오탐은 없다. 그 뒤 react_all 무시 변이는 목록이 남은 채 행이 줄어 이 가드로 거부된다(처음에는 P 목록 비교로 거부). 드라이버는 `dive_rows.rs`의 panic을 변이 거부로 세고, 빌드 오류는 자기검사 실패로 둔다.
- 첫 실행에서 드라이버 오류 두 건을 고쳤다. 이식 코드는 고치지 않았다. (1) 03d-8 nidda 캡처에도 대상 사건이 있어 S를 두 nidda 사례에 적용했고, 대상 노드·D 해시 확인을 넣었다. (2) 합격 조건에서도 실패 메시지의 `next()`가 먼저 평가돼 StopIteration이 났다.
- 관련 회귀(main `a5d68f4` 병합 뒤 트리): `cargo test -p rsolver-mip` 종료 0(단위 `dive_rows::tests::row_list` 포함, L1 테스트 2개는 ignored), `cargo test --release --workspace` 종료 0(테스트 바이너리 15개 ok), `cargo clippy --workspace --all-targets -- -D warnings` 종료 0, `python3 tools/mip_dive_rows_rust.py` 종료 0(03d-9 재생 불변).
- 실행하지 않은 검사: pytest(`crates/py`·`python/rsolver` 미변경), Gurobi(원본 재캡처 불필요).
- 계약 변경: "이미 있는 행 중복 붙이기" 변이를 거부 대상에서 판별되지 않음으로 옮겼다(§3, 첫 자기검사 뒤·판정 전, 실행 0회를 캡처로 셈).

## 7. 판정과 인수인계

- 작업 수행 상태: **완료(구현 L1)**.
- 동등성: **통과(L1, 저장 기준 재생)**. 목록 유지는 세 사례 top-level LP의 모든 X 목록 변경을 순서까지 재현한다. 교체는 nidda 대상 노드에서 줄인 D(열 앞부분 길이·ind·val, rhs)와 복원을 비트 단위로 재현한다.
- 남은 한계:
  - C만(실행에서 판별되지 않음): 비어 있지 않게 남는 빼기(마지막 원소 옮기기), 이미 목록에 있는 행의 비활성 요청, `10f448` 두 포인터 교환의 열 내 순서(입력 D가 이미 분할돼 교환 0회), `0x2716` 오류, 참조수 중첩(>1).
  - 미이식: 관문(C:1581–1610)과 C:526 재활성 조건(음성 사례 입력 없음), 비어 있지 않은 목록의 행 삭제·재번호(Rust는 panic), `10e62f`, `68341b` 경로, 중첩 MIP, `bnb.rs` 연결.
  - `10f448`은 D의 ind·val을 제자리에서 바꾸고 `10f5ae`는 되돌리지 않는다. 열 내 순서가 목록 이력에 따라 누적되므로, M2-03d-4에서 원본 궤적을 맞추려면 Rust 노드 LP도 같은 D 배열을 이어 써야 한다(매번 원본 LP에서 다시 만들면 순서가 달라질 수 있다).
- 후속:
  - 관문 입력 오라클(번호 미배정, 제안): C:1581–1610·C:526의 모든 판정(실패 포함)에서 DynamicRows, `I+0x28`, `I+0x2c`, `param_2+0x34`, `a7f0ad`·`a7f923`·`+0x18`을 캡처한다.
  - 분할 순서 오라클(번호 미배정, 제안): `X+0x18`=0으로 들어오는 `10f448`의 전후 D ind·val을 캡처한다.
  - M2-03d-4(L2): 다이브 행 선택·목록 유지·교체는 이식됐다. 남은 선행은 M2-10b, M1 요청(primal warm start·LP cutoff), 관문이다.
- 재실행: `python3 tools/mip_dive_list_rust.py --self-test`(release 빌드와 변이 7회 빌드 포함 수 분).
- 작업 branch: `work/m2-03d-10`. 검사 통과 뒤 main에 `--no-ff`로 병합한다.
- 교훈: [원본 사실](../lessons/original.md)(`10f448` 제자리 분할 누적, 만듦 표시를 지우는 함수들), [도구](../lessons/tooling.md)(판별되지 않는 변이를 "통과해야 함"으로 고정, 실패 메시지의 즉시 평가)에 추가했다.
