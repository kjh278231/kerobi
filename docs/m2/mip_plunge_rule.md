# M2-06c 검증 계약과 결과: plunge 규칙 선택·규칙 3/12 점수·꼬리 정리 유지 판정

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 앞선 작업은 [M2-06a](mip_node_select_spec.md)(노드 용기 순서, 기준 자료 `bench/oracle/m2_06a`)다. M2 인수인계의 첫 다음 작업이고, 사용자의 "다음 스프린트 진행" 지시로 착수했다(2026-11-11).

## 1. 범위와 주장

- 작업 종류: 조사·오라클(읽기 전용 원본 캡처와 규칙 재생). Rust·Python 패키지는 바꾸지 않는다.
- 조사 질문:
  1. **규칙 번호.** `6e8eb3`이 plunge 목록에서 고를 때 규칙 1–12 중 무엇을 쓰는지 정하는 식(asm `0x5b3326`–`0x5b3553`)과 그 입력.
  2. **RNG 소비.** 규칙 3/12를 가르는 난수는 어떤 상태에서 몇 번 뽑는가?
  3. **규칙 3·12 점수.** 두 규칙이 목록에서 고르는 항목(점수식·동률·제외 항목·minEst)과 work.
  4. **꼬리 정리 유지 판정.** 선택 뒤 남은 항목 중 무엇을 남기고 무엇을 힙으로 옮기거나 버리는가(δ·d′·`5fb13c` 조상·문턱·제거 목적지).
- 완료 후 주장할 것(재생 = §2.3의 규칙을 Python으로 계산):
  - R: 목록이 빈 채로 끝나지 않은 모든 최상위 `6e8eb3` 호출에서, 캡처 입력으로 재생한 규칙 번호가 dispatch 직전 `%ecx`(+1 전 값)와 같다.
  - G: 진입 때 RNG 상태 슬롯 `0x70(%rsp)`가 진입 k와 `W+0x28`로 만든 시드와 같고, 뽑기 수(0 또는 1)와 뽑은 뒤 상태, 3/12 판정이 재생과 같다.
  - S: 규칙 3·12 호출마다 재생한 선택 첨자가 `0x40(%rsp)`와, 선택 노드가 반환 노드와 같다. work 증가(minEst 3n, 점수 4n)도 같다.
  - T: 모든 최상위 호출에서 δ·d′·anc·문턱·항목별 유지/힙/버림·남은 목록 순서·`W+0x3fa8`·꼬리 work가 재생과 같다. `5fb13c`는 캡처한 부모 사슬로 직접 다시 걷는다.
  - 조사이므로 Rust 구현 일치 판정은 없다.
- 제외 범위와 후속(관측되면 검사기는 범위 밖으로 실패하거나 0회로 보고한다):
  - 규칙 1·2·4·5·6·7·9·11의 점수: 이번에는 번호만 재생하고 선택 노드는 입력으로 둔다(꼬리 판정은 모든 규칙에서 본다). 후속 **M2-06c-2**.
  - 변수 hint 경로(`r8b`: 규칙 3 강제, 꼬리 앞 형제 교환 `0x5b44ba`–`0x5b4500`·`0x5b4b1b`–`0x5b4b90`), 병렬 worker 힙(`5e2919 ≠ 0`, `0x5b47da`–`0x5b48d5`), `6e5802` 실패로 꼬리 루프가 끊기는 경로(`0x5b4a9f`, 이후 압축 `0x5b490d`–`0x5b49a9`): asm만, 0회 예상.
  - δ = 3(env+0x2710 == 4)과 δ = 10(T+0x138 > 0, 힙 ≥ 10000, k ≥ 1000), 규칙 5의 MIPFocus 덮어쓰기: 기본 설정 사례에서는 0회 예상. 0회면 asm만으로 남긴다.
  - 선택 노드 `I+0x41`(제외 표시)이 선 경우: 06a 4사례 0회. 0회면 asm만.
  - 문턱 `thr0 < 1e100`: `5f9a41(M) ≠ 0`이 필요하다(§2.3). 기본 설정에서 0이면 bound 절은 판별되지 않는다. PoolSearchMode=2 사례로 닿는지 탐색하고, 닿지 않으면 asm만으로 남긴다.
  - `W+0x28`(시드 더하기 값)의 쓰는 곳: 이번에는 입력으로 캡처만 한다. 후속 M2-06d.
  - 지연 목록·node file·status 10 힙: 06a와 같이 범위 밖.
- 선행: main `5d7629f`(M2-06f-5 병합 포함). 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 file offset이다.
- `6e8eb3` = `0x5b29d0`. 색인 크기(3,492)보다 길다. 본체와 차가운 조각이 `0x5b4c50`까지 이어지고 반환은 `0x5b49eb` 하나다(objdump `0x5b29d0`–`0x5b4c50` 전체 확인). C는 규칙 번호까지만 복원되고(C:381–461) 점프 테이블 뒤는 asm으로 읽었다.
- 점프 테이블 `0x2a6e350`(u16 12개, 기준 `0x5b3647`): 1 `0x5b4274`, 2 `0x5b412e`, 3 `0x5b3fab`, 4 `0x5b3e1e`, 5 `0x5b3daf`, 6 `0x5b3bc1`, 7 `0x5b39fc`, 8·10 `0x5b4499`, 9 `0x5b3910`, 11 `0x5b37ca`, 12 `0x5b3647`.
- 상수(`.rodata`): `0x2a6e3b0` −1e100, `0x2a6e3b8` 1e100, `0x2a6e410` 3.0, `0x2a6e418` 1e8, `0x2a6e420` 1e101, `0x2a6e428` 4.0, `0x2a6e440` 0.2, `0x2a6e3f8` 1.0, `0x2a6e448` 2.0, `0x2a6e450` 0.9, `0x2a6e458` 1e-10.
- 기호: M = 인자 1(`%rbp`), W = 인자 2(`%r13`), Mx = `M+0x620`(`%rbx`), env = `(M+8)+0x100`(`%r15`), T = `Mx+0x780`, work 포인터 = `W+0x40d0`(`%r14`). 목록: 개수 `W+0x3fa8`, 배열 `W+0x3fb0`(노드 N 포인터). N: id `N+0x00`(double), 부모 `N+0x10`, I = `N+0x18`. I: bound `+0x18`, est `+0x20`, depth `+0x2c`(int), 제외 `+0x41`(byte). k = `W+0x30`.

### 2.1 시드(asm `0x5b29e1`–`0x5b2a5b`, 진입 직후)

```text
x = (double)k_entry + W+0x28          # k_entry = 진입 때 W+0x30
seed = x < 2^63 ? (u64)trunc(x) : (u64)trunc(x − 2^63) | 2^63   # 0x70(%rsp)에 저장, 다른 쓰기 없음
h = env+0x2e2c == 0 and M+0x3560 ≠ 0 and k_entry < 10            # r8b (hint)
```

`0x70(%rsp)`의 다른 사용은 `a7b54f`(규칙 선택 `0x5b34f9`)와 규칙 5 본문(`0x5b3db3`)의 상태 포인터뿐이다. 전역 RNG 상태는 쓰지 않는다.

### 2.2 규칙 번호(asm `0x5b3326`–`0x5b3553`; C:381–461)

목록이 비지 않은 뒤 k = `W+0x30`(pop이 있었으면 pop 뒤 값, 없으면 진입 값):

```text
s = 5e70c1(M)                                       # 먼저 호출
if Mx+0x40 == −1e100 (ucomisd, NaN 아님) and lp0+0xc == lp0+0x2c:  r = 11     # lp0 = (M+8)+0xe8
elif h:                 r = 3
elif M+0x848 ≠ 0:       r = 6
elif env+0x27e0 ≠ 0:    r = 9                       # BranchDir
elif k ≤ 8:             r = (Mx+0x808 == 0) ? 6 : 1
elif k ≤ 16 or s == 0 or k % 11 == 0:  r = 2
elif k % 7 == 0:        r = 5e6efd(M) ? 4 : 2
elif k % 13 == 0:       r = 5
elif k % 19 == 0:       r = (Mx+0x808 == 0) ? 6 : 1
elif k % 43 == 9:       r = 7
elif k odd:             u = a7b54f(&seed); r = (u < 0.9) ? 3 : 12      # cmpltsd, 뽑기 1회
else:                   r = 3
if env+0x2710 == 4:     r = 5
```

`a7b54f`: `state = state·0x5851f42d4c957f2d + 0x14057b7ef767814f`(mod 2⁶⁴), 상위 32비트를 tempering해 31비트 × 4.656612873077393e-10(`crates/lp/src/common.rs` `Rng`).

### 2.3 규칙 3·12 점수와 꼬리(asm)

```text
# minEst (0x5b3541–0x5b35fc): 제외 항목도 포함, 2개씩 풀어 씀
m = 1e100; 짝 (a, b)마다 m = minsd(m, est_a); m = (est_b > m) ? m : est_b; 홀수 끝 m = minsd(m, est_last)
work += 3.0·n                                        # n = W+0x3fa8
# 점수 (규칙 12 0x5b3647–0x5b37c5, 규칙 3 0x5b3fab–0x5b4129)
best = −1e100, idx = −1
for j in 0..n−1:  I+0x41 ≠ 0 이면 건너뜀
   q = (est_j − m) / 1e8;  sc = (double)depth_j + q (규칙 12) | (double)depth_j − q (규칙 3)
   sc > best (comisd, 엄격) 이면 idx = j;  best = max(sc, best)
work += 4.0·n;  idx == −1 이면 idx = 0                # 0x5b4491–0x5b44a0
# 꼬리 (0x5b44a9–0x5b49d6)
sel = list[idx];  list[idx] = list[n−1];  n −= 1;  W+0x3fa8 = n
δ = 2                        if esi ≤ M+0x2c         # esi = 규칙 단계의 k (0x5b451b)
  = 3                        elif env+0x2710 == 4
  = 10                       elif T+0x138 > 0 and T+0x120 ≥ 10000 and k ≥ 1000
  = 6                        elif k % 23 == 7
  = clamp(trunc(log(0.2·depth_sel + 1.0) / log(2.0)), 3, 10)   otherwise
d′ = depth_sel − δ;  anc = 5fb13c(sel, d′)           # (d′ | depth−d′) < 0 → null; 아니면 부모를 depth−d′번(널에서 멈춤)
anc == null → 정리 없음(그대로 반환)
thr0 = 1e100;  5f9a41(M) ≠ 0 and 5e7796(M) > 5e8239(M, 2) − 1e-10 이면 thr0 = minsd(thr0, 5e7796(M))
thr = sel.I+0x41 ? −1e100 : thr0;  cut = 5e7204(M)
kept = 0
for j in 0..n−1:  it = list[j]
   5fb13c(it, d′) == anc and !(it.bound > thr) → list[kept++] = it
   elif 병렬(5e2919 ≠ 0) → worker 힙 (범위 밖)
   elif cut > it.bound → 5f9d4e(·, it, 4); 6e5802(M, it)    # 힙으로
   else                → 5f9d4e(·, it, 0)                   # 버림
W+0x3fa8 = kept;  work += 4.0·n
```

`5e7204(M)` = `(Mx+0x3220)+0x40`(없으면 1e100), `5e7796(M)` = 풀 첫 해 목적값, `5f9a41(M)` = 2·[`((Mx+8)+0xe8)+0xf4` > 0] + [`Mx+0x3484` > 0], `5e8239`은 읽기 전용(C 184줄, 쓰기 없음).

## 3. 비교 계약(판정 규칙, 첫 캡처 전 고정)

캡처는 06a 도구 `tools/mip_node_select.js`의 사건을 그대로 내고, 새 사건을 더한다(`tools/mip_plunge_rule.js`). 모든 새 사건은 최상위(06a 정의) `6e8eb3` 호출 안에서만 낸다.

| 사건 | 위치 | 필드 |
|---|---|---|
| `pin` | `6e8eb3` 진입 | k_entry, `W+0x28` 비트, h 입력(env+0x2e2c, M+0x3560 ≠ 0), M+0x848, Mx+0x808, Mx+0x40 비트, lp0 n·nint, env+0x27e0, env+0x2710, M+0x2c, `0x70(%rsp)` 없음(진입 전) |
| `s70` | `5e70c1`·`5e6efd` 반환(호출 지점 `0x5b3341`·`0x5b342b`) | 반환값 |
| `drw` | probe `0x5b34f9`(뽑기 직전) | `0x70(%rsp)` |
| `rpre` | probe `0x5b3539` | `%ecx`(MIPFocus 덮어쓰기 전), `%esi`, work |
| `rule` | probe `0x5b361d` | `%ecx`(최종), `%esi`, `%r8b`, n, `0x70(%rsp)`, work |
| `pick` | probe `0x5b44a9` | `0x40(%rsp)`(−1이면 0으로 바뀐 뒤 값), work, 목록 id 순서 |
| `tail` | probe `0x5b4506` | `%esi`, `M+0x2c`, k, T+0x138, T+0x120, 선택 id·depth·`I+0x41` |
| `anc` | `5fb13c` 진입·반환(호출 지점 `0x5b465b`·`0x5b4766`) | 노드 id, d′, 노드부터 위로 depth−d′+1개 id 사슬과 depth, 반환 id |
| `thr` | probe `0x5b46f4` | `0x20(%rsp)`, `0x28(%rsp)`(=`5e7204`), `(%rsp)`(=`5e7796`), d′, `5f9a41(M)`·`5e8239(M,2)`(읽기 전용 호출) |
| `rm` | `5f9d4e` 진입(호출 지점 `0x5b47aa`·`0x5b47d5`) | 노드 id, 단계 |
| `tend` | probe `0x5b49d1` | `W+0x3fa8`, 목록 id 순서, work |

판정(`tools/mip_plunge_rule_check.py`):

- **C0 대조군**: 사례마다 3회 gz 바이트 동일, manifest 해시(바이너리·모델·도구). 새 캡처에서 새 사건 종류와 새 counts 키를 빼면 `bench/oracle/m2_06a`의 같은 사례 run 1과 사건 JSON이 전부 같다(비간섭).
- **R**: §2.2 재생 번호 = `rule.ecx`, `rpre.ecx`와 MIPFocus 덮어쓰기 관계. 진입 k·pop 뒤 k 둘 다 기록하고 규칙 단계 k = `rule.esi`.
- **G**: `pin`으로 만든 seed = `drw`의 상태(뽑기 호출) 또는 `rule`의 상태(뽑기 없음). 뽑기 호출이면 `rule`의 상태 = LCG(seed), u로 재생한 3/12 = 번호.
- **S**: 규칙 3·12 호출에서 §2.3 점수 재생 첨자 = `pick`, 선택 노드 id = 06a `leave.node`, work 증가 `rpre→rule` = 3n, `rule→pick` = 4n(비트).
- **T**: 모든 호출에서 δ·d′(재생) = `anc`(선택) d′, anc는 사슬 재보행으로 다시 계산, `thr.0x20` = thr0 재생, 항목별 판정(유지/힙/버림) = `anc`·`rm`·`ins`(raw:5b47b7) 사건 순서, 남은 목록 = `tend`, work 증가 4n. anc null이면 `thr`·`rm`이 없어야 한다.
- **합격 조건**: 위 비교 전부 일치, 그리고 다음이 사례 합계에서 모두 1회 이상: 규칙 3 호출, 규칙 12 호출, 뽑기, 규칙 3/12 중 "깊이 최대 항목이 둘 이상이고 est가 다른" 판별 호출, 유지 항목, 힙으로 옮긴 항목, δ = 2·6·log 분기(δ > 3 포함). 하나라도 0이면 실패.
- **변종(V, 결과 전 고정)**: 아래를 재생에 적용해 거부되는 사례를 보고한다. 생존은 실패가 아니라 "그 절은 실행으로 판별되지 않음"이다.
  V1 규칙 3·12 부호 맞바꿈, V2 사전식(depth 우선, est 보조) 비교, V3 minEst를 제외 항목 빼고 계산, V4 동률은 뒤 항목, V5 시드를 pop 뒤 k로, V6 뽑기 문턱 0.5, V7 δ=2 절 끔, V8 δ=6 절 끔, V9 log 분기를 round로, V10 유지에서 조상 비교 무시, V11 유지에서 bound 비교 무시, V12 제거는 항상 힙, V13 δ=10 절 끔, V14 δ=3 절 끔, V15 규칙 단계 `s == 0` 절 끔, V16 k%7 절에서 `5e6efd` 무시(항상 4).
- **self-test(변조)**: 바이너리 해시 0, 빈 사건, `rule.ecx` 변경, RNG 상태 변경, `pick` 첨자 변경, d′ 변경, anc 사슬 id 변경, `tend` 목록 순서 변경, `thr` 값 변경, 3회 중 1회 사건 변경, 06a 투영 차이, `rm` 사건 삭제, 규칙 3/12 호출 전부 삭제(합격 0회), 문턱 null-null. 하나라도 거부되지 않으면 실패.
- 수치: 모든 double은 비트 비교. log는 Python `math.log`(같은 glibc)로 계산하고 δ 분포를 보고한다.

## 4. 사례

| 사례 | 실제/합성 | 노리는 분기 | 입력 |
|---|---|---|---|
| markshare_4_0 | 실제 | 규칙 3/12 다수(06a: k 홀수 548·짝수 542 호출), 동률 est | `../re/bench/data/miplib/markshare_4_0.mps.gz`, NodeLimit 2000 |
| pk1 | 실제 | 규칙 1/6·2·3/12, 꼬리 제거 다수 | `pk1.mps.gz`, NodeLimit 2000 |
| neos5 | 실제 | 꼬리 버림(06a 2회) | `neos5.mps.gz`, NodeLimit 2000 |
| tr12-30 | 실제 | 깊이 ≥ 75(06a 214회, log 분기 δ ≥ 4), 꼬리 버림 3회 | `tr12-30.mps.gz`, NodeLimit 2000 |
| (탐색 뒤 결정) | 실제 | `5f9a41 ≠ 0` 문턱(PoolSearchMode=2) | 탐색 결과로 §6에 추가하거나 asm만으로 남긴다 |

Threads=1, Seed=0, 06a와 같은 파라미터. 사례 4개는 06a 기준 자료와 투영 비교한다.

## 5. 재실행 명령

- 원본 캡처(Gurobi 한 프로세스씩): `for m in markshare_4_0 pk1 neos5 tr12-30; do for r in 1 2 3; do tools/gurun.sh node tools/mip_plunge_rule.js ../re/bench/data/miplib/$m.mps.gz bench/oracle/m2_06c/${m}_$r.json.gz '{"NodeLimit":2000}'; done; done`
- 기준 자료 고정: `python3 tools/mip_plunge_rule_check.py --write-manifest`.
- 판정: `python3 tools/mip_plunge_rule_check.py`(기대: `C0 PASS`, 사례별 `R/G/S/T PASS`, 합격 사건 수, `ACCEPTANCE PASS`, 종료 0). 변종 표는 같은 출력.
- 판별력: `python3 tools/mip_plunge_rule_check.py --self-test`(`SELF-TEST PASS`, 종료 0).
- Rust·Python 미변경: cargo·pytest 해당 없음.

## 6. 실행 결과

(첫 캡처 뒤 작성)

## 7. 판정과 인수인계

(작성 예정)
