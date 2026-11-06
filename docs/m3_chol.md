# M3 barrier 수치 비트 일치 기록 (분해·해법·합 순서)

Gurobi 13.0.3 barrier(Method=2, Presolve=0, Crossover=0, Threads=1, LP)의 수치를 비트 단위로 맞춘 기록이다. 규칙은 의사코드와 objdump(asm)로 읽고, Python 프로토타입과 Rust를 오라클과 비트 비교해 확정했다. 함수 이름은 `PRIVATE…`의 뒤 6자리, 주소는 file offset이다. F는 인수 구조체, B는 barrier 작업 구조체, D는 모델 데이터다([m3_barrier.md](m3_barrier.md) §3).

**교훈**: Ghidra 의사코드는 곱·합의 결합 순서와 SIMD 리덕션 순서를 자주 틀리게 보여준다(`a*b*c`를 `(a*b)*c`로 쓰지만 asm은 `a*(b*c)`, 8레인 리덕션을 순차 합처럼 씀). 결합 순서는 항상 asm으로 확인했다.

## 0. 결과 (2026-11-06, Netlib 64, `bench/parity_bar.csv`)

| 항목 | 결과 |
|---|---|
| 상태, 목적 ≤1e-6, Barrier statistics | 64/64, 64/64, 64/64 |
| **BarIterCount 같음** | **64/64** (이전 59/64) |
| BARRIER 콜백 궤적(pobj, dobj, PRIMINF, DUALINF, COMPL) 전 반복 repr 동일 | **56/64** |
| 같은 궤적, 상대 1e-6 안 | 63/64 (modszk1만 반복 16에서 갈림) |

단계별 비트 일치(오라클 비교):

| 단계 | 일치 | 근거 |
|---|---|---|
| 분해 L, d, 1/d, F+0xf0, F+0x100 (Python, 실제 MKL) | afiro, sc50a, adlittle, degen2, pilot4, perold, 25fv47 × 반복 0–2 전 원소 | `tools/proto/bar_chol.py` + `--chol` |
| 해법 dx, dy (Python) | 같은 7개 × 반복 0–2 전 원소 | `--solve`(콜백 안에서 Gurobi `2ee6c5`를 직접 부름) |
| Rust 분해+해법(첫 해법 x, 전 원소) | degen2, pilot4, perold, 25fv47, fit1p 전부; degen3 3306/3321 | `2d252f` 훅으로 뜬 입력 x |
| Rust 반복 상태 x, y, z, w, s | afiro·adlittle·sc50a 전 반복, perold 반복 0–3·50, fffff800 0–5, 25fv47 0–3 | `tools/proto/bar_cmp.py`(BARDUMP) |
| Rust 한 스텝(θ, 최종 방향, 마지막 corrector 방향, α) | 위와 같은 범위 | `tools/proto/bar_cmp_step.py` |

## 1. 분해 (`31c54b` → `2d8ee5`)

저장: F+0xb8[c]는 블록 c의 값이다. w×w 대각 블록의 하삼각을 행 순으로 압축한 뒤 (len−w)×w 직사각을 행 순으로 둔다. **모든 블록은 한 배열에 블록 순서대로 이어져 있고 시작이 16바이트 정렬**이다(64개 블록 주소 mod 64가 누적 크기와 일치). Threads=1이어도 F+0x17c개 작업(F+0x180/0x188 범위)으로 나뉘지만 분해 결과에는 영향이 없다.

블록 c(폭 w, 행 R)마다:

1. **조립**: w < 3 또는 w·len > 30000이면 `32d72f`, 아니면 `32e116`. 두 함수 모두 원소 곱은 fl(a_i · fl(a_k · θ_j))(k = 블록 열 쪽 행)이고, slack θ는 행 k의 열들을 다 더한 뒤 (k,k)에 더한다. 합 순서만 다르다.
   - `32d72f`: 블록 열 k마다 행 k의 열 j를 오름차순으로, 열 j의 k 이하 행 i 전부에 더한다.
   - `32e116`: 블록 행 k를 차례로 보며, 열 j의 "블록 안 첫 행"이 k일 때 열 j 전체(블록 안 행 쌍과 직사각 행)를 더한다.
   - F+0xf0(원 대각) = 조립 직후 대각.
2. **갱신** `32c49d`: 후손 supernode 목록(F+0x88/0x90/0x98 = **원천 supernode 오름차순**, f98 = 그 supernode 행 목록에서 c 이상인 첫 위치; 7개 인스턴스에서 전 블록 확인)의 각 블록 s, 그다음 같은 supernode의 앞 블록들. 원천 s의 행 위치 [p0, p1)(대상 열 범위 안)과 [p1, len)(직사각).
   - w_s = 1: DGEMM 없이 스칼라. 대상 w < 5이면 fl(L[행]·fl(−L[열]·d)), **w ≥ 5이면 fl(L[열]·fl(−L[행]·d))**(asm 0x2fe14f에서 −L·d를 미리 계산). 이 차이가 pilot4·perold·degen2 반복 1의 불일치 원인이었다.
   - w_s > 1: A = fl(−L·d)(행마다 w_s개), `DGEMM('T','N', m, n, k = w_s, 1, A, w_s, L 행, w_s, 1, C, ldc)`. 대각 부분은 n ≤ 24 덩어리, m = 누적 끝(위삼각 여분 원소 포함). 직사각 부분은 min(F+0x80 / nR1, 192)행 덩어리(대상 행이 연속이면 덩어리를 연속 구간 끝까지 늘림). 행/열이 대상에서 연속이면 C에 직접, 아니면 0으로 된 임시 C에 계산해 행 오름차순·열 오름차순으로 흩어 더한다. MKL 모델은 C + acc라서 두 경로의 값은 같다.
   - F+0x80 = max(480, max w², m) (`2cde25`).
3. **조밀 LDLᵀ**(대각 블록, 2d8ee5 0x2f20f0): 비대각이 모두 0이면 대각 경로(피벗만, DTRSM 없음). 아니면 열 j마다 wv[p] = fl(−L[j,p]·d[p]), 행 i ≥ j에 M[i,j] + Σ L[i,p]·wv[p]. 합은 j < 8이면 순차, 아니면 **8레인**(xmm 4쌍, 레인 0이 M에서 시작, 리덕션 ((l0+l2)+(l4+l6)) + ((l1+l3)+(l5+l7)), asm 0x2f2290) 뒤 꼬리 순차. 레인 앞 peel은 scratch 정렬(0)로 결정. i = j면 피벗 규칙(|d| ≤ 1e-16·|F+0xf0| → d = 0, 1/d = 0), i > j면 ×1/d.
4. `DTRSM('L','U','T','U', w, len−w, 1, 대각 버퍼, w, 직사각, w)`, 그다음 직사각 ×= 1/d(버린 피벗은 1).

## 2. MKL DGEMM/DTRSM 에뮬레이션 (`crates/lp/src/barrier/mkl.rs`)

정적 링크된 MKL(AVX2 경로, alpha = beta = 1)을 실험으로 해독했다(FPRev식 순서 탐지, 커널 asm 추적). 64개에서 나오는 DGEMM 2660개·DTRSM 746개 모양 전부, 임의 ld·정렬 0–56(mod 64)에서 실제 MKL과 비트 같다. 결과는 ld·정렬과 무관하다. 기록된 Gurobi 호출 재생: afiro 21/21, agg 744/744. Gurobi는 MKL을 다른 모드(CBWR 등)로 부르지 않는다.

- DGEMM('T','N'): m = 1 또는 n = 1 → dgemv_t(4레인, 8개 블록마다 fma(x,a, x'·a') + acc, 꼬리 마스크 fma, (a0+a1)+(a2+a3)); k = 1 → fma(a, b, C); `k ≤ 128이고 m·n/(m+n) ≤ 10이고 (m ≤ 100 또는 (n ≤ 3, k ≤ 3))` 또는 그 밖에서 m < 56 또는 n < 56 → nocopy 커널(행 8/4 블록, 열 6 묶음, 열 묶음 폭에 따라 fma 한 줄/짝홀 fma/짝홀 mul+add, C + acc, k·n 블록 분할); 나머지 → 패킹 12×4 커널(타일 모양별 누산기 수 1/2/4).
- DTRSM('L','U','T','U'): n = 1 → dtrsv; m ≤ 12 또는 (m % 4 = 0, m ≤ 88) → v1 경로(fma 사슬 + dtrsm_pst); 나머지 → 패킹 경로(행 12/8/4/2/1 블록).
- 실험 도구와 C/Rust 에뮬레이터 원본은 `tools/proto/mkl/`에 옮겨 두었다(바이너리·어셈블리·벡터 파일 제외). `build_emu.sh`가 `part_*.c`를 합쳐 `libmkl_emu.so`를 만들고, `test_all.py`가 `gshapes.json`/`tshapes.json`의 모든 모양을 libgurobi130.so 안의 진짜 MKL과 비트 비교한다. `syms.txt`는 `nm libgurobi130.so > tools/proto/mkl/syms.txt`로, 재생 입력(`../mkl_*.json.gz`, `../shapes/`)과 `vectors.jsonl`은 `tools/proto/bar_mkl_hook.py`·`gen_vectors.py`로 다시 만든다. `shortcut:` dtrsv m > 32(64개 최대 7), 패킹 DGEMM k > 256, DTRSM m > 254(블록 폭 ≤ 240이라 안 나옴), AVX-512 커널은 옮기지 않았다.

## 3. 해법 `2ee6c5`

dy = (LDLᵀ)⁻¹(rhs_y + A(θ∘rhs_x)), dx = (Aᵀdy − rhs_x)·θ(slack은 y 또는 0).

- `2d3b3e`(A·v): 행 순(D+0x118 행 사본, 열 오름차순) 8레인 dot, peel = 행 첫 값 주소의 16바이트 위상 = rbeg[i] & 1(D+0x138은 16바이트 정렬), 그 뒤 slack v[n+i]를 더한다.
- `2d3d91`(Aᵀy): 보통 열은 8레인, peel = beg[j] & 1. 공통 계수 열(B+0x648)은 계수를 broadcast하고 peel 없이 8레인 gather를 쓴다. `2d4252` 잔차 곱은 이 열에서 짝/홀 두 누산기를 쓴다. `2ca086`은 스케일된 열에서 첫 계수와의 차이 ≤1e-10을 검사하고, 해당 열 길이 제곱의 합 ≥2n일 때만 표를 유지한다(행 정렬 전). 2026-11-06 G5 복구에서 이 경로를 추가해 Presolve=0의 degen3 궤적이 일치했다([m1_driver §12](m1_driver.md#12-중단-세션-복구-공통-계수-열과-상태-13-오라클)).
- **Threads=1이면 `31cfeb`/`31d3c7`가 블록 커널을 [0, m) 한 번으로 부른다**(작업 목록 무시). 작업 목록을 쓰면 multi-task 인스턴스에서 틀린다.
- 전진 `2efdb8`(한 범위): 블록마다 블록 안 대입(삼각 행 8레인, peel = (블록 오프셋 + i(i+1)/2) & 1; 버린 피벗 자리는 0), 그다음 직사각 행에 오른쪽 방향으로 밀어넣기: 행 4개씩 2레인(peel은 4번째 행 주소), 남은 행은 8레인.
- ×1/d, 후진 `2efe1f`: 블록을 거꾸로; 직사각 행을 위에서부터 4개 묶음 x −= ((p0+p1)+p2)+p3(묶음 안은 행 오름차순), 남은 행은 하나씩, 그다음 블록 안 열 방향 갱신(원소별, 리덕션 없음).

## 4. IPM 합 순서 (`ipm.rs`)

| 값 | Gurobi | 확인 |
|---|---|---|
| compl `2d4977` | **x87 80비트 Kahan**(`3439d5`: y = p − c, t = s + y, c = (t − s) − y) 두 번: x·z(전 m+n), w·s(유한 ub 열 범위 B+0x630..0x634). compl = (τκ + dbl(S_ws + dbl(S_xz)))/ncomp | 오라클 42/42 기록 |
| `f80.rs` | 소프트웨어 80비트(64비트 가수, RNE): double×double, ±, →double | numpy.longdouble과 비교 |
| ‖r_p‖, ‖r_d‖ `2f9515` | 8레인 dot, peel은 두 번째 배열 정렬(B 배열은 16바이트 정렬 → 0) | |
| ‖r_u‖ | 유한 ub 열 범위만, peel = lo & 1 | perold 반복 0 |
| ctx, bty | (dot8(x, c) + 0) + B618·τ; ((0 + (dot8(y, b) − dot8(s, u)[lo..hi])) − 0) + τ·B618 | |
| r_d 열 | **8-SIMD 본체 열**: cτ − ((Aᵀy + z) − s), **스칼라 꼬리 열**(n − n%8 이후): ((cτ − Aᵀy) − z) + s; slack −y − z | fffff800 열 848–853, 25fv47 열 1570, perold 전 원소 |
| μ_aff `2f28c2` | 하한 있는 열 순차, 그다음 slack 전부와 유한 ub 범위 w·s를 각각 4레인(8개 블록, (l0+l2)+(l1+l3)), peel n & 1, lo & 1 | adlittle 스텝 0 |
| 초기 이동 `2d252f` | 합은 순차. sqrt 규칙의 곱은 **(‖·‖∞·1.0001)·shift**(asm 0x2ebe4e; 의사코드는 (shift·‖·‖∞)·1.0001로 보임) | afiro x⁰ 59/59 |
| 재시작 `2bf1ea` L956–994 | [m3_ipm.md](m3_ipm.md) §4.9 | perold 반복 99, pilot4 57 |

## 5. 도구

| 도구 | 용도 |
|---|---|
| `tools/bar_oracle.py --chol` | 반복마다 F의 수치 인수(블록 값, d, 1/d, F+0xf0, 블록 주소 mod 64)와 정적 구조(snode, blocks, rows, f88/f90/f98, 작업 목록) |
| `tools/bar_oracle.py --solve` | 콜백 안에서 Gurobi `2ee6c5`(와 `31cfeb`/`31d3c7`)를 시험 RHS로 직접 부르고 B 작업영역·F 스칼라를 되돌린다. D의 행 사본 시작·정렬도 기록 |
| `tools/proto/bar_chol.py` | 분해·해법 프로토타입(MKL은 libgurobi 안 것을 ctypes로) |
| `BARDUMP=<file>` (ipm.rs `dump`) | Rust 반복 상태·스텝을 비트 패턴으로 기록 |
| `tools/proto/bar_cmp.py`, `bar_cmp_step.py` | BARDUMP와 `--vec` 오라클의 원소별 비트 비교 |

함수 진입을 movabs+jmp로 패치해 인자를 뜨는 방식(`tools/proto/bar_mkl_hook.py`)으로 `2d252f` 입력도 떴다(일회성 스크립트, 저장소 밖).

## 6. 남은 차이와 `shortcut:`

- 궤적이 비트로 갈리는 8개(반복 수는 같음): 25fv47(행 22), bnl1(48), finnis(20), forplan(17), ganges(14), maros(18), modszk1(12), pilot4(79). degen3의 첫 해법 차이는 공통 계수 열의 합산 경로를 추가해 해소했다. modszk1은 반복 0–11 상태가 전 원소 같고 K = 3, N = 4, θ도 같은데 반복 11 스텝의 최종 방향과 마지막 corrector 방향이 갈린다(이 반복의 두 번째 corrector에서 ω_p = 0이 처음 나온다; `2f4fe9`의 ω = 0 분기부터 볼 것). 나머지는 어느 단계인지 아직 보지 않았다.
- `shortcut:` 정렬 가정: 블록 저장·D 값 배열·B 벡터의 시작이 16바이트 정렬이라고 둔다(관찰값). 다른 할당자에서는 peel이 바뀐다.
- `shortcut:` Bunch–Kaufman(`32af6a`, F+0xa0), 명시 행렬 경로(F+0x1e0 = 1/2), 단정밀도(`3279a1`), 밀집 열(F+0x138, B+0x7f0), SOC·QP 항(x87 `343b08`, `2f28c2` 원뿔 부분)은 옮기지 않았다(64개에서 안 탄다).
- `shortcut:` f80은 inf/NaN·subnormal double 결과를 다루지 않는다.
- `normal.rs`/`ldl.rs`(스칼라 LDLᵀ)는 IPM에서 더 쓰지 않고 `chol.rs` 시험의 비교 기준으로만 남았다.
