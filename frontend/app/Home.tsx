"use client";

import Visualizer from "@/components/Visualizer";
import {
  loadSolutionSpec,
  useKnownSolutions,
  useProblemList,
  useProblemSpec,
} from "@/components/api";
import { Solution } from "@/components/problems";
import { useCallback, useEffect, useState } from "react";
import clsx from "clsx";
import { RenderingOption } from "@/components/visualizer/renderer";
import VisualizerControl from "./VisualizerControl";
import type { EvaluationResult } from "wasm";
import SolutionList from "@/components/SolutionList";

// Tailwind (https://tailwindcss.com/docs/installation)
// を使っているので、クラス名などはそちらを参照。
//
// コンポーネントとして DaisyUI(https://daisyui.com/docs/use/)
// が入っているので、そこにあるやつはコピペで使えます。

export default function Home() {
  const [problemID, setProblemID] = useState<number | undefined>(undefined);
  const [rawSolution, setRawSolution] = useState("");
  const [solution, setSolution] = useState<Solution | null>(null);
  const [jsonParseException, setJSONParseException] = useState<any>(null);
  const [option, setOption] = useState<RenderingOption>({});
  const [evalResult, setEvalResult] = useState<EvaluationResult | null>(null);

  const { data: problems, error: errorProblems } = useProblemList();
  const { data: problem, error: errorProblem } = useProblemSpec(problemID);
  const { data: knownSolutions, error: errorKnownSolutions } =
    useKnownSolutions(problemID);

  if (knownSolutions) {
    knownSolutions.sort(
      (a, b) =>
        (b.submission?.score ?? -1e100) - (a.submission?.score ?? -1e100),
    );
  }

  const onClickSolution = useCallback(
    async (uuid: string) => {
      const solution = await loadSolutionSpec(uuid);
      setSolution(solution);
      setRawSolution(JSON.stringify(solution));
    },
    [setSolution, setRawSolution],
  );

  useEffect(() => {
    if (problems && !problemID) {
      setProblemID(problems[0].id);
    }
  }, [problems, problemID]);

  useEffect(() => {
    if (!problem || !solution) {
      return;
    }
    (async () => {
      const wasm = await import("wasm");
      console.time("wasm-eval-time");
      setEvalResult(
        wasm.EvaluationResult.from_json(
          JSON.stringify(problem),
          JSON.stringify(solution),
        ),
      );
      console.timeEnd("wasm-eval-time");
    })();
  }, [problem, solution]);

  const updateRawSolution = (rs: string) => {
    setRawSolution(rs);
    if (problemID && rawSolution !== "") {
      try {
        const s = JSON.parse(rawSolution) as Solution;
        if (s.placements) {
          if (!s.problem_id) {
            s.problem_id = problemID!;
          }
          setSolution(s);
          setJSONParseException(null);
        } else {
          setSolution(null);
          setJSONParseException("Object doesn't have placements.");
        }
      } catch (e) {
        setSolution(null);
        setJSONParseException(e);
      }
    }
    return {
      jsonParseException: jsonParseException,
      solution: solution,
      evalResult: evalResult,
    };
  };

  if (errorProblems) {
    throw errorProblems;
  }
  if (errorProblem) {
    throw errorProblem;
  }
  if (errorKnownSolutions) {
    throw errorKnownSolutions;
  }
  if (!problems) {
    return <div>Loading...</div>;
  }

  return (
    <div className="m-4">
      <div className="tabs">
        {problems.map((entry) => {
          return (
            <a
              key={entry.id}
              className={clsx(
                "tab tab-lifted",
                entry.id === problemID ? "tab-active" : null,
              )}
              onClick={() => {
                setProblemID(entry.id);
                setRawSolution("");
                setSolution(null);
                setEvalResult(null);
                setJSONParseException(null);
                setOption({});
              }}
            >
              {entry.id}
            </a>
          );
        })}
      </div>

      {problem ? (
        <div>
          <div className="flex">
            <Visualizer
              problem={problem}
              solution={solution}
              evalResult={evalResult}
              option={option}
              className="w-[800px] h-[800px] m-4 border border-slate-200"
            />
            <VisualizerControl
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
              onChange={(e) => updateRawSolution(e.target.value)}
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
      ) : null}
    </div>
  );
}
