import { Problem, Solution } from "../problems";
import type { CanvasRenderingContext2D as ServerCanvasContext2D } from "canvas";

export type Coord = [number, number];
export const CANVAS_SIZE = 4000;

export interface ViewportState {
  // Problem coordinate of Viewport width.
  // Problem coordinate of Viewport center.
  pVpSize: number;
  pVpCenter: Coord;
}

export class Viewport {
  private readonly ctx: CanvasRenderingContext2D | ServerCanvasContext2D;
  private readonly problem: Problem;
  private readonly state: ViewportState;
  private readonly setState: (s: ViewportState) => void;

  // private pVpSize: number = 0;
  // private pVpCenter: Coord = [0, 0];
  private cVpCenterTempMove: Coord = [0, 0];
  private cCursor: Coord | undefined = undefined;

  constructor(
    ctx: CanvasRenderingContext2D | ServerCanvasContext2D,
    problem: Problem,
    state: ViewportState,
    setState: (s: ViewportState) => void,
  ) {
    this.ctx = ctx;
    this.problem = problem;
    this.state = state;
    this.setState = setState;
  }

  public clear() {
    this.ctx.globalAlpha = 1;
    this.ctx.fillStyle = "rgb(255, 255, 255)";
    this.ctx.fillRect(0, 0, CANVAS_SIZE, CANVAS_SIZE);
  }

  public drawCursorPos() {
    if (!this.cCursor) {
      return;
    }
    const pVpHeight = (this.state.pVpSize * CANVAS_SIZE) / CANVAS_SIZE;
    const c = [
      this.state.pVpCenter[0] -
        this.state.pVpSize / 2 +
        this.toProblemScale(this.cCursor[0]),
      this.state.pVpCenter[1] -
        pVpHeight / 2 +
        this.toProblemScale(CANVAS_SIZE - this.cCursor[1]),
    ];
    this.ctx.font = "64px monospace";
    const text = `(${c[0]}, ${c[1]})`;
    const m = this.ctx.measureText(text);
    const h = m.actualBoundingBoxAscent + m.actualBoundingBoxDescent;
    this.ctx.fillStyle = "white";
    this.ctx.fillRect(0, 0, m.width + 30, h + 30);
    this.ctx.fillStyle = "black";
    this.ctx.fillText(text, 0, h);
  }

  public zoomWithMousePos(factor: number, cPos: Coord) {
    const pVpHeight = (this.state.pVpSize * CANVAS_SIZE) / CANVAS_SIZE;
    const pPos = [
      this.state.pVpCenter[0] -
        this.state.pVpSize / 2 +
        this.toProblemScale(cPos[0]),
      this.state.pVpCenter[1] -
        pVpHeight / 2 +
        this.toProblemScale(CANVAS_SIZE - cPos[1]),
    ];
    const newPvpSize = this.state.pVpSize * factor;
    const newPvpCenter: Coord = [
      pPos[0] + newPvpSize / 2 - (cPos[0] * newPvpSize) / CANVAS_SIZE,
      pPos[1] +
        newPvpSize / 2 -
        ((CANVAS_SIZE - cPos[1]) * newPvpSize) / CANVAS_SIZE,
    ];
    this.setState({
      pVpSize: newPvpSize,
      pVpCenter: newPvpCenter,
    });
  }

  public setVpCenterMove(diff: Coord) {
    this.cVpCenterTempMove = diff;
  }

  public setCursorPos(cPos: Coord | undefined) {
    this.cCursor = cPos;
  }

  public commitVpCenterMove() {
    if (this.cVpCenterTempMove[0] == 0 && this.cVpCenterTempMove[0] == 0) {
      return;
    }
    this.setState({
      pVpSize: this.state.pVpSize,
      pVpCenter: [
        this.state.pVpCenter[0] -
          this.toProblemScale(this.cVpCenterTempMove[0]),
        this.state.pVpCenter[1] +
          this.toProblemScale(this.cVpCenterTempMove[1]),
      ],
    });
    this.cVpCenterTempMove = [0, 0];
  }

  private toCanvasCoordX(problemScaleX: number): number {
    // Convert the problem-scale to the canvas-scale.
    const cScale = this.toCanvasScale(problemScaleX);
    // Scale-wise, this is aligned with what is drawn on canvas. Now we pan the
    // coordinate based on the viewport position.
    const cVpCenter =
      this.toCanvasScale(this.state.pVpCenter[0]) - this.cVpCenterTempMove[0];
    const cVpW = CANVAS_SIZE;
    const cOrigin = -(cVpCenter - cVpW / 2);
    return cScale + cOrigin;
  }

  private toCanvasCoordY(problemScaleY: number): number {
    // Convert the problem-scale to the canvas-scale.
    const cScale = this.toCanvasScale(this.problem.room_height - problemScaleY);
    // Scale-wise, this is aligned with what is drawn on canvas. Now we pan the
    // coordinate based on the viewport position.
    const cVpCenter =
      this.toCanvasScale(this.problem.room_height - this.state.pVpCenter[1]) -
      this.cVpCenterTempMove[1];
    const cVpH = CANVAS_SIZE;
    const cOrigin = -(cVpCenter - cVpH / 2);
    return cScale + cOrigin;
  }

  private toCanvasScale(problemScaleV: number): number {
    return (problemScaleV * CANVAS_SIZE) / this.state.pVpSize;
  }

  private toProblemScale(canvasScaleV: number): number {
    return (canvasScaleV * this.state.pVpSize) / CANVAS_SIZE;
  }

  public drawRect({
    pXY,
    pWidth,
    pHeight,
    lineWidth,
    strokeStyle,
    fillStyle,
  }: {
    pXY: Coord;
    pWidth: number;
    pHeight: number;
    lineWidth?: number;
    strokeStyle?: string | CanvasGradient | CanvasPattern;
    fillStyle?: string | CanvasGradient | CanvasPattern;
  }) {
    this.ctx.beginPath();
    this.ctx.rect(
      this.toCanvasCoordX(pXY[0]),
      this.toCanvasCoordY(pXY[1]) - this.toCanvasScale(pHeight),
      this.toCanvasScale(pWidth),
      this.toCanvasScale(pHeight),
    );

    if (fillStyle) {
      this.ctx.fillStyle = fillStyle;
      this.ctx.fill();
    }
    if (strokeStyle && lineWidth) {
      this.ctx.lineWidth = lineWidth;
      this.ctx.strokeStyle = strokeStyle;
      this.ctx.stroke();
    }
  }

  public drawCircle({
    pXY,
    pRadius,
    cRadius,
    lineWidth,
    strokeStyle,
    fillStyle,
  }: {
    pXY: Coord;
    pRadius?: number;
    cRadius?: number;
    lineWidth?: number;
    strokeStyle?: string | CanvasGradient | CanvasPattern;
    fillStyle?: string | CanvasGradient | CanvasPattern;
  }) {
    this.ctx.beginPath();
    if (pRadius) {
      this.ctx.arc(
        this.toCanvasCoordX(pXY[0]),
        this.toCanvasCoordY(pXY[1]),
        this.toCanvasScale(pRadius),
        0,
        2 * Math.PI,
        false,
      );
    } else if (cRadius) {
      this.ctx.arc(
        this.toCanvasCoordX(pXY[0]),
        this.toCanvasCoordY(pXY[1]),
        cRadius,
        0,
        2 * Math.PI,
        false,
      );
    }

    if (fillStyle) {
      this.ctx.fillStyle = fillStyle;
      this.ctx.fill();
    }
    if (strokeStyle && lineWidth) {
      this.ctx.lineWidth = lineWidth;
      this.ctx.strokeStyle = strokeStyle;
      this.ctx.stroke();
    }
  }
}

export function initialViewportState(
  problem: Problem,
  solution: Solution | null,
): ViewportState {
  let minX = problem.room_width;
  let maxX = 0;
  let minY = problem.room_height;
  let maxY = 0;

  const f = (x: number, y: number) => {
    minX = Math.min(minX, x);
    maxX = Math.max(maxX, x);
    minY = Math.min(minY, y);
    maxY = Math.max(maxY, y);
  };
  f(problem.stage_bottom_left[0], problem.stage_bottom_left[0]);
  f(
    problem.stage_bottom_left[0] + problem.stage_width,
    problem.stage_bottom_left[0] + problem.stage_height,
  );
  problem.attendees.forEach((a) => f(a.x, a.y));
  if (solution) {
    solution.placements.forEach((m) => f(m.x, m.y));
  }

  const pVpCenter: Coord = [(maxX + minY) / 2, (maxY + minY) / 2];
  let vpSize = maxX - minX + 100;
  if (vpSize <= maxY - minY + 100) {
    vpSize = maxY - minY + 100;
  }
  return {
    pVpCenter: pVpCenter,
    pVpSize: vpSize,
  };
}
