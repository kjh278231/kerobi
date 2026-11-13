# M3-04f-4c crossover cleanup simplex 본체 조사 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 선행: [04f-2 cleanup 진입](crossover_cleanup_entry.md), [04f-3 Rust cleanup 진입](crossover_cleanup_rust.md), [04f-4a 쌍대값](crossover_dual_values.md), [04f-4b 연결](crossover_dual_link.md). M1 simplex 명세: [primal](../m1/m1_primal_spec.md), [04 단체법](../04_simplex.md).

## 1. 범위와 주장

- 작업 종류: **조사**(읽기 전용 원본 관측 포함). Rust 구현은 하지 않는다. 사용자 지시 "다음 M3 스프린트 진행해 너가 작업자야"로 이 세션이 직접 작업한다(작업자 선택 질문 생략).
- 조사 질문.
  1. `13f716`이 부른 cleanup `0cda5b`는 어떤 경로를 타는가: 진입 직후 "이미 풀린 상태" 조기 반환(C 142–160: spx 존재, model+100 = spx+0xf0, alg = spx+0x358, 상태 2–5, env+0x2454 / `12dd49`), setup `02f7e1`, 웜스타트 `0325b4`·`194efc`, 엔진 분기(primal `12e6ec`→`0478d3`, dual `12ed7c`→`06268b`, sifting `12e738`), 종료 뒤 검사(`2e9ae`·`2e315`·`12f282`).
  2. 자연 사례에서 cleanup이 실제로 하는 일의 양: simplex 반복 수(spx+0x2a0 증분), 재분해(`036ade`)·FT 갱신(`05b257`) 수, primal↔dual 전환, 진입 basis(S+0x168) 대비 최종 basis 변화.
  3. cleanup이 crossover workspace에서 이어받는 입력 상태(factor·x·d·섭동·pricing 가중·허용오차·카운터)와 cleanup 시작에서 다시 만드는 상태를, 실제로 탄 경로의 C·asm으로 특정한다.
  4. M1 Rust simplex(`crates/lp/src/simplex.rs`: dual `06268b`, primal `0478d3`, 외곽 `drive`)를 이 경로에 재사용할 수 있는 범위와 빠진 것(상태 import, 이어받는 factor, 조기 반환 등).
  5. 최종 VBasis/CBasis 비교 경계와 후속 구현 번호.
- 포함: double(S+0x28 = 0), Threads = 1, Presolve = 0, Method = 2, 04f-2와 같은 사례 집합(Netlib 소형 45 × Crossover {−1, 1, 2, 3, 4}, 특수 자연 arms·retry·interior × 5값). mode 1(arms·retry) 사례는 04f-2처럼 cleanup 선택 이후만 본다.
- 제외와 후속: 합성 편집 사례(04f-2 `synthetic.json`, 이번에는 자연 경로만 조사), x87, Crossover 5·threads ≥ 2, 재시작(04g), Rust 구현과 동등성 판정.
- 선행 commit: main `8a12121`(Merge M3-04f-4b). 미커밋 선행 변경 없음(main의 미추적 `.claude/settings.local.json`은 무관).

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 ELF file offset.
- `0cda5b` = `../re/decomp/c/00153cc0_PRIVATE00000000000cda5b.c`(2,113줄), `02f7e1` = `000d06f0`, `0478d3` = `000e3db0`, `06268b` = `000f6a00`, `13f716` = `001bc5d0`(C 1523–1690 호출 지점).
- 시작 경계: `0cda5b` 진입(반환 주소가 `13f716` 안). 종료 경계: 그 `0cda5b` 반환.
- 관측(읽기 전용 훅, 쓰기 없음): 진입 인자·model+100·spx+0xf0·spx+0x358·spx+0x35c·spx+0x2a0·spx+0x20·spx+0x180·spx+0x4d8·spx+0x450·env+0x2454·env+0x2438·S+0x168(n+m); 내부 호출의 진입·반환 횟수와 반환값(`02f7e1` 0xd06f0, `0325b4` 0xd3150, `194efc` 0x1fcd50, `12e6ec` 0x1bbb20, `12ed7c` 0x1bbd80, `12e738` 0x1bbb40, `0478d3` 0xe3db0, `06268b` 0xf6a00, `036ade` 0xd4190, `05b257` 0xf15c0, `2e9ae` 0xcff40, `2e315` 0xcfbc0, `12f282` 0x1bbf80, `12dd49` 0x1bb6f0); 반환 시 같은 필드와 S+0x168.

## 3. 비교 계약(조사 산출물)

| 필드·사건 | 의미 | 판정 규칙 | 캡처 |
|---|---|---|---|
| 경로 사건열 | cleanup 안 호출 순서·횟수·반환 | 사례당 독립 3회 정확 동일 | 훅 |
| 반복·재분해 수 | spx+0x2a0 증분, `036ade`·`05b257` 횟수 | 정확(3회 동일) | 훅 |
| basis 변화 | 진입 S+0x168 대 반환 S+0x168, 최종 VBasis/CBasis | 정확 | 훅·Python |
| 조사 답 | Q1–Q5 | C·asm 위치와 관측 사건으로 근거 연결 | 문서 |

- 원본 재현성: 사례당 독립 3회의 사건열이 같아야 한다. 다르면 첫 차이 필드를 기록하고 원인을 조사한다.
- 수치 비교는 하지 않는다(조사). 시간·work는 기록만 한다.
- 판정: 조사 완료는 Q1–Q5에 근거로 답하고 미확인 범위·후속 번호를 특정했을 때. 동등성은 **미검증**(Rust 구현 없음).

## 4. 사례

| 사례 | 실제/합성 | 노리는 분기 |
|---|---|---|
| Netlib 45 × Crossover −1·1·3 | 실제 | primal cleanup(basis 인자 없음, remaining = 0) |
| Netlib 45 × Crossover 2·4 | 실제 | dual cleanup |
| arms·retry × 5값 | 실제(mode 1) | remaining ≠ 0 primal + `0325b4` 설치, 자연 쌍대 검사 → dual |
| interior × 5값 | 실제 | 04f-2 특수 자연 |

## 5. 재실행 명령

- 캡처: `tools/hrun.sh --timeout 590000 -- 'python3 tools/xo_simplex_runs.py --reps 3'`(작업 worktree, Gurobi 한 프로세스).
- 검사·집계: `python3 tools/xo_simplex_check.py --controls --summary`, 판별력 `--self-test`.

## 6. 실행 결과

(실행 후 작성)

## 7. 판정과 인수인계

(실행 후 작성)
