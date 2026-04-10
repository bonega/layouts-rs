use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Deserializer};

use crate::matrix::{Matrix, Pos};

const NONE_CHAR: char = '_';

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

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub finger_assignment: Matrix<u8>,
    pub finger_effort: Matrix<f64>,
    #[serde(deserialize_with = "deserialize_finger_home_positions")]
    pub finger_home_positions: HashMap<u8, Pos>,
}

fn deserialize_finger_home_positions<'de, D>(deserializer: D) -> Result<HashMap<u8, Pos>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: HashMap<u8, [usize; 2]> = HashMap::deserialize(deserializer)?;
    Ok(raw
        .into_iter()
        .map(|(k, [row, col])| (k, Pos::new(row, col)))
        .collect())
}

#[derive(Clone)]
pub struct Layout {
    keys: Matrix<Option<Key>>,
}

impl Layout {
    pub fn new(definition: &str, config: &Config) -> anyhow::Result<Self> {
        let definition = Matrix::new(
            definition
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(|line| line.chars().filter(|c| !c.is_whitespace()).collect())
                .collect(),
        )?;

        Self::check_matrix("finger assignment", &definition, &config.finger_assignment)?;
        Self::check_matrix("finger effort", &definition, &config.finger_effort)?;
        Self::check_finger_home_positions(
            &definition,
            &config.finger_assignment,
            &config.finger_home_positions,
        )?;

        let mut keys = Matrix::filled(definition.rows, definition.columns, None);

        for r in 0..definition.rows {
            for c in 0..definition.columns {
                let pos = Pos::new(r, c);

                let ch = *definition.get(&pos).unwrap();
                let finger_index = *config.finger_assignment.get(&pos).unwrap();

                if finger_index == 0 || ch == NONE_CHAR {
                    continue;
                }

                let finger = Finger::from(finger_index);
                let effort = *config.finger_effort.get(&pos).unwrap();
                let finger_home = config
                    .finger_home_positions
                    .get(&finger.into())
                    .is_some_and(|hp| hp.r == r && hp.c == c);
                let key = Key::new(ch, finger, pos, effort, finger_home);

                *keys.get_mut(&pos).unwrap() = Some(key);
            }
        }

        Ok(Self { keys })
    }

    fn check_matrix<T, Z>(
        matrix_name: &str,
        matrix_to_check: &Matrix<T>,
        matrix_reference: &Matrix<Z>,
    ) -> anyhow::Result<()> {
        if matrix_to_check.rows != matrix_reference.rows
            || matrix_to_check.columns != matrix_reference.columns
        {
            anyhow::bail!(
                "expected {matrix_name} to have {} rows and {} columns, received {} rows and {} columns",
                matrix_reference.rows,
                matrix_reference.columns,
                matrix_to_check.rows,
                matrix_to_check.columns
            );
        }

        Ok(())
    }

    fn check_finger_home_positions(
        definition: &Matrix<char>,
        finger_assignment: &Matrix<u8>,
        finger_home_positions: &HashMap<u8, Pos>,
    ) -> anyhow::Result<()> {
        let mut positions_by_finger: HashMap<u8, Vec<(Pos, char)>> = HashMap::new();
        for r in 0..finger_assignment.rows {
            for c in 0..finger_assignment.columns {
                let pos = Pos::new(r, c);

                let f = *finger_assignment
                    .get(&pos)
                    .ok_or_else(|| anyhow::anyhow!("finger assignment out of bounds at {pos}"))?;

                if f == 0 {
                    continue;
                }

                let ch = *definition
                    .get(&pos)
                    .ok_or_else(|| anyhow::anyhow!("definition out of bounds at {pos}"))?;

                positions_by_finger
                    .entry(f)
                    .or_default()
                    .push((Pos::new(r, c), ch));
            }
        }

        for (&finger_value, pos) in finger_home_positions {
            let actual = *finger_assignment
                .get(pos)
                .ok_or_else(|| anyhow::anyhow!("home position out of bounds at {pos}"))?;
            if actual != finger_value {
                anyhow::bail!("finger home position at {pos} does not match finger {finger_value}");
            }
        }

        for &finger_value in positions_by_finger.keys() {
            if !finger_home_positions.contains_key(&finger_value) {
                anyhow::bail!("finger {finger_value} does not have a home position");
            }
        }

        for (&finger_value, home_pos) in finger_home_positions {
            let home_char = *definition
                .get(home_pos)
                .ok_or_else(|| anyhow::anyhow!("home position out of bounds at {home_pos}"))?;
            if home_char != NONE_CHAR {
                continue;
            }

            if let Some(positions) = positions_by_finger.get(&finger_value) {
                for (pos, ch) in positions {
                    if (pos.r != home_pos.r || pos.c != home_pos.c) && *ch != NONE_CHAR {
                        anyhow::bail!(
                            "finger {finger_value} has empty home at {home_pos} but non-empty key '{ch}' at {pos}",
                        );
                    }
                }
            }
        }

        Ok(())
    }

    pub fn char_at(&self, pos: &Pos) -> Option<char> {
        self.key_at(pos).map(|key| key.ch)
    }

    pub fn key_at(&self, pos: &Pos) -> Option<&Key> {
        self.keys.get(pos).and_then(|k| k.as_ref())
    }

    pub fn key_for(&self, ch: char) -> Option<&Key> {
        self.keys().find(|key| key.ch == ch)
    }

    pub fn set_char(&mut self, pos: &Pos, ch: char) {
        if let Some(Some(key)) = self.keys.get_mut(pos) {
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

    pub fn keys(&self) -> impl Iterator<Item = &Key> {
        self.keys
            .rows_iter()
            .flat_map(|row| row.iter().filter_map(|key| key.as_ref()))
    }
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let split = self.keys.columns / 2;
        for row in self.keys.rows_iter() {
            let left = row[..split]
                .iter()
                .map(|k| k.map(|kk| kk.ch).unwrap_or(NONE_CHAR).to_string())
                .collect::<Vec<_>>()
                .join(" ");
            let right = row[split..]
                .iter()
                .map(|k| k.map(|kk| kk.ch).unwrap_or(NONE_CHAR).to_string())
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
    pub fn qwerty() -> Layout {
        Layout::new(
            r#"
            _ q w e r t   y u i o p _
            " a s d f g   h j k l ; '
            _ z x c v b   n m , . / _
            "#,
            &Config {
                finger_assignment: matrix!([
                    [1, 1, 2, 3, 4, 4, 7, 7, 8, 9, 10, 10],
                    [1, 1, 2, 3, 4, 4, 7, 7, 8, 9, 10, 10],
                    [1, 1, 2, 3, 4, 4, 7, 7, 8, 9, 10, 10],
                ]),
                finger_effort: matrix!([
                    [3.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 3.0],
                    [2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0],
                    [3.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 3.0],
                ]),
                finger_home_positions: [
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
            },
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
                Layout::new(
                    "ab\ncd",
                    &Config {
                        finger_assignment: matrix!([[1, 2], [1, 2]]),
                        finger_effort: matrix!([[1.0, 1.0], [1.0, 1.0]]),
                        finger_home_positions: [(1, pos!(0, 0)), (2, pos!(0, 1))].into()
                    }
                )
                .is_ok()
            );
            check!(
                Layout::new(
                    "abc\ndef",
                    &Config {
                        finger_assignment: matrix!([[1, 2, 3], [1, 2, 3]]),
                        finger_effort: matrix!([[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]]),
                        finger_home_positions: [(1, pos!(0, 0)), (2, pos!(0, 1)), (3, pos!(0, 2))]
                            .into()
                    }
                )
                .is_ok()
            );
            check!(
                Layout::new(
                    "aa\naa",
                    &Config {
                        finger_assignment: matrix!([[1, 2], [1, 2]]),
                        finger_effort: matrix!([[1.0, 1.0], [1.0, 1.0]]),
                        finger_home_positions: [(1, pos!(0, 0)), (2, pos!(0, 1))].into()
                    }
                )
                .is_ok()
            );

            check!(
                Layout::new(
                    "abcde",
                    &Config {
                        finger_assignment: matrix!([[1, 2], [1, 2]]),
                        finger_effort: matrix!([[1.0, 1.0], [1.0, 1.0]]),
                        finger_home_positions: [(1, pos!(0, 0)), (2, pos!(0, 1))].into()
                    }
                )
                .is_err()
            );
            check!(
                Layout::new(
                    "ab\ncd",
                    &Config {
                        finger_assignment: matrix!([[1, 2, 3], [1, 2, 3]]),
                        finger_effort: matrix!([[1.0, 1.0], [1.0, 1.0]]),
                        finger_home_positions: [(1, pos!(0, 0)), (2, pos!(0, 1))].into()
                    }
                )
                .is_err()
            );
            check!(
                Layout::new(
                    "ab\ncd",
                    &Config {
                        finger_assignment: matrix!([[1, 2], [1, 2]]),
                        finger_effort: matrix!([[1.0, 1.0], [1.0, 1.0]]),
                        finger_home_positions: [].into()
                    }
                )
                .is_err()
            );
            check!(
                Layout::new(
                    "ab\ncd",
                    &Config {
                        finger_assignment: matrix!([[1, 2], [1, 2]]),
                        finger_effort: matrix!([[1.0, 1.0], [1.0, 1.0]]),
                        finger_home_positions: [(1, pos!(0, 0)), (2, pos!(0, 0))].into()
                    }
                )
                .is_err()
            );
            check!(
                Layout::new(
                    "ab\ncd",
                    &Config {
                        finger_assignment: matrix!([[1, 2], [1, 2]]),
                        finger_effort: matrix!([[1.0, 1.0], [1.0, 1.0]]),
                        finger_home_positions: [(1, pos!(0, 0)), (2, pos!(1, 0))].into()
                    }
                )
                .is_err()
            );
            check!(
                Layout::new(
                    "_b\ncd",
                    &Config {
                        finger_assignment: matrix!([[1, 2], [1, 2]]),
                        finger_effort: matrix!([[1.0, 1.0], [1.0, 1.0]]),
                        finger_home_positions: [(1, pos!(0, 0)), (2, pos!(0, 1))].into()
                    }
                )
                .is_err()
            );
        }

        #[test]
        fn it_builds_with_underscores_as_none() {
            let layout = Layout::new(
                "_ab\n_cd",
                &Config {
                    finger_assignment: matrix!([[1, 1, 2], [1, 1, 2]]),
                    finger_effort: matrix!([[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]]),
                    finger_home_positions: [(1, pos!(0, 1)), (2, pos!(0, 2))].into(),
                },
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
                Layout::new(
                    r#"
            a b c
            d e f
            "#,
                    &Config {
                        finger_assignment: matrix!([[1, 2, 3], [1, 2, 3]]),
                        finger_effort: matrix!([[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]]),
                        finger_home_positions: [(1, pos!(0, 0)), (2, pos!(0, 1)), (3, pos!(0, 2))]
                            .into()
                    }
                )
                .is_ok()
            );
        }

        #[test]
        fn it_returns_key_by_pos() {
            let layout = Layout::new(
                "abc\ndef",
                &Config {
                    finger_assignment: matrix!([[1, 2, 3], [1, 2, 3]]),
                    finger_effort: matrix!([[1.0, 1.0, 1.0], [2.0, 2.0, 2.0]]),
                    finger_home_positions: [(1, pos!(0, 0)), (2, pos!(0, 1)), (3, pos!(0, 2))]
                        .into(),
                },
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
            let layout = Layout::new(
                "ab\ncd",
                &Config {
                    finger_assignment: matrix!([[1, 2], [1, 2]]),
                    finger_effort: matrix!([[1.0, 1.0], [1.0, 1.0]]),
                    finger_home_positions: [(1, pos!(0, 0)), (2, pos!(0, 1))].into(),
                },
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
            let layout = Layout::new(
                "ab\ncd",
                &Config {
                    finger_assignment: matrix!([[1, 2], [1, 2]]),
                    finger_effort: matrix!([[1.0, 1.0], [1.0, 1.0]]),
                    finger_home_positions: [(1, pos!(0, 0)), (2, pos!(0, 1))].into(),
                },
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
            let mut layout = Layout::new(
                "ab\ncd",
                &Config {
                    finger_assignment: matrix!([[1, 2], [1, 2]]),
                    finger_effort: matrix!([[1.0, 1.0], [1.0, 1.0]]),
                    finger_home_positions: [(1, pos!(0, 0)), (2, pos!(0, 1))].into(),
                },
            )
            .unwrap();

            layout.set_char(&Pos::new(0, 0), 'x');
            check!(layout.char_at(&Pos::new(0, 0)) == Some('x'));
        }

        #[test]
        fn it_swaps_chars() {
            let mut layout = Layout::new(
                "ab\ncd",
                &Config {
                    finger_assignment: matrix!([[1, 2], [1, 2]]),
                    finger_effort: matrix!([[1.0, 1.0], [1.0, 1.0]]),
                    finger_home_positions: [(1, pos!(0, 0)), (2, pos!(0, 1))].into(),
                },
            )
            .unwrap();

            layout.swap_chars(&Pos::new(0, 0), &Pos::new(1, 0));
            check!(layout.char_at(&Pos::new(0, 0)) == Some('c'));
            check!(layout.char_at(&Pos::new(1, 0)) == Some('a'));
        }

        #[test]
        fn it_returns_keys() {
            let layout = Layout::new(
                "ab\ncd",
                &Config {
                    finger_assignment: matrix!([[1, 2], [1, 2]]),
                    finger_effort: matrix!([[1.0, 1.0], [1.0, 1.0]]),
                    finger_home_positions: [(1, pos!(0, 0)), (2, pos!(0, 1))].into(),
                },
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
