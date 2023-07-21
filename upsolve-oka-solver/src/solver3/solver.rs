use anyhow::Context;
use common::Problem;
use log::info;
use lyon_geom::Vector;
use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};
use rayon::prelude::*;

use crate::{
    params::Params,
    pretty::pretty,
    solver3::{etx_problem::split_problem, mini_solver::MiniSolver},
};

use super::types::Board;

const D: f64 = 200.;

pub fn solve(problem_id: u32, problem: Problem, num_iter: usize, params: Params) -> Board {
    let mini_problems = split_problem(problem.clone(), D);

    let mut seed = 0;
    let mut rng = SmallRng::seed_from_u64(seed);
    seed += 1;

    let mut ms = (0..problem.musicians.len()).collect::<Vec<_>>();
    ms.shuffle(&mut rng);

    let mut available_musicians = ms
        .chunks((ms.len() + mini_problems.len() - 1) / mini_problems.len())
        .into_iter()
        .map(|v| v.into_iter().map(|&i| i).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    assert_eq!(mini_problems.len(), available_musicians.len());

    let mut initial_locations = vec![None; mini_problems.len()];

    let params = Params {
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

        for mini_board in mini_boards {
            for m in 0..mini_board.musicians().len() {
                if let Some((p, _)) = mini_board.musicians()[m] {
                    board.try_place(m, p.to_point()).unwrap();
                }
            }
        }

        board.hungarian();

        info!(
            "{:>3}%  score: {:>14}",
            ((outer_iter + 1) * 100) / num_outer_iter,
            pretty(board.score() as i64),
        );

        if outer_iter == num_outer_iter - 1 {
            return board;
        }

        // Update available_musicians and initial_locations.
        available_musicians.iter_mut().for_each(|v| v.clear());
        let mut locs = vec![vec![]; mini_problems.len()];
        for (m, p) in board.musicians().iter().enumerate() {
            let (p, _) = p.with_context(|| format!("musician {}", m)).unwrap();

            for (i, mp) in mini_problems.iter().enumerate() {
                let mut cell = mp.problem.stage;
                cell.min += Vector::new(5., 5.);
                cell.max -= Vector::new(5., 5.);

                if cell.contains(p.to_point()) {
                    assert!(!available_musicians.concat().contains(&m));

                    available_musicians[i].push(m);
                    locs[i].push(p);
                    break;
                }
            }
        }
        initial_locations = locs.into_iter().map(|v| Some(v)).collect();
    }

    unreachable!()
}
