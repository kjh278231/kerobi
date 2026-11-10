# M3-04c-7 `056973` asm 판독 명세

[작업 기록](primal_push_retry.md) §2의 근거다. 2026-11-10 objdump·점프표·rodata 상수를 읽어 정리했다(subagent 판독). 이 문서만으로 실행 동등성을 주장하지 않는다. 실행으로 확인한 범위는 작업 기록 §6(합성 74호출·자연 16호출 비트 일치)이고, 재진입 패스의 허용오차 결합·NaN 처리는 실행 근거가 없다.


Sources: objdump of libgurobi130.so, jump tables read from .rodata (2a5dc00 word, 2a5dc30 word, 2a5dc60 byte, 2a5dc78 byte), constants 2a5dc90.. read as doubles.
Note: dense pass-1 code lives at f1343..f15b8 (past the nominal function end text); `dc.py --asm` default window cuts off at f1343. objdump at f1343 shows junk until f1348 (f1343 = `mov 0x78(%rsp),%r9`).

## 0. Signature (confirmed from prologue f0120-f01bd and loop uses)
fn(h=xmm0, pivtol=xmm1, I=xmm2, cap=xmm3, S=rdi, m=esi, alpha=rdx, x=rcx, w=r8, marks=r9,
   stack: theta*=0xa0(rsp after prologue), pivot*=0xa8, iscr=0xb0 (int*), dscr=0xb8 (double*), work*=0xc0)  -> int (eax)
Regs: xmm2=h, xmm6=pivtol (mutable), xmm7=I, xmm8=cap, xmm3=0.0, xmm4=4.0 (2a5dd18), xmm5=2.0 (2a5dd30).
alpha: count=*(int*)alpha (r9d); idx=*(int**)(alpha+8) (r12/r13); val=*(double**)(alpha+16) (r12/r14).
count<0 => dense (length m, row=position); count>=0 => sparse (row = idx[k]). x[row], w[row], marks[row] always indexed by ROW; iscr stores POSITION k (sparse) or row i (dense). Return value = position k (or -1).
Local state: first = cl (init 1, f0158); prev = ebx (init -1, f013e); chosen ebp; ncand r11d; acc9 (xmm9, "AB"), acc10 (xmm10, "at-bound").
Constants: scale table 2a5dcc0..: 0.1, 10, 1e4, 1e7, 1e10, 1e13, 1e-5(2a5dcf0), 1e-6(2a5dcf8), 1e-4(2a5dd00), 1e-3(2a5dd08), 1e-2(2a5dd10). 3.0 at 2a5dd20.
Masks: 2a5dc90 = scalar |.| (7fff.. low), 2a5dcb0 = packed |.|, 2a5dca0 = sign bit (xorps => negate).
All FP is scalar SSE2 mulsd/subsd/divsd; NO FMA anywhere. comisd only (never ucomisd).
comisd forms used: `comisd B,A ; ja/jbe/jb` tests A?B : ja <=> A>B (ordered); jbe <=> A<=B OR unordered; jb <=> A<B OR unordered. I write the tests below as the condition that CONTINUES; NaN => the "continue" test is false (falls to skip) in all ja cases, true for jb-skip cases (so `jb skip` => NaN also skips). Summary: all "proceed" conditions below are ordered strict/non-strict comparisons that are false on NaN.

## Loop (f01b2 loop top), do { ... } while(true)
```
loop:                                   // f01b2
  acc10=0.0; acc9=0.0; ncand=0; chosen=-1      // f01b2,f01b6,f01b9,f01bd   (r9d=count NOT reloaded; stays valid)
  // ---------- PASS 1 ----------
  negpt = -pivtol                       // xorps sign mask, f01ef/f1356
  if count>=0 (sparse) { for k in 0..count: a=val[k]; row=idx[k]; ... }       // f0211..f03c4
  else (dense, count<0) { for i in 0..m: a=val[i]; row=i; ... }                // f1389..f1532
```
### 1. Pass 1 per entry (identical for sparse and dense except where marked)
Test order: `if a > pivtol` (comisd %xmm6,a; jbe) POS arm; else `if -pivtol > a` (comisd a,%xmm1) NEG arm; else nothing. (|a|<=pivtol, NaN: nothing.)
mark dispatch via jump tables on (mark-0x41), idx>0x17 => nothing. Only A,B,L,N,U,X do anything ('F', others: nothing).
Let cand(r) mean: `if cap > r { iscr[ncand]=pos; dscr[ncand]=r; ncand++ }` (comisd r,%xmm8; jbe skip; i.e. r < cap strict).
POS arm (a > tol):
- A: r1=(x[row]-w[row])/a; cand(r1); r2=x[row]/a; cand(r2) (x reloaded); acc9 -= a     [sparse f0283-f02d4, dense f13f8-f1524; push order r1 then r2; BOTH paths filtered with cap]
- B: acc9 += a                                                          [f0279 / f151d]
- L,X: acc10 += a                                                       [f026f / f13ee]
- N,U: r=x[row]/a;
    SPARSE: push UNCONDITIONALLY (no cap test)  [f024c-f026a: divsd, store idx/ratio, inc r11d, no comisd]  <-- CONFIRMED (C was right)
    DENSE : cand(r) filtered by cap                                      [f13c3-f13e6]
NEG arm (a < -tol):
- A: acc9 -= a                                                          [f03b6 / f1524]
- B: r1=x[row]/a; cand(r1); wv=w[row]; if I > wv { r2=(x[row]-wv)/a; cand(r2) }; acc9 += a   [f0356-f03af / f14c7-f151d]   (note r1 pushed first)
- L,N: wv=w[row]; if I > wv { r=(x[row]-wv)/a; cand(r) }                  [f0319 / f148d]  (comisd %xmm11,%xmm7: I>wv)
- U,X: acc10 -= a                                                       [f030f / f1483]
Dense only: nnz counter r14d++ for every i with a>tol (inc at f139d before dispatch) or a<-tol (inc at f145d after the -tol>a test), regardless of mark.
(Sparse reloads count after some pushes (`mov (%r10),%r9d`); aliasing-only, ignore.)
Work (pass 1), skipped when work*==NULL (test %rax at f03f3 / f1563; for count==0: f1331 -> f03f8 if work else slope=0 at f133a):
  SPARSE (f03f8-f0423): *work = ((double)ncand*4.0 + (double)n + (double)n) * S.coef148 + *work     where n=count (loop idx r13d)
        evaluation order: t=ncand*4.0; t=t+n; t=t+n; t=t*coef; t=t+*work. (left-assoc: ((c*4 + n) + n))
  DENSE  (f156c-f15a5): t=ncand*4.0; t=t+(double)nnz; t=t+(double)m; t=t*coef; t=t+*work       // m = loop idx r13d (=m)
  coef = *(double*)(S+0x148). (also executed when count==0 / m<=0 with zeros -> adds +0.0*coef.)
slope = acc10 + acc9    (f0427 `addsd %xmm9,%xmm10`; single add, operand order irrelevant)  -- confirmed "at-bound (L/X/U) + AB"

### 2. Branch after pass 1 (f0427-f0441, f0a07-f0a56)
```
slope = acc10+acc9
if ncand==0 {                                   // f042c
   if 0 > slope {                               // f0431 comisd %xmm10,%xmm3; ja -> f0a07
      if first {                                // f0a16 test cl
          theta = (cap < I) ? cap : I           // f0a1f minsd %xmm7,%xmm8: dst=cap; result = cap<I ? cap : I (NaN/equal => I). C's version differs only on NaN.
          pivot = 0.0 (xmm3);  return -1        // f0a24-f0a2d (work NOT touched, theta/pivot written)
      } else {
          n = fn_12dd49(S)                      // f0a32 call 1bb6f0, rdi=S
          if n > 16 (signed; jle taken if n<=16) { *(int*)(S+0x294)=2; *(int*)(S+0x2f8)=-1; return -1 }   // f0a3c-f0a52
          else return prev (ebx)                // f0a54: prev = chosen from previous loop iteration
          // theta/pivot NOT written in this branch (keep values from previous iteration's pass 2)
      }
   }
   // else (slope >= 0 or NaN) fall to f043c
}
// f043c
if 0 > slope  -> QUICKSELECT (f0447)           // comisd %xmm10,%xmm3; jbe f04eb  (slope>=0 or NaN => t=0)
else          -> t = 0.0 ; goto PASS2           // f04eb: xmm0=0
```
Note: the reentry branch can only be reached when first==0 (loop iteration >=2) because first==1 returns earlier.

### 3. Quickselect (slope<0 and ncand>=1)   f0447-f0d2f
```
lo=0; hi=ncand-1;
if hi > 0 {                                      // f0451 test r9d; jle f0d13
  do {                                           // loop head f0477
    p = dscr[lo];                                // pivot VALUE = first element of range; fixed for this partition
    i = lo-1; j = hi+1;
    for(;;) {                                    // f04a6
       do { j--; } while (dscr[j] > p);          // f04a6-f04cc : stop when !(dscr[j] > p) (NaN stops)
       do { i++; } while (p > dscr[i]);          // f04ce-f04dc : continue while p > dscr[i] strict ordered (NaN stops)
       if (i < j) { swap dscr[i]<->dscr[j] AND iscr[i]<->iscr[j]; continue }   // f04de jl f048c; swap f048c-f04a2
       break                                     // -> f0a58 with j (eax)
    }
    // slope update, uses CURRENT slope (xmm10) sign BEFORE update
    if 0 > slope {                               // f0a58 comisd %xmm10,%xmm3; jbe f0b90
        // add |alpha| of LEFT part [lo..j]  (only if lo<=j: f0a63 cmp; jg skip)
        cnt = j-lo+1;  base = &iscr[lo];  absv(e)=|val[iscr[e]]|  (val[position])
        slope = SUM_ADD(slope, base, cnt)
    } else {                                     // f0b90 : slope>=0 or NaN
        // subtract |alpha| of RIGHT part [j+1..hi]   (only if j+1<=hi: f0b97 cmp %r9d,%esi; jg skip)
        cnt = hi-j;  base = &iscr[j+1]
        slope = SUM_SUB(slope, base, cnt)
    }
    if 0 > slope { lo = j+1 } else { hi = j }    // f0cc2 comisd %xmm10,%xmm3 ; cmovbe: !(0>slope) (>=0 or NaN) => hi=j (both r9d and rbx) ; else lo=j+1 (r12)
  } while (lo < hi);                             // f0cdd cmp %r9,%r12; jl f0477  (signed 64)
}
t = dscr[lo];                                    // f0d13
if (I > cap) && (cap <= t)  { *theta = cap; *pivot = 0.0; return -1 }   // f0d19 comisd %xmm8,%xmm7; jbe->pass2 ; f0d24 comisd %xmm8,%xmm0; jb->pass2 (t<cap or NaN => pass2)
   // chosen restored to -1 (f0ce6 mov (%rsp),%ebp); note: work not touched, no restore of caller flags needed
goto PASS2 with t
```
IMPORTANT: slope variable (xmm10) is carried across iterations and only changed by the sums; the sign test at loop head uses it as is. Final slope value is not used after.
SUM_ADD / SUM_SUB (identical structure; the two use addpd vs subpd):
```
SUM_ADD(slope, e[0..cnt)):                        // f0a79 cmp $8,%rdx ; if cnt<8 => skip vector part, k=0
  if cnt >= 8:
     nv = cnt & ~7
     A=[slope,0.0]  B=[0,0]  C=[0,0]  D=[0,0]     // xmm0=[slope,0] (movsd into zeroed reg), xmm10, xmm9, xmm1
     for b in 0, 8, 16, ... < nv:                  // loop f0aa9-f0b3c, 8 elements/iter, ascending
        A.lane0 += |v[e[b+0]]|; A.lane1 += |v[e[b+1]]|      // xmm11=[e0,e1] (movsd/movhpd), addpd xmm11,xmm0
        B.lane0 += |v[e[b+2]]|; B.lane1 += |v[e[b+3]]|      // xmm12 -> xmm10
        C.lane0 += |v[e[b+4]]|; C.lane1 += |v[e[b+5]]|      // xmm13 -> xmm9
        D.lane0 += |v[e[b+6]]|; D.lane1 += |v[e[b+7]]|      // xmm14 -> xmm1
     A = A+B (f0b42 addpd %xmm10,%xmm0);  C = C+D (f0b47 addpd %xmm1,%xmm9);  A = A+C (f0b4c addpd %xmm9,%xmm0)   // per lane: (A+B)+(C+D)
     slope = A.lane0 + A.lane1                       // f0b51 unpckhpd; f0b58 addsd %xmm1,%xmm0 => low+high
     k = nv
  else k = 0
  for k..cnt-1 ascending: slope = slope + |v[e[k]]|   // f0b6d-f0b89 (scalar remainder)
SUM_SUB: identical but accumulators A=[slope,0],B,C,D start the same and do `-= |v|` per lane (subpd xmm11,xmm0 etc., f0c06..f0c6e),
   element-to-lane map identical (A:0,1 B:2,3 C:4,5 D:6,7 within each 8-block); combine addpd A+B, C+D, A+C, then low+high (f0c7c-f0c96);
   remainder: slope = slope - |v[e[k]]|  (f0ca4-f0cc0, subsd) with e base = &iscr[j+1] and k starting at nv.
```
NB C pseudo-code shows ((A0+B0)+C0)+D0 for lane0; ASM is (A0+B0)+(C0+D0). Use asm.
Elements are always |val[position]| with position = iscr[e] (val = alpha->values, f0a75 / f0ba9 `mov 0x10(%r10),%rsi`), NOT |ratio|. For dense, position == row.

### 4. Pass 2 (selection at t)   sparse f04ee-f0795 ; dense f0f79-f120d (identical logic; dense uses row i for everything, chosen=i)
Entry state: xmm0 = t, best (xmm1) = 0.0, chosen ebp = -1, loop idx ebx=0. negpt=-pivtol. (f04eb sets t=0.0 for slope>=0 branch.)
Define res = |num - (t*a)|  computed as: p = t*a (mulsd, t first operand); d = num - p (subsd); res = |d| (andps). Un-fused. `*2.0` = mulsd with xmm5 after abs.
OK(res,scale): `!(h < res*scale)` i.e. proceed iff res*scale <= h (comisd res,h(xmm2); jb skip: skip if h<res or NaN).
Tie rule: strictly better only: `a > best` (POS) / `-a > best` (NEG)  (comisd %xmm1,a ; jbe skip). ebp=k, best=|a|-like value, *theta written immediately at each improvement.
```
for each entry (a=val[k], row):
  if a > pivtol {                                         // f0544 comisd %xmm6,%xmm11; jbe f065b
     mark N or U  [f0d61 / f1261]: num=x[row]; if OK(|num - t*a|*2.0)   && a > best { theta=num/a; best=a; chosen=k }
     mark A       [f05c8 / f103d]: n1 = x[row]-w[row];
                      if OK(|n1 - t*a|*2.0) && a > best { theta=n1/a; best=a; chosen=k }       // first try (n1 divide)
                      else { n2 = x[row];  if OK(|n2 - t*a|*2.0) && a > best { theta=n2/a; best=a; chosen=k } }  // 2nd try runs when EITHER cond of first failed (f0603 jbe also -> f061a)
     mark L or X  [f057d / f0ff2]: num=x[row]; if (|num - t*a| <= h)  /*NO *2*/ && a > best { theta=num/a; best=a; chosen=k }
     else skip (B, F, ...)
  } else if -pivtol > a {                                  // f065b comisd %xmm11,%xmm12; jbe skip
     mark L or N  (m&~2==0x4c) [f0ef9 / f12b1]: wv=w[row]; if I > wv { n=x[row]-wv; if OK(|n - t*a|*2.0) && (-a) > best { theta=n/a; best=-a; chosen=k } }
     mark B       [f06ec / f1164]: n1=x[row];
                      if OK(|n1 - t*a|*2.0) && (-a) > best { theta=n1/a; best=-a; chosen=k }
                      else { wv=w[row]; if I > wv { n2 = x[row]-wv; if OK(|n2 - t*a|*2.0) && (-a) > best { theta=n2/a; best=-a; chosen=k } } }
     mark U or X  [f068f / f1107]: n=x[row]-w[row]; if (|n - t*a| <= h) /*NO *2*/ && (-a) > best { theta=n/a; best=-a; chosen=k }
  }
```
(-a computed by xorps sign flip; `p=t*a` is computed once per arm before the first residual and reused for the second try. In B(neg) first try theta divides by a (negative): numerator not negated.)
Check vs C: matches decompile exactly (incl. *2.0 vs none; `U,X` pos N/U use *2, L/X pos none; U/X neg none; L/N neg *2; B both arms *2; A both arms *2).
Pass-2 work, skipped if work*==NULL:
  SPARSE (f07c6-f07df): *work = ((double)count * 4.0) * coef + *work    (count=ebx at loop end; also when count==0 via f0f6b)
  DENSE  (f1237-f1258): *work = ((double)m * 3.0) * coef + *work         (m<=0 case f1323 -> same with 0)
  order: t=n*K; t=t*coef; t=t+*work.
Then: if chosen == -1 -> FALLBACK (f07e3 cmp $-1 ; je f0e4f).  (Also when work==NULL and count==0/m<=0: f0e4f directly.)

### 5. Fallback (f0e4f-f0ef4)   chosen==-1
```
*theta = I;  best = I
if ncand <= 0 { *pivot = 0.0; return -1 }      // f0e56 jle f0e37  (ncand = r11d, preserved/restored across quickselect; f0ce9)
for e in 0..ncand ascending (2-unrolled pairs 2k,2k+1 then odd tail; visiting order = ascending index):
   if best > dscr[e] (strict; comisd %xmm1,%xmm0; jbe skip) { *theta = dscr[e]; best = dscr[e]; chosen = iscr[e] }
// f0e70-f0eb2 pairs, f0eb8-f0edb tail, n==1 special case f0ee0-f0ef2 (I > dscr[0] => chosen=iscr[0], theta=dscr[0])
if chosen still -1 -> *pivot=0.0 ; return -1   (theta stays I)   // f07ec test ebp; jl f0e37 -> f0e46 movsd %xmm3,(%rdi)
else continue to step 6 with this chosen
```
NOTE: dscr after quickselect is PERMUTED, but fallback is reached only when pass 2 found nothing; ties => FIRST in permuted order wins (strict <). The fallback min uses dscr as permuted by the preceding quickselect (not original order) when quickselect ran.
Note: fallback best starts at I, not cap; sparse N/U pushes may exceed cap.

### 6. Final (f07ec-f09e5)
```
pivotv = |val[chosen]|;  *pivot = pivotv                          // f07f7-f0807 (val[position])
if !first            return chosen                                // f080b test cl; je f09ea
// second abs (no-op) then:
if !(0.1 > pivotv)   return chosen                                // f081a comisd %xmm1,%xmm0(=0.1); jbe -> return : returns when 0.1 <= pivotv or NaN
big = max_k |val[k]|, start big=0.0, k over [0,count) sparse or [0,m) dense   // dense f0db1-f0e32, sparse f083a-f08a6
     pairs (2k,2k+1): big = maxsd(big,|v[2k]|) [maxsd %xmm9,%xmm0: big = big>|v| ? big : |v|];  then if !(big > |v[2k+1]|) big=|v[2k+1]|  [comisd %xmm10,%xmm0; ja keep]
     odd tail: big = maxsd(big,|v[last]|)
     (n<=0 => big=0.0). NaN semantics only differ from plain max when NaN present (NaN propagates/gets replaced by next element depending on step).
scale(big) = 0.1 if 10>big; elif 1e4>big: 0.01; elif 1e7>big: 1e-3; elif 1e10>big: 1e-4; elif 1e13>big: 1e-5; else 1e-6
     (each test is `comisd big,Kconst; jbe next` i.e. K > big strict; big>=K or NaN advances; last (1e13) NaN/>= => 1e-6)   // f08ab-f0939
th = (pivtol*big)*scale                                          // f093d mulsd %xmm0,xmm9(=pivtol); mulsd %xmm11 : (pivtol*big)*scale
if th <= pivotv  (i.e. !(th > pivotv); comisd %xmm1,%xmm9; jbe) return chosen
// tighten:
scale recomputed identically (f0956-f09d5);
big2 = big*scale                                                  // f09d9 mulsd %xmm1,%xmm0
pivtol = pivtol * big2                                            // f09df mulsd %xmm0,%xmm6  => pivtol*(big*scale)  *** NOT (pivtol*big)*scale as C shows ***
first = 0;  prev = chosen (ebx=ebp, f09e3);  goto loop top (f01b2)
```
State reset at loop top: acc9=acc10=0, ncand=0, chosen=-1 (ebp); first=0; prev=old chosen; pivtol updated; theta*/pivot* keep last written values; h,I,cap,x,w,marks,work unchanged; work keeps accumulating across iterations. Note the quickselect permutes dscr/iscr but pass 1 rebuilds them.
At tighten iteration pass 1 uses new pivtol and `-pivtol` recomputed (f01ef).
Return value: eax = chosen (32-bit int) (f09f9).

### 7. C misrepresentations / gotchas
1. Tighten: asm pivtol_new = pivtol * (big*scale); C writes (pivtol*big)*scale. Test threshold itself is (pivtol*big)*scale.
2. Lane combine: asm per lane (A+B)+(C+D); C shows ((A+B)+C)+D. Lane 1 combine identical. Final low+high.
3. cap<I min on first-pass slope<0 no-cand exit: `minsd` semantic = (cap < I ? cap : I); C `(I <= cap) ? I : cap` equal except NaN.
4. Pass 1 sparse N/U positive pushes with NO cap filter (confirmed); dense N/U positive filtered. All other pushes filtered in both. Sparse pass 1 records position k; dense records row i.
5. Dense nnz counter counts every |a|>tol entry regardless of mark (used only for work).
6. NaN: all comisd. unordered sets ZF=PF=CF=1 => ja false, jbe true, jb true. C's `<`-style rewrite equals asm for these given forms except max-scan (maxsd/ja form) and minsd.
7. No FMA, no reassociation: res = |num - (t*a)| (mulsd then subsd), *2.0 after abs where listed. |.| by andps, negation by xorps (no 0-a).
8. theta* is written eagerly at each pass-2 improvement; if later candidates fail, last written stays. On `chosen==-1` fallback theta=I is written first.
9. In reentry (first==0, no candidates, slope<0) theta/pivot are untouched; returns prev chosen unless fn_12dd49(S)>16.
10. Hoare partition uses pivot VALUE dscr[lo] (not mid), do-while scans, break when i>=j; after break j is the split (left [lo..j], right [j+1..hi]); lo<hi loop condition. Reaching index out of range with NaNs is not guarded.
