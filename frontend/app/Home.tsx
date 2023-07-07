"use client";

import Visualizer from "@/components/Visualizer";
import { Solution, problems } from "@/components/problems";
import { useState } from "react";

// Tailwind (https://tailwindcss.com/docs/installation)
// を使っているので、クラス名などはそちらを参照。
//
// コンポーネントとして DaisyUI(https://daisyui.com/docs/use/)
// が入っているので、そこにあるやつはコピペで使えます。

export default function Home() {
  const problemNames = Array.from(problems.keys());
  const [problemName, setProblemName] = useState(problemNames[0]);
  const [rawSolution, setRawSolution] = useState("");

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
        onChange={(e) => setProblemName(e.target.value)}
      >
        {problemNames.map((name) => {
          return (
            <option key={name} selected={name === problemName}>
              {name}
            </option>
          );
        })}
      </select>
      <textarea
        placeholder="Solution"
        className="textarea textarea-bordered textarea-xs w-full max-w-xs"
        onChange={(e) => setRawSolution(e.target.value)}
      >
        {rawSolution}
      </textarea>
      {jsonParseException ? `${jsonParseException}` : null}

      <Visualizer problem={problems.get(problemName)!} solution={solution} />
    </div>
  );
}
