// DTRSM('L','U','T','U') v1 path (m <= 12, or m % 4 == 0 && m <= 88). Port of part_trsmv1.c.

/// t - sum_{p<i} u[p]*x[p] in mkl_blas_avx2_dtrsm_pst's order (Left/Upper/Trans branch)
fn dot_trsm_pst(mut t: f64, u: &[f64], x: &[f64], i: usize) -> f64 {
    let nb16 = i & !15;
    if nb16 > 0 {
        let mut a = [[0.0f64; 4]; 4];
        a[0][0] = t;
        for b in (0..nb16).step_by(16) {
            for q in 0..4 {
                for l in 0..4 {
                    let p = b + 4 * q + l;
                    a[q][l] = (-u[p]).mul_add(x[p], a[q][l]);
                }
            }
        }
        let s: Vec<f64> = (0..4).map(|l| (a[0][l] + a[1][l]) + (a[2][l] + a[3][l])).collect();
        t = (s[0] + s[2]) + (s[1] + s[3]);
    }
    let nb4 = (i - nb16) & !3;
    if nb4 > 0 {
        let mut v = [t, 0.0, 0.0, 0.0];
        for g in (nb16..nb16 + nb4).step_by(4) {
            for l in 0..4 {
                v[l] = v[l] - u[g + l] * x[g + l];
            }
        }
        t = (v[0] + v[2]) + (v[1] + v[3]);
    }
    for p in nb16 + nb4..i {
        t = t - u[p] * x[p];
    }
    t
}

/// mkl_blas_avx2_dtrsm_pst('L','U','T','U', m, n, 1.0, A, lda, B, ldb); a and b start at the block's (0,0)
fn trsm_pst_lutu(m: usize, n: usize, a: &[f64], lda: usize, b: &mut [f64], ldb: usize) {
    for j in 0..n {
        let x = &mut b[j * ldb..];
        for i in 1..m {
            x[i] = dot_trsm_pst(x[i], &a[i * lda..], x, i);
        }
    }
}

pub fn trsm_v1(m: usize, n: usize, a: &[f64], lda: usize, b: &mut [f64], ldb: usize) {
    let (m4, mr) = (m & !3, m & 3);
    let mut n4 = n & !3;
    if m4 > 0 && n4 > 0 {
        // ker_llu_a4_b8/b4: one fused chain per element, p ascending
        for j in 0..n4 {
            let x = &mut b[j * ldb..];
            for i in 0..m4 {
                let mut acc = x[i];
                for p in 0..i {
                    acc = (-a[p + i * lda]).mul_add(x[p], acc);
                }
                x[i] = acc;
            }
        }
        if mr > 0 {
            // dgemm_pst('T','N', mr, n4, m4, -1, ..., 1, ...): 4 lanes of mul+add, (l0+l2)+(l1+l3), then C - s
            for j in 0..n4 {
                let x = &mut b[j * ldb..];
                for i in m4..m {
                    let u = &a[i * lda..];
                    let mut v = [0.0f64; 4];
                    for g in (0..m4).step_by(4) {
                        for l in 0..4 {
                            v[l] = u[g + l] * x[g + l] + v[l];
                        }
                    }
                    let s = (v[0] + v[2]) + (v[1] + v[3]);
                    x[i] = 1.0f64.mul_add(x[i], -s);
                }
            }
            trsm_pst_lutu(mr, n4, &a[m4 + m4 * lda..], lda, &mut b[m4..], ldb);
        }
    }
    if m4 == 0 {
        n4 = 0;
    }
    if n > n4 {
        trsm_pst_lutu(m, n - n4, a, lda, &mut b[n4 * ldb..], ldb);
    }
}
