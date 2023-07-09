import { Problem, Solution } from "@/components/problems";
import { NextRequest, NextResponse } from "next/server";

interface Request {
  problem: Problem;
  solution: Solution;
}

export async function POST(request: NextRequest) {
  const { problem, solution } = (await request.json()) as Request;

  const wasm = await import("wasm");
  const evalResult = wasm.Evaluator.from_json(
    JSON.stringify(problem),
    JSON.stringify(solution),
    "",
    0,
  );
  return NextResponse.json(JSON.parse(evalResult));
}
