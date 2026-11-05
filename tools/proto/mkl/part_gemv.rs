// Port of part_gemv.c: MKL AVX2 dgemv_t (alpha = beta = 1, incx = 1) and the m == 1 / n == 1 DGEMM('T','N') routed to it.
// Per column j, 4 lanes (start +0): 8-blocks acc[l] = fma(x[p+l], a[p+l], x[p+4+l]*a[p+4+l]) + acc[l]; one 4-block if
// k%8 >= 4 (fma for columns in 4-column groups, mul+add for the n&2 / n&1 tail columns); k%4 tail masked fma
// (lanes >= k%4: fma(0,0,acc)); y[j] += (acc0 + acc1) + (acc2 + acc3).

/// y[j*incy] += sum_p a[p + j*lda] * x[p], for j < n
pub fn emu_gemv_t(k: usize, n: usize, a: &[f64], lda: usize, x: &[f64], y: &mut [f64], incy: usize) {
    let (k8, r, n4) = (k & !7, k & 3, n & !3);
    let k4 = (k - k8) & 4 != 0;
    for j in 0..n {
        let a = &a[j * lda..];
        let mut acc = [0.0f64; 4];
        let mut p = 0;
        while p < k8 {
            for l in 0..4 {
                acc[l] = x[p + l].mul_add(a[p + l], x[p + 4 + l] * a[p + 4 + l]) + acc[l];
            }
            p += 8;
        }
        if k4 {
            for l in 0..4 {
                acc[l] = if j < n4 { x[p + l].mul_add(a[p + l], acc[l]) } else { x[p + l] * a[p + l] + acc[l] };
            }
            p += 4;
        }
        if r != 0 {
            for l in 0..4 {
                acc[l] = if l < r { x[p + l].mul_add(a[p + l], acc[l]) } else { 0.0f64.mul_add(0.0, acc[l]) };
            }
        }
        y[j * incy] += (acc[0] + acc[1]) + (acc[2] + acc[3]);
    }
}

/// DGEMM('T','N', m, n, k, 1, A, lda, B, ldb, 1, C, ldc) for m == 1 or n == 1 (m == 1 checked first)
pub fn emu_gemm_tn_gemv(m: usize, n: usize, k: usize, a: &[f64], lda: usize, b: &[f64], ldb: usize, c: &mut [f64], ldc: usize) {
    if m == 1 {
        emu_gemv_t(k, n, b, ldb, a, c, ldc)
    } else {
        emu_gemv_t(k, m, a, lda, b, c, 1)
    }
}
