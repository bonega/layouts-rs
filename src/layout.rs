use std::collections::HashSet;

pub struct Pos {
    pub r: usize,
    pub c: usize,
}
impl Pos {
    pub fn new(r: usize, c: usize) -> Self {
        Self { r, c }
    }
}

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

    pub fn char_at(&self, position: Pos) -> Option<char> {
        self.key_at(position).map(|key| key.ch)
    }

    pub fn key_at(&self, position: Pos) -> Option<&Key> {
        if position.r >= ROWS || position.c >= COLUMNS {
            return None;
        }
        self.keys[position.r][position.c].as_ref()
    }

    pub fn key_for(&self, ch: &str) -> Option<&Key> {
        (0..ROWS)
            .flat_map(|row| (0..COLUMNS).map(move |col| (row, col)))
            .find_map(|(row, col)| {
                self.keys[row][col]
                    .as_ref()
                    .filter(|key| key.ch.to_string() == ch)
            })
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

    #[test]
    fn it_returns_key_by_pos() {
        let layout = Layout::<2, 3>::new("abcdef").unwrap();

        check!(layout.key_at(Pos::new(0, 0)) == Some(&Key::new('a')));
        check!(layout.key_at(Pos::new(0, 1)) == Some(&Key::new('b')));
        check!(layout.key_at(Pos::new(0, 2)) == Some(&Key::new('c')));
        check!(layout.key_at(Pos::new(1, 0)) == Some(&Key::new('d')));
        check!(layout.key_at(Pos::new(1, 1)) == Some(&Key::new('e')));
        check!(layout.key_at(Pos::new(1, 2)) == Some(&Key::new('f')));
        check!(layout.key_at(Pos::new(2, 0)) == None);
        check!(layout.key_at(Pos::new(0, 3)) == None);
    }

    #[test]
    fn it_returns_key_by_char() {
        let layout = Layout::<2, 3>::new("abcdef").unwrap();

        check!(layout.key_for("a") == Some(&Key::new('a')));
        check!(layout.key_for("b") == Some(&Key::new('b')));
        check!(layout.key_for("c") == Some(&Key::new('c')));
        check!(layout.key_for("d") == Some(&Key::new('d')));
        check!(layout.key_for("e") == Some(&Key::new('e')));
        check!(layout.key_for("f") == Some(&Key::new('f')));
        check!(layout.key_for("g") == None);
    }

    #[test]
    fn it_returns_char_by_pos() {
        let layout = Layout::<2, 3>::new("abcdef").unwrap();

        check!(layout.char_at(Pos::new(0, 0)) == Some('a'));
        check!(layout.char_at(Pos::new(0, 1)) == Some('b'));
        check!(layout.char_at(Pos::new(0, 2)) == Some('c'));
        check!(layout.char_at(Pos::new(1, 0)) == Some('d'));
        check!(layout.char_at(Pos::new(1, 1)) == Some('e'));
        check!(layout.char_at(Pos::new(1, 2)) == Some('f'));
        check!(layout.char_at(Pos::new(2, 0)) == None);
        check!(layout.char_at(Pos::new(0, 3)) == None);
    }
}
