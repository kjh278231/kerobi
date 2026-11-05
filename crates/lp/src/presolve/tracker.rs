//! Change tracker (state+0x3d8, 12 §4.5): which rows and columns changed since a reduction last looked, per level
//! (0 finish, 1 outer reductions, 2 inner loop). Reductions read their work list through `rows`/`cols`, so the
//! order of these lists decides the order reductions are applied in.

/// Per kind (rows or columns): flag byte per index (bit0/1 level-1 queued/marked-this-round, bit2/3 level 2),
/// per level a list with a "current" length `a` and a total length `b` (marks made while the level is active go
/// after `a` and become current at the next `reset`).
#[derive(Clone, Debug, Default)]
pub struct Queue {
    pub flag: Vec<u8>,
    pub a: [usize; 3],
    pub b: [usize; 3],
    pub list: [Vec<i32>; 3],
    /// `538f30`/`5399fc` result cache per level (tracker+0x94/+0xb8 count, -1 = stale).
    pub cache: [Option<Vec<i32>>; 3],
}

#[derive(Clone, Debug, Default)]
pub struct Tracker {
    /// tracker+0x10
    pub level: usize,
    /// tracker+0x18 / +0x28.. / +0x40..
    pub rows: Queue,
    /// tracker+0x20 / +0x58.. / +0x70..
    pub cols: Queue,
    /// tracker+0x88.. : the level has been reset at least once (marks then go to the "next" part)
    pub active: [bool; 3],
}

impl Queue {
    pub fn new(n: usize) -> Queue {
        let all: Vec<i32> = (0..n as i32).collect();
        Queue { flag: vec![0; n], a: [n; 3], b: [n; 3], list: [all.clone(), all.clone(), all], cache: [None, None, None] }
    }

    /// `53763f` (rows) / `5379f1` (columns).
    fn mark(&mut self, i: i32, active: &[bool; 3]) {
        let f = &mut self.flag[i as usize];
        for (lv, bits) in [(1usize, 0u8), (2, 2)] {
            if *f & (3 << bits) == 0 {
                let at = if active[lv] { self.b[lv] } else { self.a[lv] };
                if self.list[lv].len() <= at {
                    self.list[lv].resize(at + 1, 0);
                }
                self.list[lv][at] = i;
                if !active[lv] {
                    self.a[lv] += 1;
                    *f |= 1 << bits;
                }
                self.b[lv] += 1;
            }
        }
        if active[1] {
            *f |= 2;
        }
        if active[2] {
            *f |= 8;
        }
    }

    /// Compaction half of `5369b8`: keep live entries; above level 0 keep only those marked this round.
    /// Returns the number of entries visited (for work).
    fn reset(&mut self, lv: usize, live: &[i32]) -> usize {
        let (keep_bit, set, and_mask, clear) = if lv == 2 { (8u8, 0xcu8, 0xf7u8, 0xf3u8) } else { (2, 3, 0xfd, 0xfc) };
        let n = self.b[lv];
        let mut k = 0;
        for t in 0..n {
            let i = self.list[lv][t];
            if live[i as usize] < 0 {
                continue;
            }
            if lv == 0 {
                self.list[lv][k] = i;
                k += 1;
                continue;
            }
            let f = &mut self.flag[i as usize];
            if *f & keep_bit == 0 {
                *f &= clear;
            } else {
                *f = (*f | set) & and_mask;
                self.list[lv][k] = i;
                k += 1;
            }
        }
        self.a[lv] = k;
        self.b[lv] = k;
        n
    }
}

impl Tracker {
    pub fn new(nrows: usize, ncols: usize) -> Tracker {
        Tracker { level: 0, rows: Queue::new(nrows), cols: Queue::new(ncols), active: [false; 3] }
    }

    pub fn mark_row(&mut self, i: i32) {
        self.rows.mark(i, &self.active);
    }

    pub fn mark_col(&mut self, j: i32) {
        self.cols.mark(j, &self.active);
    }

    /// `5369b8`: start a new round at the current level. Work: level 0 counts 2 per row and 3 per column visited,
    /// other levels 3 per visit (times state+0x448).
    pub fn reset(&mut self, row_cnt: &[i32], col_cnt: &[i32], work: &mut f64, mult: f64) {
        let lv = self.level;
        if self.active[lv] {
            let r = self.rows.reset(lv, row_cnt) as f64;
            *work += if lv == 0 { (r + r) * mult } else { mult * 3.0 * r };
            let c = self.cols.reset(lv, col_cnt) as f64;
            *work += mult * 3.0 * c;
            if lv != 0 {
                self.rows.cache[lv] = None;
                self.cols.cache[lv] = None;
            }
        }
        self.active[lv] = true;
    }
}
