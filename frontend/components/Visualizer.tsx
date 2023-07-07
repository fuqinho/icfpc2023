"use client";

import { useEffect, useRef } from "react";
import { Attendee, Musician, Problem, Solution } from "./problems";
import { Viewport } from "./visualizer/viewport";

const ATTENDEE_RADIUS = 10;
const MUSICIAN_RADIUS = 10;

export default function Visualizer({
  problem,
  solution,
  className,
}: {
  problem: Problem;
  solution: Solution | null;
  className?: string;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }
    const ctx = canvas.getContext("2d")!;
    const vp = new Viewport(ctx, problem, solution);
    let animationFrameId: number = 0;

    const wheelEvent = (e: WheelEvent) => {
      if (e.deltaY < 0) {
        vp.zoom(0.8);
      } else {
        vp.zoom(1.2);
      }
      return false;
    };

    canvas.addEventListener("wheel", wheelEvent);

    const render = () => {
      vp.clear();
      drawRoomAndStage(vp, problem);
      problem.attendees.forEach((a) => drawAttendee(vp, a));
      if (solution) {
        solution.placements.forEach((m) => drawMusician(vp, m));
      }
      animationFrameId = window.requestAnimationFrame(render);
    };
    render();
    return () => {
      canvas.removeEventListener("wheel", wheelEvent);
      window.cancelAnimationFrame(animationFrameId);
    };
  }, [canvasRef, problem, solution]);

  return (
    <canvas
      className={className}
      ref={canvasRef}
      width={4000}
      height={4000}
    ></canvas>
  );
}

function drawRoomAndStage(vp: Viewport, problem: Problem) {
  vp.drawRect({
    pXY: [problem.stage_bottom_left[0], problem.stage_bottom_left[1]],
    pWidth: problem.stage_width,
    pHeight: problem.stage_height,
    fillStyle: "#fecaca",
  });
  vp.drawRect({
    pXY: [0, 0],
    pWidth: problem.room_width,
    pHeight: problem.room_height,
    strokeStyle: "blue",
    lineWidth: 10,
  });
}

function drawAttendee(vp: Viewport, attendee: Attendee) {
  vp.drawCircle({
    pXY: [attendee.x, attendee.y],
    cRadius: ATTENDEE_RADIUS,
    strokeStyle: "red",
    fillStyle: "red",
  });
}

function drawMusician(vp: Viewport, musician: Musician) {
  vp.drawCircle({
    pXY: [musician.x, musician.y],
    pRadius: MUSICIAN_RADIUS,
    fillStyle: "blue",
    lineWidth: 2,
    strokeStyle: "rgb(255, 0, 0)",
  });
}
