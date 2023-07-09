import { SolutionMetadata, solutionURL } from "@/components/api";
import { formatNumber } from "@/components/number_format";
import { DateTime } from "luxon";
import Link from "next/link";

interface MismatchedSolutionListRowProps {
  solution: SolutionMetadata;
  onClickSolution?: (uuid: string) => void;
}

function MismatchedSolutionListRow({
  solution,
}: MismatchedSolutionListRowProps) {
  const submission = solution.submission!;
  const evaluation = solution.evaluation!;

  const created = DateTime.fromISO(solution.created).setZone("Asia/Tokyo");

  return (
    <tr>
      <td className="font-mono text-center">
        <Link href={`/problem/${solution.problem_id}`}>
          {solution.problem_id}
        </Link>
      </td>
      <td className="font-mono text-center">{solution.uuid}</td>
      <td className="font-mono text-right">{formatNumber(submission.score)}</td>
      <td className="font-mono text-right text-error">
        {formatNumber(evaluation.score)}
      </td>
      <td className="font-mono">
        {created.toFormat("ccc HH:mm:ss ZZZ")} ({created.toRelative()})
      </td>
      <td className="space-x-2">
        <a
          className="btn btn-secondary btn-sm"
          target="_blank"
          href={solutionURL(solution.uuid)}
        >
          直リンク
        </a>
      </td>
    </tr>
  );
}

interface MismatchedSolutionListProps {
  solutions: SolutionMetadata[];
}

export default function MismatchedSolutionList({
  solutions,
}: MismatchedSolutionListProps) {
  return (
    <div className="overflow-x-auto">
      <table className="table w-auto">
        <thead>
          <tr>
            <th className="text-center">Problem</th>
            <th className="text-center">UUID</th>
            <th className="text-center">Score (公式)</th>
            <th className="text-center">Score (俺俺)</th>
            <th className="text-center">Created</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {solutions.map((solution) => (
            <MismatchedSolutionListRow
              key={solution.uuid}
              solution={solution}
            />
          ))}
        </tbody>
      </table>
    </div>
  );
}
