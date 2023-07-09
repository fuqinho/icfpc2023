import clsx from "clsx";
import Link from "next/link";
import { ProblemMetadata } from "./api";

export default function ProblemListBar({
  problemID,
  problems,
}: {
  problemID?: number;
  problems: ProblemMetadata[];
}) {
  return (
    <div className="mx-2">
      <div className="tabs tabs-boxed">
        {problems.map((problem) => {
          return (
            <Link
              key={problem.id}
              className={clsx(
                "tab tab-sm",
                problemID == problem.id ? "tab-active" : null,
              )}
              href={`/problem/${problem.id}`}
            >
              {problem.id}
            </Link>
          );
        })}
      </div>
    </div>
  );
}
