# M1-02p 부록 B: work 포인터 전파의 정적 상위 집합

[M1-02p](m1_02p_work.md) §6.5의 부록이다. 하위 에이전트가 `06268b` param_2에서 asm 수준 오염 추적(디컴파일 C는 레지스터·스택 인자를 빠뜨려서 쓰지 않음)으로 만든 결과를 그대로 옮겼다(영어). 정적 분석이고 원본 실행으로 확인한 것은 "OBSERVED" 지점(캡처 69곳)뿐이다. 오래된 레지스터를 통한 과대 추정이 있을 수 있다. `sites.tsv`는 `bench/oracle/m1_02p/sites.tsv`, 보조 표(간선·조건·T2 목록)는 `bench/oracle/m1_02p/static/`에 있다.

## Work-counter writer propagation from dual driver 06268b (Gurobi 13.0.3, double engine)

Static superset of code that can write `*work` (offset 0 of the pointer passed as `param_2` of `06268b`). Read-only analysis; Gurobi not run; no repo file modified.

### 0. Method and trust levels

- Method: asm-level forward taint (may-analysis) over `objdump -d` of every function, NOT the decompiled C. Decompiled C hides arguments (e.g. `28c93 -> 9e376` passes the work pointer in rcx but C shows 3 args; `12f4f3` is a tail-jmp stub whose C shows no args; `9cf9b` passes it as stack arg 11 to `2207de`; `2a264(param_1, work)` never uses param_2 in C). Taint covers rdi,rsi,rdx,rcx,r8,r9, stack args (rsp-relative incl. `lea` address tracking), spills, `mov/lea/add/sub`, push/pop, tail jumps, `.cold` fragments. Derived pointers (`lea 8(%r)`) are tracked with offset; only offset-0 stores are "work stores"; other-offset stores are listed separately.
- Param index convention: "asm idx" = integer-class argument position (0..5 = rdi,rsi,rdx,rcx,r8,r9; >=6 = stack slot idx-6). Decompiled C counts `double` by-value params too, so C idx = asm idx + number of leading double params (shown as `C<n>`).
- Self-check: all 69 store sites of sites.tsv (the hardware-watchpoint list) are re-found by this static analysis (69/69, each "pc after store" equals the next-instruction address of a static offset-0 store with tainted base). So the analysis is complete for the observed behaviour and the unobserved stores below are by construction extra.
- Over-approximation: flow-insensitive-ish joins; a stale tainted register at a call can create a spurious edge (callee must actually use that parameter to matter). Spurious edges seen: infrastructure (log/alloc/time/mutex helpers, see section 5). Argument slots above callee C param count + 2 are ignored (outgoing-area vs local spill ambiguity).
- OBSERVED = static store's "next instruction address" is in sites.tsv. File offset == virtual address in this binary.
- C line numbers refer to `../re/decomp/c/<off>_PRIVATE...<name>.c`. C-line <-> asm-store pairing is exact only where the function has 1 store or where all asm stores share one status. For MIXED functions the asm addresses are authoritative and C lines are listed unpaired (a constant-literal match was tried and is ambiguous).

### 1. Tiers

- T1 = reachable from `06268b` without entering a "gateway": 72 functions (about half are store-free helpers: log/alloc/time/mutex/limit-check trampolines).
- TQ = reachable only through `12f4f3 -> 1eca78` (the quad/x87 init). `12f4f3` is `if (spx->+0x28 == 0) 534f7 else 1eca78`, and `12ed7c` (caller of 06268b) already requires `spx+0x28 == 0`, so TQ is statically reachable but contradicted by the caller's guard (dead for a double dual run). 14 extra functions.
- T2 / gateways = functions below `534f7 -> 1e7531` (C:551, unguarded in C), `1eca78 -> 1e7531(quad)` and anything reaching `cda5b` (`1e7531` C:1741 calls it with the work pointer; `12ed7c`, `12e738`, `12e6ec`, `12e790` are the dispatch stubs it uses, i.e. a nested full simplex re-solve). `1e7531` forwards the pointer (its param 18, asm 16) to LP presolve `390989` and then to essentially the whole solver (829 further functions). Not hand-verified; machine list in `bench/oracle/m1_02p/static/propagation_t2.tsv`. None of T2 executed in kb2/degen2 (no sites there).

### 2. Tree (first-discovery path; asm int param idx carrying work; call-site asm address)

Only the first discovery parent is shown; the full edge list is in `bench/oracle/m1_02p/static/propagation_edges.tsv`. `[N store(s): a obs / b NOT]` = offset-0 stores of that function reached through that parameter.

```
- 6268b @0xf6a00 param(asm int idx)=1
  - 1ca57 @0xbf5b0 param(asm int idx)=1 [5 store(s): 2 obs / 3 NOT]  <- call @0xf6d0a
    - 1bae9c @0xc0050 param(asm int idx)=2 [17 store(s): 10 obs / 7 NOT]  <- call @0xbffe9
      - 1b7106 @0xc3f80 param(asm int idx)=21 [1 store(s): 1 obs / 0 NOT]  <- call @0xc1a4e
      - a7aa8f @0x88a100 param(asm int idx)=2 [INDIRECT-CALL w/ tainted args]  <- call @0xc0c8e
    - 118d6a @0x1aa960 param(asm int idx)=3 [3 store(s): 0 obs / 3 NOT]  <- call @0xbfe05
  - 28c93 @0xc4fe0 param(asm int idx)=2 [1 store(s): 1 obs / 0 NOT]  <- call @0xf8b26
    - 1c35c5 @0xc74a0 param(asm int idx)=5 [10 store(s): 7 obs / 3 NOT]  <- call @0xc54b0
      - 1df51 @0xcd250 param(asm int idx)=5  <- call @0xcaadc
      - a7aa8f @0x88a100 param(asm int idx)=5 [INDIRECT-CALL w/ tainted args]  <- call @0xca6c3
      - a7b4d6 @0x88a870 param(asm int idx)=5  <- call @0xcad84
      - a7b54f @0x88a8d0 param(asm int idx)=5  <- call @0xcacb4
    - 1c090a @0xcc6a0 param(asm int idx)=2 [1 store(s): 0 obs / 1 NOT]  <- call @0xc54ce
      - a7b4d6 @0x88a870 param(asm int idx)=2  <- call @0xcc9e3
    - 2896a3 @0xccbc0 param(asm int idx)=1 [3 store(s): 3 obs / 0 NOT]  <- call @0xc50b6
      - 15d2d @0xbdc10 param(asm int idx)=24 [4 store(s): 4 obs / 0 NOT]  <- call @0xccfca
      - 15d2d @0xbdc10 param(asm int idx)=27  <- call @0xccfca
    - 2896a3 @0xccbc0 param(asm int idx)=2  <- call @0xc50b6
    - 1c8477 @0xcd720 param(asm int idx)=3 [3 store(s): 3 obs / 0 NOT]  <- call @0xc5d5e
    - 9e376 @0x131710 param(asm int idx)=3 [1 store(s): 1 obs / 0 NOT]  <- call @0xc51d8
    - a4109 @0x134d90 param(asm int idx)=8 [1 store(s): 1 obs / 0 NOT]  <- call @0xc537c
  - 2a264 @0xce3c0 param(asm int idx)=1  <- call @0xf9064
    - a4109 @0x134d90 param(asm int idx)=10  <- call @0xce6d5
    - 1876a1 @0x1f4d20 param(asm int idx)=3 [3 store(s): 0 obs / 3 NOT]  <- call @0xce84a
  - 48d5f @0xe45c0 param(asm int idx)=1 [1 store(s): 1 obs / 0 NOT]  <- call @0xf852a
    - 9e288 @0x12de10 param(asm int idx)=3 [1 store(s): 1 obs / 0 NOT]  <- call @0xe53a2
  - 5b257 @0xf15c0 param(asm int idx)=1 [22 store(s): 21 obs / 1 NOT]  <- call @0xf6c82
    - 16b0a @0xbe6b0 param(asm int idx)=5 [1 store(s): 1 obs / 0 NOT]  <- call @0xf470f
    - 17248 @0xbe7b0 param(asm int idx)=1 [2 store(s): 0 obs / 2 NOT]  <- call @0xf52a7
    - 36ade @0xd4190 param(asm int idx)=16 [9 store(s): 5 obs / 4 NOT]  <- call @0xf34ed
      - a83b2f @0x88eed0 param(asm int idx)=2  <- call @0xd5e56
    - 9e47c @0x131760 param(asm int idx)=3 [1 store(s): 0 obs / 1 NOT]  <- call @0xf1967
    - 9e60b @0x132460 param(asm int idx)=4 [3 store(s): 3 obs / 0 NOT]  <- call @0xf48f8
    - ade4c2 @0x8eabf0 param(asm int idx)=1  <- call @0xf51e0
  - 8b129 @0x11f430 param(asm int idx)=1  <- call @0xf6e52
  - 8b473 @0x11f620 param(asm int idx)=1 [1 store(s): 1 obs / 0 NOT]  <- call @0xf90b3
    - 9cf9b @0x1205d0 param(asm int idx)=4  <- call @0x120069
      - 2207de @0x120740 param(asm int idx)=10 [1 store(s): 1 obs / 0 NOT]  <- call @0x12066e
    - 9d112 @0x125f50 param(asm int idx)=7 [1 store(s): 1 obs / 0 NOT]  <- call @0x1201fb
  - a002d @0x133cc0 param(asm int idx)=1  <- call @0xf6adc
  - d07d1 @0x164c60 param(asm int idx)=3  <- call @0xf6dfb
    - a7a18c @0x889be0 param(asm int idx)=3 [INDIRECT-CALL w/ tainted args]  <- call @0x164dbf
    - ac55a2 @0x8d1630 param(asm int idx)=3  <- call @0x164cc9
  - 12f4f3 @0x1bc090 param(asm int idx)=2  <- call @0xf6b4b
    - 534f7 @0xe7c50 param(asm int idx)=2 [2 store(s): 1 obs / 1 NOT]  <- call @0x1bc09d
      - 1ca57 @0xbf5b0 param(asm int idx)=2  <- call @0xea72a
      - 81cc2 @0x10d300 param(asm int idx)=2  <- call @0xe7d4f
        - ac661f @0x8d2460 param(asm int idx)=2 [INDIRECT-CALL w/ tainted args]  <- call @0x10d51d
      - ac661f @0x8d2460 param(asm int idx)=3 [INDIRECT-CALL w/ tainted args]  <- call @0xe9aa5
    - 1eca78 @0x23a9b0 param(asm int idx)=2 [2 store(s): 0 obs / 2 NOT]  <- call @0x1bc0a2
      - 1be884 @0x217600 param(asm int idx)=1 [5 store(s): 0 obs / 5 NOT]  <- call @0x23d7f2
        - 1bae9c @0x218010 param(asm int idx)=2 [17 store(s): 0 obs / 17 NOT]  <- call @0x217f8f
          - 1b7106 @0x21c300 param(asm int idx)=21 [1 store(s): 0 obs / 1 NOT]  <- call @0x219c81
      - 1be884 @0x217600 param(asm int idx)=2  <- call @0x23d7f2
      - 1f4140 @0x244cd0 param(asm int idx)=1 [21 store(s): 0 obs / 21 NOT]  <- call @0x23d86f
        - 1b864b @0x2165c0 param(asm int idx)=5 [1 store(s): 0 obs / 1 NOT]  <- call @0x247d2c
        - 1b8cf4 @0x216700 param(asm int idx)=1 [2 store(s): 0 obs / 2 NOT]  <- call @0x2488e5
        - 1cf516 @0x226ed0 param(asm int idx)=16 [9 store(s): 0 obs / 9 NOT]  <- call @0x246cb3
          - a83b2f @0x88eed0 param(asm int idx)=3  <- call @0x228cf5
            - ac5a29 @0x8d17d0 param(asm int idx)=3  <- call @0x88f0aa
        - 222baa @0x270900 param(asm int idx)=3 [1 store(s): 0 obs / 1 NOT]  <- call @0x245086
        - 222d31 @0x271750 param(asm int idx)=4 [3 store(s): 0 obs / 3 NOT]  <- call @0x247f1e
      - 22170d @0x25df40 param(asm int idx)=4  <- call @0x23c4a8
        - 2207de @0x25e0e0 param(asm int idx)=10 [1 store(s): 0 obs / 1 NOT]  <- call @0x25dffe
      - 2229b6 @0x26ce00 param(asm int idx)=3 [1 store(s): 0 obs / 1 NOT]  <- call @0x23c058
  - 193d1d @0x1fc6c0 param(asm int idx)=1  <- call @0xf6b25
  - 1943e5 @0x1fca70 param(asm int idx)=1  <- call @0xf93bf
  - 9a91f3 @0x7cd7f0 param(asm int idx)=5 [STRUCT-CARRIER]  <- call @0xf92df
    - 6de7ca @0x5a8110 param(asm int idx)=5 [13 store(s): 0 obs / 13 NOT]  <- call @0x7cdd5b
      - 5e6f57 @0x509b60 param(asm int idx)=5  <- call @0x5a8185
    - 895d22 @0x6e1110 param(asm int idx)=2 [1 store(s): 0 obs / 1 NOT]  <- call @0x7cdda2
      - 5e8239 @0x50abc0 param(asm int idx)=2  <- call @0x6e121c
      - 897f79 @0x6d73b0 param(asm int idx)=1 [6 store(s): 0 obs / 6 NOT]  <- call @0x6e1498
        - 8a1cd9 @0x6d89a0 param(asm int idx)=2 [4 store(s): 0 obs / 4 NOT]  <- call @0x6d767f
        - 8a2ecb @0x6d9250 param(asm int idx)=2 [3 store(s): 0 obs / 3 NOT]  <- call @0x6d7639
        - 8a37dc @0x6d98a0 param(asm int idx)=2 [2 store(s): 0 obs / 2 NOT]  <- call @0x6d76bd
          - 618398 @0x52a7d0 param(asm int idx)=18 [4 store(s): 0 obs / 4 NOT]  <- call @0x6d9d11
            - a7e18d @0x88ca60 param(asm int idx)=10 [2 store(s): 0 obs / 2 NOT]  <- call @0x52aa4a
        - ac426e @0x8d0da0 param(asm int idx)=1 [1 store(s): 0 obs / 1 NOT]  <- call @0x6d7e71
        - ade736 @0x8eac90 param(asm int idx)=1  <- call @0x6d73f8
        - adf00e @0x8eb1e0 param(asm int idx)=1  <- call @0x6d7452
        - adf2be @0x8eb340 param(asm int idx)=1  <- call @0x6d752b
        - b406ca @0x947870 param(asm int idx)=8 [INDIRECT-CALL w/ tainted args]  <- call @0x6d77e7
      - 8986cb @0x6d7f60 param(asm int idx)=5 [6 store(s): 0 obs / 6 NOT]  <- call @0x6e17ec
        - 89d32b @0x6e4500 param(asm int idx)=4 [4 store(s): 0 obs / 4 NOT]  <- call @0x6d815b
          - ac3f7f @0x8d0ca0 param(asm int idx)=4  <- call @0x6e49ee
      - 8a9773 @0x6d9f20 param(asm int idx)=2 [3 store(s): 0 obs / 3 NOT]  <- call @0x6e11d9
        - ac3c8b @0x8d0b40 param(asm int idx)=4 [1 store(s): 0 obs / 1 NOT]  <- call @0x6da811
          - ac64e6 @0x8d1bb0 param(asm int idx)=4 [INDIRECT-CALL w/ tainted args]  <- call @0x8d0b6b
            - ac7f9a @0x8d3570 param(asm int idx)=4  <- call @0x8d1c39
      - 8a55b2 @0x6daa60 param(asm int idx)=3 [4 store(s): 0 obs / 4 NOT]  <- call @0x6e12d5
        - a7d68e @0x88c340 param(asm int idx)=5  <- call @0x6db1cd
        - a7d7cf @0x88c380 param(asm int idx)=5  <- call @0x6db443
        - a7d9ff @0x88c420 param(asm int idx)=5  <- call @0x6db4ac
    - ac7f9a @0x8d3570 param(asm int idx)=3  <- call @0x7cd978
    - ac7f9a @0x8d3570 param(asm int idx)=5  <- call @0x7ce533
    - ae1983 @0x8ecb00 param(asm int idx)=3  <- call @0x7cd88b
    - ae1983 @0x8ecb00 param(asm int idx)=5  <- call @0x7cd88b
  - a83b2f @0x88eed0 param(asm int idx)=1  <- call @0xf6e36
    - ac55a2 @0x8d1630 param(asm int idx)=1  <- call @0x88f073

GATEWAYS (not expanded here, see section 4):
  534f7 -> 1e7531 @0xe82ef  (C:551, no enclosing guard in C), 1e7531 asm param 16 / C 17, receives the work pointer as its last stack arg
  1eca78 -> 1e7531(quad @0x23db90) @0x23b004
  1e7531 -> cda5b @0x153cc0 (C:1741), 1e7531 -> 390989 (C:1521/1483)
```

### 3. Store table (offset-0 stores of the work counter)

Columns: function | asm idx / C idx | C line(s) | asm store instruction address(es) | status | guard.  Status per asm address where mixed.

| tier | function | param asm/C | C lines | asm store addr (observed) | asm store addr (NOT observed) | status | guard condition |
|---|---|---|---|---|---|---|---|
| T1 | 15d2d @0xbdc10 | 24 / C26 | 108, 163, 166, 485 | 0xbe035 0xbe09d 0xbe17a 0xbe349 | - | OBSERVED | - |
| T1 | 16b0a @0xbe6b0 | 5 / C7 | 51 | 0xbe78e | - | OBSERVED | - |
| T1 | 17248 @0xbe7b0 | 1 / C1 | 138, 293 | - | 0xbea13 0xbec94 | NOT OBSERVED | store guard (first line): if ((dVar28 / (double)iVar15 <= 5.0) && (*(int *)(param_1 + 0x414) < 10)) {; caller 5b257 C:L2472 [if (*(int *)(param_1 + 0x3ec) == 2) {; if (-1 < iVar35) {; if ((*(int *)(param_1 + 0x290) == 0) && (-1 < *(int *)(param_1 + 0x2fc))) {] |
| T1 | 1ca57 @0xbf5b0 | 1 / C1 | 140, 316, 321, 360, 393 | 0xbf97b 0xbfecc | 0xbf7c2 0xbf84a 0xbfbe1 | MIXED 2 obs / 3 NOT | per-store guard not resolved (C<->asm pairing ambiguous), C-line guards in `bench/oracle/m1_02p/static/propagation_guards.tsv` |
| T1 | 1bae9c @0xc0050 | 2 / C2 | 353, 415, 461, 642, 727, 778, 1037, 1044, 1048, 1260, 1591, 1714, 1969, 2206, 2253, 2469, 2506 | 0xc040b 0xc06c2 0xc0c1f 0xc1426 0xc1fde 0xc26e7 0xc275c 0xc283d 0xc2acf 0xc391a | 0xc09b8 0xc1779 0xc2117 0xc3232 0xc34f5 0xc362a 0xc3d35 | MIXED 10 obs / 7 NOT | per-store guard not resolved (C<->asm pairing ambiguous), C-line guards in `bench/oracle/m1_02p/static/propagation_guards.tsv` |
| T1 | 1b7106 @0xc3f80 | 21 / C23 | 146 | 0xc4324 | - | OBSERVED | - |
| T1 | 28c93 @0xc4fe0 | 2 / C2 | 294 | 0xc5614 | - | OBSERVED | - |
| T1 | 1c35c5 @0xc74a0 | 5 / C5 | 438, 689, 752, 873, 1026, 1472, 1564, 1617, 1838, 2140 | 0xc7bc8 0xc7c4a 0xc82e9 0xc8b62 0xc92b8 0xc9431 0xcbca9 | 0xc856d 0xc8e73 0xca68a | MIXED 7 obs / 3 NOT | per-store guard not resolved (C<->asm pairing ambiguous), C-line guards in `bench/oracle/m1_02p/static/propagation_guards.tsv` |
| T1 | 1c090a @0xcc6a0 | 2 / C2 | 197 | - | 0xccb00 | NOT OBSERVED | store guard (first line): if (param_3 != (double *)0x0) {; caller 28c93 C:L225 [else {] |
| T1 | 2896a3 @0xccbc0 | 1 / C1 | 252, 259, 263 | 0xcce6d 0xcd04d 0xcd18c | - | OBSERVED | - |
| T1 | 1c8477 @0xcd720 | 3 / C3 | 158, 256, 470 | 0xcdbb1 0xce1eb 0xce312 | - | OBSERVED | - |
| T1 | 36ade @0xd4190 | 16 / C18 | 538, 762, 819, 1158, 1533, 2817, 3401, 5753, 5934 | 0xd4de9 0xd52fc 0xd85e2 0xdc7ae 0xdcb77 | 0xd5489 0xd5e35 0xd896f 0xd93db | MIXED 5 obs / 4 NOT | per-store guard not resolved (C<->asm pairing ambiguous), C-line guards in `bench/oracle/m1_02p/static/propagation_guards.tsv` |
| T1 | 48d5f @0xe45c0 | 1 / C1 | 2188 | 0xe6ffc | - | OBSERVED | - |
| T1 | 534f7 @0xe7c50 | 2 / C2 | 1373, 1513 | 0xe84b5 | 0xea700 | MIXED 1 obs / 1 NOT | per-store guard not resolved (C<->asm pairing ambiguous), C-line guards in `bench/oracle/m1_02p/static/propagation_guards.tsv` |
| T1 | 5b257 @0xf15c0 | 1 / C1 | 418, 492, 551, 560, 596, 794, 798, 884, 987, 1006, 1040, 1065, 1176, 1208, 1317, 2029, 2546, 2615, 2633, 2669, 2674, 2729 | 0xf1be6 0xf1cfd 0xf1f32 0xf1f7d 0xf203f 0xf2075 0xf251c 0xf2964 0xf2a4c 0xf2e37 0xf3c9f 0xf3d7e 0xf3ea7 0xf3f6c 0xf406e 0xf461a 0xf48a9 0xf4f96 0xf4ff8 0xf5075 0xf51aa | 0xf5156 | MIXED 21 obs / 1 NOT | per-store guard not resolved (C<->asm pairing ambiguous), C-line guards in `bench/oracle/m1_02p/static/propagation_guards.tsv` |
| T1 | 8b473 @0x11f620 | 1 / C1 | 485 | 0x12010f | - | OBSERVED | - |
| T1 | 2207de @0x120740 | 10 / C10 | 1047 | 0x1219d2 | - | OBSERVED | - |
| T1 | 9d112 @0x125f50 | 7 / C7 | 2437 | 0x12966d | - | OBSERVED | - |
| T1 | 9e288 @0x12de10 | 3 / C3 | 15 | 0x12de4c | - | OBSERVED | - |
| T1 | 9e376 @0x131710 | 3 / C3 | 9 | 0x131749 | - | OBSERVED | - |
| T1 | 9e47c @0x131760 | 3 / C3 | 441 | - | 0x1322e9 | NOT OBSERVED | store guard (first line): if (param_4 != (double *)0x0) {; caller 5b257 C:L316 [if (iVar10 != *(int *)(lVar20 + 0x1d8)) {; if (iVar35 != iVar52) {; if (*(int *)(param_1 + 0x290) == 0) {] |
| T1 | 9e60b @0x132460 | 4 / C4 | 252, 255, 347 | 0x1327d7 0x1329bc 0x1329e1 | - | OBSERVED | - |
| T1 | a4109 @0x134d90 | 8 / C8 | 1000 | 0x135e8e | - | OBSERVED | - |
| T1 | 118d6a @0x1aa960 | 3 / C3 | 187, 325, 474 | - | 0x1aab4e 0x1aaca0 0x1aae1d | NOT OBSERVED | store guard (first line): if (param_4 != (double *)0x0) { > if (iVar20 < 0) { > if ((((lVar3 != 0) && (lVar4 != 0)) && (lVar5 != 0)) && (lVar7 != 0)) {; caller 1ca57 C:L483 [if ((*(int *)(param_1 + 0x514) != 0) && (*(int *)(param_1 + 0x28) == 0)) {; else {] |
| T1 | 1876a1 @0x1f4d20 | 3 / C3 | 375, 2167, 2246 | - | 0x1f54cd 0x1f837a 0x1f8503 | NOT OBSERVED | store guard (first line): if (param_4 != (double *)0x0) {; caller 2a264 C:L223 [] |
| T1 | 618398 @0x52a7d0 | 18 / CNone |  | - | 0x52b263 0x52b4fa 0x52b6d1 0x52b7d9 | NOT OBSERVED | caller 8a37dc C:L172 [else {; if ((bVar6) &&] |
| T1 | 6de7ca @0x5a8110 | 5 / C5 | 366, 477, 492, 589, 598, 602, 690, 703, 770, 848, 853, 861, 932 | - | 0x5a8705 0x5a8b67 0x5a8ee3 0x5a8f53 0x5a90c2 0x5a91c9 0x5a92e5 0x5a93ec 0x5a93f7 0x5a9561 0x5a9763 0x5a9ab1 0x5a9e03 | NOT OBSERVED | store guard (first line): if (param_6 != (double *)0x0) { > if (!bVar8) { > if ((((*(int *)((long)plVar5 + 0x3c) == 0) && (*(int *)((long)plVar5 + 0x2c) != 0)) &&; caller 9a91f3 C:L371 [if ((((local_450 == 0) && (*(long *)(local_430 + 0x1a) != 0)) && (0 < (int)local_430[0x1c])); if ((local_430[1] >> (param_3 & 0x1f) & 1) == 0) {; else {] |
| T1 | 897f79 @0x6d73b0 | 1 / C1 | 212, 244, 269, 274, 277, 392 | - | 0x6d79be 0x6d7a2d 0x6d7a87 0x6d7b86 0x6d7b93 0x6d7dbd | NOT OBSERVED | store guard (first line): if (param_2 != (double *)0x0) { > if (param_2 == (double *)0x0) { > else {; caller 895d22 C:L157 [if (0 < *(int *)((long)param_2 + 0x14)) {] |
| T1 | 8986cb @0x6d7f60 | 5 / C6 | 81, 97, 114, 220, 248, 327 | - | 0x6d8055 0x6d80d3 0x6d8277 0x6d84b5 0x6d8712 0x6d886f | NOT OBSERVED | store guard (first line): if (param_7 != (double *)0x0) { > if (cVar8 == '>') {; caller 895d22 C:L136 [if (0.0 < dVar9) {; if (*(double *)(lVar13 + lVar18 * 8) - *(double *)(lVar12 + lVar18 * 8) == 1.0) {; do {],L140 [else {; if (*(double *)(lVar13 + lVar18 * 8) - *(double *)(lVar12 + lVar18 * 8) == 1.0) {; do {] |
| T1 | 8a1cd9 @0x6d89a0 | 2 / C3 | 71, 168, 242, 276 | - | 0x6d8b19 0x6d8dd4 0x6d8f14 0x6d8fc2 | NOT OBSERVED | store guard (first line): if (((0.0 < param_1) && (*(int *)(*(long *)(param_2 + 0xc0) + lVar21) == 1)) //; caller 897f79 C:L121 [else {; if (iVar7 < 0) {; if (iVar7 < local_70) {],L125 [else {; if (iVar7 < local_70) {; if (iVar7 < local_60) {] |
| T1 | 8a2ecb @0x6d9250 | 2 / C3 | 64, 133, 156 | - | 0x6d9352 0x6d95a1 0x6d963b | NOT OBSERVED | store guard (first line): if (param_4 != (double *)0x0) { > if (((param_1 <= 0.0) // (*(int *)(param_2[0x18] + lVar12 * 4) != 1)) &&; caller 897f79 C:L116 [else {; if (iVar7 < local_80) {; if (iVar7 < 0) {],L129 [else {; if (iVar7 < local_60) {; if (iVar7 < local_58) {] |
| T1 | 8a37dc @0x6d98a0 | 2 / C3 | 120, 209 | - | 0x6d9b14 0x6d9ec8 | NOT OBSERVED | store guard (first line): if (param_4 != (double *)0x0) {; caller 897f79 C:L112 [if (iVar7 < local_88) {; if (iVar7 < local_80) {; if (iVar7 < 0) {],L133 [else {; if (iVar7 < local_58) {; if (iVar7 < local_48) {] |
| T1 | 8a9773 @0x6d9f20 | 2 / C2 | 362, 366, 380 | - | 0x6da650 0x6da71d 0x6da7d5 | NOT OBSERVED | store guard (first line): if (0 < iVar12) { > else {; caller 895d22 C:L65 [if ((*(long *)(lVar22 + 0x3240) != 0) &&; if (lVar22 != 0) {] |
| T1 | 8a55b2 @0x6daa60 | 3 / C3 | 214, 341, 392, 516 | - | 0x6daf37 0x6db3ac 0x6db691 0x6dbb0f | NOT OBSERVED | store guard (first line): if (param_4 != (double *)0x0) { > do { > if ((long)uVar31 < lVar20) {; caller 895d22 C:L94 [] |
| T1 | 895d22 @0x6e1110 | 2 / C2 | 153 | - | 0x6e148d | NOT OBSERVED | store guard (first line): if (param_3 != (double *)0x0) { > if ((-1e+100 < dVar28) && (dVar28 = (double)param_2[0x24], dVar28 < 1e+100)) { > if (param_2[0xe] != 0) {; caller 9a91f3 C:L375 [if ((((local_450 == 0) && (*(long *)(local_430 + 0x1a) != 0)) && (0 < (int)local_430[0x1c])); if ((local_430[1] >> (param_3 & 0x1f) & 1) == 0) {; else {] |
| T1 | 89d32b @0x6e4500 | 4 / C5 | 590, 595, 762, 810 | - | 0x6e4c73 0x6e50f3 0x6e57c3 0x6e5a4b | NOT OBSERVED | store guard (first line): if (lVar36 != 0) { > else {; caller 8986cb C:L172 [] |
| T1 | a7e18d @0x88ca60 | 10 / C11 | 83, 188 | - | 0x88cbb1 0x88cde2 | NOT OBSERVED | store guard (first line): else {; caller 618398 C:L111 [if ((0.0 <= local_48) // (param_5 == 1e+101)) {; if ((local_48 <= 0.0) // (param_4 == 1e+101)) {] |
| T1 | ac3c8b @0x8d0b40 | 4 / C4 | 41 | - | 0x8d0bec | NOT OBSERVED | store guard (first line): if (param_5 != (double *)0x0) { > else {; caller 8a9773 C:L384 [],L390 [if (param_2[0x32] != 0) {; if ((int)uVar17 == 0) {] |
| T1 | ac426e @0x8d0da0 | 1 / C1 | 26 | - | 0x8d0dee | NOT OBSERVED | store guard (first line): if (param_2 != (double *)0x0) {; caller 897f79 C:L430 [] |
| TQ | 1b864b @0x2165c0 | 5 / C6 | 64 | - | 0x2166cc | NOT OBSERVED | store guard (first line): if (param_7 != (double *)0x0) {; caller 1f4140 C:L2418 [else {; if (-1 < iVar10) {; if ((*(int *)((long)param_1 + 0x290) == 0) && (-1 < *(int *)((long)param_1 + 0x2fc))) {] |
| TQ | 1b8cf4 @0x216700 | 1 / C1 | 136, 273 | - | 0x216978 0x216b91 | NOT OBSERVED | store guard (first line): if ((dVar6 / (double)iVar12 <= 5.0) && (*(int *)(param_1 + 0x414) < 10)) {; caller 1f4140 C:L2409 [if (*(int *)((long)param_1 + 0x3ec) == 2) {; if (-1 < iVar10) {; if ((*(int *)((long)param_1 + 0x290) == 0) && (-1 < *(int *)((long)param_1 + 0x2fc))) {] |
| TQ | 1be884 @0x217600 | 1 / C1 | 125, 204, 210, 214, 295 | - | 0x217878 0x217910 0x217a70 0x217ba9 0x217e93 | NOT OBSERVED | store guard (first line): else { > if (iVar7 == 0) {; caller 1eca78 C:L659 [else if (local_68 < 2) {; if (((*(long *)(*(long *)(lVar12 + 0x4b8) + 0xf0) == 0) && (*(int *)(lVar12 + 0xf4) < 0)) &&; if (*(int *)(lVar44 + 0x2524) != 0) {] |
| TQ | 1bae9c @0x218010 | 2 / C2 | 246, 382, 437, 623, 717, 780, 1055, 1062, 1066, 1285, 1585, 1695, 1995, 2133, 2161, 2375, 2408 | - | 0x21841d 0x2187cc 0x218b3c 0x218df8 0x219595 0x2198d5 0x21a310 0x21a470 0x21ab09 0x21ab81 0x21ac80 0x21af51 0x21b756 0x21b9ac 0x21ba96 0x21bcea 0x21c0b1 | NOT OBSERVED | store guard (first line): if (param_3 != (double *)0x0) { > if ((dVar9 / (double)iVar57 <= 5.0) && (*(int *)(param_1 + 0x414) < 10)) { > if (dVar8 != 0.0) {; caller 1be884 C:L42 [],L48 [if (((*(int *)(lVar13 + 0x48) < 1) && (*(int *)(lVar13 + 0x44) == 0)) &&] |
| TQ | 1b7106 @0x21c300 | 21 / C23 | 166 | - | 0x21c6e3 | NOT OBSERVED | store guard (first line): ; caller 1bae9c C:L2168 [if (*(int *)(lVar44 + 0x48) == 0) {; if ((lVar74 <= (longdouble)local_198 * lVar68) //; if (lVar74 <= (longdouble)(local_198 * local_198)) {] |
| TQ | 1cf516 @0x226ed0 | 16 / C18 | 576, 982, 1012, 1362, 2912, 3183, 3716, 5394, 5581 | - | 0x227aaf 0x22823f 0x228399 0x228cd4 0x22b846 0x22c31c 0x22ca3c 0x230ef6 0x231277 | NOT OBSERVED | store guard (first line): else { > if (iVar49 == 0) {; caller 1f4140 C:L1357 [if ((*(int *)(lVar21 + 0x20c) == 0) //; else {] |
| TQ | 1eca78 @0x23a9b0 | 2 / C2 | 656, 1586 | - | 0x23b1c4 0x23d7c8 | NOT OBSERVED | store guard (first line): if (param_3 != (double *)0x0) { > else if (local_68 < 2) { > if (((*(long *)(*(long *)(lVar12 + 0x4b8) + 0xf0) == 0) && (*(int *)(lVar12 + 0xf4) < 0)) &&; caller 12f4f3 C:L10 [] |
| TQ | 1f4140 @0x244cd0 | 1 / C1 | 298, 379, 448, 457, 476, 679, 766, 876, 892, 930, 956, 1068, 1100, 1220, 1950, 2507, 2576, 2594, 2630, 2635, 2691 | - | 0x2451de 0x245362 0x2455be 0x2455fd 0x24569a 0x245bdc 0x24608b 0x2461ac 0x24661e 0x247453 0x247545 0x247683 0x247753 0x24786a 0x247c28 0x247ed0 0x2485e8 0x24864a 0x2486c5 0x24879c 0x2487ed | NOT OBSERVED | store guard (first line): if (param_2 != (double *)0x0) { > if (0 < (int)uVar11) { > if (iVar10 != local_d0) {; caller 1eca78 C:L620 [if (*(int *)(lVar12 + 0x294) == 1) {; if (((*(long *)(*(long *)(lVar12 + 0x4b8) + 0xf0) == 0) && (*(int *)(lVar12 + 0xf4) < 0)) &&; if (*(int *)(lVar44 + 0x2524) != 0) {],L622 [while (local_68 == 1) {; if (*(int *)(lVar12 + 0x294) == 1) {; if (((*(long *)(*(long *)(lVar12 + 0x4b8) + 0xf0) == 0) && (*(int *)(lVar12 + 0xf4) < 0)) &&],L961 [if ((*(int *)(lVar12 + 0x294) != 0) // (-1 < *(int *)(lVar12 + 0x2f8))) {; else {; do {] |
| TQ | 2207de @0x25e0e0 | 10 / C10 | 1035 | - | 0x25f614 | NOT OBSERVED | store guard (first line): if (param_11 != (double *)0x0) {; caller 22170d C:L46 [] |
| TQ | 2229b6 @0x26ce00 | 3 / C3 | 15 | - | 0x26ce3c | NOT OBSERVED | store guard (first line): if (param_4 != (double *)0x0) { > if (iVar1 != 0) {; caller 1eca78 C:L1083 [do {; if (0 < local_e0) {; if ((local_68 < 2) && (-1000 < *(int *)(lVar12 + 0xf0))) {] |
| TQ | 222baa @0x270900 | 3 / C3 | 502 | - | 0x2715a7 | NOT OBSERVED | store guard (first line): if (param_4 != (double *)0x0) {; caller 1f4140 C:L248 [if (iVar9 != *(int *)(lVar21 + 0x1f0)) {; if (iVar10 != local_d0) {; if (*(int *)((long)param_1 + 0x290) == 0) {] |
| TQ | 222d31 @0x271750 | 4 / C4 | 252, 255, 347 | - | 0x271ac7 0x271cac 0x271cd1 | NOT OBSERVED | store guard (first line): if (*param_2 == 2) { > else {; caller 1f4140 C:L2513 [if (((**(int **)(*(long *)((long)param_1 + 0x4b8) + 0xe8) != 4) //; else {; if ((*(int *)(lVar21 + 0x20c) == 0) //] |

Resolved/hinted mixed cases:

- 534f7: asm 0xea700 (NOT) = C:1373 (the only C store without the 2.0 constant; guards: `param_3 != 0`, `else if (local_68 < 2)`, `*(*(lVar10+0x4b8)+0xf0)==0 && *(lVar10+0xf4)<0 ...`), asm 0xe84b5 (OBS) = C:1513. Constant-literal match, unique.
- 5b257: only asm 0xf5156 is NOT (21 of 22 observed); no constant in its window, so the C line is one of the const-free lines {560,596,794,884,987,1006,1065,1176,2029,2615,2633,2669,2674,2729}; address-order hint = C:2729 (last, `*param_2 = auVar43._0_8_ + (double)(...)*dVar49`), unverified.
- 1ca57: 3 NOT (0xbf7c2 0xbf84a 0xbfbe1) / 2 OBS; 1bae9c: 7 NOT / 10 OBS; 1c35c5: 3 NOT / 7 OBS; 36ade: 4 NOT / 5 OBS: see table addresses.

### 4. Unobserved writers, grouped

Core dual path (reachable in a normal double dual run; not executed in kb2/degen2):
- 17248 (2 stores, 0xbe7b0): called from 5b257 C:2472 under `*(int*)(param_1+0x3ec)==2` / `-1 < iVar35` / `spx+0x290==0 && -1 < spx+0x2fc`; stores guarded by `work != NULL` (C:293) and `dVar28/iVar15 <= 5.0 && *(int*)(param_1+0x414) < 10` (C:138).
- 1c090a (1 store, 0xcc6a0): called from 28c93 C:225 (in the `else` branch; the known suspected path); store C:197 under `work != NULL`.
- 9e47c (1 store, 0x131760): called from 5b257 C:316 under `iVar10 != *(lVar20+0x1d8)`, `iVar35 != iVar52`, `spx+0x290==0`, `spx+0x294==0`; store C:441 under `work != NULL`.
- 118d6a (3 stores, 0x1aa960): called from 1ca57 C:483 under `(spx+0x514 != 0) && (spx+0x28 == 0)` and from 48d5f C:221 under `1 < spx+0x514 && spx+0x388*100 < ABS(..)`, `spx+0x290 != 1`; stores C:187/325/474 all under `work != NULL` and four non-null arrays.
- 1876a1 (3 stores, 0x1f4d20): called from 2a264 C:223 (unguarded in 2a264; 2a264 is called from 6268b C:1015 in the do-loop before 28c93, `iVar8 == 1` branch); stores C:375/2167/2246 under `work != NULL`.
- Partial: 5b257 (1 of 22), 534f7 (C:1373), 1ca57 (3 of 5), 1bae9c (7 of 17), 1c35c5 (3 of 10), 36ade (4 of 9) - addresses in table.
- 12f4f3/2a264/6268b/a002d/... have no store themselves.
Callback/heuristic subtree under 9a91f3 (called from 6268b C:979 and 534f7 C:718/1077 behind `*(lVar24+0x2e58+..) != 0 && *(lVar24+0x2e80)==0 ...`; looks like MIP callback / solution-polish machinery (calls `GRBcbsolution`), role inferred not confirmed): 6de7ca (13), 897f79 (6), 8986cb (6), 89d32b (4), 8a1cd9 (4), 8a2ecb (3), 8a37dc (2), 8a9773 (3), 8a55b2 (4), 895d22 (1), 618398 (4), a7e18d (2), ac3c8b (1), ac426e (1): 54 stores, none observed. 9a91f3 also stores the pointer into a struct (`mov %rcx,0x298(%r12)`, C:66 `*(double**)(puVar17+0xa6) = param_6`) - see section 5.
TQ (dead by the `spx+0x28` guard): 1eca78 (2), 1be884 (5), 1bae9c quad @0x218010 (17), 1b7106 quad @0x21c300 (1), 1f4140 (21), 1b864b (1), 1b8cf4 (2), 1cf516 (9), 22170d->2207de quad @0x25e0e0 (1), 2229b6 (1), 222baa (1), 222d31 (3): 64 stores, none observed.

Also unobserved among the 20 functions that DO have observed sites: none besides the partial ones above (all other T1 stores in 15d2d, 16b0a, 1b7106, 1c8477, 28c93, 2896a3, 2207de, 48d5f, 8b473, 9d112, 9e288, 9e376, 9e60b, a4109 are OBSERVED).

Other-offset stores on the work object (not the counter): a83b2f (asm 0x88eed0) writes `+0x10` and `+0x18` of the work pointer (`movsd %xmm2,0x10(%r15)` @0x88f086 and `movsd %xmm3,0x18(%r15)` @0x88f08c, reached from 6268b C:981 / 534f7 C:720 via a83b2f(spx, work)) - the object is therefore a small struct (+0 work, +8 limit/start-time, +0x10/+0x18 written by a83b2f), not a bare double.

### 5. Unresolved points

- Struct carrier: `9a91f3` (0x7cd7f0) stores the work pointer into a struct field `+0x298` (`mov %rcx,0x298(%r12)` @0x7cd861; C:66). Readers of that field (callback "get WORK/RUNTIME" style) not traced; any later write through it is NOT in this superset. Not followed.
- Indirect calls/jumps receiving tainted args (possible stale-register noise): variadic/log trampolines `a7aa8f` (`jmp *%r11`, 52 callers, C: `(*(code*)(&LAB_0098a15c + in_AL*-4))()`), `a7a18c`, `b406ca` (`jmp *%rax`), allocator hooks `ac661f`/`ac64e6` (`call *%rax`). Their targets are unresolved; they look like logging/allocation helpers, the tainted registers are probably stale, but this is unverified.
- External calls with tainted args (memcpy/memset/strncpy/sprintf/floor/ceil/log2/pow/gettimeofday/pthread_*/`DGETRF`): taint at those sites is mostly stale arg registers; `memcpy`/`memset` with the work pointer as destination would be a store - none identified, not proven absent (5b257 memcpy@plt x5, 6268b memset@plt/`__isnan`, 36ade `DGETRF`).
- `1e7531`/`cda5b` gateway closure (~829 functions, see section 1) is not hand-verified; the same stale-register over-approximation applies and stack-argument cut-offs (callee C param count + 2) may hide some deep stack-arg flows.
- Decompiler arg loss was handled by asm, but flows through memory other than rsp-relative slots (heap structs holding the pointer, besides 9a91f3's +0x298 store) are invisible to this analysis.
- 6268b itself has no offset-0 store (its `*pdVar17 = ...` lines C:1607/1673 are vector divisions on a reused variable; the work pointer is spilled at 0xe0(%rsp)).

### 6. Counts

- T1: 72 functions, 152 offset-0 stores, 69 observed, 83 NOT observed.
- TQ: 14 extra functions, 64 stores, 0 observed.
- T2 (gateway closure): 829 functions, 2239 offset-0 stores (unverified), 0 observed.

Side files (same directory): `bench/oracle/m1_02p/static/propagation_guards.tsv` (every C store line with enclosing conditions), propagation_edges.tsv (all call edges carrying work), propagation_t2.tsv (gateway closure).