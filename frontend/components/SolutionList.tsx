import { useCallback } from "react";
import { SolutionMetadata } from "./api";

interface SolutionListRowProps {
  solution: SolutionMetadata;
  onClickSolution?: (uuid: string) => void;
}

function SolutionListRow({ solution, onClickSolution }: SolutionListRowProps) {
  const submission = solution.submission;

  const [badgeType, badgeCaption] =
    submission?.state === "FINISHED"
      ? submission.accepted
        ? ["success", "Accepted"]
        : ["error", "Rejected"]
      : submission?.state === "PROCESSING"
      ? ["warning", "Processing"]
      : ["primary", "Pending"];

  const onClick = useCallback(() => {
    if (onClickSolution) {
      onClickSolution(solution.uuid);
    }
  }, [onClickSolution, solution.uuid]);

  return (
    <tr>
      <td className="text-center">
        <span className={`badge badge-${badgeType}`} title={submission?.error}>
          {badgeCaption}
        </span>
      </td>
      <td className="font-mono text-right">{submission?.score}</td>
      <td className="font-mono">
        <span title={solution.uuid}>{solution.created}</span>
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
      <table className="table">
        <thead>
          <tr>
            <th className="text-center">Status</th>
            <th className="text-center">Score</th>
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
