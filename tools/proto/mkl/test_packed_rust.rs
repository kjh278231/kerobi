// stdin: lines "G|P m n k lda ldb ldc" then A, B, C as u64 bit patterns (one line each); stdout: C bits after the call
include!("part_gemv.rs");
include!("part_packed.rs");
use std::io::{self, BufRead, Write};
fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines().map(|l| l.unwrap());
    let mut out = io::stdout().lock();
    while let Some(h) = lines.next() {
        let f: Vec<&str> = h.split_whitespace().collect();
        let v: Vec<usize> = f[1..].iter().map(|s| s.parse().unwrap()).collect();
        let mut rd = || -> Vec<f64> { lines.next().unwrap().split_whitespace().map(|s| f64::from_bits(s.parse().unwrap())).collect() };
        let (a, b, mut c) = (rd(), rd(), rd());
        if f[0] == "G" {
            emu_gemm_tn_gemv(v[0], v[1], v[2], &a, v[3], &b, v[4], &mut c, v[5]);
        } else {
            emu_gemm_packed_tn(v[0], v[1], v[2], &a, v[3], &b, v[4], &mut c, v[5]);
        }
        writeln!(out, "{}", c.iter().map(|x| x.to_bits().to_string()).collect::<Vec<_>>().join(" ")).unwrap();
    }
}
