import { Problem, Solution } from "@/components/problems";
import { Renderer } from "@/components/visualizer/renderer";
import { createCanvas } from "canvas";
import { NextRequest } from "next/server";

interface Request {
  problem: Problem;
  solution?: Solution;
}

export async function POST(request: NextRequest) {
  const { problem, solution } = (await request.json()) as Request;

  const offscreen = createCanvas(4000, 4000);
  const ctx = offscreen.getContext("2d")!;
  const renderer = new Renderer(
    ctx,
    4000,
    4000,
    problem,
    solution ?? null,
    null,
    {},
  );
  renderer.render();
  const pngBlob = offscreen.toBuffer("image/png");

  const res = new Response(pngBlob);
  res.headers.set("Content-Type", "image/png");
  return res;
}
