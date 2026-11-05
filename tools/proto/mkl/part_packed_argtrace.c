/* int3 tracer on ONE function whose first 3 args are int64 pointers (m, n, k of dgemm_kernel_0): logs *rdi,*rsi,*rdx,rcx,r8,r9 */
#define _GNU_SOURCE
#include <signal.h>
#include <stdint.h>
#include <string.h>
#include <sys/mman.h>
#include <ucontext.h>
static uint64_t bp; static unsigned char orig; static int pending, armed;
static int64_t logv[100000][8]; static long nlog;
static void wr(uint64_t a, unsigned char v) { mprotect((void *)(a & ~0xfffUL), 0x2000, 7); *(volatile unsigned char *)a = v; }
static void h(int sig, siginfo_t *si, void *ctx) {
    greg_t *g = ((ucontext_t *)ctx)->uc_mcontext.gregs;
    if (pending) { wr(bp, 0xcc); pending = 0; g[REG_EFL] &= ~0x100; return; }
    if ((uint64_t)g[REG_RIP] - 1 != bp) return;
    if (nlog < 100000) {
        int64_t *r = logv[nlog++];
        r[0] = *(int64_t *)g[REG_RDI]; r[1] = *(int64_t *)g[REG_RSI]; r[2] = *(int64_t *)g[REG_RDX];
        r[3] = g[REG_RCX]; r[4] = g[REG_R8]; r[5] = g[REG_R9];
        r[6] = *(int64_t *)(g[REG_RSP] + 8); r[7] = *(int64_t *)(g[REG_RSP] + 16);
    }
    wr(bp, orig); g[REG_RIP] -= 1; g[REG_EFL] |= 0x100; pending = 1;
}
void at_set(uint64_t a) {
    struct sigaction sa; memset(&sa, 0, sizeof sa); sa.sa_sigaction = h; sa.sa_flags = SA_SIGINFO; sigaction(SIGTRAP, &sa, 0);
    bp = a; orig = *(unsigned char *)a; wr(a, 0xcc); armed = 1;
}
void at_off(void) { if (armed) wr(bp, orig); armed = 0; }
long at_get(int64_t *out) { long n = nlog; memcpy(out, logv, n * 64); nlog = 0; return n; }
