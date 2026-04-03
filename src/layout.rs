#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pos {
    pub r: usize,
    pub c: usize,
}

impl Pos {
    pub fn new(r: usize, c: usize) -> Self {
        Self { r, c }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Key {
    ch: char,
    pub finger: Finger,
    pub position: Pos,
}

impl Key {
    pub fn new(ch: char, finger: Finger, position: Pos) -> Self {
        Self {
            ch,
            finger,
            position,
        }
    }

    pub fn same_finger(&self, other: &Key) -> bool {
        self.finger == other.finger
    }

    pub fn row_distance(&self, other: &Key) -> usize {
        self.position.r.abs_diff(other.position.r)
    }

    pub fn column_distance(&self, other: &Key) -> usize {
        self.position.c.abs_diff(other.position.c)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Finger {
    pub hand: Hand,
    pub kind: FingerKind,
}

impl Finger {
    pub fn new(hand: Hand, kind: FingerKind) -> Self {
        Self { hand, kind }
    }

    pub fn distance(&self, other: &Finger) -> Option<usize> {
        if self.hand != other.hand || self.kind == other.kind {
            return None;
        }

        Some((self.kind as i32).abs_diff(other.kind as i32) as usize)
    }
}

impl From<Finger> for u8 {
    fn from(value: Finger) -> Self {
        match (value.hand, value.kind) {
            (Hand::Left, FingerKind::Pinky) => 1,
            (Hand::Left, FingerKind::Ring) => 2,
            (Hand::Left, FingerKind::Middle) => 3,
            (Hand::Left, FingerKind::Index) => 4,
            (Hand::Left, FingerKind::Thumb) => 5,
            (Hand::Right, FingerKind::Thumb) => 6,
            (Hand::Right, FingerKind::Index) => 7,
            (Hand::Right, FingerKind::Middle) => 8,
            (Hand::Right, FingerKind::Ring) => 9,
            (Hand::Right, FingerKind::Pinky) => 10,
        }
    }
}

impl From<u8> for Finger {
    fn from(value: u8) -> Self {
        let (hand, kind) = match value {
            1 => (Hand::Left, FingerKind::Pinky),
            2 => (Hand::Left, FingerKind::Ring),
            3 => (Hand::Left, FingerKind::Middle),
            4 => (Hand::Left, FingerKind::Index),
            5 => (Hand::Left, FingerKind::Thumb),
            6 => (Hand::Right, FingerKind::Thumb),
            7 => (Hand::Right, FingerKind::Index),
            8 => (Hand::Right, FingerKind::Middle),
            9 => (Hand::Right, FingerKind::Ring),
            10 => (Hand::Right, FingerKind::Pinky),
            _ => panic!("invalid finger value: {value}"),
        };
        Self { hand, kind }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Hand {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FingerKind {
    Pinky,
    Ring,
    Middle,
    Index,
    Thumb,
}

#[derive(Clone, Copy)]
pub struct Layout<const ROWS: usize = 3, const COLUMNS: usize = 12> {
    keys: [[Option<Key>; COLUMNS]; ROWS],
}

impl<const ROWS: usize, const COLUMNS: usize> Layout<ROWS, COLUMNS> {
    pub fn new(definition: &str, finger_assignment: Vec<Vec<u8>>) -> anyhow::Result<Self> {
        let definition = definition
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();

        if definition.len() != ROWS * COLUMNS {
            anyhow::bail!(
                "expected {ROWS} rows and {COLUMNS} columns, received {} characters",
                definition.len()
            );
        }

        if finger_assignment.len() != ROWS
            || finger_assignment.iter().any(|row| row.len() != COLUMNS)
        {
            anyhow::bail!(
                "expected finger assignment to have {ROWS} rows and {COLUMNS} columns, received {} rows and {} columns",
                finger_assignment.len(),
                finger_assignment
                    .iter()
                    .map(|row| row.len())
                    .max()
                    .unwrap_or(0)
            );
        }

        let mut keys = Self::default_keys();
        for (index, ch) in definition.chars().enumerate() {
            let row = index / COLUMNS;
            let column = index % COLUMNS;

            keys[row][column] = Some(Key::new(
                ch,
                Finger::from(finger_assignment[row][column]),
                Pos::new(row, column),
            ));
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

    pub fn key_for(&self, ch: char) -> Option<&Key> {
        (0..ROWS)
            .flat_map(|row| (0..COLUMNS).map(move |col| (row, col)))
            .find_map(|(row, col)| self.keys[row][col].as_ref().filter(|key| key.ch == ch))
    }

    pub fn set_char(&mut self, position: Pos, ch: char) -> anyhow::Result<()> {
        if position.r >= ROWS || position.c >= COLUMNS {
            anyhow::bail!("position out of bounds");
        }

        if let Some(key) = self.keys[position.r][position.c].as_mut() {
            key.ch = ch;
        }
        Ok(())
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
pub mod fixtures {
    use rstest::fixture;

    use super::*;

    #[fixture]
    pub fn qwerty() -> Layout<3, 12> {
        Layout::new(
            r#"
            _ q w e r t   y u i o p _
            " a s d f g   h j k l ; '
            _ z x c v b   n m , . / _
            "#,
            vec![
                vec![1, 1, 2, 3, 4, 4, 7, 7, 8, 9, 10, 10],
                vec![1, 1, 2, 3, 4, 4, 7, 7, 8, 9, 10, 10],
                vec![1, 1, 2, 3, 4, 4, 7, 7, 8, 9, 10, 10],
            ],
        )
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use assert2::check;

    use super::*;

    mod finger {
        use super::*;

        #[test]
        fn it_calculates_distance() {
            let finger1 = Finger::new(Hand::Left, FingerKind::Index);
            let finger2 = Finger::new(Hand::Left, FingerKind::Pinky);
            let finger3 = Finger::new(Hand::Right, FingerKind::Index);
            let finger4 = Finger::new(Hand::Right, FingerKind::Middle);

            check!(finger1.distance(&finger1) == None);
            check!(finger1.distance(&finger2) == Some(3));
            check!(finger1.distance(&finger3) == None);
            check!(finger3.distance(&finger4) == Some(1));
        }
    }

    mod key {
        use super::*;

        #[test]
        fn it_checks_if_keys_are_same_finger() {
            let key1 = key!('q', 1, pos!(0, 0));
            let key2 = key!('w', 2, pos!(0, 1));
            let key3 = key!('a', 1, pos!(0, 1));

            check!(key1.same_finger(&key3));
            check!(!key1.same_finger(&key2));
        }

        #[test]
        fn it_checks_row_distance() {
            let key1 = key!('q', 1, pos!(0, 0));
            let key2 = key!('w', 1, pos!(0, 1));
            let key3 = key!('a', 1, pos!(1, 0));
            let key4 = key!('z', 1, pos!(2, 0));

            check!(key1.row_distance(&key2) == 0);
            check!(key1.row_distance(&key3) == 1);
            check!(key1.row_distance(&key4) == 2);
            check!(key4.row_distance(&key1) == 2);
        }

        #[test]
        fn it_checks_column_distance() {
            let key1 = key!('q', 1, pos!(0, 0));
            let key2 = key!('w', 1, pos!(0, 1));
            let key3 = key!('a', 1, pos!(1, 0));
            let key4 = key!('e', 1, pos!(1, 2));

            check!(key1.column_distance(&key2) == 1);
            check!(key1.column_distance(&key3) == 0);
            check!(key1.column_distance(&key4) == 2);
            check!(key4.column_distance(&key1) == 2);
        }
    }

    mod layout {
        use super::*;

        #[test]
        fn it_builds_from_configuration() {
            check!(Layout::<2, 2>::new("abcd", vec![vec![1, 2], vec![1, 2]]).is_ok());
            check!(Layout::<2, 3>::new("abcdef", vec![vec![1, 2, 3], vec![1, 2, 3]]).is_ok());
            check!(Layout::<2, 2>::new("aaaa", vec![vec![1, 2], vec![1, 2]]).is_ok());

            check!(Layout::<2, 2>::new("abcde", vec![vec![1, 2], vec![1, 2]]).is_err());
            check!(Layout::<2, 2>::new("abcd", vec![vec![1, 2], vec![1, 2, 3]]).is_err());
        }

        #[test]
        fn it_builds_from_string_normalizing() {
            check!(
                Layout::<2, 3>::new(
                    r#"
            a b c
            d e f
            "#,
                    vec![vec![1, 2, 3], vec![1, 2, 3]]
                )
                .is_ok()
            );
        }

        #[test]
        fn it_returns_key_by_pos() {
            let layout = Layout::<2, 3>::new("abcdef", vec![vec![1, 2, 3], vec![1, 2, 3]]).unwrap();

            check!(layout.key_at(Pos::new(0, 0)) == Some(&key!('a', 1, pos!(0, 0))));
            check!(layout.key_at(Pos::new(0, 1)) == Some(&key!('b', 2, pos!(0, 1))));
            check!(layout.key_at(Pos::new(0, 2)) == Some(&key!('c', 3, pos!(0, 2))));
            check!(layout.key_at(Pos::new(1, 0)) == Some(&key!('d', 1, pos!(1, 0))));
            check!(layout.key_at(Pos::new(1, 1)) == Some(&key!('e', 2, pos!(1, 1))));
            check!(layout.key_at(Pos::new(1, 2)) == Some(&key!('f', 3, pos!(1, 2))));
            check!(layout.key_at(Pos::new(2, 0)) == None);
            check!(layout.key_at(Pos::new(0, 3)) == None);
        }

        #[test]
        fn it_returns_key_by_char() {
            let layout = Layout::<2, 2>::new("abcd", vec![vec![1, 2], vec![1, 2]]).unwrap();

            check!(layout.key_for('a') == Some(&key!('a', 1, pos!(0, 0))));
            check!(layout.key_for('b') == Some(&key!('b', 2, pos!(0, 1))));
            check!(layout.key_for('c') == Some(&key!('c', 1, pos!(1, 0))));
            check!(layout.key_for('d') == Some(&key!('d', 2, pos!(1, 1))));
            check!(layout.key_for('e') == None);
        }

        #[test]
        fn it_returns_char_by_pos() {
            let layout = Layout::<2, 2>::new("abcd", vec![vec![1, 2], vec![1, 2]]).unwrap();

            check!(layout.char_at(Pos::new(0, 0)) == Some('a'));
            check!(layout.char_at(Pos::new(0, 1)) == Some('b'));
            check!(layout.char_at(Pos::new(1, 0)) == Some('c'));
            check!(layout.char_at(Pos::new(1, 1)) == Some('d'));
            check!(layout.char_at(Pos::new(2, 0)) == None);
            check!(layout.char_at(Pos::new(0, 2)) == None);
        }

        #[test]
        fn it_sets_char_for_key() {
            let mut layout =
                Layout::<2, 3>::new("abcdef", vec![vec![1, 2, 3], vec![1, 2, 3]]).unwrap();

            check!(layout.set_char(Pos::new(0, 0), 'x').is_ok());
            check!(layout.char_at(Pos::new(0, 0)) == Some('x'));
        }
    }
}
