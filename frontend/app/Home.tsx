"use client";

import Visualizer from "@/components/Visualizer";
import { useProblemList, useProblemSpec } from "@/components/api";
import { Solution } from "@/components/problems";
import { useEffect, useState } from "react";

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
      solution = JSON.parse(rawSolution) as Solution;
    } catch (e) {
      jsonParseException = e;
    }
  }
  return (
    <div>
      <select
        className="select select-bordered select-sm w-full max-w-xs"
        onChange={(e) => setProblemID(e.target.value)}
        value={problemID}
      >
        {problems.map((entry) => {
          return (
            <option key={entry.id} value={entry.id}>
              {entry.id}
            </option>
          );
        })}
      </select>
      <textarea
        placeholder="Solution"
        className="textarea textarea-bordered textarea-xs w-full max-w-xs"
        onChange={(e) => setRawSolution(e.target.value)}
        defaultValue={rawSolution}
      ></textarea>
      {jsonParseException ? `${jsonParseException}` : null}

      {problem ? (
        <Visualizer
          problem={problem}
          solution={solution}
          className="w-[800px] h-[800px] m-4 border border-slate-200"
        />
      ) : null}
    </div>
  );
}
