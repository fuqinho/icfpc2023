#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct F64 {
    value: i64,
}

impl F64 {
    pub fn new(value: f64) -> Self {
        let mut v = unsafe { std::mem::transmute::<f64, i64>(value) };

        // sign bit is set
        if v < 0 {
            v ^= 0x7fff_ffff_ffff_ffff;
        }

        Self { value: v }
    }

    pub fn get(&self) -> f64 {
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
