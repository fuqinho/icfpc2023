"use client";

import Image from "next/image";
import Link from "next/link";
import {
  ProblemMetadata,
  SolutionMetadata,
  problemImage,
  solutionImage,
  useBestSolutions,
  useProblemList,
  useProblemSpec,
} from "@/components/api";
import { formatNumber } from "@/components/number_format";
import { useState } from "react";
import { orderBy } from "natural-orderby";

// Tailwind (https://tailwindcss.com/docs/installation)
// を使っているので、クラス名などはそちらを参照。
//
// コンポーネントとして DaisyUI(https://daisyui.com/docs/use/)
// が入っているので、そこにあるやつはコピペで使えます。

function ProblemListItem({
  problem,
  bestSolution,
}: {
  problem: ProblemMetadata;
  bestSolution: SolutionMetadata | undefined;
}) {
  const { data: problemSpec, error: errorProblemSpec } = useProblemSpec(
    problem.id,
  );

  if (errorProblemSpec) {
    throw errorProblemSpec;
  }
  if (!problemSpec) {
    return null;
  }

  return (
    <tr>
      <td>{problem.id}</td>
      <td>
        <Link href={`/problem/${problem.id}`}>
          {bestSolution ? (
            <Image
              src={solutionImage(bestSolution.uuid)}
              alt=""
              width={200}
              height={200}
            />
          ) : (
            <Image
              src={problemImage(problem.id)}
              alt=""
              width={200}
              height={200}
            />
          )}
        </Link>
      </td>
      <td className="text-mono">
        <div className="stat">
          <div className="stat-value">
            {formatNumber(bestSolution?.submission?.score)}
          </div>
        </div>
      </td>
    </tr>
  );
}

function ProblemList() {
  const { data: problems, error: errorProblems } = useProblemList();
  const { data: bestSolutions, error: errorBestSolutions } = useBestSolutions();
  const [order, setOrder] = useState("by-id");

  if (errorProblems) {
    throw errorProblems;
  }
  if (errorBestSolutions) {
    throw errorBestSolutions;
  }
  if (!problems || bestSolutions.size == 0) {
    return <div>Loading...</div>;
  }

  let problemKeys = Array.from(problems.keys());
  switch (order) {
    case "by-id":
      problemKeys = orderBy(problemKeys, [(i) => problems[i].id], ["asc"]);
      break;
    case "by-score-desc":
      problemKeys = orderBy(
        problemKeys,
        [
          (i) => bestSolutions.get(problems[i].id)?.submission?.score,
          (i) => problems[i].id,
        ],
        ["desc", "asc"],
      );
      break;
    case "by-score-asc":
      problemKeys = orderBy(
        problemKeys,
        [
          (i) => bestSolutions.get(problems[i].id)?.submission?.score,
          (i) => problems[i].id,
        ],
        ["asc", "asc"],
      );
      break;
  }

  return (
    <div className="overflow-x-auto">
      <select
        className="select select-bordered select-sm m-2"
        onChange={(e) => setOrder(e.target.value)}
        value={order}
      >
        <option value="by-id">ID順</option>
        <option value="by-score-desc">スコアの高い順</option>
        <option value="by-score-asc">スコアの低い順</option>
      </select>
      <table className="table">
        <thead>
          <tr>
            <th>#</th>
            <th>Image</th>
            <th>Best Score (公式)</th>
          </tr>
        </thead>
        <tbody>
          {problemKeys.map((i) => {
            const problem = problems[i];
            return (
              <ProblemListItem
                key={problem.id}
                problem={problem}
                bestSolution={bestSolutions.get(problem.id)}
              />
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

export default function Home() {
  return (
    <div className="m-4">
      <h1 className="text-3xl">Problems</h1>

      <ProblemList />
    </div>
  );
}
