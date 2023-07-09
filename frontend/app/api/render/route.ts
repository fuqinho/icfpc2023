import { Problem, Solution } from "@/components/problems";
import { Renderer } from "@/components/visualizer/renderer";
import {
  CANVAS_SIZE,
  initialViewportState,
} from "@/components/visualizer/viewport";
import axios, { AxiosResponse } from "axios";
import { createCanvas } from "canvas";
import { NextRequest } from "next/server";

interface Request {
  problem: Problem;
  solution?: Solution;
}

export async function POST(request: NextRequest) {
  const { problem, solution } = (await request.json()) as Request;

  const offscreen = createCanvas(CANVAS_SIZE, CANVAS_SIZE);
  const ctx = offscreen.getContext("2d")!;
  const renderer = new Renderer(
    ctx,
    problem,
    solution ?? null,
    null,
    { scoreHeatmapAttendees: true },
    initialViewportState(problem, solution ?? null),
    () => {},
  );
  renderer.render();
  const pngBlob = offscreen.toBuffer("image/png");

  const res = new Response(pngBlob);
  res.headers.set("Content-Type", "image/png");
  return res;
}

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const problemID = parseInt(searchParams.get("problem")!);
  const solutionID = searchParams.get("solution");

  const response: AxiosResponse<Problem> = await axios.get(
    `https://icfpc2023-backend-uadsges7eq-an.a.run.app/api/problems/${problemID}/spec`,
  );
  const problem = response.data;

  let solution: Solution | null = null;
  if (solutionID) {
    const solResp = await axios.get<Solution>(
      `https://icfpc2023-backend-uadsges7eq-an.a.run.app/api/solutions/${solutionID}/spec`,
    );
    solution = solResp.data;
  }

  const offscreen = createCanvas(CANVAS_SIZE, CANVAS_SIZE);
  const ctx = offscreen.getContext("2d")!;
  const renderer = new Renderer(
    ctx,
    problem,
    solution ?? null,
    null,
    { scoreHeatmapAttendees: true },
    initialViewportState(problem, solution ?? null),
    () => {},
  );
  renderer.render();
  const pngBlob = offscreen.toBuffer("image/png");

  const res = new Response(pngBlob);
  res.headers.set("Content-Type", "image/png");
  return res;
}
