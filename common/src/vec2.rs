#[derive(Clone, Debug)]
pub struct Vec2<T> {
    n: usize,
    m: usize,
    // i -> vec[i/m][i%m]
    v: Vec<T>,

    lens: Vec<usize>,
}

impl<T> Vec2<T> {
    pub fn new_with_fill(n: usize, m: usize, t: T) -> Self
    where
        T: Clone,
    {
        let v = vec![t; n * m];
        let lens = vec![m; n];

        Self { n, m, v, lens }
    }

    pub fn new(n: usize, m: usize) -> Self
    where
        T: Default + Clone,
    {
        let v = vec![Default::default(); n * m];
        let lens = vec![0; n];

        Self { n, m, v, lens }
    }

    pub fn get(&self, i: usize, j: usize) -> &T {
        debug_assert!(i < self.n && j < self.row_len(i));

        &self.v[i * self.m + j]
    }

    pub fn get_mut(&mut self, i: usize, j: usize) -> &mut T {
        debug_assert!(i < self.n && j < self.row_len(i));

        &mut self.v[i * self.m + j]
    }

    pub fn set(&mut self, i: usize, j: usize, t: T) {
        debug_assert!(i < self.n && j < self.row_len(i));

        self.v[i * self.m + j] = t;
    }

    pub fn clear(&mut self, i: usize) {
        self.lens[i] = 0;
    }

    pub fn push(&mut self, i: usize, t: T) {
        debug_assert!(i < self.n && self.lens[i] < self.m);

        self.lens[i] += 1;
        self.set(i, self.lens[i] - 1, t);
    }

    pub fn num_rows(&self) -> usize {
        self.n
    }

    pub fn row_len(&self, i: usize) -> usize {
        self.lens[i]
    }

    pub fn row_capacity(&self) -> usize {
        self.m
    }

    pub fn row(&self, i: usize) -> &[T] {
        debug_assert!(i < self.n);

        &self.v[i * self.m..i * self.m + self.lens[i]]
    }

    pub fn row_mut(&mut self, i: usize) -> &mut [T] {
        debug_assert!(i < self.n);

        &mut self.v[i * self.m..i * self.m + self.lens[i]]
    }

    pub fn swap_rows(&mut self, i1: usize, i2: usize) {
        let l = self.row_len(i1).max(self.row_len(i2));

        for j in 0..l {
            self.v.swap(i1 * self.m + j, i2 * self.m + j);
        }
        self.lens.swap(i1, i2);
    }

    pub fn swap(&mut self, i1: usize, j1: usize, i2: usize, j2: usize) {
        debug_assert!(i1 < self.n && j1 < self.row_len(i1));
        debug_assert!(i2 < self.n && j2 < self.row_len(i2));

        let x = i1 * self.m + j1;
        let y = i2 * self.m + j2;

        self.v.swap(x, y)
    }
}
