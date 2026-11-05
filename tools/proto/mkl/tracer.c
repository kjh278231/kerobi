/* int3 function-entry tracer: records the sequence of hit breakpoints (re-armed after a single step) */
#define _GNU_SOURCE
#include <signal.h>
#include <stdint.h>
#include <string.h>
#include <sys/mman.h>
#include <ucontext.h>
#define MAXBP 8192
#define MAXLOG 4000000
static uint64_t bp[MAXBP]; static unsigned char orig[MAXBP]; static int nbp, pending = -1;
static uint64_t logv[MAXLOG]; static long nlog;
static void wr(uint64_t a, unsigned char v) {
    uint64_t pg = a & ~0xfffUL;
    mprotect((void *)pg, 0x2000, PROT_READ | PROT_WRITE | PROT_EXEC);
    *(volatile unsigned char *)a = v;
}
static int find(uint64_t a) { for (int i = 0; i < nbp; i++) if (bp[i] == a) return i; return -1; }
static void h(int sig, siginfo_t *si, void *ctx) {
    ucontext_t *uc = ctx; greg_t *g = uc->uc_mcontext.gregs;
    if (pending >= 0) { wr(bp[pending], 0xcc); pending = -1; g[REG_EFL] &= ~0x100; return; }
    uint64_t rip = g[REG_RIP] - 1; int i = find(rip);
    if (i < 0) return;
    if (nlog < MAXLOG) logv[nlog++] = rip;
    wr(rip, orig[i]); g[REG_RIP] = rip; g[REG_EFL] |= 0x100; pending = i;
}
int trace_set(const uint64_t *a, int n) {
    struct sigaction sa; memset(&sa, 0, sizeof sa); sa.sa_sigaction = h; sa.sa_flags = SA_SIGINFO; sigaction(SIGTRAP, &sa, 0);
    for (int i = 0; i < n && nbp < MAXBP; i++) { bp[nbp] = a[i]; orig[nbp] = *(unsigned char *)a[i]; wr(a[i], 0xcc); nbp++; }
    return nbp;
}
void trace_off(void) { for (int i = 0; i < nbp; i++) wr(bp[i], orig[i]); nbp = 0; }
long trace_get(uint64_t *out, long max) { long n = nlog < max ? nlog : max; memcpy(out, logv, n * 8); nlog = 0; return n; }
