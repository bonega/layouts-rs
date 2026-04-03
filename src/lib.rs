#[cfg(test)]
macro_rules! pos {
    ($row:expr, $col:expr) => {
        crate::layout::Pos::new($row, $col)
    };
}

#[cfg(test)]
macro_rules! key {
    ($ch:expr, $finger_number:expr, $pos:expr) => {
        crate::layout::Key::new(
            $ch,
            crate::layout::Finger::from($finger_number),
            $pos,
            false,
        )
    };
}

#[cfg(test)]
macro_rules! finger_home_key {
    ($ch:expr, $finger_number:expr, $pos:expr) => {
        crate::layout::Key::new($ch, crate::layout::Finger::from($finger_number), $pos, true)
    };
}

pub mod analyzer;
pub mod corpus;
pub mod layout;
pub mod ngrams;
pub mod report;
