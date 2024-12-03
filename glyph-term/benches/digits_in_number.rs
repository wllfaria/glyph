fn main() {
    divan::main();
}

#[divan::bench_group]
mod digits_in_number {
    #[divan::bench(args = [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000])]
    #[inline]
    fn to_string(line_num: usize) -> usize {
        line_num.to_string().len()
    }

    #[divan::bench(args = [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000])]
    #[inline]
    fn dividing(mut line_num: usize) -> usize {
        if line_num == 0 {
            return 1;
        }

        let mut count = 0;
        while line_num > 0 {
            line_num /= 10;
            count += 1;
        }

        count
    }

    #[divan::bench(args = [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000])]
    #[inline]
    fn log10(line_num: usize) -> usize {
        (f32::log10(line_num as f32) + 1.0) as usize
    }

    #[divan::bench(args = [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000])]
    fn mixed(line_num: usize) -> usize {
        if line_num < 1000 {
            dividing(line_num)
        } else {
            log10(line_num)
        }
    }
}
