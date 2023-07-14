#[derive(Debug, Clone)]
pub struct BoardOptions {
    pub(crate) important_attendees_ratio: f64,
    // pub(crate) important_musicians_ratio: f64,
}

impl Default for BoardOptions {
    fn default() -> Self {
        Self {
            important_attendees_ratio: 1.0,
            // important_musicians_ratio: 1.0,
        }
    }
}

impl BoardOptions {
    pub fn with_important_attendees_ratio(mut self, ratio: f64) -> Self {
        self.important_attendees_ratio = ratio;
        self
    }

    // pub fn with_important_musicians_ratio(mut self, ratio: f64) -> Self {
    //     self.important_musicians_ratio = ratio;
    //     self
    // }
}
