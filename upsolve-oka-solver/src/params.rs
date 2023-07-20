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

    pub v2_unplace: usize,  // 1 - 20
    pub v2_place: usize,    // 1 - 20
    pub v2_move_dir: usize, // 1 - 20
    pub v2_swap: usize,     // 1 - 20
}
