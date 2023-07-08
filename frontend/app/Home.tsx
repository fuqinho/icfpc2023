"use client";

import Image from "next/image";
import Link from "next/link";
import {
  ProblemMetadata,
  SolutionMetadata,
  useBestSolutions,
  useProblemList,
  useProblemSpec,
} from "@/components/api";
import { formatNumber } from "@/components/number_format";

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
          <Image
            src={`/api/render/problem/${problem.id}`}
            alt=""
            width={200}
            height={200}
          />
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

  if (errorProblems) {
    throw errorProblems;
  }
  if (errorBestSolutions) {
    throw errorBestSolutions;
  }
  if (!problems || bestSolutions.size == 0) {
    return <div>Loading...</div>;
  }

  return (
    <div className="overflow-x-auto">
      <table className="table">
        <thead>
          <tr>
            <th>#</th>
            <th>Map</th>
            <th>Best Score</th>
          </tr>
        </thead>
        <tbody>
          {(problems ?? []).map((problem) => (
            <ProblemListItem
              key={problem.id}
              problem={problem}
              bestSolution={bestSolutions.get(problem.id)}
            />
          ))}
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
