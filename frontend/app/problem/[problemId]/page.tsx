"use client";

import Visualizer, { VisualizerElement } from "@/components/Visualizer";
import {
  loadSolutionSpec,
  useKnownSolutions,
  useProblemSpec,
} from "@/components/api";
import { Solution } from "@/components/problems";
import { ChangeEvent, useCallback, useEffect, useRef, useState } from "react";
import { RenderingOption } from "@/components/visualizer/renderer";
import SolutionList from "@/components/SolutionList";
import { EvaluationResult } from "@/components/evaluation_result";
import VisualizerControl from "@/components/VisualizerControl";

// Tailwind (https://tailwindcss.com/docs/installation)
// を使っているので、クラス名などはそちらを参照。
//
// コンポーネントとして DaisyUI(https://daisyui.com/docs/use/)
// が入っているので、そこにあるやつはコピペで使えます。

function parseSolution(rawSolution: string, problemID: number): Solution {
  const s = JSON.parse(rawSolution) as Solution;

  if (!Array.isArray(s.placements)) {
    throw new Error("Object doesn't have placements.");
  }
  // TODO: Inspect placements.

  s.problem_id = problemID;
  return s;
}

export default function Home({ params }: { params: { problemId: string } }) {
  const problemID = Number(params.problemId);

  const [rawSolution, setRawSolution] = useState(
    JSON.stringify({ problem_id: problemID, placements: [] }),
  );
  const [solution, setSolution] = useState<Solution | null>(null);
  const [jsonParseException, setJSONParseException] = useState<any>(null);
  const [option, setOption] = useState<RenderingOption>({});
  const [evalResult, setEvalResult] = useState<EvaluationResult | null>(null);
  const visualizer = useRef<VisualizerElement>(null);

  const { data: problem, error: errorProblem } = useProblemSpec(problemID);
  const { data: knownSolutions, error: errorKnownSolutions } =
    useKnownSolutions(problemID);

  useEffect(() => {
    if (!problem || !solution) {
      return;
    }
    (async () => {
      const wasm = await import("wasm");
      console.time("wasm-eval-time");
      setEvalResult(
        JSON.parse(
          wasm.Evaluator.from_json(
            JSON.stringify(problem),
            JSON.stringify(solution),
          ),
        ),
      );
      console.timeEnd("wasm-eval-time");
    })();
  }, [problem, solution]);

  useEffect(() => {
    try {
      const solution = parseSolution(rawSolution, problemID);
      setSolution(solution);
      setJSONParseException(null);
    } catch (e) {
      setSolution(null);
      setJSONParseException(e);
      return;
    }
  }, [rawSolution, problemID]);

  const onRawSolutionChange = useCallback(
    (event: ChangeEvent<HTMLTextAreaElement>) => {
      setRawSolution(event.target.value);
    },
    [setRawSolution],
  );

  const onClickSolution = useCallback(
    async (uuid: string) => {
      const solution = await loadSolutionSpec(uuid);
      setRawSolution(JSON.stringify(solution));
      setSolution(solution);
    },
    [setSolution, setRawSolution],
  );

  if (errorProblem) {
    throw errorProblem;
  }
  if (errorKnownSolutions) {
    throw errorKnownSolutions;
  }
  if (!problem) {
    return <div>Loading problem...</div>;
  }

  return (
    <div className="m-4">
      <h1 className="text-3xl">Problem {problemID}</h1>

      <div>
        <div className="flex">
          <Visualizer
            ref={visualizer}
            problem={problem}
            solution={solution}
            evalResult={evalResult}
            option={option}
            className="w-[800px] h-[800px] m-4 border border-slate-200"
          />
          <VisualizerControl
            visualizer={visualizer.current}
            problem={problem}
            evalResult={evalResult}
            option={option}
            setOption={setOption}
          />
        </div>
        <div className="m-4">
          <h2 className="text-xl my-2">解答</h2>
          <textarea
            placeholder="Solution"
            className="textarea textarea-bordered w-[800px] h-[100px] font-mono"
            onChange={onRawSolutionChange}
            value={rawSolution}
          ></textarea>
          <pre>
            <code>{jsonParseException ? `${jsonParseException}` : null}</code>
          </pre>
        </div>
        <SolutionList
          solutions={knownSolutions ?? []}
          onClickSolution={onClickSolution}
        />
      </div>
    </div>
  );
}
