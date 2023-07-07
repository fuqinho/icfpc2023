"use client";

import { useEffect, useRef } from "react";
import { Attendee, Musician, Problem, Solution } from "./problems";

const ATTENDEE_RADIUS = 1;
const MUSICIAN_RADIUS = 10;

export default function Visualizer({
  problem,
  solution,
}: {
  problem: Problem;
  solution: Solution | null;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }
    const ctx = canvas.getContext("2d")!;
    let animationFrameId: number = 0;

    const resetCanvas = () => {
      ctx.globalAlpha = 1;
      ctx.fillStyle = "rgb(255, 255, 255)";
      ctx.fillRect(0, 0, problem.room_width, problem.room_height);
    };
    const render = () => {
      resetCanvas();
      drawStage(ctx, problem);
      problem.attendees.forEach((a) => drawAttendee(ctx, problem, a));
      if (solution) {
        solution.placements.forEach((m) => drawMusician(ctx, problem, m));
      }
      animationFrameId = window.requestAnimationFrame(render);
    };
    render();
    return () => window.cancelAnimationFrame(animationFrameId);
  }, [canvasRef, problem, solution]);

  return (
    <div>
      <canvas
        ref={canvasRef}
        style={{ width: "100vw", height: "100vh" }}
        width={problem.room_width}
        height={problem.room_height}
      ></canvas>
    </div>
  );
}

function coordToCanvas(
  problem: Problem,
  x: number,
  y: number
): [number, number] {
  return [x, problem.room_height - y];
}

function drawStage(ctx: CanvasRenderingContext2D, problem: Problem) {
  ctx.beginPath();
  ctx.rect(
    ...coordToCanvas(
      problem,
      problem.stage_bottom_left[0],
      problem.stage_bottom_left[1] + problem.stage_height
    ),
    problem.stage_width,
    problem.stage_height
  );

  ctx.lineWidth = 1;
  ctx.strokeStyle = "rgb(255, 0, 0)";
  ctx.stroke();
}

function drawAttendee(
  ctx: CanvasRenderingContext2D,
  problem: Problem,
  attendee: Attendee
) {
  ctx.beginPath();
  ctx.arc(
    ...coordToCanvas(problem, attendee.x, attendee.y),
    ATTENDEE_RADIUS,
    0,
    2 * Math.PI,
    false
  );

  ctx.fillStyle = "green";
  ctx.fill();

  ctx.lineWidth = 1;
  ctx.strokeStyle = "rgb(255, 0, 0)";
  ctx.stroke();
}

function drawMusician(
  ctx: CanvasRenderingContext2D,
  problem: Problem,
  musician: Musician
) {
  ctx.beginPath();
  ctx.arc(
    ...coordToCanvas(problem, musician.x, musician.y),
    MUSICIAN_RADIUS,
    0,
    2 * Math.PI,
    false
  );

  ctx.fillStyle = "blue";
  ctx.fill();

  ctx.lineWidth = 1;
  ctx.strokeStyle = "rgb(255, 0, 0)";
  ctx.stroke();
}
