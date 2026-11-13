# M3-04f-3 crossover cleanup 선택·retry 재설정·basis 설치 Rust 이식 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 선행: [04f-2 cleanup 진입 조사·오라클](crossover_cleanup_entry.md), [04f-1 해 설치](primal_push_install.md).

## 1. 범위와 주장

- 작업 종류: **구현**(Rust 이식 + 저장 원본 자료 재생 비교). 원본 재캡처는 하지 않는다(04f-2 자료 768회를 기대값으로 쓴다).
- 완료 후 주장할 동작: 같은 입력 상태(push 반환 retry·remaining, Crossover, S+0x2d8·S+0x120·S+0x28, OptimalityTol, local_98, S+0x178, 상태 배열·x·bound·tolerance)에서 Rust가
  1. retry 잔존 재설정(`13f716` C 1363–1521, env+0x2340 = 0): 열 상태 −1/−2/−3/−4, 행 slack basis(위치 i), remaining = 0, S+0x184 = 0, env+0x24a0 = 진입 PerturbValue,
  2. cleanup 선택(C 1523–1690·asm 0x1bf6c2–0x1bf7b9·0x1bfa59–0x1bfbda): 선택 bool(r14d), 쌍대 검사 경유 여부, `0cda5b` 호출 지점(dual / primal 인자 없음 / primal + basis 인자), alg, S+0x180 = 1, S+0x178(+10 iff local_98 = 2),
  3. `0325b4` 상태 배열 설치(스케일 없는 경로, S+0x4d8 ≠ 0): 설치 뒤 상태(n+m), superbasic 목록 수 S+0x450·위치 S+0x440·값 S+0x448, x·slack 보존, factor 무효화 표시
  를 원본과 정확·비트 일치로 낸다.
- 포함 분기: 04f-2 자료의 모든 분기(Netlib 45 × Crossover 5값, arms·retry·interior 자연, 합성 16종: auto 양수/음수/0, 쌍대 검사 참·거짓, retry 재설정 3종, Crossover 1–4 양·음 remaining, superbasic 설치 27회).
- 입력 전제: double(S+0x28 = 0), Threads = 1, 자료의 mode 0·1(선택 코드는 mode를 읽지 않음, 04f-2 §1).
- 제외와 후속:
  - 쌍대값 재계산 `251b4f`(d = c − Aᵀy, factor·BTRAN 필요)는 이식하지 않는다. 쌍대 검사의 d(n+m)는 원본 `sel:dual` 캡처값을 주입한다(04f-1의 조건수 주입과 같은 방식). 검사 규칙 자체(최댓값·건너뜀·합 < OptTol)는 Rust가 계산한다.
  - `0325b4` 스케일 경로(S+0x4d8 = 0의 x 나눗셈·slack 곱셈), x87, env+0x2340 ≠ 0의 −1006 종료, 할당 실패(0x2711)는 구현·비교하지 않는다. sense flag(D+0x3c = 1: −2 → −1, x 사본 부호 반전)는 C대로 구현하되 자연·합성 모두 0회라 미검증으로 둔다.
  - retry 재설정의 basis header(위치 i → n + i)는 Rust 출력에 있으나 원본 캡처에 없어 비교하지 않는다.
  - cleanup simplex 본체(`0cda5b` 반복)와 최종 basis, solver 경로 연결, work·시간은 후속(04f-4 이후)이다.
- 선행 commit: main `abf6d74`(Merge M3-04f-2). 미커밋 선행 변경 없음(main의 미추적 `.claude/settings.local.json`은 무관).

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 ELF file offset.
- `13f716` = `../re/decomp/c/001bc5d0_PRIVATE000000000013f716.c` C 1363–1690, asm 0x1bf2ee–0x1bfc95(쌍대 검사 maxsd 피연산자 순서 확인: 기저 `maxsd bmax → |d| > bmax ? |d| : bmax`, 비기저 `v = (0 > −d) ? 0 : −d`, `comisd v, tol; ja` = tol > v면 건너뜀, 합 `nmax + bmax`, `cmpltsd tol`).
- `0325b4` = `../re/decomp/c/000d3150_PRIVATE00000000000325b4.c` 전체: 복사(C 67–208) → `a0011`(S+0x1d0 = 0, C 210) → sense flag(C 211–216·444–472, x 사본 부호 반전 C 640–667) → 열(C 599–637 `LAB_001d3764`) → 행(C 224–250 `LAB_001d39a3`) → 목록(C 372–436 `LAB_001d39d4`) → factor +0xc = 0(C 437–443 `LAB_001d3b51`). param_2 또는 param_3이 null이면 아무것도 하지 않고 0(C 63–67).
- 시작 경계: `0x1bf2ee` `sel:enter`(push 시도 루프·뒤처리 뒤). 종료 경계: `i325b4:leave`(basis 설치 경로) 또는 `cda5b:enter`(그 밖).
- 입력: `sel:enter`(retry, remaining, crossover, s2d8, s120, precision, opttol_slot, local_98, s178, vstat, x(n), d_lb·d_ub(D bound), tol 370), `xo:enter`(fixtol, feastol, perturb), `sel:dual` d(쌍대 검사 경로만, 주입), `i325b4:enter`의 S 배열(lb·ub(n+m), cost·collen(n), tol 380·388·3b0, d_flag3c·d_flags, slack(m)). 설치 입력 상태 vb·cb와 x는 Rust 자신의 재설정·선택 결과와 `sel:enter` x를 쓴다(연결).

## 3. 비교 계약

| 필드·사건 | 의미 | 비교 규칙 | 기대값 |
|---|---|---|---|
| reset vstat(n+m) | retry 재설정 뒤(재설정 없으면 진입값 그대로) | 정확 | `sel:reset.vstat` |
| reset remaining·S+0x184·env+0x24a0 | | 정확·비트 | `sel:reset` |
| select r14·쌍대 검사 경유 | 선택 bool, `sel:dual` 유무 | 정확 | `sel:decide.r14`, `sel:dual` 존재 |
| `0cda5b` 호출 | 지점(3종)·alg·basis 인자 유무·S+0x180·S+0x178 | 정확 | `cda5b:enter` |
| 설치 유무 | `0325b4` 호출 여부 | 정확 | `i325b4:enter` 존재 |
| 설치 vstat(n+m)·S+0x450·목록 위치·값 | | 정확·값 비트 | `i325b4:leave` |
| 설치 x·slack | 보존 | 비트 | `i325b4:leave` x·slack |

- 오차 허용 없음. 부동소수 값은 IEEE 비트 비교(16진).
- 시간·work: 비교하지 않는다.
- 재현성: 04f-2 `--controls`(256사례 × 3회 동일)를 선행 확인한다. 비교는 768실행 모두에 대해 한다.
- 판별력: 비교기 자료 변조(Rust 출력·기대값 필드)와 Rust 규칙 변이(환경 변수로 켜는 변이)가 거부되는지 확인한다.

## 4. 사례

04f-2 `bench/oracle/m3_xo_cleanup/`(manifest.json 해시) 768실행 전부. 분기 집계는 04f-2 §6과 같다(auto 7경로, Crossover 1–4 각 rem+·rem0·rem−·retry, 쌍대 검사 24, retry 재설정 9, 설치 27·superbasic 345).

## 5. 재실행 명령

- `python3 tools/xo_cleanup_check.py --controls --acceptance`(선행: 원본 자료 재현·규칙)
- `python3 tools/xo_cleanup_rust.py --write` → `cargo test --release -p rsolver-lp --test xo_cleanup` → `python3 tools/xo_cleanup_rust.py --compare`
- 판별력: `python3 tools/xo_cleanup_rust.py --self-test`, `--mutate-rust`
- 회귀: `cargo test -p rsolver-lp`(관련 push 테스트), `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, 04f-1 `push_install_rust.py --compare`.

## 6. 실행 결과

(실행 후 작성)

## 7. 판정과 인수인계

(실행 후 작성)
