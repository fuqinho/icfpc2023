"use client";

import Visualizer, { VisualizerElement } from "@/components/Visualizer";
import {
  loadSolutionSpec,
  useKnownSolutions,
  useProblemList,
  useProblemSpec,
} from "@/components/api";
import { Solution } from "@/components/problems";
import { useCallback, useEffect, useRef, useState } from "react";
import { RenderingOption } from "@/components/visualizer/renderer";
import SolutionList from "@/components/SolutionList";
import { EvaluationResult } from "@/components/evaluation_result";
import VisualizerControl from "@/components/VisualizerControl";
import clsx from "clsx";
import Link from "next/link";
import VisualizerAnnealer from "@/components/VisualizerAnnealer";
import HoverInfo from "@/components/HoverInfo";
import ProblemListBar from "@/components/ProblemListBar";

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
  const [option, setOption] = useState<RenderingOption>({
    attendeeHeatmapByScore: true,
  });
  const [evalResult, setEvalResult] = useState<EvaluationResult | null>(null);
  const [firstLoad, setFirstLoad] = useState(true);
  const visualizer = useRef<VisualizerElement>(null);

  const { data: problems, error: errorProblems } = useProblemList();
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
            option.lockedItem?.kind ?? "",
            option.lockedItem?.index ?? 0,
          ),
        ),
      );
      console.timeEnd("wasm-eval-time");
    })();
  }, [problem, solution, option.lockedItem]);

  const parseAndSetSolution = useCallback(
    (s: string) => {
      setRawSolution(s);
      setOption({
        attendeeHeatmapByScore: true,
      });
      try {
        const solution = parseSolution(s, problemID);
        setSolution(solution);
        setJSONParseException(null);
      } catch (e) {
        setSolution(null);
        setJSONParseException(e);
      }
    },
    [problemID, setSolution, setRawSolution, setJSONParseException],
  );

  const onClickSolution = useCallback(
    async (uuid: string) => {
      parseAndSetSolution("");
      window.scrollTo(0, 0);
      const solution = await loadSolutionSpec(uuid);
      parseAndSetSolution(JSON.stringify(solution));
    },
    [parseAndSetSolution],
  );

  useEffect(() => {
    (async () => {
      if (firstLoad && knownSolutions && knownSolutions.length > 0) {
        setFirstLoad(false);
        const solution = await loadSolutionSpec(knownSolutions[0].uuid);
        parseAndSetSolution(JSON.stringify(solution));
      }
    })();
  }, [firstLoad, knownSolutions, parseAndSetSolution]);

  if (errorProblems) {
    throw errorProblems;
  }
  if (errorProblem) {
    throw errorProblem;
  }
  if (errorKnownSolutions) {
    throw errorKnownSolutions;
  }
  if (!problems || !problem) {
    return <div>Loading problem...</div>;
  }

  return (
    <div>
      <ProblemListBar problemID={problemID} problems={problems} />

      <div className="m-4">
        <h1 className="text-3xl">Problem {problemID}</h1>

        <div>
          <div className="flex">
            <div>
              <Visualizer
                ref={visualizer}
                problem={problem}
                solution={solution}
                evalResult={evalResult}
                option={option}
                className="w-[800px] h-[800px] m-4 border border-slate-200"
              />
              <HoverInfo
                visualizer={visualizer.current}
                problem={problem}
                evalResult={evalResult}
                solution={solution}
                option={option}
                setOption={setOption}
              />
              <VisualizerAnnealer
                problem={problem}
                solution={solution}
                setRawSolution={parseAndSetSolution}
              />
            </div>
            <VisualizerControl
              problem={problem}
              evalResult={evalResult}
              option={option}
              setOption={setOption}
              rawSolution={rawSolution}
              setRawSolution={parseAndSetSolution}
              parseError={jsonParseException}
            />
          </div>
          <SolutionList
            solutions={knownSolutions ?? []}
            onClickSolution={onClickSolution}
          />
        </div>
      </div>
    </div>
  );
}
