import clsx from "clsx";
import { useCallback, useState } from "react";
import { Problem, Solution } from "./problems";

interface VisualizerAnnealerProps {
  problem: Problem;
  solution: Solution | null;
  setRawSolution: (s: string) => void;
}

async function doAnnealing(
  problem: Problem,
  solution: Solution | null,
  setRawSolution: (s: string) => void,
  temp: number,
): Promise<void> {
  if (!solution) {
    return;
  }

  const wasm = await import("wasm");
  wasm.init_panic_hook();
  const problemHandle = wasm.ProblemHandle.from_json(JSON.stringify(problem));
  const solutionHandle = wasm.SolutionHandle.from_json(
    JSON.stringify(solution),
  );
  const newSolutionHandle = wasm.perform_annealing(
    problemHandle,
    solutionHandle,
    temp,
    0.2,
    BigInt(Math.floor(Math.random() * 1000000000000)),
  );
  setRawSolution(newSolutionHandle.as_json());
}

export default function VisualizerAnnealer({
  problem,
  solution,
  setRawSolution,
}: VisualizerAnnealerProps) {
  const [temp, setTemp] = useState("1.0");

  const maybeSetTemp = useCallback(
    (newTemp: string) => {
      if (newTemp !== "" && !Number.isNaN(Number(newTemp))) {
        setTemp(newTemp);
      }
    },
    [setTemp],
  );

  const onClick = useCallback(async () => {
    await doAnnealing(problem, solution, setRawSolution, Number(temp));
  }, [problem, solution, setRawSolution, temp]);

  return (
    <div className="m-4">
      <h2 className="text-xl mb-4">Web焼きなまし(仮)</h2>
      <div className="form-control w-full max-w-xs">
        <label className="label">
          <span className="label-text">温度</span>
        </label>
        <input
          type="text"
          className="input input-bordered w-full max-w-xs"
          value={temp}
          onChange={(e) => maybeSetTemp(e.target.value)}
        />
      </div>
      <button
        className={clsx("btn", "btn-sm", "btn-primary")}
        onClick={onClick}
      >
        焼きなます
      </button>
    </div>
  );
}
