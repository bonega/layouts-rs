use std::collections::HashSet;

pub struct Pos(usize, usize);

#[derive(Clone, Copy, PartialEq)]
pub struct Key {
    ch: char,
}

impl Key {
    fn new(ch: char) -> Self {
        Self { ch }
    }
}

#[derive(Clone, Copy)]
pub struct Layout<const ROWS: usize = 3, const COLUMNS: usize = 12> {
    keys: [[Option<Key>; COLUMNS]; ROWS],
}

impl<const ROWS: usize, const COLUMNS: usize> Layout<ROWS, COLUMNS> {
    pub fn new(definition: &str) -> anyhow::Result<Self> {
        if definition.len() != ROWS * COLUMNS {
            anyhow::bail!(
                "expected {ROWS} rows and {COLUMNS} columns, received {} characters",
                definition.len()
            );
        }

        if definition.chars().collect::<HashSet<_>>().len() != definition.len() {
            anyhow::bail!("duplicate characters are not allowed in the layout");
        }

        let mut keys = Self::default_keys();
        for (index, ch) in definition.chars().enumerate() {
            let row = index / COLUMNS;
            let column = index % COLUMNS;
            keys[row][column] = Some(Key::new(ch));
        }

        Ok(Self { keys })
    }

    fn default_keys() -> [[Option<Key>; COLUMNS]; ROWS] {
        [[None; COLUMNS]; ROWS]
    }
}

impl<const ROWS: usize, const COLUMNS: usize> Default for Layout<ROWS, COLUMNS> {
    fn default() -> Self {
        Self {
            keys: Self::default_keys(),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert2::check;

    use super::*;

    #[test]
    fn it_checks_valid_layout() {
        check!(Layout::<2, 2>::new("abcd").is_ok());
        check!(Layout::<2, 3>::new("abcdef").is_ok());
        check!(Layout::<2, 2>::new("abcde").is_err());
        check!(Layout::<2, 2>::new("aaaa").is_err());
    }
}
