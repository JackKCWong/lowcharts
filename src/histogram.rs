use std::fmt;
use std::ops::Range;

use yansi::Color::{Blue, Green, Red};

use crate::stats::Stats;

#[derive(Debug)]
pub struct Bucket {
    range: Range<f64>,
    count: usize,
}

impl Bucket {
    fn new(range: Range<f64>) -> Bucket {
        Bucket { range, count: 0 }
    }

    fn inc(&mut self) {
        self.count += 1;
    }
}

#[derive(Debug)]
pub struct Histogram {
    vec: Vec<Bucket>,
    max: f64,
    step: f64,
    top: usize,
    last: usize,
    stats: Stats,
}

impl Histogram {
    pub fn new(size: usize, step: f64, stats: Stats) -> Histogram {
        let mut b = Histogram {
            vec: Vec::with_capacity(size),
            max: stats.min + (step * size as f64),
            step,
            top: 0,
            last: size - 1,
            stats,
        };
        let mut lower = b.stats.min;
        for _ in 0..size {
            b.vec.push(Bucket::new(lower..lower + step));
            lower += step;
        }
        b
    }

    pub fn load(&mut self, vec: &[f64]) {
        for x in vec {
            self.add(*x);
        }
    }

    pub fn add(&mut self, n: f64) {
        if let Some(slot) = self.find_slot(n) {
            self.vec[slot].inc();
            self.top = self.top.max(self.vec[slot].count);
        }
    }

    fn find_slot(&self, n: f64) -> Option<usize> {
        if n < self.stats.min || n > self.max {
            None
        } else {
            Some((((n - self.stats.min) / self.step) as usize).min(self.last))
        }
    }
}

impl fmt::Display for Histogram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.stats)?;
        let writer = HistWriter {
            width: f.width().unwrap_or(110),
        };
        writer.write(f, &self)
    }
}

struct HistWriter {
    width: usize,
}

impl HistWriter {
    pub fn write(&self, f: &mut fmt::Formatter, hist: &Histogram) -> fmt::Result {
        let width_range = Self::get_width(hist);
        let width_count = ((hist.top as f64).log10().ceil() as usize).max(1);
        let divisor = 1.max(hist.top / self.get_max_bar_len(width_range + width_count));
        writeln!(
            f,
            "each {} represents a count of {}",
            Red.paint("∎"),
            Blue.paint(divisor.to_string()),
        )?;
        for x in hist.vec.iter() {
            self.write_bucket(f, x, divisor, width_range, width_count)?;
        }
        Ok(())
    }

    fn write_bucket(
        &self,
        f: &mut fmt::Formatter,
        bucket: &Bucket,
        divisor: usize,
        width: usize,
        width_count: usize,
    ) -> fmt::Result {
        let bar = Red.paint(format!("{:∎<width$}", "", width = bucket.count / divisor));
        writeln!(
            f,
            "[{range}] [{count}] {bar}",
            range = Blue.paint(format!(
                "{:width$.3} .. {:width$.3}",
                bucket.range.start,
                bucket.range.end,
                width = width,
            )),
            count = Green.paint(format!("{:width$}", bucket.count, width = width_count)),
            bar = bar
        )
    }

    fn get_width(hist: &Histogram) -> usize {
        format!("{:.3}", hist.stats.min)
            .len()
            .max(format!("{:.3}", hist.max).len())
    }

    fn get_max_bar_len(&self, fixed_width: usize) -> usize {
        const EXTRA_CHARS: usize = 10;
        if self.width < fixed_width + EXTRA_CHARS {
            75
        } else {
            self.width - fixed_width - EXTRA_CHARS
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yansi::Paint;

    #[test]
    fn basic_test() {
        let stats = Stats::new(&[-2.0, 14.0]);
        let mut hist = Histogram::new(8, 2.5, stats);
        hist.load(&[
            -1.0, -1.1, 2.0, 2.0, 2.1, -0.9, 11.0, 11.2, 1.9, 1.99, 1.98, 1.97, 1.96,
        ]);

        assert_eq!(hist.top, 8);
        let bucket = &hist.vec[0];
        assert_eq!(bucket.range, -2.0..0.5);
        assert_eq!(bucket.count, 3);
        let bucket = &hist.vec[1];
        assert_eq!(bucket.count, 8);
        assert_eq!(bucket.range, 0.5..3.0);
    }

    #[test]
    fn display_test() {
        let stats = Stats::new(&[-2.0, 14.0]);
        let mut hist = Histogram::new(8, 2.5, stats);
        hist.load(&[
            -1.0, -1.1, 2.0, 2.0, 2.1, -0.9, 11.0, 11.2, 1.9, 1.99, 1.98, 1.97, 1.96,
        ]);
        Paint::disable();
        let display = format!("{}", hist);
        assert!(display.find("[-2.000 ..  0.500] [3] ∎∎∎\n").is_some());
        assert!(display.find("[ 0.500 ..  3.000] [8] ∎∎∎∎∎∎∎∎\n").is_some());
        assert!(display.find("[10.500 .. 13.000] [2] ∎∎\n").is_some());
    }
}
