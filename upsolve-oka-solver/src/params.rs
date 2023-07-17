use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Params {
    pub placed_musicians_ratio: f64,    // 0.4 - 0.6
    pub important_attendees_ratio: f64, // 0.2 - 0.3
    pub important_musician_range: f64,  // 300 - 500

    pub max_temp: f64, // 1_000_000 - 20_000_000
    pub min_temp: f64, // 0 - 100_000

    pub temp_func_power: f64, // 1.0 - 3.0

    pub max_move_dist: f64, // 40 - 100
    pub min_move_dist: f64, // 1 - 40

    // x -> forbidden.max == stage.max - (stage.max - stage.min) * important_musician_ragnge * x
    pub forbidden_area_coeff: f64, // 0.5 - 1.0

    pub hungarian_rarity: usize, // 1_000_000 - 100_000_000

    pub swap: usize,        // 1 - 20
    pub move_random: usize, // 1 - 20
    pub move_dir: usize,    // 1 - 20
}

impl Default for Params {
    fn default() -> Self {
        Self {
            placed_musicians_ratio: 0.5,
            important_attendees_ratio: 0.2,
            important_musician_range: 300.0,

            max_temp: 10_000_000.0,
            min_temp: 0.0,

            temp_func_power: 2.0,

            max_move_dist: 40.0,
            min_move_dist: 5.0,

            forbidden_area_coeff: 0.5,

            hungarian_rarity: 1_000_000,

            swap: 8,
            move_random: 2,
            move_dir: 10,
        }
    }
}
