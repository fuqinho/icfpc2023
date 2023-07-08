"use client";

import { useEffect, useRef } from "react";
import { Attendee, Musician, Problem, Solution } from "./problems";
import { Viewport } from "./visualizer/viewport";
import tinycolor from "tinycolor2";

const ATTENDEE_RADIUS = 10;
const MUSICIAN_RADIUS = 5;

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

    const instruments = new Map<number, number[]>();
    for (let i = 0; i < problem.musicians.length; i++) {
      const instr = problem.musicians[i];
      if (!instruments.has(instr)) {
        instruments.set(instr, []);
      }
      instruments.get(instr)?.push(i);
    }

    const ctx = canvas.getContext("2d")!;
    const vp = new Viewport(ctx, problem, solution);
    let animationFrameId: number = 0;
    let dragStartCoord: [number, number] | undefined = undefined;

    const getMouseCCoord = (e: MouseEvent): [number, number] => {
      const c = canvas.getBoundingClientRect();
      return [
        ((e.pageX - c.left) * canvas.width) / c.width,
        ((e.pageY - c.top) * canvas.height) / c.height,
      ];
    };
    const wheelEvent = (e: WheelEvent) => {
      if (e.deltaY < 0) {
        vp.zoom(0.8);
      } else {
        vp.zoom(1.2);
      }
      return false;
    };
    const mousedownEvent = (e: MouseEvent) => {
      dragStartCoord = getMouseCCoord(e);
    };
    const mouseupEvent = () => {
      dragStartCoord = undefined;
      vp.commitVpCenterMove();
    };
    const mousemoveEvent = (e: MouseEvent) => {
      const current = getMouseCCoord(e);
      vp.setCursorPos(current);
      if (dragStartCoord) {
        vp.setVpCenterMove([
          current[0] - dragStartCoord[0],
          current[1] - dragStartCoord[1],
        ]);
      }
    };
    const mouseleaveEvent = () => {
      vp.setCursorPos(undefined);
    };

    canvas.addEventListener("wheel", wheelEvent);
    canvas.addEventListener("mouseleave", mouseleaveEvent);
    canvas.addEventListener("mousedown", mousedownEvent);
    canvas.addEventListener("mousemove", mousemoveEvent);
    canvas.addEventListener("mouseup", mouseupEvent);

    const render = () => {
      vp.clear();
      drawRoomAndStage(vp, problem);
      problem.attendees.forEach((a) => drawAttendee(vp, a));
      if (solution) {
        solution.placements.forEach((m, i) =>
          drawMusician(vp, m, i, instruments.size),
        );
      }
      vp.drawCursorPos();
      animationFrameId = window.requestAnimationFrame(render);
    };
    render();
    return () => {
      canvas.removeEventListener("wheel", wheelEvent);
      canvas.removeEventListener("mouseleave", mouseleaveEvent);
      canvas.removeEventListener("mousedown", mousedownEvent);
      canvas.removeEventListener("mousemove", mousemoveEvent);
      canvas.removeEventListener("mouseup", mousedownEvent);
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
    fillStyle: "#cbd5e1",
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

function drawMusician(
  vp: Viewport,
  musician: Musician,
  index: number,
  totalInstruments: number,
) {
  const col = tinycolor({
    h: (index / totalInstruments) * 360,
    s: 100,
    v: 100,
  });

  vp.drawCircle({
    pXY: [musician.x, musician.y],
    pRadius: MUSICIAN_RADIUS,
    fillStyle: col.toRgbString(),
  });
}
