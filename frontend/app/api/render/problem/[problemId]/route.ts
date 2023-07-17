import { NO_BACKEND } from "@/components/env";
import { Problem, readProblem } from "@/components/problems";
import { Renderer } from "@/components/visualizer/renderer";
import {
  CANVAS_SIZE,
  initialViewportState,
} from "@/components/visualizer/viewport";
import axios, { AxiosResponse } from "axios";
import { createCanvas } from "canvas";
import { NextRequest } from "next/server";

export async function GET(
  request: NextRequest,
  { params }: { params: { problemId: string } },
) {
  const problemID = Number(params.problemId);
  if (!problemID) {
    throw new Error("Invalid problem ID");
  }

  let problem: Problem;
  if (NO_BACKEND) {
    problem = readProblem(problemID);
  } else {
    const response: AxiosResponse<Problem> = await axios.get(
      `https://icfpc2023-backend-uadsges7eq-an.a.run.app/api/problems/${problemID}/spec`,
    );
    problem = response.data;
  }

  const offscreen = createCanvas(CANVAS_SIZE, CANVAS_SIZE);
  const ctx = offscreen.getContext("2d")!;
  const renderer = new Renderer(
    ctx,
    problem,
    null,
    null,
    { attendeeHeatmapByScore: true },
    initialViewportState(problem, null),
    () => {},
  );
  renderer.render();

  const resized = createCanvas(400, 400);
  resized.getContext("2d").drawImage(offscreen, 0, 0, 400, 400);

  const pngBlob = resized.toBuffer("image/png");

  const res = new Response(pngBlob);
  res.headers.set("Content-Type", "image/png");
  return res;
}
