/* int3 tracer that also records integer argument registers and 8 stack args at each hit */
#define _GNU_SOURCE
#include <signal.h>
#include <stdint.h>
#include <string.h>
#include <sys/mman.h>
#include <ucontext.h>
#define MAXBP 256
#define MAXLOG 100000
static uint64_t bp[MAXBP]; static unsigned char orig[MAXBP]; static int nbp, pending = -1;
static uint64_t logv[MAXLOG][16]; static long nlog;
static void wr(uint64_t a, unsigned char v) {
    mprotect((void *)(a & ~0xfffUL), 0x2000, PROT_READ | PROT_WRITE | PROT_EXEC);
    *(volatile unsigned char *)a = v;
}
static void h(int sig, siginfo_t *si, void *ctx) {
    ucontext_t *uc = ctx; greg_t *g = uc->uc_mcontext.gregs;
    if (pending >= 0) { wr(bp[pending], 0xcc); pending = -1; g[REG_EFL] &= ~0x100; return; }
    uint64_t rip = g[REG_RIP] - 1; int i;
    for (i = 0; i < nbp && bp[i] != rip; i++);
    if (i == nbp) return;
    if (nlog < MAXLOG) {
        uint64_t *l = logv[nlog++], *sp = (uint64_t *)g[REG_RSP];
        l[0] = rip; l[1] = g[REG_RDI]; l[2] = g[REG_RSI]; l[3] = g[REG_RDX]; l[4] = g[REG_RCX]; l[5] = g[REG_R8]; l[6] = g[REG_R9];
        for (int s = 0; s < 9; s++) l[7 + s] = sp[1 + s];
    }
    wr(rip, orig[i]); g[REG_RIP] = rip; g[REG_EFL] |= 0x100; pending = i;
}
int at_set(const uint64_t *a, int n) {
    struct sigaction sa; memset(&sa, 0, sizeof sa); sa.sa_sigaction = h; sa.sa_flags = SA_SIGINFO; sigaction(SIGTRAP, &sa, 0);
    for (int i = 0; i < n && nbp < MAXBP; i++) { bp[nbp] = a[i]; orig[nbp] = *(unsigned char *)a[i]; wr(a[i], 0xcc); nbp++; }
    return nbp;
}
void at_off(void) { for (int i = 0; i < nbp; i++) wr(bp[i], orig[i]); nbp = 0; }
long at_get(uint64_t *out, long max) { long n = nlog < max ? nlog : max; memcpy(out, logv, n * 128); nlog = 0; return n; }
