"use client";

import {
  ForwardedRef,
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import { Problem, Solution } from "./problems";
import {
  Renderer,
  RenderingOption,
  UpdateHoveredItemEvent,
} from "./visualizer/renderer";
import { EvaluationResult } from "./evaluation_result";
import { CANVAS_SIZE, initialViewportState } from "./visualizer/viewport";

export interface VisualizerElement {
  onUpdateHoveredItemEvent(fn: (e: UpdateHoveredItemEvent) => void): void;
}

const Visualizer = forwardRef(function Visualizer(
  {
    problem,
    solution,
    evalResult,
    className,
    option,
  }: {
    problem: Problem;
    solution: Solution | null;
    evalResult: EvaluationResult | null;
    option: RenderingOption;
    className?: string;
  },
  ref: ForwardedRef<VisualizerElement>,
) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [viewportState, setViewportState] = useState(() =>
    initialViewportState(problem, solution),
  );

  useImperativeHandle(
    ref,
    () => {
      return {
        onUpdateHoveredItemEvent(fn: (e: UpdateHoveredItemEvent) => void) {
          canvasRef.current?.addEventListener("updateHoveredItem", (e) =>
            fn(e as UpdateHoveredItemEvent),
          );
        },
      };
    },
    [],
  );

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }

    const renderer = new Renderer(
      canvas.getContext("2d")!,
      problem,
      solution,
      evalResult,
      option,
      viewportState,
      setViewportState,
    );
    const remove = renderer.addEventListeners(canvas);

    let animationFrameId: number = 0;
    const render = () => {
      renderer.render();
      animationFrameId = window.requestAnimationFrame(render);
    };
    render();
    return () => {
      remove();
      window.cancelAnimationFrame(animationFrameId);
    };
  }, [
    canvasRef,
    problem,
    solution,
    evalResult,
    option,
    viewportState,
    setViewportState,
  ]);

  return (
    <canvas
      className={className}
      ref={canvasRef}
      width={CANVAS_SIZE}
      height={CANVAS_SIZE}
    ></canvas>
  );
});
export default Visualizer;
