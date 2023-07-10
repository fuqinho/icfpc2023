import clsx from "clsx";
import { useCallback, useEffect, useState } from "react";
import { Problem, Solution } from "./problems";
import type * as wasm from "wasm";

interface VisualizerAnnealerProps {
  problem: Problem;
  solution: Solution | null;
  setRawSolution: (s: string) => void;
}

function useFloatState(defaultValue: string): [string, (s: string) => void] {
  const [value, setValue] = useState(defaultValue);

  const maybeSetValue = useCallback(
    (newValue: string) => {
      if (newValue !== "" && !Number.isNaN(Number(newValue))) {
        setValue(newValue);
      }
    },
    [setValue],
  );
  return [value, maybeSetValue];
}

function useWasm(): typeof wasm | null {
  const [wasmModule, setWasmModule] = useState<typeof wasm | null>(null);
  useEffect(() => {
    (async () => {
      const wasmModule = await import("wasm");
      setWasmModule(wasmModule);
    })();
  }, [setWasmModule]);
  return wasmModule;
}

function useSolverHandle(
  problem: Problem,
  solution: Solution | null,
  wasmModule: typeof wasm | null,
): wasm.SolverHandle | null {
  const [solverHandle, setSolverHandle] = useState<wasm.SolverHandle | null>(
    null,
  );

  useEffect(() => {
    if (!wasmModule || !solution) {
      setSolverHandle(null);
      return;
    }

    const problemHandle = wasmModule.ProblemHandle.from_json(
      JSON.stringify(problem),
    );
    const solutionHandle = wasmModule.SolutionHandle.from_json(
      JSON.stringify(solution),
    );
    const solverHandle = wasmModule.SolverHandle.new(
      problemHandle,
      solutionHandle,
    );

    problemHandle.free();
    solutionHandle.free();

    setSolverHandle(solverHandle);

    return () => {
      solverHandle.free();
    };
  }, [wasmModule, problem, solution]);

  return solverHandle;
}

export default function VisualizerAnnealer({
  problem,
  solution,
  setRawSolution,
}: VisualizerAnnealerProps) {
  const [temp, setTempString] = useFloatState("1.0");
  const wasmModule = useWasm();
  const solverHandle = useSolverHandle(problem, solution, wasmModule);

  const onClick = useCallback(() => {
    if (!solverHandle) {
      return;
    }
    solverHandle.run(
      Number(temp),
      1.0,
      BigInt(Math.floor(Math.random() * 1000000000000)),
    );
    const solutionHandle = solverHandle.solution();
    setRawSolution(solutionHandle.as_json());
    solutionHandle.free();
  }, [solverHandle, temp, setRawSolution]);

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
          onChange={(e) => setTempString(e.target.value)}
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
