import { Problem, Solution } from "../problems";
import type { CanvasRenderingContext2D as ServerCanvasContext2D } from "canvas";

type Coord = [number, number];

export class Viewport {
  private readonly ctx: CanvasRenderingContext2D | ServerCanvasContext2D;
  private readonly canvasWidth: number;
  private readonly canvasHeight: number;
  private readonly problem: Problem;
  private readonly solution: Solution | null;

  // Problem coordinate of Viewport width.
  // Problem coordinate of Viewport center.
  private pVpWidth: number = 0;
  private pVpCenter: Coord = [0, 0];
  private cVpCenterTempMove: Coord = [0, 0];
  private cCursor: Coord | undefined = undefined;

  constructor(
    ctx: CanvasRenderingContext2D | ServerCanvasContext2D,
    canvasWidth: number,
    canvasHeight: number,
    problem: Problem,
    solution: Solution | null,
  ) {
    this.ctx = ctx;
    this.canvasWidth = canvasWidth;
    this.canvasHeight = canvasHeight;
    this.problem = problem;
    this.solution = solution;
    this.calculateInitialViewport();
  }

  private calculateInitialViewport() {
    let minX = this.problem.room_width;
    let maxX = 0;
    let minY = this.problem.room_height;
    let maxY = 0;

    const f = (x: number, y: number) => {
      minX = Math.min(minX, x);
      maxX = Math.max(maxX, x);
      minY = Math.min(minY, y);
      maxY = Math.max(maxY, y);
    };
    f(this.problem.stage_bottom_left[0], this.problem.stage_bottom_left[0]);
    f(
      this.problem.stage_bottom_left[0] + this.problem.stage_width,
      this.problem.stage_bottom_left[0] + this.problem.stage_height,
    );
    this.problem.attendees.forEach((a) => f(a.x, a.y));
    if (this.solution) {
      this.solution.placements.forEach((m) => f(m.x, m.y));
    }

    this.pVpCenter = [(maxX + minY) / 2, (maxY + minY) / 2];

    let vpW = maxX - minX + 100;
    let vpH = (vpW * this.canvasHeight) / this.canvasWidth;
    if (vpH > maxY - minY + 100) {
      this.pVpWidth = vpW;
      return;
    }
    vpH = maxY - minY + 100;
    vpW = (vpH * this.canvasWidth) / this.canvasHeight;
    this.pVpWidth = vpW;
  }

  public clear() {
    this.ctx.globalAlpha = 1;
    this.ctx.fillStyle = "rgb(255, 255, 255)";
    this.ctx.fillRect(0, 0, this.canvasWidth, this.canvasHeight);
  }

  public drawCursorPos() {
    if (!this.cCursor) {
      return;
    }
    const pVpHeight = (this.pVpWidth * this.canvasHeight) / this.canvasWidth;
    const c = [
      this.pVpCenter[0] -
        this.pVpWidth / 2 +
        this.toProblemScale(this.cCursor[0]),
      this.pVpCenter[1] -
        pVpHeight / 2 +
        this.toProblemScale(this.canvasHeight - this.cCursor[1]),
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

  public zoom(factor: number) {
    this.pVpWidth *= factor;
  }

  public zoomWithMousePos(factor: number, cPos: Coord) {
    const pVpHeight = (this.pVpWidth * this.canvasHeight) / this.canvasWidth;
    const pPos = [
      this.pVpCenter[0] - this.pVpWidth / 2 + this.toProblemScale(cPos[0]),
      this.pVpCenter[1] -
        pVpHeight / 2 +
        this.toProblemScale(this.canvasHeight - cPos[1]),
    ];
    const newPvpWidth = this.pVpWidth * factor;
    const newPvpHeight = (newPvpWidth * this.canvasHeight) / this.canvasWidth;
    const newPvpCenter: Coord = [
      pPos[0] + newPvpWidth / 2 - (cPos[0] * newPvpWidth) / this.canvasWidth,
      pPos[1] +
        newPvpHeight / 2 -
        ((this.canvasHeight - cPos[1]) * newPvpWidth) / this.canvasWidth,
    ];
    this.pVpWidth = newPvpWidth;
    this.pVpCenter = newPvpCenter;
  }

  public setVpCenterMove(diff: Coord) {
    this.cVpCenterTempMove = diff;
  }

  public setCursorPos(cPos: Coord | undefined) {
    this.cCursor = cPos;
  }

  public commitVpCenterMove() {
    this.pVpCenter = [
      this.pVpCenter[0] - this.toProblemScale(this.cVpCenterTempMove[0]),
      this.pVpCenter[1] + this.toProblemScale(this.cVpCenterTempMove[1]),
    ];
    this.cVpCenterTempMove = [0, 0];
  }

  private toCanvasCoordX(problemScaleX: number): number {
    // Convert the problem-scale to the canvas-scale.
    const cScale = this.toCanvasScale(problemScaleX);
    // Scale-wise, this is aligned with what is drawn on canvas. Now we pan the
    // coordinate based on the viewport position.
    const cVpCenter =
      this.toCanvasScale(this.pVpCenter[0]) - this.cVpCenterTempMove[0];
    const cVpW = this.canvasWidth;
    const cOrigin = -(cVpCenter - cVpW / 2);
    return cScale + cOrigin;
  }

  private toCanvasCoordY(problemScaleY: number): number {
    // Convert the problem-scale to the canvas-scale.
    const cScale = this.toCanvasScale(this.problem.room_height - problemScaleY);
    // Scale-wise, this is aligned with what is drawn on canvas. Now we pan the
    // coordinate based on the viewport position.
    const cVpCenter =
      this.toCanvasScale(this.problem.room_height - this.pVpCenter[1]) -
      this.cVpCenterTempMove[1];
    const cVpH = this.canvasHeight;
    const cOrigin = -(cVpCenter - cVpH / 2);
    return cScale + cOrigin;
  }

  private toCanvasScale(problemScaleV: number): number {
    return (problemScaleV * this.canvasWidth) / this.pVpWidth;
  }

  private toProblemScale(canvasScaleV: number): number {
    return (canvasScaleV * this.pVpWidth) / this.canvasWidth;
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
