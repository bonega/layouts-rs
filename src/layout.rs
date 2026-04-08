use std::collections::{HashMap, HashSet};
use std::fmt;

const NONE_CHAR: char = '_';

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
    pub ch: char,
    pub finger: Finger,
    pub position: Pos,
    pub finger_home: bool,
    pub effort: f64,
}

impl Key {
    pub fn new(ch: char, finger: Finger, position: Pos, effort: f64, finger_home: bool) -> Self {
        Self {
            ch,
            finger,
            position,
            effort,
            finger_home,
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
    pub fn new(
        definition: &str,
        finger_assignment: Vec<Vec<u8>>,
        finger_effort: Vec<Vec<f64>>,
        finger_home_positions: HashMap<u8, Pos>,
    ) -> anyhow::Result<Self> {
        let definition: Vec<Vec<char>> = definition
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<Vec<char>>()
            .chunks(COLUMNS)
            .map(|chunk| chunk.to_vec())
            .collect();

        Self::check_matrix("definition", &definition)?;
        Self::check_matrix("finger assignment", &finger_assignment)?;
        Self::check_matrix("finger effort", &finger_effort)?;
        Self::check_finger_home_positions(&finger_assignment, &finger_home_positions)?;

        let mut keys = Self::default_keys();
        for (row, row_chars) in definition.iter().enumerate() {
            for (column, &ch) in row_chars.iter().enumerate() {
                if ch == NONE_CHAR {
                    continue;
                }

                let finger = Finger::from(finger_assignment[row][column]);
                let finger_value: u8 = finger.into();
                let pos = Pos::new(row, column);
                let effort = finger_effort[row][column];
                let homerow = finger_home_positions
                    .get(&finger_value)
                    .is_some_and(|hp| hp.r == row && hp.c == column);

                keys[row][column] = Some(Key::new(ch, finger, pos, effort, homerow));
            }
        }

        Ok(Self { keys })
    }

    fn check_matrix<T>(matrix_name: &str, matrix: &[Vec<T>]) -> anyhow::Result<()> {
        if matrix.len() != ROWS || matrix.iter().any(|row| row.len() != COLUMNS) {
            anyhow::bail!(
                "expected {matrix_name} to have {ROWS} rows and {COLUMNS} columns, received {} rows and {} columns",
                matrix.len(),
                matrix.iter().map(|row| row.len()).max().unwrap_or(0)
            );
        }

        Ok(())
    }

    fn check_finger_home_positions(
        finger_assignment: &[Vec<u8>],
        finger_home_positions: &HashMap<u8, Pos>,
    ) -> anyhow::Result<()> {
        let fingers_in_layout: HashSet<u8> = finger_assignment
            .iter()
            .flat_map(|row| row.iter().copied())
            .collect();

        for (&finger_value, pos) in finger_home_positions {
            if pos.r >= finger_assignment.len() || pos.c >= finger_assignment[pos.r].len() {
                anyhow::bail!("finger home position {pos:?} is out of bounds");
            }

            let finger_at_pos = finger_assignment[pos.r][pos.c];
            if finger_at_pos != finger_value {
                anyhow::bail!("finger home position {pos:?} does not match finger {finger_value}");
            }
        }

        for finger_value in &fingers_in_layout {
            if !finger_home_positions.contains_key(finger_value) {
                anyhow::bail!("finger {finger_value} does not have an home position");
            }
        }

        Ok(())
    }

    pub fn char_at(&self, position: &Pos) -> Option<char> {
        self.key_at(position).map(|key| key.ch)
    }

    pub fn key_at(&self, position: &Pos) -> Option<&Key> {
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

    pub fn set_char(&mut self, position: &Pos, ch: char) {
        if position.r >= ROWS || position.c >= COLUMNS {
            return;
        }

        if let Some(key) = self.keys[position.r][position.c].as_mut() {
            key.ch = ch;
        }
    }

    pub fn swap_chars(&mut self, pos1: &Pos, pos2: &Pos) {
        let Some(ch1) = self.char_at(pos1) else {
            return;
        };
        let Some(ch2) = self.char_at(pos2) else {
            return;
        };

        self.set_char(pos1, ch2);
        self.set_char(pos2, ch1);
    }

    fn default_keys() -> [[Option<Key>; COLUMNS]; ROWS] {
        [[None; COLUMNS]; ROWS]
    }

    pub fn keys(&self) -> impl Iterator<Item = &Key> {
        self.keys
            .iter()
            .flat_map(|row| row.iter().filter_map(|key| key.as_ref()))
    }
}

impl<const ROWS: usize, const COLUMNS: usize> Default for Layout<ROWS, COLUMNS> {
    fn default() -> Self {
        Self {
            keys: Self::default_keys(),
        }
    }
}

impl<const ROWS: usize, const COLUMNS: usize> std::fmt::Display for Layout<ROWS, COLUMNS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.keys.iter() {
            let left = row[..6]
                .iter()
                .map(|k| k.map(|kk| kk.ch).unwrap_or('_').to_string())
                .collect::<Vec<_>>()
                .join(" ");
            let right = row[6..]
                .iter()
                .map(|k| k.map(|kk| kk.ch).unwrap_or('_').to_string())
                .collect::<Vec<_>>()
                .join(" ");
            writeln!(f, "  {}   {}", left, right)?;
        }
        Ok(())
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
            vec![
                vec![3.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 3.0],
                vec![2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0],
                vec![3.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 3.0],
            ],
            [
                (1, pos!(1, 1)),
                (2, pos!(1, 2)),
                (3, pos!(1, 3)),
                (4, pos!(1, 4)),
                (7, pos!(1, 7)),
                (8, pos!(1, 8)),
                (9, pos!(1, 9)),
                (10, pos!(1, 10)),
            ]
            .into(),
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
            check!(
                Layout::<2, 2>::new(
                    "abcd",
                    vec![vec![1, 2], vec![1, 2]],
                    vec![vec![1.0, 1.0], vec![1.0, 1.0]],
                    [(1, pos!(0, 0)), (2, pos!(0, 1))].into()
                )
                .is_ok()
            );
            check!(
                Layout::<2, 3>::new(
                    "abcdef",
                    vec![vec![1, 2, 3], vec![1, 2, 3]],
                    vec![vec![1.0, 1.0, 1.0], vec![1.0, 1.0, 1.0]],
                    [(1, pos!(0, 0)), (2, pos!(0, 1)), (3, pos!(0, 2))].into()
                )
                .is_ok()
            );
            check!(
                Layout::<2, 2>::new(
                    "aaaa",
                    vec![vec![1, 2], vec![1, 2]],
                    vec![vec![1.0, 1.0], vec![1.0, 1.0]],
                    [(1, pos!(0, 0)), (2, pos!(0, 1))].into()
                )
                .is_ok()
            );

            check!(
                Layout::<2, 2>::new(
                    "abcde",
                    vec![vec![1, 2], vec![1, 2]],
                    vec![vec![1.0, 1.0], vec![1.0, 1.0]],
                    [(1, pos!(0, 0)), (2, pos!(0, 1))].into()
                )
                .is_err()
            );
            check!(
                Layout::<2, 2>::new(
                    "abcd",
                    vec![vec![1, 2], vec![1, 2, 3]],
                    vec![vec![1.0, 1.0], vec![1.0, 1.0]],
                    [(1, pos!(0, 0)), (2, pos!(0, 1))].into()
                )
                .is_err()
            );

            check!(
                Layout::<2, 2>::new(
                    "abcd",
                    vec![vec![1, 2], vec![1, 2]],
                    vec![vec![1.0, 1.0], vec![1.0]],
                    [(1, pos!(0, 0)), (2, pos!(0, 1))].into()
                )
                .is_err()
            );
            check!(
                Layout::<2, 2>::new(
                    "abcd",
                    vec![vec![1, 2], vec![1, 2]],
                    vec![vec![1.0, 1.0], vec![1.0, 1.0]],
                    [].into()
                )
                .is_err()
            );
            check!(
                Layout::<2, 2>::new(
                    "abcd",
                    vec![vec![1, 2], vec![1, 2]],
                    vec![vec![1.0, 1.0], vec![1.0, 1.0]],
                    [(1, pos!(0, 0)), (2, pos!(0, 0))].into()
                )
                .is_err()
            );
            check!(
                Layout::<2, 2>::new(
                    "abcd",
                    vec![vec![1, 2], vec![1, 2]],
                    vec![vec![1.0, 1.0], vec![1.0, 1.0]],
                    [(1, pos!(0, 0)), (2, pos!(1, 0))].into()
                )
                .is_err()
            );
        }

        #[test]
        fn it_builds_with_underscores_as_none() {
            let layout = Layout::<2, 3>::new(
                "_ab_cd",
                vec![vec![1, 1, 2], vec![1, 1, 2]],
                vec![vec![1.0, 1.0, 1.0], vec![1.0, 1.0, 1.0]],
                [(1, pos!(0, 1)), (2, pos!(0, 2))].into(),
            )
            .unwrap();

            check!(layout.key_at(&Pos::new(0, 0)) == None);
            check!(layout.char_at(&Pos::new(0, 1)) == Some('a'));
            check!(layout.char_at(&Pos::new(0, 2)) == Some('b'));
            check!(layout.key_at(&Pos::new(1, 0)) == None);
            check!(layout.char_at(&Pos::new(1, 1)) == Some('c'));
            check!(layout.char_at(&Pos::new(1, 2)) == Some('d'));
            check!(layout.keys().count() == 4);
        }

        #[test]
        fn it_builds_from_string_normalizing() {
            check!(
                Layout::<2, 3>::new(
                    r#"
            a b c
            d e f
            "#,
                    vec![vec![1, 2, 3], vec![1, 2, 3]],
                    vec![vec![1.0, 1.0, 1.0], vec![1.0, 1.0, 1.0]],
                    [(1, pos!(0, 0)), (2, pos!(0, 1)), (3, pos!(0, 2))].into()
                )
                .is_ok()
            );
        }

        #[test]
        fn it_returns_key_by_pos() {
            let layout = Layout::<2, 3>::new(
                "abcdef",
                vec![vec![1, 2, 3], vec![1, 2, 3]],
                vec![vec![1.0, 1.0, 1.0], vec![2.0, 2.0, 2.0]],
                [(1, pos!(0, 0)), (2, pos!(0, 1)), (3, pos!(0, 2))].into(),
            )
            .unwrap();

            check!(
                layout.key_at(&Pos::new(0, 0)) == Some(&finger_home_key!('a', 1, pos!(0, 0), 1.0))
            );
            check!(
                layout.key_at(&Pos::new(0, 1)) == Some(&finger_home_key!('b', 2, pos!(0, 1), 1.0))
            );
            check!(
                layout.key_at(&Pos::new(0, 2)) == Some(&finger_home_key!('c', 3, pos!(0, 2), 1.0))
            );
            check!(layout.key_at(&Pos::new(1, 0)) == Some(&key!('d', 1, pos!(1, 0), 2.0)));
            check!(layout.key_at(&Pos::new(1, 1)) == Some(&key!('e', 2, pos!(1, 1), 2.0)));
            check!(layout.key_at(&Pos::new(1, 2)) == Some(&key!('f', 3, pos!(1, 2), 2.0)));
            check!(layout.key_at(&Pos::new(2, 0)) == None);
            check!(layout.key_at(&Pos::new(0, 3)) == None);
        }

        #[test]
        fn it_returns_key_by_char() {
            let layout = Layout::<2, 2>::new(
                "abcd",
                vec![vec![1, 2], vec![1, 2]],
                vec![vec![1.0, 1.0], vec![1.0, 1.0]],
                [(1, pos!(0, 0)), (2, pos!(0, 1))].into(),
            )
            .unwrap();

            check!(layout.key_for('a') == Some(&finger_home_key!('a', 1, pos!(0, 0))));
            check!(layout.key_for('b') == Some(&finger_home_key!('b', 2, pos!(0, 1))));
            check!(layout.key_for('c') == Some(&key!('c', 1, pos!(1, 0))));
            check!(layout.key_for('d') == Some(&key!('d', 2, pos!(1, 1))));
            check!(layout.key_for('e') == None);
        }

        #[test]
        fn it_returns_char_by_pos() {
            let layout = Layout::<2, 2>::new(
                "abcd",
                vec![vec![1, 2], vec![1, 2]],
                vec![vec![1.0, 1.0], vec![1.0, 1.0]],
                [(1, pos!(0, 0)), (2, pos!(0, 1))].into(),
            )
            .unwrap();

            check!(layout.char_at(&Pos::new(0, 0)) == Some('a'));
            check!(layout.char_at(&Pos::new(0, 1)) == Some('b'));
            check!(layout.char_at(&Pos::new(1, 0)) == Some('c'));
            check!(layout.char_at(&Pos::new(1, 1)) == Some('d'));
            check!(layout.char_at(&Pos::new(2, 0)) == None);
            check!(layout.char_at(&Pos::new(0, 2)) == None);
        }

        #[test]
        fn it_sets_char_for_key() {
            let mut layout = Layout::<2, 2>::new(
                "abcd",
                vec![vec![1, 2], vec![1, 2]],
                vec![vec![1.0, 1.0], vec![1.0, 1.0]],
                [(1, pos!(0, 0)), (2, pos!(0, 1))].into(),
            )
            .unwrap();

            layout.set_char(&Pos::new(0, 0), 'x');
            check!(layout.char_at(&Pos::new(0, 0)) == Some('x'));
        }

        #[test]
        fn it_swaps_chars() {
            let mut layout = Layout::<2, 2>::new(
                "abcd",
                vec![vec![1, 2], vec![1, 2]],
                vec![vec![1.0, 1.0], vec![1.0, 1.0]],
                [(1, pos!(0, 0)), (2, pos!(0, 1))].into(),
            )
            .unwrap();

            layout.swap_chars(&Pos::new(0, 0), &Pos::new(1, 0));
            check!(layout.char_at(&Pos::new(0, 0)) == Some('c'));
            check!(layout.char_at(&Pos::new(1, 0)) == Some('a'));
        }

        #[test]
        fn it_returns_keys() {
            let layout = Layout::<2, 2>::new(
                "abcd",
                vec![vec![1, 2], vec![1, 2]],
                vec![vec![1.0, 1.0], vec![1.0, 1.0]],
                [(1, pos!(0, 0)), (2, pos!(0, 1))].into(),
            )
            .unwrap();

            check!(
                layout.keys().collect::<Vec<_>>()
                    == vec![
                        &finger_home_key!('a', 1, pos!(0, 0)),
                        &finger_home_key!('b', 2, pos!(0, 1)),
                        &key!('c', 1, pos!(1, 0)),
                        &key!('d', 2, pos!(1, 1))
                    ]
            );
        }
    }
}
