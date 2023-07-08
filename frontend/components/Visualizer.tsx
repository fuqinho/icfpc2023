"use client";

import {
  ForwardedRef,
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
} from "react";
import { Problem, Solution } from "./problems";
import { Renderer, RenderingOption } from "./visualizer/renderer";
import type { EvaluationResult } from "wasm";

export interface VisualizerElement {}

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

  useImperativeHandle(ref, () => {
    return {};
  });

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
    );
    const remove = renderer.addEventListeners();

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
  }, [canvasRef, problem, solution, evalResult, option]);

  return (
    <canvas
      className={className}
      ref={canvasRef}
      width={4000}
      height={4000}
    ></canvas>
  );
});
export default Visualizer;
