# M1-02p 부록 A: dual work 쓰기 지점별 식

[M1-02p](m1_02p_work.md) §6.2의 부록이다. 지점마다 저장 명령과 피연산자 사슬을 asm에서 읽고, 입력의 의미는 C에서 읽었다. 하위 에이전트 둘이 작성했고 원문(영어)을 그대로 옮겼다. "confirmed in asm"은 저장·피연산자·상수를 asm으로 확인했다는 뜻이다. **캡처로 식을 확인한 것은 §6.2 Q의 6지점뿐이다.** 나머지 식은 후속 구현(M1-02q–u)에서 입력을 캡처해 확인한다. "Rust hint"는 추정이다.

정정(캡처 근거): 아래 1부의 "안쪽 항은 모두 정수·반정수" 서술은 LU 풀이 넷(`9e288`·`9e376`·`9d112`·`2207de`)에는 맞지 않는다. 이 넷은 증분/x148이 반정수가 아닌 경우가 있다(§6.2). `1ca57` 0xbf981의 플래그 spx+0x338은 같은 호출 안 `1bae9c` 뒤에 바뀐다.

## 1부: Work-counter increments, one dual-simplex iteration (06268b -> 028c93), part 1

Conventions. `spx` = param_1 of the dual kernels. `x148` = `*(double*)(spx+0x148)` (1.3). `w` = the work pointer (`*param_N`).
`LU = *(spx+0x470)`; `LU+0x1f8` is a copy of spx+0x148 (251b4f L107 restores it; it is temporarily 1.0 around some calls, 251b4f L105-107).
`m = *(int*)(spx+0x64)` (rows / basic count), `n = *(int*)(spx+0x68)` (structural columns); `m+n` = total variables.
`sv = {int nnz; int* idx(+8); double* val(+0x10)}`; spx+0x208 = alpha_r sparse row, spx+0x218 / 0x220 / 0x228 = other work sparse vectors (all consistent with the Rust notes).
C line numbers are in `re/decomp/c/<file>.c` of the function (file name = `<off>_PRIVATE...<name>.c`; the first copy at the 0x1.... offset, not the 0x26.... twin).
Trap PC = instruction after the store. "store@" = the store instruction itself.
Constants verified by reading .rodata bytes (file offset == vaddr): 2a5d060=3.0, 2a5d068=5.0, 2a5d070=1.0, 2a5d200=5.0, 2a5d210=3.0, 2a5d258=0.5, 2a5d328=4.0, 2a5d338=2.0, 2a5d340=7.0, 2a5d2e0=10.0.

**FP order note (matters for bit-exactness).** All amounts inside the `(...)` are integers or multiples of 0.5 and exact in a double, so inner summation order is irrelevant. Only the last
multiply-add rounds. The grouping `(A*c)*x148` vs `A*(c*x148)` differs in the last bit for x148=1.3, so the grouping given below must be kept.
Exact patterns seen: `(A)*x148 + w` (mul then add, no FMA), `w + (A*4.0)*x148` (c8b67), `(x148*7.0)*n + w` (ce1ef), `((n*3.0)*x148) + w` (c7bcd).

| trap PC | function | C line | formula (increment of `*w`) | inputs (C lines) | guard | FP order | Rust hint (guess) | confidence |
|---|---|---|---|---|---|---|---|---|
| 0xbe039 (store@be035) | 15d2d | 108 (value at 104-106) | `w += (3.0*e + ((p12==1)+5.0)*(n_c - e)) * x148`, n_c = param_4 (= m, candidate scan length), e = counter `iVar22`: if `param_26==0` (spx+0x2c8==0) loop L43-95 counts every element so e = n_c and the second term is 0; else e counts only feasible elements (flag set to 2, L345) | loop 41-95 (param_26==0) or 332-390 (else); p12 = param_12 (cost-mode P+0x44); n_c L41, L325 | `param_27!=NULL` (L98/326/392) and stored only if `*param_25 > 0` (old list length, L103/107); otherwise the amount is carried in a register to the be0a1 store | xmm3=3.0*e; xmm1=(sete+5.0)*(n_c-e); (xmm3+xmm1)*x148; then +*w (asm be000-be02d) | pricing.rs `Pricing::rebuild` / `full_init` (CHUZR candidate list rebuild) | confirmed in asm |
| 0xbe0a1 (store@be09d) | 15d2d | 166 (value L160) | `w += (2*k + L)*x148`: k = `iVar20` = number kept in the sorted old list scan (L112-154), L = `iVar15` = old list length scanned = `*param_25` (0 if list empty) | L103-154 | `param_27!=NULL` and `n_c > 0` (else carried to be17e store) | `(2k+L)` as `(double)k` doubled then `+ (double)L`, `*x148`, `+ prior` (asm be074-be091) | same | confirmed in asm |
| 0xbe17e (store@be17a) | 15d2d | 163 | `w += x148 * (2*f)`, f = `iVar22` = number of newly flagged entries (flag==1 -> 0) appended by scanning n_c entries (L168-199) | L168-199 | `param_27!=NULL` | `xmm2 = x148*(f+f)`, then `xmm3 + xmm2` (asm be169-be17a); also reached with f=0 when n_c<=0 | same | confirmed in asm |
| 0xbe34d (store@be349) | 15d2d | 485 | `w += (double)(int)t * x148` where t = final length of the merged output index list written to param_6 (= new `*param_25` when merging, = iVar20 when not) | L209-483 | `param_27!=NULL` | `(t*x148) + *w` (asm be335-be349) | same | confirmed in asm (t = final list length: C only) |
| 0xc561a (store@c5614) | 28c93 | 295 | `w += (2*N0 + 5.0*N1)*x148`: N0 = `lVar13` = iterations over the list `lVar22` of length `local_40[0]` = bound-flip candidate count returned by 1c35c5 (before), N1 = kept = new `local_40[0]` (entries whose `lVar14[...] != spx+0x2fc` map test) | list loop L275-292 | `param_3!=NULL` and `local_40[0]!=0`; `local_40[0]==0` skips (asm c5561) | `((N0+N0) + N1*5.0)*x148 + *w` (asm c55e5-c5614) | bfrt.rs `bfrt` flip list | confirmed in asm |
| 0xc7bcd (store@c7bc8) | 1c35c5 | 438 | `w += ((double)nnz*3.0)*x148`, nnz = `*(int*)spx+0x208` (alpha_r sparse row count, both slack part [param_2,nnz) and structural part [0,param_2) scanned) | L297-436 scans | `param_6!=NULL` | `(nnz*3.0)*x148 + *w` (asm c7baa-c7bc8) | simplex.rs `chuzr`/ratio-test (alpha_r) | confirmed in asm |
| 0xc7c4f (store@c7c4a) | 1c35c5 | 752 | `w += (double)n1*5.0*x148` n1 = `uVar46` = number of ratio-test candidates in `param_5` list (`*param_4`-ish) after the first pass (L435). In the quick-select path L686 first does `w = (Dq*3.0*x148) + w` and the L751 value is added in `dVar109` form (same store insn, entered at c7c32) | L297-435, quickselect L478-692 | `param_6!=NULL`; store itself is c7c4a (asm c7c25-c7c4a) | `*w + (n1*5.0)*x148` (xmm6=n1*5.0; *x148; xmm5+xmm6) | bfrt.rs `bfrt` (candidate sort) | confirmed in asm (n1 meaning C only) |
| 0xc82ee (store@c82e9) | 1c35c5 | 873 | `w += ((double)A*3.0 + (double)B)*x148`: A = `lVar50 = uVar46 - uVar35`..(candidates scanned in the second filtering pass, L824-865), B = `iVar29` = number accepted (L832-858) | L813-873 | `param_6!=NULL` | `(A*3.0 + B)*x148 + *w` (asm c82c0-c82e9; ecx=A, edx=B) | bfrt.rs | confirmed in asm |
| 0xc8b67 (store@c8b62) | 1c35c5 | 1471 | `w += (4.0*C)*x148`, C = `uVar43` = accepted candidate count at LAB_001c8b0b | L1426-1470 | `param_6!=NULL` | `w + (C*4.0)*x148` (xmm0=4.0*C; *x148; xmm2(w)+xmm0) (asm c8b4d-c8b62; note order differs from the other sites) | bfrt.rs | confirmed in asm |
| 0xc92bd (store@c92b8) | 1c35c5 | 1564 | `w += ((double)(int)uVar35*4.0)*x148`, uVar35 = size of the candidate prefix used in the final selection loop (L1517-1561) | L1517-1561 | `param_6!=NULL` and `spx+0x3fc (iVar7) != 0` | `(Cs*4.0)*x148 + *w` (asm c929b-c92b8) | bfrt.rs | confirmed in asm |
| 0xc9436 (store@c9431) | 1c35c5 | 1618 | `w += ((double)uVar35*4.0 + (double)uVar43)*x148`, uVar43 = survivors after re-filter (L1582-1616) | L1582-1618 | `param_6!=NULL` (and a candidate was found, `lVar34 >= 0`) | `(Cs*4.0 + Kp)*x148 + *w` (asm c940f-c9431) | bfrt.rs | confirmed in asm; executes once in kb2, 0 in degen2 |
| 0xcbcae (store@cbca9) | 1c35c5 | 688-689 | quick-select partition cost: `w += ((Q*3.0)*x148)` where `Q = dVar109 = auVar87` = sum over partition rounds of `(double)(hi - lo + 4)` i.e. `((iVar29+1)-(lo-1))+2` per round (L487-492), reset to 0 at each outer repetition (L487) | partition loop L476-672 | `param_6!=NULL`; this store runs only when `dVar76 > dVar63`-test fails (`dVar76 <= dVar63` skips to c7c32 with the value kept in a register) | `(Q*3.0)*x148 + *w` (asm cbc89-cbc9e) | bfrt.rs `bfrt` sort/partition | confirmed in asm |
| 0xcce72 (store@cce6d) | 2896a3 | 252 (value 249-250) | `w += ((double)i27*0.5 + (double)i23 + (double)(i18 - i23)*2.0)*x148`: i27 = candidates kept in compacted list `W[0x10]` (flag<2, value >= threshold W[0x30]); i23 = entries with flag >= 2; i18 = total scanned = `iVar26`; (0 when W[0x28] < 1) | list loop L145-171 | `param_2!=NULL`; this store taken when `spx+0x2f8 != -1` (a candidate chosen), asm cce64 | `((0.5*i27 + i23) + 2.0*(i18-i23))*x148 + *w` (asm cce25-cce6d) | pricing.rs `next_listed` / list partial pricing | confirmed in asm |
| 0xcd052 (store@cd04d) | 2896a3 | 259 | stores the same value dVar32 as L249 when no candidate chosen (`spx+0x2f8==-1`) and `m>0` (uVar4>0) | L249-260 | `param_2!=NULL`, `spx+0x2f8 == -1`, `m > 0` | as above; value already includes `+*w` | same | confirmed in asm |
| 0xcd191 (store@cd18c) | 2896a3 | 263 | `w += ((double)i21 + (double)i18 + (double)i18)*x148`: i21 = entries added to the list in the full rescan L181-237 (flag 0 -> 1, value >= thr/10), i18 = number of rescan positions scanned (`uVar4` rounded: pair loop count*2 or last index) | L181-237 | `param_2!=NULL` (also reached with i21=i18=0 when `m<=0`) | `((i21+i18)+i18)*x148` added to prior `w` (asm cd173-cd18c: xmm3=edx+r8d+r8d, *x148, xmm5+xmm3) | same | confirmed in asm |
| 0xcdbb5 (store@cdb1) | 1c8477 | 158 | `w += (dv + dv)*x148` with dv = `4.0*S` (sparse path, L186: S = sum over flipped columns of column length `spx+0x78[j]` for structural j<n, 1 for slack, L169-174) or `dv = 2m` (dense path, `iVar9<2*S`, L101); so increment = `(8*S)` or `(4*m)` units | L164-182 | `param_4!=NULL`; same insn for both paths (jump to LAB_001cdb99) | `((dv+dv)*x148) + *w` (asm cdba1-cdbb1) | simplex bound-flip update of x_B (no Rust work yet) | confirmed in asm |
| 0xce1ef (store@ce1eb) | 1c8477 | 470 | `w += ((x148*7.0)*(double)F)`, F = final `*puVar18` = nnz of the FTRAN result list (spx+0x228) after the x_B update loops (L261-467) | loops L261-467 | `param_4!=NULL` | `(7.0*x148)*F + *w` (xmm1=7.0; *x148; *F; +w) (asm ce1cb-ce1eb): grouping `(x148*7.0)*F` | same | confirmed in asm |
| 0xce316 (store@ce312) | 1c8477 | 256 | `w += ((double)M + (double)K)*x148`, M = loop count `lVar38` = m (dense FTRAN result compacted, L241-252), K = kept nnz `uVar33` | L237-254 | `param_4!=NULL` and FTRAN result dense (`*puVar18<0`) | `(M+K)*x148 + *w` (asm ce2f2-ce312) | same | confirmed in asm |
| 0xe7000 (store@e6ffc) | 48d5f | 2188 | `w += T*x148`, `T=local_118`: sum of stage terms, **path dependent**, see Appendix B. Both callee BTRAN 9e288 (L661) and kernels add to `*w` before this final fold | see Appendix B | `param_2!=NULL` | `(T*x148) + *w` (asm e6fe8-e6ffc), re-reads `*w` after callees | simplex.rs dual update after ratio test (guess) | confirmed in asm for the store; T per-path C only |
| 0x120114 (store@12010f) | 8b473 | 485 | `w += T*x148`, T = `dVar46`: **path dependent**, see Appendix C (e.g. 4*len + |rho| + ... + m + 2*kept); callees 9d112 / 9cf9b add before the fold | Appendix C | `param_2!=NULL` | `(T*x148) + *w` (asm 120101-12010f) | simplex.rs `ftran_col` + DSE update (guess) | confirmed in asm for store; T C only |
| 0x1219d6 (store@1219d2) | 2207de | 1047 | `w += local_68*LU[0x1f8]`, local_68 (stack 0x70) accumulated by: L484, 683, 805, 888, 903, 945, 959 and by callee kernels given `&local_68` (21b9de, 214344, 21686f, 2126cd, 21574d). See Appendix D | Appendix D | `param_11!=NULL` | `local_68*LU[0x1f8] + *w`: multiplier is LU+0x1f8 (not spx+0x148!) (asm 1219bf-1219d2) | lu.rs `ftran_*` (guess) | confirmed in asm for fold; kernel content C only |
| 0x129671 (store@12966d) | 9d112 | 2437 | `w += LU[0x200]*LU[0x1f8]`, LU+0x200 reset to 0 at L127 then accumulated inside the body and inside kernels passed `LU+0x200`. See Appendix D | Appendix D | `param_8!=NULL` | `LU[0x200]*LU[0x1f8] + *w` (asm 129657-12966d) | lu.rs `ftran2` (guess) | confirmed in asm for fold |
| 0x12de50 (store@12de4c) | 9e288 | 17 | `w += LU[0x200]*LU[0x1f8]`; LU+0x200 zeroed at entry (asm 12de1c) then incremented by callee 221173 (BTRAN) and kernel 21a941; LU+0x1f8 may be 1.0 when called from 251b4f (L105-107) | Appendix D (221173) | `rcx(param_4)!=NULL` and `*param_2 != 0` (input nnz != 0, else `*param_3=0` and no fold) | `LU[0x200]*LU[0x1f8] + *w` (asm 12de38-12de4c) | lu.rs `btran_dense` / `btran_updated` (guess) | confirmed in asm for fold |
| 0x13174d (store@131749) | 9e376 | 10 | same as 9e288 but without the `*param_2!=0` test: `w += LU[0x200]*LU[0x1f8]` | Appendix D (221173) | `param_4!=NULL` (28c93 passes its own `w`, asm c518d `mov %r13,%rcx`) | same | lu.rs `btran` | confirmed in asm |
| 0x135e92 (store@135e8e) | a4109 | 1394 | `w += W*x148`, **W** = case-dependent sum below (matches m1_notes 8: 3.5*work + 6*|rho| + 3*touched for the sparse-order path) | see Appendix A | `param_10!=NULL` | `(W*x148) + *w` (asm 135e81-135e8e) | simplex.rs `pivot_row` (L440) | confirmed in asm for fold; W per C |

### Appendix A: a4109 (alpha_r), full W

Inputs: R = |rho| = `*param_7` (L63/1001: re-read `uVar46=*param_7` after the case block, so = |rho| in the tail), W0 = `param_5` (int, caller's `work = sum of row lengths`, from a171f/a17b5),
n = `uVar39` = `*(spx+0x68)`, `param_4` = caller flag `(double)n < 1.8*work` (28c93 L189 call), `touched = uVar53` = number of structural columns touched (sparse-order path).
Case selector (L91 `uVar51==1`):

1. **R == 1** (L91-482): `len` = `uVar46` = row length of the single row `*(row-copy+0x20)[r]` (= `dVar66 = (double)len`, L443). If `|rho_r| < 1.0` (filter pass `param_1<|v|`): `W1 = 3.0*len` (L481); else `W1 = len + len` (L554).
   Then `W1 = W1 + len + 5.0 + len` (L556, left to right: `((W1 + len) + 5.0) + len`) => `5*len+5.0` or `4*len+5.0`.
2. **R>1, `param_4 == 0`** (L559-792): `W1 = (double)W0*3.5 + (double)R*6.0 + (double)touched*3.0` (L792).
3. **R>1, `param_4 != 0`** (dense accumulate, L793-997): `W1 = (double)(W0*2) + (double)n*1.5 + (double)n + (double)R*3.0` (L996; note `W0*2` is an **int** multiply converted afterwards).

Tail (all cases, L1353-1390): `W2 = (double)R*1.5 + W1 + (double)uVar57*4.0` with `uVar57 = R` (slack append count; 0 if R<1) =>
`W2 = ((R*1.5 + W1) + R*4.0)`. Then if `spx+0x2cc != 0 && *(double*)(param_2+0x10) == -1.0` (param_2 = spx+0x1d0 row copy struct):
`W = ((R + R) + W2) + iVar63 + iVar63`, iVar63 = R (L1390; asm 135e4c-135e64: `xmm2=2R; +W2; +iVar63; +iVar63`); else `W = W2`.
Final fold: `*param_10 = W * x148 + *param_10`.
(All values are exact integers/half-integers; only the final fold rounds.)

### Appendix B: 48d5f stage terms (`T = local_118`, C lines in 000e45c0 file; branch not traced, which branch runs per iteration is NOT determined)

`m=uVar9`, `n=uVar10`, `local_a8=(double)m`, `local_e0=(double)n` (L621-622), `local_d8 = m+n`.
Main-path pieces (L596-880):
- L258 / L4c9d branch: `T = m + n + n` (rescale branch, `local_118 = uVar9+uVar10+uVar10`).
- L211: `T = 3.0*iVar32` (small-update branch, iVar32 = |changed list|).
- copy of c_B (L596-619): `auVar52 = T + iVar19 + iVar19` (iVar19 = m copied) or L211-like `3.0*iVar19` (L205).
- L742: `T = m + n + n` (when `spx+0x338 == 0`) or L841 `T = m + m + 3.0*n` (else).
- L844: `T += auVar52`.
- then callees: `9e288` (L661, BTRAN, adds to `*w` itself) and `a4268` (no `w` arg).
- L876: `T += (double)iVar19` (iVar19 = `spx+0x52c` list copied count).
- L937: `T += (double)lVar48` (scan of `local_d8` entries).
- L1009 / L1124: `T += n*1.5`; L1040: `T += (double)iVar32 + n`; L1168: `T += (double)uVar23*0.5` (uVar23 = n + iVar19);
- L1412: `T += n + n + (double)uVar27`.
- scaling branch: L1564 `local_e0 = n + m + n`, L1566/1641/1703: `T = auVar52 + local_e0`, L1761: `T = auVar52 + local_e0 + (double)iVar19`, L1956: `T += iVar32 + iVar32 + iVar19`.
- L421: `T += iVar19 + iVar19 + iVar32 + iVar32`.
Final fold L2188 `*w = T*x148 + *w`.

### Appendix C: 8b473 T (`dVar46`) stage terms

`m=uVar2 (=spx+0x64)`, `n=iVar29 (=spx+0x68)`, `q = spx+0x2fc` entering var.
1. L98: if `q < n`: copy column q: `T = 4.0*len_q` (len = `*(spx+0x78)[q]`, loop count `local_38[0]`); else `T = 0`.
2. if `spx+0x3ec != 0`: L224 `T += (double)cnt`, cnt = nnz of spx+0x218 vector (or m if dense, 0 if empty) (norm^2 into spx+0x310). Then:
   - `spx+0x3ec != 3`: no more terms here.
   - `==3` and `3*nnz*(double)*(long*)(spx+0x3d0) < (double)n*(double)m` (L ~236): sparse accumulate: per entry structural `T += 2.0*len_col` (L252), end `T += nnz*3.0 + K + K + Q + Q`, K = touched count, Q = kept after tolerance filter (L311).
   - else dense accumulate: per entry structural `T += 3.0*len_col` (L363); end `T += (double)m + (double)cnt*3.0` (L416).
3. callee `9d112` or `9cf9b`->`2207de` adds to `*w` directly.
4. L452: if result `*piVar5 < 0` (dense): `T += (double)iVar29 + Q + Q` with `iVar29 = m` (loop var), Q = nnz kept.
Fold L485.

### Appendix D: LU solve kernels (C-only extraction, guards not traced)

LU+0x08 = dimension (m), LU+0x180 = number of FT/R etas, LU+0x184 = their length. Each statement adds to the work cell (`*param_6` of the kernel, `LU+0x200`, or `local_68`).

- **221173 (BTRAN, used by 9e288/9e376)** (file 0012de60): L114 `2*nnz_in + m` (dense U^T branch, input nnz>0); L190 `m + m`; L203 `2*len(U column)` per nonzero (`(double)(long)uVar16 * 2.0`); L1171 adds `dVar54` (accumulated); L1381/1572 `2*LU[0x184]` (R etas); L1490 `Dq + (n1 - uVar38)*2.0 + n1*3.0`; L1592 `Dq + 2*uVar38 + m`; L1674 `Dq + m + m`; L1683 `2*len` per nonzero. Kernel 21a941 (sparse U^T) final fold L427: `LU[0x200] = dVar37 + LU[0x200]`; dVar37 = `3*uVar26` (L70) + `2*len` per column (L104, 164, 345, 387) + `2*(param_3 - iVar25)` (L154).
- **2207de (FTRAN)**: L484 `Dq + (double)local_c0*3.0`, L683 `iVar34*2.0` (R etas), L805 `m + local_68 + 2*uVar19`, L888 `(m - iVar5)*5.0`, L903 `((double)uVar27+2.0)*2.0`, L945 `2*local_60`, L959 `((lVar31-lVar38)+2.0)*2.0`. Kernels with `&local_68`: 214344 (L58 `m + 2*nnz`, L130 `2*(m-n_b) + 2*iVar5`, L136/193 `2*len`), 21b9de (fold L896 `dVar47 + *w`, dVar47 L842 `(param_3-iVar41)*2.0 + ...`, L854 `2*len`, L255/603 `(.. *7.0*iVar37)/..`), 21686f (L626-628 `*param_8 = dVar46 + dVar45`), 2126cd (L62 `m + 2*nnz`, L153 `(len+2.0)*2.0`, L206 `3.0*...`).
- **9d112 (FTRAN with save/update, two RHS)**: reset L127; L225 `dVar71 = uVar5 + 2*uVar21`; L315 `+= uVar5 + 2*uVar44`; L399/451/532/584 `dVar91 += 2*len`, L499/632 `+= 4*len`; L655 `LU[0x200] += ((uVar5 - iVar48) + iVar27)*4.0 + dVar71 + dVar91 + dVar84`; L1360 `LU[0x200] += 4*LU[0x184]`; L1556/1741 `dVar71 + 3*lVar49 + LU[0x200]`; L2427 `LU[0x200] += ((dVar84 + max(0,uVar21))*5.0 + dVar71 + dVar91 + auVar85 + dVar59 + dVar59)`; kernels 214344/21574d/21686f/2126cd/21b9de/21f83a/21841f take `LU+0x200` and add by C as in the lists above (21f83a L263 `2*iVar32`, L880 `dVar50 + 3*lVar20 + 2*iVar32`; 21841f L876 `dVar60 + dVar51`).

Unresolved / not traced: exact per-iteration branch of 48d5f and 8b473, the semantic meaning of `uVar35/uVar43` inside 1c35c5 beyond the labelled counts, and kernel internal accumulators (dVar71/dVar91/dVar84/auVar85 in 9d112 beyond the listed lines). These are C-only, not asm-checked.

## 2부: Work-counter increments: 5b257, 36ade, 9e60b, 16b0a, 1ca57, 1bae9c, 1b7106, 534f7

Notation. `w` = `*work`. `x148` = `*(double*)(spx+0x148)` (1.3). LU = `*(spx+0x470)`. m = `*(int*)(spx+0x64)`, n = `*(int*)(spx+0x68)`.
C line numbers = lines of the decompiled C file of that function (`../re/decomp/c/<addr>_PRIVATE...<name>.c`).
Constants read from the binary (file offset = vaddr): 2a5d060=3.0, 2a5d068=5.0, 2a5d098=4.0, 2a5d0a0=2.0, 2a5d0f0=7.0, 2a5dd98=3.0, 2a5dda8=5.0, 2a5dde8=2.0, 2a5ddf0=9.0, 2a5db20=2.0, 2a5d7c0=2.0, 2a5d790=5.0, 2a5d788=100.0, 2a5d798=3.0, 2a5d7a8=10.0, 2a5d7b0=7.0, 2a5d7d8=6.0, 2a5d838=24.0, 2a5d848=4e6, 2a5d850=9.0.
Counts = kb2/degen2 executions from sites.tsv. conf "asm" = the store instruction, operand chain and constants were read in the asm; where an input's meaning was only read from C or inferred, that is said in the row.

Layout used (m1_notes sec 8 / m1_02n): U columns +0x88 start, +0x98 section boundary, +0x90 end, +0xa8 row idx, +0xa0 val. U rows +0xb0 start, +0xb8 end, +0xd0 col idx, +0xc8 val. LU+0x40 pos, +0x48 ord, +0x100 prefix t, +0xe8 empty-U-column cache, +0xdc, +0x1c8 spike {nnz,idx,val}, +0x1d0 U^T e_r row-eta vec {nnz,idx,val}, +0x58 L column start (long[]), +0x1f8 LU-local copy of the multiplier (251b4f L107 sets it to 1.0 and restores it from spx+0x148; assumed = x148, not proven for every LU).
In 05b257: r = `*(int*)(spx+0x2f8)` (iVar10, leaving position), nS = `*(uint*)(LU+0x1c8)` (iVar33, spike nnz), local_40 = new prefix bound, local_80 in {1,2,3} (1: pos[r] <= t-1, 2: pos[r] > t-1, 3: spike empty and l==p).

### A. 05b257 (spx, &w). Update path 0xf1beb..0xf4073; refactor-return path 0xf461f..0xf51af (after 36ade; 9e60b is called between 0xf48ae and 0xf4f9b)

| trap PC (kb2/degen2) | C line | formula | inputs | guard | FP order | Rust hint (guess) | conf |
|---|---|---|---|---|---|---|---|
| 0xf1beb (25/422) | 417-418 | `w += 3.0*x148*(double)nRow` | nRow = `*piVar15` = nnz of U^T e_r vec LU+0x1d0 (C 300-415, dot product S[r]=sum S[idx]*val loop count) | param_2!=0; in `iVar35 != iVar52` branch; only if nRow>0 | asm: `((3.0*x148)*(double)(int)n) + w` | row-eta length in `Lu::update` (`retas`) | asm |
| 0xf1d02 (25/422) | 490-492 | `w += (double)(e8 - i)*3.0*x148`; e8 = LU+0xe8 before, i = stop position of a scan downward from e8 | C 468-492: i=e8 down while `i > iVar52` and U column of `ord[i]` non-empty (`LU+0x90[c] != LU+0x88[c]`); the else path (iVar52 >= e8) gives 0 | param_2!=0; `else if (pos[r] < iVar52)` branch | asm: `((double)(ecx-edx)*3.0)*x148 + w` | `e8` field, empty-column search in `update` | asm |
| 0xf1f37 (1/97) | 550-551 | `w += x148*3.0` per event | event = entry j of U column r (loop C 525-570) whose U row j has length 1 (`start+1==end`) and `pos[j] <= local_40` (row j leaves LU+0xec list) | param_2!=0, per event | asm: `x148*3.0 + w` | `f0`/`f8` list pop | asm |
| 0xf1f82 (53/945) | 559-560 | `w += (double)(p - s)*x148` per event | same loop; row j of length != 1: s = LU+0xb0[j], p = first index in [s, LU+0xb8[j]) with col idx (LU+0xd0) == r (data dependent offset) | param_2!=0, per event (not for the singleton case, which jumps past the store) | asm: `(double)(int64)(p-s)*x148 + w` | urow scan removing column r | asm |
| 0xf2044 (33/601) | 593-596 | `w += (double)cU*5.0*x148`, cU = LU+0x90[r] - LU+0x88[r] = nnz of U column r | lVar12 after the loop = end of column; local_150 = (int)cU | param_2!=0 and `local_150 < nS` | asm: `((double)(int64)cU*5.0)*x148 + w` | `ucol[r].len()` | asm |
| 0xf207a (2/19) | 593-596 (`*param_2=dVar49; goto 207f`) | same amount as 0xf2044 | same | param_2!=0, `local_150 >= nS`, nS>0. If nS<=0 the amount is carried (no store) into the 0xf2521 store: 601+19+1 = 621 | identical | same | asm |
| 0xf2521 (35/621) | 797-798 | `w += (double)nS*3.0*x148` (nS<=0 gives 0) | iVar14 ends = iVar33 (loops C 610-790) | param_2!=0 | asm: w reloaded, `w + ((double)nS*3.0)*x148` (addsd xmm1,xmm2) | spike length | asm |
| 0xf2969 (19/545) | 1004-1006 | `w += (double)(p - s)*x148` per column | `local_80 != 1` (=2) with iVar27>0: for each column j in U row r (LU+0xd0[LU+0xb0[r]..LU+0xb8[r])), scan U col j section 1 [LU+0x88[j], LU+0x98[j]) for row r | param_2!=0 per column | asm: `(double)(int64)(p-s)*x148 + w` | section-1 swap-remove | asm |
| 0xf2a51 (35/528) | 1039-1041 | `w += (double)(lVar12 - LU+0xb0[r])*3.0*x148` = nnz of U row r | lVar12 = end of the loop over U row r (=LU+0xb8[r]) | param_2!=0; reached after the eta branch (iVar27>0) and after the `local_80==3` branch; no store when `iVar27<1 && local_80!=3` | asm: `((double)d*3.0)*x148 + w` | `urow[r].len()` | asm |
| 0xf2e3c (35/621) | 1316-1317 | `w += (double)nS*5.0*x148` | lVar21 = loop counter of C 1240-1310 (r15 in asm, bound 0x38(%rsp)=iVar33); 0 if nS<=0 | param_2!=0 | asm: `((double)(int)nS*5.0)*x148 + w` | spike length | asm |
| 0xf3ca4 (25/510) | 1175-1176 | `w += acc*LU[0x1f8]`, acc = sum_k ((double)len_k*2.0 + 5.0), acc starts 0.0 | C 1141-1171: each step moves one U position (U column c=ord[iVar35], len_k = LU+0x90[c]-LU+0x88[c]) in the cyclic shift | param_2!=0; branch `iVar35 > iVar52` | asm acc: `(acc + len*2.0) + 5.0`; store `acc*LU[0x1f8] + w` (LU+0x1f8, not x148) | order-cycle loop in `update` | asm; LU+0x1f8 == x148 assumed |
| 0xf3d83 (25/510) | 1207-1208 | `w += (double)k*2.0*x148`, k = iVar27 - iVar52 = number of U positions shifted by the cycle | C 1180-1205 | param_2!=0, same branch | asm: `(2.0*(double)k)*x148 + w` | position shift | asm |
| 0xf3eac (1/103) | 1064-1065 | `w += (double)(p - s)*x148` per column | `local_148==1` block (C 1086-1115): for each column j in U row of `ord[iVar52]`, scan section 1 [LU+0x88[j], LU+0x98[j]) for that row | param_2!=0 per column | same as 0xf2969 | same | asm |
| 0xf3f71 (63/769) | 986-988 | `w += (double)(p - s)*x148` per column | `local_80==1`: per column j in U row r scan section 2 [LU+0x98[j], LU+0x90[j]) for row r, s = LU+0x98[j] | param_2!=0 per column | asm: `(double)(int64)(p-s)*x148 + w` | section-2 swap-remove | asm |
| 0xf4073 (5/41) | 883-884 | `w += (double)(p - s)*x148` per column | `local_80==3`: same scan as 0xf3f71 | param_2!=0 per column | same | same | asm |
| 0xf461f (3/13) | 2028-2029 | `w += ((double)k+(double)k)*x148`, k = nS (spike nnz, loop clearing spike values C 2010-2027) | `*(uint*)(LU+0x1c8)` | param_2!=0, k>0, refactor-return path | asm: `(cvt(k)+cvt(k))*x148 + w` | spike clear | asm |
| 0xf48ae (5/16) | 2545-2546 | `w += (double)m*3.0*LU[0x1f8]` | m = `*(uint*)(spx+100)` (uVar11 in the block C 2420-2545) | param_2!=0, every refactor | asm: `((double)(int)m*3.0)*LU[0x1f8] + w` | post-factor reset | asm |
| 0xf4f9b (5/16) | 2611-2615 | `w += (double)m*9.0*x148` (m>0) | loop count uVar31 over `i<m` (C 2574-2590) | param_2!=0; stored only if `LU+0xdc > 0`, otherwise carried into the next store | asm: `((double)m*9.0)*x148 + w` | refactor reset | asm |
| 0xf4ffd (3/13) | 2625-2633 | `acc += x148*(double)d1`, d1 = LU+0xdc (if >0, loop C 2620-2632 sets LU+0xf8[0..d1) = -1) | | stored if `d1 < LU+0x100` | asm: `acc + x148*(double)d1` | `dc` | asm |
| 0xf507a (3/13) | 2651-2674 | `acc += ((double)d+(double)d)*x148`, d = max(LU+0xdc, LU+0x100) (iVar10 after the init loops C 2650-2673) | | stored only if `d < m` (cmp 0xf0(%rsp)=m); the d>=m store at 0xf5156 was never hit | asm: `acc + ((double)d+(double)d)*x148` | `prefix`, `f8` init | asm |
| 0xf51af (5/16) | 2725-2729 | `w += (double)((m - i) - 1)*x148`, i = final LU+0xe8: scan down from m-1 while `LU+0x88[i+1] != LU+0x88[i]` (C 2700-2715, stops at i <= LU+0x100); 0 steps if LU+0x100 >= m-1 | | param_2!=0 | asm: w reloaded, `w + (double)(m-i-1)*x148` | `e8` recompute | asm |

Correction (M1-02r, docs/m1/m1_02r_ft_work.md): on the update path nS in 0xf2044/0xf207a/0xf2521/0xf2e3c is the spike list length after r is taken out (iVar33 = `*(uint*)(LU+0x1c8)` − 1 when r is listed); only 0xf461f uses the full LU+0x1c8 length. 0xf1d02 runs 19 times on kb2 (not 25). 0xf2a51 stores only when the row eta is non-empty or local_80 = 3, 0xf3ca4/0xf3d83 only when l > p; 0xf1f37 needs pos <= local_40 (else no store).

Running sum order on the refactor-return path, whatever stores are skipped: `((w + m*3*LU1f8) [9e60b adds here] ) ... + m*9*x148 + x148*d1 + (d+d)*x148 + (m-i-1)*x148` (C 2574-2729; skipped stores carry the partial sum in a register).

### B. 036ade (LU factor; param_5 = nb = m, param_19 = &w, param_3 = spx)

nb=(double)(int)param_5 (dVar81). nnzB = local_128 = sum over basis columns of (col length if col idx < param_6 structural, else 1) (C 296-330). P0 = `param_15[4]` (initial `local_110`). `1da0b2` updates `local_110` (pool pointer moved down per emitted element) and `local_100` (U elements written).

| trap PC | C line | formula | inputs | guard | FP order | Rust hint | conf |
|---|---|---|---|---|---|---|---|
| 0xd4ded (5/16) | 535-541 | `w += (nb*5.0 + 100.0 + (double)((P0 + local_100) - local_110)*3.0)*x148`; also `spx+0x150 = w_old` | local_100, local_110 after the triangular phase `1da0b2` (C 372) | param_19!=0 (else `spx+0x150 = 0`) | asm: `((5.0*nb + 100.0) + (double)(int64)q*3.0)*x148`, `w_new = w_old + that` | `Lu::factor` singleton phase (`n_singleton`) | asm formula; meaning of q = guess |
| 0xd5300 (5/16) | 762-764 | `w += (10.0*nb + 3.0*nnzB + 7.0*act)*x148`, act = uVar38 = nnz left in the active submatrix after the triangular phase (basis entries with row unpivoted `local_a8[row]<0` and column unpivoted `local_c8[col]<0`, slack counts 1, min 1; C 543-585) | | param_19!=0, once per factor | asm: `(10*nb + 3*nnzB) + (double)act*7.0`, then `*x148`, then `+w` | kernel nnz in `factor` | asm |
| 0xd85e6 (5/16) | 1531-1533 (`LAB_001d85c7`) | `w += acc*x148`; acc = Markowitz-loop work accumulator: starts 0.0 (C 801), flushed early if >1e8 (C 817-822); per pivot step: singleton column pivot `acc += ((double)cnt+1.0)*10.0` (C 924, asm 0xd647b), singleton row pivot same (C 1072, asm 0xd6077), general pivot `acc += (double)k1*6.0*c` (C 2563, asm 0xd6ea4) then `acc += (double)k2*10.0` (asm 0xd7178 / 0xd78eb) | | param_19!=0; every factor exits the pivot loop through C 1526/1626/2569 (flag local_70=1 on 2569) | asm: `acc*x148 + w` | pivot loop in `factor` | asm for the flush; individual per-pivot terms only partly traced |
| 0xdc7b2 (5/16) | 5752-5754 | `w += ((nb+nb) + (double)Lnnz + (double)Unnz)*2.0*f*x148`; Lnnz = `*(long*)(LU+0x58 + 8*nb)` (L column start end), Unnz = `LU+0xb0[nb]` (U row start end); f = 1.0 unless `24*(Lnnz+Unnz) > 2e6` (then `1+((24*(L+U)-2e6)*9)/2e6`, 10.0 if >= 4e6) (C 5743-5751) | | param_19!=0, every factor | asm: `((((nb+nb)+L)+U)*2.0*f)*x148 + w`, L/U via 64-bit cvtsi2sd | `factor_nnz()` | asm |
| 0xdcb7b (0/16) | 5934-5936 | `w += (double)(Lnnz + nb)*2.0*x148` | Lnnz as above | param_19!=0; only in the Lt level-DAG branch (C 5920; `nb >= 250`, so degen2 yes, kb2 no) | asm: `(2.0*(double)(int64)(Lnnz+nb))*x148 + w` | level DAG build (`level`, `llev`) | asm |

### C. 09e60b (`9e60b(LU, int *mode, int n, double *p, double *w)`; called from 05b257 L2552 and 1bae9c L1282 with LU = spx+0x470 content, n = m)

f = LU+0x1f8, B = LU+0xb0 (U row start long[]).

| trap PC | C line | formula | guard | FP order | conf |
|---|---|---|---|---|---|
| 0x1327dc (0/8) | 246-250 | `w += (double)(B[n]-B[0])*f` (total U row nnz) | w!=0 and `*mode != 2` | asm: `(double)(int64)d*f + w` | asm |
| 0x1329e6 (0/8) | 246-251 | same amount | w!=0, `*mode == 2`, n>=1 | same | asm |
| 0x1329c1 (0/16) | 345-347 | `w += f*3.0*(double)n` | w!=0, both cases (when `*mode==2 && n<1` the first amount is carried in a register and added here) | asm: `w + ((f*3.0)*(double)n)` | asm |

### D. 016b0a (DSE weight update; 05b257 L2478 passes vecA = `*(spx+0x210)` = alpha_q)

| trap PC | C line | formula | guard | FP order | conf |
|---|---|---|---|---|---|
| 0xbe793 (3/13) | 46-48 | `w += x148*3.0*(double)nA`, nA = `*param_5` (nnz of alpha_q; 0 if loop skipped) | param_8 != NULL and `param_2 != 0.0` (otherwise early return L56 without a store) | asm: `((3.0*x148)*(double)(int)k) + w` | asm |

### E. 01ca57 (`1ca57(spx,&w)`)

| trap PC | C line | formula | guard | FP order | conf |
|---|---|---|---|---|---|
| 0xbf981 (5/12) | 139-141 | `w += (double)(m+n)*((double)(spx+0x338 != 0) + 2.0)*x148` | w!=0 and `*(int*)(spx+0x2cc) == 0` | asm: `(cvt(m+n)*(2.0+flag))*x148 + w` | asm |
| 0xbfed2 (0/4) | 312-316 | same amount | w!=0, `spx+0x2cc != 0`, `*(long*)(spx+0x490) == 0` (`*param_2 = dVar35`) | same | asm for formula; branch from control flow (`jmp bf850`) |

### F. 01bae9c (dual iteration core; param_3 = &w; m=`*(uint*)(spx+100)`, n=`*(uint*)(spx+0x68)`, nA=`**(uint**)(spx+0x210)`, P=`*(spx+0x478)`)

| trap PC | C line | formula | inputs | guard | FP order | Rust hint | conf |
|---|---|---|---|---|---|---|---|
| 0xc0410 (35/621) | 460-462 | `w += (double)nA*3.0*x148` (inlined body of 16b0a) | loop counter = `*puVar19` | param_3!=0; `spx+0x3ec != 0` and `spx+0x310 != 0` (C 421-463) | asm: `((double)(int)k*3.0)*x148 + w` | DSE update | asm |
| 0xc06c7 (5/272) | 777-778 | `w += (double)nA*7.0*x148` | iVar60 = loop count over alpha_q (C 733-776, update of x_B and infeasibility flags, both `local_b0` variants) | param_3!=0, `*(int*)(P+0x40) == 0` | asm: `((double)(int)k*7.0)*x148 + w` | primal update | asm |
| 0xc0c24 (30/349) | 641-642 | `w += (double)nA*7.0*x148` | lVar49 loop count (C 560-640) | param_3!=0, `P+0x40 != 0`, `spx+0x2c8 == 0` (the `!= 0` twin C 727 uses 8.0, not hit) | same | | asm |
| 0xc142b (5/16) | 1590-1592 | `w += ((double)m*4.0 + (double)S + (double)S)*x148`, S = sum over structural j<n with nonbasic-at-nonzero-bound (vstat in -1,-4,-2,-5, bound != 0) of the column length `*(int*)(spx+0x78+4j)` (C 1432-1500) | | param_3!=0; after FTRAN of the bound RHS (`9cf9b`) | asm: `(((double)m*4.0 + cvt(S)) + cvt(S))*x148 + w` | `basic_solution` in lu.rs | asm; S meaning C-only |
| 0xc1fe3 (4/9) | 2468-2470 | `w += ((double)m*5.0 + (double)nInf*4.0)*x148`, nInf = number of infeasibilities (`uVar36`, also stored to `P+0x3c`) | loop C 2268-2410 | param_3!=0, `P+0x40 != 0` | asm: `((double)m*5.0 + (double)nInf*4.0)*x148 + w` | | asm |
| 0xc26ec (0/51) | 881-889 | `w' = ((double)c0*3.0 + ((double)(P+0x44 == 1) + 5.0)*(double)(m - c0))*x148 + w`, stored if `P+0x3c > 0` | c0 = iVar60 = entries flagged `2` in loops C 770-860 / 940-980 | param_3!=0; else partial sum carried to 0xc2761 | asm: `(c0*3.0 + (flag+5.0)*(m-c0))*x148 + w` | | asm |
| 0xc2761 (0/51) | 1008-1010 | `w' += ((double)k+(double)k+(double)s)*x148`; k = iVar74 kept candidates, s = iVar57 scanned | C 874-908 | stored if m>0, else carried to 0xc2842 | asm: `((k+k)+s)*x148` added to carried/loaded w | | asm |
| 0xc2842 (0/51) | 1048 | `w' += ((double)q+(double)q)*x148`, q = iVar54 (compacted entries, C 905-935) | | param_3!=0 | asm: `w + ((q+q)*x148)` | | asm |
| 0xc2ad4 (0/51) | 1259-1260 | `w += (double)len*x148`, len = lVar49 = end of the merge into the `P+0x10` list, expected iVar74+iVar54 (= new `P+0x3c`) | C 1060-1258 | param_3!=0 | asm: `(double)(int)len*x148 + w` | | asm; len inferred from C |
| 0xc391f (1/1) | 2205-2206 | `w += (double)m*x148` (loop count over basis positions of the DSE weight init, bound at 0x10(%rsp)) | C 2150-2205 | param_3!=0 | asm: `(double)(int)cnt*x148 + w` | norm init | asm; m inferred from loop bound |

### G. 01b7106 (called from 1bae9c L2262 when `P+0x40 == 0`)

| trap PC | C line | formula | guard | FP order | conf |
|---|---|---|---|---|---|
| 0xc4329 (1/7) | 146-147 | `w += ((double)m*5.0 + (double)nInf*4.0)*x148` (param_4 = m, nInf = `(int)lVar3` = entries written, = `*param_22` = P+0x3c) | param_24 != NULL | asm: `((double)m*5.0 + (double)nInf*4.0)*x148 + w` | asm |

### H. 0534f7 (`534f7(ctx, p2, &w)`, ctx+0xe0 = spx)

| trap PC | C line | formula | guard | FP order | conf |
|---|---|---|---|---|---|
| 0xe84b9 (1/1) | 1512-1514 | `w += (double)A_nnz*2.0*x148`, A_nnz = `*(long*)(*(long*)(ctx+0xe8)+0x10)` (model matrix nnz: other functions compare it with 0x77359401 and multiply by 1000) | param_3 != NULL, end of the `spx+0x3ec > 1` init branch | asm: `((double)(int64)nnz*2.0)*x148 + w` | asm formula; field meaning inferred |

### Weak points

1. 36ade 0xd85e6: flush amount is the Markowitz accumulator. The increment instructions (asm 0xd6077, 0xd647b, 0xd6ea4, 0xd7178, 0xd78eb) and C expressions (924, 1072, 2563) are mapped, but not every pivot-type branch was traced. Terms are small integers, so only the final `acc*x148 + w` rounding matters.
2. 36ade 0xd4ded: `(P0 + local_100) - local_110` confirmed in asm (stack 0x18, 0x238, 0x228); the human meaning (pool slots consumed plus U elements emitted by `1da0b2`) is read from 1da0b2 control flow only.
3. LU+0x1f8 == x148 not proven (affects 0xf3ca4, 0xf48ae, all 9e60b sites).
4. 0xbfed2 attributed to C 316 by control flow. 0xc2ad4 and 0xc391f amounts are loop-counter finals inferred from loop structure.
5. The 5b257 paths at 0xf5145/0xf5156 (d >= m) were never executed, so no site exists there.
