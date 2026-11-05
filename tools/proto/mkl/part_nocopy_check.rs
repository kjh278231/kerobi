// test driver for part_nocopy.rs: stdin lines "m n k lda ldb ldc <A bits hex...> <B...> <C...>", prints C bits hex
include!("part_nocopy.rs");
use std::io::{self, BufRead, Write};
fn main() {
    let out = io::stdout();
    let mut out = out.lock();
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut it = line.split_whitespace();
        let mut u = || it.next().unwrap().parse::<usize>().unwrap();
        let (m, n, k, lda, ldb, ldc) = (u(), u(), u(), u(), u(), u());
        let v: Vec<f64> = line.split_whitespace().skip(6).map(|h| f64::from_bits(u64::from_str_radix(h, 16).unwrap())).collect();
        let (a, rest) = v.split_at(lda * m);
        let (b, c0) = rest.split_at(ldb * n);
        let mut c = c0.to_vec();
        assert!(mkl_gemm_is_nocopy(m, n, k) || m == 0);
        emu_gemm_nocopy_tn(m, n, k, a, lda, b, ldb, &mut c, ldc);
        let s: Vec<String> = c.iter().map(|x| format!("{:x}", x.to_bits())).collect();
        writeln!(out, "{}", s.join(" ")).unwrap();
    }
}
