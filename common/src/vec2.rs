#[derive(Clone, Debug)]
pub struct Vec2<T> {
    n: usize,
    m: usize,
    // i -> vec[i/m][i%m]
    v: Vec<T>,
}

impl<T> Vec2<T> {
    pub fn new(n: usize, m: usize, t: T) -> Self
    where
        T: Clone,
    {
        let v = vec![t; n * m];

        Self { n, m, v }
    }

    pub fn get(&self, i: usize, j: usize) -> &T {
        debug_assert!(i < self.n && j < self.m);

        &self.v[i * self.m + j]
    }

    pub fn get_mut(&mut self, i: usize, j: usize) -> &mut T {
        debug_assert!(i < self.n && j < self.m);

        &mut self.v[i * self.m + j]
    }

    pub fn set(&mut self, i: usize, j: usize, t: T) {
        debug_assert!(i < self.n && j < self.m);

        self.v[i * self.m + j] = t;
    }

    pub fn len1(&self) -> usize {
        self.n
    }

    pub fn len2(&self) -> usize {
        self.m
    }

    pub fn row(&self, i: usize) -> &[T] {
        debug_assert!(i < self.n);

        &self.v[i * self.m..(i + 1) * self.m]
    }

    pub fn row_mut(&mut self, i: usize) -> &mut [T] {
        debug_assert!(i < self.n);

        &mut self.v[i * self.m..(i + 1) * self.m]
    }

    pub fn swap(&mut self, i1: usize, j1: usize, i2: usize, j2: usize) {
        let x = i1 * self.m + j1;
        let y = i2 * self.m + j2;
        self.v.swap(x, y)
    }
}
