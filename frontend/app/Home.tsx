"use client";

import Visualizer from "@/components/Visualizer";
import { useProblemList, useProblemSpec } from "@/components/api";
import { Solution } from "@/components/problems";
import { useEffect, useState } from "react";
import clsx from "clsx";
import { RenderingOption } from "@/components/visualizer/renderer";
import VisualizerControl from "./VisualizerControl";

// Tailwind (https://tailwindcss.com/docs/installation)
// を使っているので、クラス名などはそちらを参照。
//
// コンポーネントとして DaisyUI(https://daisyui.com/docs/use/)
// が入っているので、そこにあるやつはコピペで使えます。

export default function Home() {
  const { data: problems, error: errorProblems } = useProblemList();
  const [problemID, setProblemID] = useState<string | number | undefined>(
    undefined,
  );
  const { data: problem, error: errorProblem } = useProblemSpec(problemID);
  const [rawSolution, setRawSolution] = useState("");
  const [option, setOption] = useState<RenderingOption>({});

  useEffect(() => {
    if (problems && !problemID) {
      setProblemID(problems[0].id);
    }
  }, [problems, problemID]);

  if (errorProblems) {
    throw errorProblems;
  }
  if (errorProblem) {
    throw errorProblem;
  }
  if (!problems) {
    return <div>Loading...</div>;
  }

  let jsonParseException = null;
  let solution: Solution | null = null;
  if (rawSolution !== "") {
    try {
      const s = JSON.parse(rawSolution) as Solution;
      if (Object.hasOwn(s, "placements")) {
        solution = s;
      } else {
        jsonParseException = "Object doesn't have placements.";
      }
    } catch (e) {
      jsonParseException = e;
    }
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
                setOption(() => {
                  return {};
                });
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
              option={option}
              className="w-[800px] h-[800px] m-4 border border-slate-200"
            />
            <VisualizerControl
              problem={problem}
              option={option}
              setOption={setOption}
            />
          </div>
          <div className="m-4">
            <h2 className="text-xl my-2">解答</h2>
            <textarea
              placeholder="Solution"
              className="textarea textarea-bordered w-[800px] h-[100px] font-mono"
              onChange={(e) => setRawSolution(e.target.value)}
              value={rawSolution}
            ></textarea>
            <pre>
              <code>{jsonParseException ? `${jsonParseException}` : null}</code>
            </pre>
          </div>
        </div>
      ) : null}
    </div>
  );
}
