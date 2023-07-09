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
  useScoreboard,
} from "@/components/api";
import { formatNumber, formatPercentage } from "@/components/number_format";
import { useCallback } from "react";
import { orderBy } from "natural-orderby";
import { usePathname, useRouter, useSearchParams } from "next/navigation";

// Tailwind (https://tailwindcss.com/docs/installation)
// を使っているので、クラス名などはそちらを参照。
//
// コンポーネントとして DaisyUI(https://daisyui.com/docs/use/)
// が入っているので、そこにあるやつはコピペで使えます。

function ProblemListItem({
  problem,
  bestSolution,
  totalScore,
}: {
  problem: ProblemMetadata;
  bestSolution: SolutionMetadata | undefined;
  totalScore: number;
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
      <td className="text-mono">
        <div className="stat">
          <div className="stat-value">
            {(bestSolution?.submission?.score ?? -1) < 0
              ? null
              : formatPercentage(
                  (bestSolution?.submission?.score ?? 0) / totalScore,
                )}
          </div>
        </div>
      </td>
    </tr>
  );
}

function ProblemList() {
  const { data: problems, error: errorProblems } = useProblemList();
  const { data: bestSolutions, error: errorBestSolutions } = useBestSolutions();
  const { data: scoreboard, error: errorScoreboard } = useScoreboard();

  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams()!;

  const createQueryString = useCallback(
    (name: string, value: string) => {
      const params = new URLSearchParams(searchParams.toString());
      params.set(name, value);
      return params.toString();
    },
    [searchParams],
  );

  const order = searchParams.get("order") ?? "by-id";
  const showV1 = searchParams.get("showV1") !== "false";
  const showV2 = searchParams.get("showV2") !== "false";

  const winnerScore = scoreboard?.scoreboard[0]?.score ?? 999999999999;

  if (errorProblems) {
    throw errorProblems;
  }
  if (errorBestSolutions) {
    throw errorBestSolutions;
  }
  if (!problems || bestSolutions.size == 0) {
    return <div>Loading...</div>;
  }
  let totalScore = 0;
  let v1TotalScore = 0;
  let v2TotalScore = 0;
  bestSolutions.forEach((v) => {
    const score = v.submission?.score ?? 0;
    if (score > 0) {
      totalScore += score;
      if (v.problem_id <= 55) {
        v1TotalScore += score;
      } else {
        v2TotalScore += score;
      }
    }
  });

  let problemKeys = Array.from(problems.keys());
  if (!showV1) {
    problemKeys = problemKeys.filter((i) => problems[i].id > 55);
  }
  if (!showV2) {
    problemKeys = problemKeys.filter((i) => problems[i].id <= 55);
  }

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
      <div className="flex justify-between">
        <div className="flex w-fit">
          <select
            className="select select-bordered select-sm m-2"
            onChange={(e) =>
              router.push(
                pathname + "?" + createQueryString("order", e.target.value),
              )
            }
            value={order}
          >
            <option value="by-id">ID順</option>
            <option value="by-score-desc">スコアの高い順</option>
            <option value="by-score-asc">スコアの低い順</option>
          </select>
          <button
            className="btn btn-sm"
            onClick={() =>
              navigator.clipboard.writeText(createScript(problems, problemKeys))
            }
          >
            この順番で雑ローラースクリプトをコピー
          </button>
          <label className="label cursor-pointer space-x-2">
            <span className="label-text">v1を表示 (1-55)</span>
            <input
              type="checkbox"
              className="toggle toggle-primary"
              checked={showV1}
              onChange={(e) =>
                router.push(
                  pathname +
                    "?" +
                    createQueryString("showV1", `${e.target.checked}`),
                )
              }
            />
          </label>
          <label className="label cursor-pointer space-x-2">
            <span className="label-text">v2を表示 (56+)</span>
            <input
              type="checkbox"
              className="toggle toggle-primary"
              checked={showV2}
              onChange={(e) =>
                router.push(
                  pathname +
                    "?" +
                    createQueryString("showV2", `${e.target.checked}`),
                )
              }
            />
          </label>
        </div>
        <div className="text-lg text-right font-mono">
          <p>トータルスコア: {formatNumber(totalScore)}</p>
          <div className="text-sm">
            <p>
              V1トータルスコア: {formatNumber(v1TotalScore)} (
              {formatPercentage(v1TotalScore / totalScore)})
            </p>
            <p>
              V2トータルスコア: {formatNumber(v2TotalScore)} (
              {formatPercentage(v2TotalScore / totalScore)})
            </p>
            <p>
              1位まであと: {formatNumber(winnerScore - totalScore)} (+
              {formatPercentage(winnerScore / totalScore - 1)})
            </p>
          </div>
        </div>
      </div>
      <table className="table">
        <thead>
          <tr>
            <th>#</th>
            <th>Image</th>
            <th>Best Score (公式)</th>
            <th>スコア寄与率</th>
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
                totalScore={totalScore}
              />
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

function createScript(problems: ProblemMetadata[], problemKeys: number[]) {
  let lines: string[] = problemKeys.map((i) => {
    const problem = problems[i];
    return (
      "cargo run --release --time-limit=${TIME_LIMIT:-600} --threads=${THREADS:-8} --from-current-best --start-temp=${START_TEMP:-1e5} " +
      `${problem.id}`
    );
  });
  return lines.join("\n");
}

export default function Home() {
  return (
    <div className="m-4">
      <h1 className="text-3xl">Problems</h1>

      <ProblemList />
    </div>
  );
}
