#!/usr/bin/env python3

import json
import multiprocessing
import os
import shutil
import subprocess
import sys
import tempfile

import optuna

TESTING = False

DIVIDE = 10 if TESTING else 1

N_TRIALS = 200 // DIVIDE

N_ITER = 5_000_000 // DIVIDE

PROBLEM_ID = 1

N_JOBS = 14


def objective(trial: optuna.Trial, temp_dir: str, exe: str) -> float:
    temp_dir = os.path.join(temp_dir, str(trial.number))
    os.mkdir(temp_dir)

    params = {
        "placed_musicians_ratio": trial.suggest_float(
            "placed_musicians_ratio", 0.2, 0.5
        ),
        "important_attendees_ratio": trial.suggest_float(
            "important_attendees_ratio", 0.1, 0.2
        ),
        "important_musician_range": (
            trial.suggest_int("important_musician_range", 200, 500, step=10)
        ),
        "max_temp": trial.suggest_int(
            "max_temp", 1_000_000, 20_000_000, step=1_000_000
        ),
        "min_temp": trial.suggest_int("min_temp", 0, 100_000, step=10_000),
        "temp_func_power": trial.suggest_float("temp_func_power", 1.0, 3.0),
        "max_move_dist": trial.suggest_int("max_move_dist", 40, 100),
        "min_move_dist": trial.suggest_int("min_move_dist", 1, 40),
        "forbidden_area_coeff": trial.suggest_float("forbidden_area_coeff", 0.4, 1.0),
        "hungarian_rarity": trial.suggest_int(
            "hungarian_rarity", 1_000_000, 100_000_000, step=1_000_000
        ),
        "swap": trial.suggest_int("swap", 1, 10),
        "move_random": trial.suggest_int("move_random", 1, 10),
        "move_dir": trial.suggest_int("move_dir", 1, 10),
    }

    param_file = os.path.join(temp_dir, "params.json")
    with open(param_file, "w") as f:
        json.dump(params, f)

    output_file = os.path.join(temp_dir, "output.json")

    try:
        subprocess.check_call(
            [
                exe,
                str(PROBLEM_ID),
                "--params",
                param_file,
                "--output",
                output_file,
                "--quiet",
                "-n",
                str(N_ITER),
            ]
        )
        with open(output_file) as f:
            return float(json.load(f)["score"])

    except Exception as e:
        print(f"Exception: {e}", file=sys.stderr)
        return 0.0


def main():
    cur = os.path.dirname(os.path.abspath(__file__))
    os.chdir(os.path.join(cur, ".."))

    with tempfile.TemporaryDirectory() as temp_dir:
        dest = os.path.join(temp_dir, "upsolve-oka-solver")
        subprocess.run(["cargo", "build", "-r", "--bin", "upsolve-oka-solver"])
        shutil.copy("target/release/upsolve-oka-solver", dest)

        study = optuna.create_study(
            direction="maximize",
        )

        study.optimize(
            lambda trial: objective(trial, temp_dir, dest),
            n_trials=N_TRIALS,
            n_jobs=N_JOBS,
        )

        print("Writing params.json")
        with open("upsolve-oka-solver/params.json", "w") as f:
            json.dump(study.best_params, f, indent=4)


if __name__ == "__main__":
    multiprocessing.freeze_support()
    main()
