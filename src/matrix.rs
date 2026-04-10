use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pos {
    pub r: usize,
    pub c: usize,
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.r, self.c)
    }
}

impl Pos {
    pub fn new(r: usize, c: usize) -> Self {
        Self { r, c }
    }
}

#[derive(Debug, Clone)]
pub struct Matrix<T> {
    pub rows: usize,
    pub columns: usize,
    data: Vec<T>,
}

impl<T> Matrix<T> {
    pub fn new(data: Vec<Vec<T>>) -> anyhow::Result<Self> {
        let rows = data.len();
        let Some(columns) = data.first().map(|r| r.len()) else {
            anyhow::bail!("matrix cannot be empty");
        };

        if data.iter().any(|row| row.len() != columns) {
            anyhow::bail!("matrix has inconsistent row lengths");
        }

        let flat: Vec<T> = data.into_iter().flatten().collect();

        Ok(Self {
            rows,
            columns,
            data: flat,
        })
    }

    pub fn rows_iter(&self) -> impl Iterator<Item = &[T]> {
        self.data.chunks(self.columns)
    }

    pub fn get(&self, position: &Pos) -> Option<&T> {
        (position.r < self.rows && position.c < self.columns)
            .then(|| &self.data[position.r * self.columns + position.c])
    }

    pub fn get_mut(&mut self, position: &Pos) -> Option<&mut T> {
        (position.r < self.rows && position.c < self.columns)
            .then(|| &mut self.data[position.r * self.columns + position.c])
    }
}

impl<T: Clone> Matrix<T> {
    pub fn filled(rows: usize, columns: usize, value: T) -> Self {
        Self {
            rows,
            columns,
            data: vec![value; rows * columns],
        }
    }
}

#[cfg(test)]
mod tests {
    use assert2::check;

    use super::*;

    #[test]
    fn it_validates_correctness_of_input() {
        check!(Matrix::<()>::new(vec![]).is_err());
        check!(Matrix::new(vec![vec![1, 2], vec![3]]).is_err());
        check!(Matrix::new(vec![vec![1, 2], vec![3, 4]]).is_ok());
    }

    #[test]
    fn it_gets_and_sets_values() {
        let mut matrix = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
        check!(matrix.get(&Pos::new(0, 0)) == Some(&1));
        check!(matrix.get(&Pos::new(0, 1)) == Some(&2));
        check!(matrix.get(&Pos::new(1, 0)) == Some(&3));
        check!(matrix.get(&Pos::new(1, 1)) == Some(&4));
        check!(matrix.get(&Pos::new(2, 0)) == None);
        check!(matrix.get_mut(&Pos::new(0, 0)).map(|it| *it = 10).is_some());
        check!(matrix.get(&Pos::new(0, 0)) == Some(&10));
    }

    #[test]
    fn it_fills_with_value() {
        let matrix = Matrix::filled(2, 3, 5);
        check!(matrix.rows == 2);
        check!(matrix.columns == 3);
        check!(matrix.data == vec![5; 6]);
    }
}
