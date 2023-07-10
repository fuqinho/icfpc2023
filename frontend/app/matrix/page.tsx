"use client";

import { SolutionMetadata, useSolutions } from "@/components/api";
import { formatNumber, formatPercentage } from "@/components/number_format";
import { num_attendees, num_musicians } from "@/components/static_metadata";
import clsx from "clsx";
import { DateTime } from "luxon";
import { orderBy } from "natural-orderby";

export default function Page() {
  const { data: solutions, error: errorSolutions } = useSolutions();

  if (errorSolutions) throw errorSolutions;
  if (!solutions) return <div>Loading...</div>;

  const sols = new Map<number, SolutionMetadata[]>();
  solutions.forEach((sol) => {
    if (!sols.has(sol.problem_id)) {
      sols.set(sol.problem_id, [sol]);
    } else {
      sols.get(sol.problem_id)!.push(sol);
    }
  });

  const problemIDs = orderBy(Array.from(sols.keys()));
  problemIDs.forEach((problemID) => {
    const ordered = orderBy(
      sols.get(problemID)!,
      [
        (sol) => sol.submission?.score,
        (sol) => DateTime.fromISO(sol.created).toUnixInteger(),
      ],
      ["desc", "desc"],
    );
    sols.set(problemID, ordered);
  });

  return (
    <div className="m-4">
      <div className="overflow-x-auto">
        <table className="table">
          <thead>
            <tr>
              <th>#</th>
              <th>奏者</th>
              <th>観客</th>
              <th className="text-right">No.1</th>
              <th className="text-right">No.2</th>
              <th className="text-right">No.3</th>
              <th className="text-right">No.4</th>
              <th className="text-right">No.5</th>
            </tr>
          </thead>
          <tbody>
            {problemIDs.map((problemID) => (
              <Ranking
                key={problemID}
                problemID={problemID}
                solutions={sols.get(problemID)!}
              />
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function Ranking({
  problemID,
  solutions,
}: {
  problemID: number;
  solutions: SolutionMetadata[];
}) {
  return (
    <tr>
      <th>{problemID}</th>
      <th>{num_musicians.get(problemID)}</th>
      <th>{num_attendees.get(problemID)}</th>
      <td>
        {solutions.length <= 0 ? null : (
          <RankSolution
            solution={solutions[0]}
            prevSol={solutions.at(1)}
            top={true}
          />
        )}
      </td>
      <td>
        {solutions.length <= 1 ? null : (
          <RankSolution solution={solutions[1]} prevSol={solutions.at(2)} />
        )}
      </td>
      <td>
        {solutions.length <= 2 ? null : (
          <RankSolution solution={solutions[2]} prevSol={solutions.at(3)} />
        )}
      </td>
      <td>
        {solutions.length <= 3 ? null : (
          <RankSolution solution={solutions[3]} prevSol={solutions.at(4)} />
        )}
      </td>
      <td>
        {solutions.length <= 4 ? null : (
          <RankSolution solution={solutions[4]} prevSol={solutions.at(5)} />
        )}
      </td>
    </tr>
  );
}

function RankSolution({
  top,
  solution,
  prevSol,
}: {
  top?: boolean;
  solution: SolutionMetadata;
  prevSol?: SolutionMetadata;
}) {
  let diffScore = undefined;
  let increase = undefined;
  if (prevSol) {
    diffScore = solution.submission?.score! - prevSol.submission?.score!;
    increase = diffScore / prevSol.submission?.score!;
  }
  const created = DateTime.fromISO(solution.created);
  const old = top && created.diffNow().milliseconds < -3600 * 1000;
  return (
    <div className="font-mono text-right">
      <p className="text-lg">{formatNumber(solution.submission?.score)}</p>
      <p className="text-sm">+{formatNumber(diffScore)}</p>
      <p className="text-sm">+{formatPercentage(increase)}</p>
      <p className={clsx("text-sm", old ? "text-red-700 font-bold" : null)}>
        {created.toRelative()}
      </p>
    </div>
  );
}
