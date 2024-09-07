#[macro_export]
macro_rules! mask {
    ($size:expr) => {
        if $size < 64 {
            (1 << $size) - 1
        } else {
            !0
        }
    };
}

#[macro_export]
macro_rules! prev_pow_of_2 {
    ($n:expr) => {
        (1 << (usize::BITS - ($n).leading_zeros() - 1))
    };
}

#[macro_export]
macro_rules! round_up {
    ($n:expr, $align:expr) => {
        ($n + $align - 1) & !($align - 1)
    };
}

#[macro_export]
macro_rules! round_down {
    ($n:expr, $align:expr) => {
        ($n & !($align - 1))
    };
}
