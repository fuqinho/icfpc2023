"use client";

import { useMismatchedSolutions } from "@/components/api";
import MismatchedSolutionList from "./MismatchedSolutionList";

export default function MismatchedSolutions() {
  const { data: mismatchedSolutions, error: errorMismatchedSolutions } =
    useMismatchedSolutions();

  if (mismatchedSolutions) {
    mismatchedSolutions.sort((a, b) => b.created.localeCompare(a.created));
  }

  if (errorMismatchedSolutions) {
    throw errorMismatchedSolutions;
  }
  if (mismatchedSolutions === undefined) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      <MismatchedSolutionList solutions={mismatchedSolutions} />
    </div>
  );
}
