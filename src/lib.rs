#[cfg(test)]
macro_rules! pos {
    ($row:expr, $col:expr) => {
        crate::matrix::Pos::new($row, $col)
    };
}

#[macro_export]
macro_rules! matrix {
    ([$([$($x:expr),* $(,)?]),+ $(,)?]) => {
        $crate::matrix::Matrix::new(vec![
            $(vec![$($x),*]),+
        ]).unwrap()
    };
}

#[cfg(test)]
macro_rules! key {
    ($ch:expr, $finger_number:expr, $pos:expr) => {
        key!($ch, $finger_number, $pos, 1.0)
    };
    ($ch:expr, $finger_number:expr, $pos:expr, $effort:expr) => {
        crate::layout::Key::new(
            $ch,
            crate::layout::Finger::from($finger_number),
            $pos,
            $effort,
            false,
        )
    };
}

#[cfg(test)]
macro_rules! finger_home_key {
    ($ch:expr, $finger_number:expr, $pos:expr) => {
        finger_home_key!($ch, $finger_number, $pos, 1.0)
    };
    ($ch:expr, $finger_number:expr, $pos:expr, $effort:expr) => {
        crate::layout::Key::new(
            $ch,
            crate::layout::Finger::from($finger_number),
            $pos,
            $effort,
            true,
        )
    };
}

#[cfg(test)]
macro_rules! optimizer_target {
    ($value:expr, $weight:expr) => {
        crate::optimizer::Target {
            value: $value,
            weight: $weight,
            scale: 1.0,
        }
    };
}

pub mod analyzer;
pub mod config;
pub mod corpus;
pub mod layout;
pub mod matrix;
pub mod ngrams;
pub mod optimizer;
pub mod report;
pub mod swaps;
