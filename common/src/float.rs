pub trait Float: std::fmt::Debug + Clone + Copy + Eq + Ord + From<f64> + Into<f64> {
    fn new(value: f64) -> Self;
    fn get(&self) -> f64;
    const EPS: f64;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct F64 {
    value: i64,
}

impl Float for F64 {
    const EPS: f64 = 1e-12;

    fn new(value: f64) -> Self {
        let mut v = unsafe { std::mem::transmute::<f64, i64>(value) };

        // sign bit is set
        if v < 0 {
            v ^= 0x7fff_ffff_ffff_ffff;
        }

        Self { value: v }
    }

    fn get(&self) -> f64 {
        let mut v = self.value;

        if v < 0 {
            v ^= 0x7fff_ffff_ffff_ffff;
        }

        unsafe { std::mem::transmute::<i64, f64>(v) }
    }
}

impl From<f64> for F64 {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl From<F64> for f64 {
    fn from(value: F64) -> Self {
        value.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct F32 {
    value: i32,
}

impl Float for F32 {
    const EPS: f64 = 1e-6;

    #[inline]
    fn new(value: f64) -> Self {
        let value = value as f32;

        let mut v = unsafe { std::mem::transmute::<f32, i32>(value) };

        // sign bit is set
        if v < 0 {
            v ^= 0x7fff_ffff;
        }

        Self { value: v }
    }

    #[inline]
    fn get(&self) -> f64 {
        let mut v = self.value;

        if v < 0 {
            v ^= 0x7fff_ffff;
        }

        (unsafe { std::mem::transmute::<i32, f32>(v) }) as f64
    }
}

impl From<f64> for F32 {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl From<F32> for f64 {
    fn from(value: F32) -> Self {
        value.get()
    }
}
