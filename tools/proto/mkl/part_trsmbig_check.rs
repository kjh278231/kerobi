// reads cases from argv[1] (little-endian: u64 m, n, lda, ldb, then A[lda*m], B[ldb*n] as f64), writes B results to argv[2]
mod part_trsmbig;
use std::io::{Read, Write};
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut buf = Vec::new();
    std::fs::File::open(&args[1]).unwrap().read_to_end(&mut buf).unwrap();
    let w: Vec<u64> = buf.chunks(8).map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect();
    let mut out = std::fs::File::create(&args[2]).unwrap();
    let mut i = 0;
    while i < w.len() {
        let (m, n, lda, ldb) = (w[i] as usize, w[i + 1] as usize, w[i + 2] as usize, w[i + 3] as usize);
        i += 4;
        let a: Vec<f64> = w[i..i + lda * m].iter().map(|&u| f64::from_bits(u)).collect();
        i += lda * m;
        let mut b: Vec<f64> = w[i..i + ldb * n].iter().map(|&u| f64::from_bits(u)).collect();
        i += ldb * n;
        if n == 1 { part_trsmbig::emu_trsv_utu(m, &a, lda, &mut b) } else { part_trsmbig::emu_trsm_big(m, n, &a, lda, &mut b, ldb) }
        for v in b { out.write_all(&v.to_bits().to_le_bytes()).unwrap(); }
    }
}
