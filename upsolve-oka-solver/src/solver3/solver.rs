use anyhow::Context;
use common::{Problem, Solution};
use log::info;
use lyon_geom::Size;
use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;

use crate::{
    params::Params,
    pretty::pretty,
    solver3::{
        ext_problem::{split_problem, split_problem_from_cut},
        mini_solver::MiniSolver,
        types::P,
    },
};

use super::types::Board;

const D: f64 = 120.;
const MIN_SIDE: f64 = 80.;

const MOVE_CUT_POSITIONS: bool = false;

const MOVE_CUT_STD_DEV: f64 = 10.0;

pub fn solve(
    problem_id: u32,
    problem: Problem,
    num_iter: usize,
    params: Params,
    initial_solution: Option<Solution>,
) -> Board {
    let mut seed = 0;
    let mut rng = SmallRng::seed_from_u64(seed);
    seed += 1;

    let mut mini_problems = split_problem(problem.clone(), D);

    let corner_stage = mini_problems
        .iter()
        .find(|mp| mp.walls.len() == 2)
        .unwrap()
        .problem
        .stage;

    let last_cut_pos = corner_stage.min + P::new(5., 5.);
    let base_cell_size = corner_stage.size() - Size::new(10., 10.);

    let mut fixed_positions: Vec<(usize, P)> = vec![];

    let mut ms = (0..problem.musicians.len()).collect::<Vec<_>>();
    ms.shuffle(&mut rng);

    // Initialize available musicians and their initial locations par mini problem.

    let (mut available_musicians, mut initial_locations) = if let Some(solution) = initial_solution
    {
        let mut available_musicians = vec![vec![]; mini_problems.len()];
        let mut initial_locations = vec![vec![]; mini_problems.len()];

        let mut used_musicians = vec![false; problem.musicians.len()];
        for (m, p) in solution.placements.into_iter().enumerate() {
            for (i, mp) in mini_problems.iter().enumerate() {
                if !mp.contains(p.position.to_vector()) {
                    continue;
                }
                used_musicians[m] = true;
                available_musicians[i].push(m);
                initial_locations[i].push(p.position.to_vector().into());
            }
        }

        let mut unused = (0..problem.musicians.len())
            .filter(|&m| !used_musicians[m])
            .collect::<Vec<_>>();

        if unused.len() > 0 {
            unused.shuffle(&mut rng);
            unused
                .chunks((unused.len() + mini_problems.len() - 1) / mini_problems.len())
                .into_iter()
                .zip(available_musicians.iter_mut())
                .zip(initial_locations.iter_mut())
                .for_each(|((ms, ams), ils)| {
                    ams.extend_from_slice(ms);
                    ils.extend_from_slice(&ms.iter().map(|_| None).collect::<Vec<_>>());
                });
        }

        (available_musicians, initial_locations)
    } else {
        let mut available_musicians = ms
            .chunks((ms.len() + mini_problems.len() - 1) / mini_problems.len())
            .into_iter()
            .map(|v| v.into_iter().map(|&i| i).collect::<Vec<_>>())
            .collect::<Vec<_>>();

        available_musicians.iter_mut().for_each(|x| x.sort());

        assert_eq!(mini_problems.len(), available_musicians.len());

        let initial_locations = available_musicians
            .iter()
            .map(|ms| ms.iter().map(|_| None).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        (available_musicians, initial_locations)
    };

    info!("parallelism = {}", mini_problems.len());

    let params = Params {
        placed_musicians_ratio: 1.0,
        important_attendees_ratio: 1.0,
        important_musician_range: f64::INFINITY,
        ..params
    };

    let num_outer_iter = 100;
    let num_inner_iter = num_iter / num_outer_iter;

    for outer_iter in 0..num_outer_iter {
        let inner_iter_range = num_inner_iter * outer_iter..num_inner_iter * (outer_iter + 1);

        let mut solvers = vec![];
        for i in 0..mini_problems.len() {
            let solver = MiniSolver::new(
                problem_id,
                mini_problems[i].clone(),
                num_iter,
                inner_iter_range.clone(),
                params.clone(),
                available_musicians[i].clone(),
                initial_locations[i].clone(),
                seed,
                false,
            );
            seed += 1;

            solvers.push(solver);
        }

        let mini_boards = solvers
            .par_iter_mut()
            .map(|solver| solver.solve())
            .collect::<Vec<_>>();

        let mut board = Board::new(problem_id, problem.clone(), "upsolver-oka-solver3", false);
        for m in 0..board.musicians().len() {
            board.set_volume(m, 10.0);
        }

        for mini_board in mini_boards {
            for m in 0..mini_board.musicians().len() {
                if let Some((p, _)) = mini_board.musicians()[m] {
                    board.try_place(m, p.to_point()).unwrap();
                }
            }
        }

        // Set fixed positions.
        for (m, p) in fixed_positions.iter() {
            board.try_place(*m, p.to_point()).unwrap();
        }

        board.hungarian();

        info!(
            "{:>3}%  score: {:>14}",
            ((outer_iter + 1) * 100) / num_outer_iter,
            pretty(board.score() as i64),
        );

        // Handle zero scores.
        {
            let mut zero_score_points = vec![];
            let mut zero_score_musicians = vec![];
            for m in 0..board.musicians().len() {
                if let Some((p, _)) = board.musicians()[m] {
                    if board.contribution2(m) <= 0. {
                        zero_score_musicians.push(m);
                        zero_score_points.push(Some(p));
                        board.unplace(m);
                    }
                } else {
                    zero_score_musicians.push(m);
                    zero_score_points.push(None);
                }
            }
            zero_score_musicians.shuffle(&mut rng);

            let mut solvers_iter_order = (0..solvers.len()).collect::<Vec<_>>();
            solvers_iter_order.shuffle(&mut rng);

            for i in 0..zero_score_musicians.len() {
                if let Some(p) = zero_score_points[i] {
                    board
                        .try_place(zero_score_musicians[i], p.to_point())
                        .unwrap();
                }
            }

            for i in 0..zero_score_musicians.len() {
                let m = zero_score_musicians[i];

                if zero_score_points[i].is_none() {
                    'outer: loop {
                        for j in solvers_iter_order.iter() {
                            let p = solvers[*j].random_place();

                            let score = board.score();
                            if board
                                .try_place(zero_score_musicians[i], p.to_point())
                                .is_ok()
                            {
                                if board.score() >= score {
                                    break 'outer;
                                } else {
                                    board.unplace(m);
                                }
                            }
                        }
                    }
                }
            }
        }

        if outer_iter == num_outer_iter - 1 {
            return board;
        }

        // Update mini problems.
        let num_x_cut = mini_problems.iter().filter(|prob| prob.is_top).count() - 1;
        let num_y_cut = mini_problems.iter().filter(|prob| prob.is_right).count() - 1;

        let mut x_cut = vec![];
        let mut y_cut = vec![];

        for (min, cut, n, last, base_step) in [
            (
                problem.stage.min.x + 5.0,
                &mut x_cut,
                num_x_cut,
                last_cut_pos.x,
                base_cell_size.width,
            ),
            (
                problem.stage.min.y + 5.0,
                &mut y_cut,
                num_y_cut,
                last_cut_pos.y,
                base_cell_size.height,
            ),
        ] {
            loop {
                cut.clear();
                cut.push(min);
                for i in 0..n - 1 {
                    let c = min + base_step * (i + 1) as f64;

                    let d = if MOVE_CUT_POSITIONS {
                        Normal::new(c, MOVE_CUT_STD_DEV).unwrap().sample(&mut rng)
                    } else {
                        c
                    };

                    cut.push(d);
                }
                cut.push(last);

                cut.sort_by(|x, y| x.partial_cmp(&y).unwrap());

                let mut ok = true;
                for i in 0..cut.len() - 1 {
                    let d = cut[i + 1] - cut[i];
                    if d < MIN_SIDE {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    cut.remove(0);
                    break;
                }
            }
        }

        let pillar_cands = if MOVE_CUT_POSITIONS {
            board
                .musicians()
                .clone()
                .into_iter()
                .filter_map(|x| x.map(|x| x.0))
                .collect::<Vec<_>>()
                .into()
        } else {
            None
        };

        let mini_problems_len = mini_problems.len();

        mini_problems = split_problem_from_cut(problem.clone(), x_cut, y_cut, pillar_cands);

        assert_eq!(mini_problems_len, mini_problems.len());

        // Update available_musicians, initial_locations, fixed_positions.
        available_musicians.iter_mut().for_each(|v| v.clear());
        initial_locations.iter_mut().for_each(|v| v.clear());
        fixed_positions.clear();
        for (m, p) in board.musicians().iter().enumerate() {
            let (p, _) = p.with_context(|| format!("musician {}", m)).unwrap();

            let mut is_fixed = true;
            for (i, mp) in mini_problems.iter().enumerate() {
                if mp.contains(p) {
                    debug_assert!(!available_musicians.concat().contains(&m));

                    is_fixed = false;

                    available_musicians[i].push(m);
                    initial_locations[i].push(p.into());
                    break;
                }
            }

            if is_fixed && MOVE_CUT_POSITIONS {
                fixed_positions.push((m, p.into()));
            }
        }
    }

    unreachable!()
}
