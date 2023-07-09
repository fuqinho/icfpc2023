"use client";

import { useMismatchedSolutions } from "@/components/api";
import MismatchedSolutionList from "./MismatchedSolutionList";

async function rerunEvaluation() {
  if (
    !window.confirm(
      "本当に再計算しなおしますか？ 評価器はフロントエンドの API を使っているので実行前に最新版がデプロイされていることを確認してください",
    )
  ) {
    return;
  }
  await fetch(
    "https://icfpc2023-backend-uadsges7eq-an.a.run.app/api/solutions-mismatched/refresh",
    {
      method: "POST",
    },
  );
  window.alert("再計算をリクエストしました！ ちょっと待ってね");
}

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
      <h1 className="text-3xl">Mismatched solutions</h1>
      <button className="btn btn-sm btn-error" onClick={rerunEvaluation}>
        Re-run evaluation
      </button>
      <MismatchedSolutionList solutions={mismatchedSolutions} />
    </div>
  );
}
