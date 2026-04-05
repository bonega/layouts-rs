use crate::layout::{Layout, Pos};

#[derive(Clone, Debug, PartialEq)]
pub enum SwapMove {
    Single(Pos, Pos),
    Column(Vec<(Pos, Pos)>),
}

impl SwapMove {
    pub fn single_moves(positions: &[Pos]) -> Vec<Self> {
        let mut moves = Vec::new();
        for (i, &p1) in positions.iter().enumerate() {
            for &p2 in positions.iter().skip(i + 1) {
                moves.push(Self::Single(p1, p2));
            }
        }
        moves
    }

    pub fn column_moves(positions: &[Pos]) -> Vec<Self> {
        use std::collections::BTreeMap;

        let mut cols: BTreeMap<usize, Vec<Pos>> = BTreeMap::new();
        for &pos in positions {
            cols.entry(pos.c).or_default().push(pos);
        }

        let col_keys: Vec<usize> = cols.keys().copied().collect();
        let mut moves = Vec::new();

        for (i, &c1) in col_keys.iter().enumerate() {
            for &c2 in col_keys.iter().skip(i + 1) {
                let col1 = &cols[&c1];
                let col2 = &cols[&c2];
                let pairs: Vec<(Pos, Pos)> =
                    col1.iter().copied().zip(col2.iter().copied()).collect();
                if !pairs.is_empty() {
                    moves.push(Self::Column(pairs));
                }
            }
        }

        moves
    }

    pub fn apply<const C: usize, const R: usize>(&self, layout: &mut Layout<C, R>) {
        match self {
            SwapMove::Single(p1, p2) => {
                layout.swap_chars(p1, p2);
            }
            SwapMove::Column(pairs) => {
                for &(p1, p2) in pairs {
                    layout.swap_chars(&p1, &p2);
                }
            }
        }
    }
}

#[cfg(test)]
mod single_moves_tests {
    use assert2::check;

    use super::*;

    #[test]
    fn it_builds() {
        let positions = vec![pos!(0, 0), pos!(1, 0), pos!(0, 1)];
        let swap_moves = SwapMove::single_moves(&positions);
        check!(
            swap_moves
                == vec![
                    SwapMove::Single(pos!(0, 0), pos!(1, 0)),
                    SwapMove::Single(pos!(0, 0), pos!(0, 1)),
                    SwapMove::Single(pos!(1, 0), pos!(0, 1)),
                ]
        );
    }

    #[test]
    fn it_builds_from_single_position() {
        let swap_moves = SwapMove::single_moves(&[pos!(0, 0)]);
        check!(swap_moves == vec![]);
    }

    #[test]
    fn it_builds_from_empty() {
        let swap_moves = SwapMove::single_moves(&[]);
        check!(swap_moves == vec![]);
    }

    #[test]
    fn it_applies() {
        let mut layout = Layout::<2, 2>::new(
            "abcd",
            vec![vec![1, 2], vec![1, 2]],
            vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            vec![pos!(0, 0), pos!(0, 1)],
        )
        .unwrap();

        let swap = SwapMove::Single(pos!(0, 0), pos!(1, 0));
        swap.apply(&mut layout);

        check!(layout.key_for('a').unwrap().position == pos!(1, 0));
        check!(layout.key_for('c').unwrap().position == pos!(0, 0));
    }

    #[test]
    fn it_reverts_when_applied_twice() {
        let original = Layout::<2, 2>::new(
            "abcd",
            vec![vec![1, 2], vec![1, 2]],
            vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            vec![pos!(0, 0), pos!(0, 1)],
        )
        .unwrap();

        let mut layout = original;
        let swap = SwapMove::Single(pos!(0, 0), pos!(1, 0));
        swap.apply(&mut layout);
        swap.apply(&mut layout);

        check!(layout.key_for('a').unwrap().position == pos!(0, 0));
        check!(layout.key_for('c').unwrap().position == pos!(1, 0));
    }
}

#[cfg(test)]
mod column_moves_tests {
    use assert2::check;

    use super::*;

    #[test]
    fn it_builds() {
        let positions = vec![pos!(0, 0), pos!(1, 0), pos!(0, 1), pos!(1, 1)];
        let col_moves = SwapMove::column_moves(&positions);
        check!(
            col_moves
                == vec![SwapMove::Column(vec![
                    (pos!(0, 0), pos!(0, 1)),
                    (pos!(1, 0), pos!(1, 1)),
                ])]
        );
    }

    #[test]
    fn it_builds_with_n_columns() {
        let positions = vec![
            pos!(0, 0),
            pos!(1, 0),
            pos!(0, 1),
            pos!(1, 1),
            pos!(0, 2),
            pos!(1, 2),
        ];
        let col_moves = SwapMove::column_moves(&positions);
        check!(
            col_moves
                == vec![
                    SwapMove::Column(vec![(pos!(0, 0), pos!(0, 1)), (pos!(1, 0), pos!(1, 1))]),
                    SwapMove::Column(vec![(pos!(0, 0), pos!(0, 2)), (pos!(1, 0), pos!(1, 2))]),
                    SwapMove::Column(vec![(pos!(0, 1), pos!(0, 2)), (pos!(1, 1), pos!(1, 2))]),
                ]
        );
    }

    #[test]
    fn it_builds_with_n_rows() {
        let positions = vec![
            pos!(0, 0),
            pos!(1, 0),
            pos!(2, 0),
            pos!(0, 1),
            pos!(1, 1),
            pos!(2, 1),
        ];
        let col_moves = SwapMove::column_moves(&positions);
        check!(
            col_moves
                == vec![SwapMove::Column(vec![
                    (pos!(0, 0), pos!(0, 1)),
                    (pos!(1, 0), pos!(1, 1)),
                    (pos!(2, 0), pos!(2, 1)),
                ])]
        );
    }

    #[test]
    fn it_builds_zips_to_shorter_column() {
        let positions = vec![pos!(0, 0), pos!(1, 0), pos!(2, 0), pos!(0, 1), pos!(1, 1)];
        let col_moves = SwapMove::column_moves(&positions);
        check!(
            col_moves
                == vec![SwapMove::Column(vec![
                    (pos!(0, 0), pos!(0, 1)),
                    (pos!(1, 0), pos!(1, 1)),
                ])]
        );
    }

    #[test]
    fn it_builds_from_single_column() {
        let positions = vec![pos!(0, 0), pos!(1, 0)];
        let col_moves = SwapMove::column_moves(&positions);
        check!(col_moves == vec![]);
    }

    #[test]
    fn it_builds_from_empty() {
        let col_moves = SwapMove::column_moves(&[]);
        check!(col_moves == vec![]);
    }

    #[test]
    fn it_applies() {
        let mut layout = Layout::<2, 2>::new(
            "abcd",
            vec![vec![1, 2], vec![1, 2]],
            vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            vec![pos!(0, 0), pos!(0, 1)],
        )
        .unwrap();

        let swap = SwapMove::Column(vec![(pos!(0, 0), pos!(0, 1)), (pos!(1, 0), pos!(1, 1))]);
        swap.apply(&mut layout);

        check!(layout.key_for('a').unwrap().position == pos!(0, 1));
        check!(layout.key_for('b').unwrap().position == pos!(0, 0));
        check!(layout.key_for('c').unwrap().position == pos!(1, 1));
        check!(layout.key_for('d').unwrap().position == pos!(1, 0));
    }

    #[test]
    fn it_reverts_when_applied_twice() {
        let original = Layout::<2, 2>::new(
            "abcd",
            vec![vec![1, 2], vec![1, 2]],
            vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            vec![pos!(0, 0), pos!(0, 1)],
        )
        .unwrap();

        let mut layout = original;
        let swap = SwapMove::Column(vec![(pos!(0, 0), pos!(0, 1)), (pos!(1, 0), pos!(1, 1))]);
        swap.apply(&mut layout);
        swap.apply(&mut layout);

        check!(layout.key_for('a').unwrap().position == pos!(0, 0));
        check!(layout.key_for('b').unwrap().position == pos!(0, 1));
        check!(layout.key_for('c').unwrap().position == pos!(1, 0));
        check!(layout.key_for('d').unwrap().position == pos!(1, 1));
    }
}
