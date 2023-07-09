import { useCallback } from "react";
import { SolutionMetadata } from "./api";
import clsx from "clsx";
import { formatNumber } from "./number_format";
import { DateTime } from "luxon";

interface SolutionListRowProps {
  solution: SolutionMetadata;
  onClickSolution?: (uuid: string) => void;
}

function SolutionListRow({ solution, onClickSolution }: SolutionListRowProps) {
  const submission = solution.submission;

  const [badgeType, badgeCaption] =
    submission?.state === "FINISHED"
      ? submission.accepted
        ? ["badge-success", "Accepted"]
        : ["badge-error", "Rejected"]
      : submission?.state === "PROCESSING"
      ? ["badge-warning", "Processing"]
      : ["badge-primary", "Pending"];

  const onClick = useCallback(() => {
    if (onClickSolution) {
      onClickSolution(solution.uuid);
    }
  }, [onClickSolution, solution.uuid]);

  const created = DateTime.fromISO(solution.created).setZone("Asia/Tokyo");

  return (
    <tr>
      <td className="text-center">
        <span className={clsx("badge", badgeType)} title={submission?.error}>
          {badgeCaption}
        </span>
      </td>
      <td className="font-mono text-right">
        {formatNumber(submission?.score)}
      </td>
      <td className="font-mono">
        <span title={solution.uuid}>
          {created.toFormat("ccc HH:mm:ss ZZZ")} ({created.toRelative()})
        </span>
      </td>
      <td>
        <button className="btn btn-primary btn-sm" onClick={onClick}>
          Load
        </button>
      </td>
    </tr>
  );
}

interface SolutionListProps {
  solutions: SolutionMetadata[];
  onClickSolution?: (uuid: string) => void;
}

export default function SolutionList({
  solutions,
  onClickSolution,
}: SolutionListProps) {
  return (
    <div className="overflow-x-auto">
      <table className="table w-auto">
        <thead>
          <tr>
            <th className="text-center">Status</th>
            <th className="text-center">Score (公式)</th>
            <th className="text-center">Created</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {solutions.map((solution) => (
            <SolutionListRow
              key={solution.uuid}
              solution={solution}
              onClickSolution={onClickSolution}
            />
          ))}
        </tbody>
      </table>
    </div>
  );
}
