// Port of part_packed.c: MKL AVX2 packed DGEMM('T','N') path (fullacopybcopy + dgemm_kernel_0), alpha = beta = 1.
// Column blocks of nc = 24 (k < 48) or 192; per block 4-column tiles then 2, 1. Rows: 12-row tiles, then m%12 as 8,4,2,1.
// Per element q accumulators by p mod q (p < 4*(k/4), fma, start +0); combine when k >= 4: q=2 a0+a1,
// q=4 (a0+a1)+(a2+a3); k%4 tail fma'd into the sum; C = C + sum. 12x1 tiles add an extra +0.0 after the loop.

fn dot_q(a: &[f64], b: &[f64], k: usize, q: usize, zadd: bool) -> f64 {
    let k4 = k & !3;
    let mut acc = [0.0f64; 4];
    for p in 0..k4 {
        acc[p % q] = a[p].mul_add(b[p], acc[p % q]);
    }
    let mut s = acc[0];
    if k4 != 0 {
        if q == 2 {
            s = acc[0] + acc[1];
        } else if q == 4 {
            s = (acc[0] + acc[1]) + (acc[2] + acc[3]);
        } else if zadd {
            s = acc[0] + 0.0;
        }
    }
    for p in k4..k {
        s = a[p].mul_add(b[p], s);
    }
    s
}

// tile size along a dimension of length len: full tiles of `full`, then the rest by its bits (largest first)
fn tile_of(idx: usize, len: usize, full: usize) -> usize {
    let nf = len - len % full;
    if idx < nf {
        return full;
    }
    let (mut t, mut off) = (if full == 12 { 8 } else { 2 }, nf);
    while t >= 1 {
        if (len - nf) & t != 0 {
            if idx < off + t {
                return t;
            }
            off += t;
        }
        t >>= 1;
    }
    unreachable!()
}

fn q_of(mr: usize, nr: usize) -> usize {
    match (mr, nr) {
        (8, 1) | (4, 1) => 4,
        (4, 4) | (2, 1) | (1, _) => 2,
        _ => 1,
    }
}

/// C[i + j*ldc] += sum_p A[p + i*lda] * B[p + j*ldb]. shortcut: assumes k <= 256 (MKL splits k above), extend if k grows
pub fn emu_gemm_packed_tn(m: usize, n: usize, k: usize, a: &[f64], lda: usize, b: &[f64], ldb: usize, c: &mut [f64], ldc: usize) {
    let nc = if k < 48 { 24 } else { 192 };
    let mut j0 = 0;
    while j0 < n {
        let nb = (n - j0).min(nc);
        for jj in 0..nb {
            let (j, nr) = (j0 + jj, tile_of(jj, nb, 4));
            for i in 0..m {
                let mr = tile_of(i, m, 12);
                c[i + j * ldc] += dot_q(&a[i * lda..], &b[j * ldb..], k, q_of(mr, nr), mr == 12 && nr == 1);
            }
        }
        j0 += nc;
    }
}
