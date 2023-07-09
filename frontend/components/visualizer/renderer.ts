import tinycolor from "tinycolor2";
import { Attendee, Musician, Pillar, Problem, Solution } from "../problems";
import { Viewport, ViewportState } from "./viewport";
import type { CanvasRenderingContext2D as ServerCanvasContext2D } from "canvas";
import { EvaluationResult } from "../evaluation_result";

const ATTENDEE_RADIUS = 10;
const MUSICIAN_RADIUS = 5;

export interface RenderingOption {
  tasteHeatmapInstrument?: number;
  scoreHeatmapMusicians?: boolean;
}

export class Renderer {
  private readonly vp: Viewport;
  private readonly problem: Problem;
  private readonly solution: Solution | null;
  private readonly evalResult: EvaluationResult | null;
  private readonly option: RenderingOption;

  private readonly instruments: Map<number, number[]>;

  private dragStartCoord: [number, number] | undefined = undefined;

  constructor(
    ctx: CanvasRenderingContext2D | ServerCanvasContext2D,
    problem: Problem,
    solution: Solution | null,
    evalResult: EvaluationResult | null,
    option: RenderingOption,
    viewportState: ViewportState,
    setViewportState: (s: ViewportState) => void,
  ) {
    this.vp = new Viewport(ctx, problem, viewportState, setViewportState);
    this.problem = problem;
    this.solution = solution;
    this.option = option;
    this.evalResult = evalResult;

    this.instruments = new Map<number, number[]>();
    for (let i = 0; i < problem.musicians.length; i++) {
      const instr = problem.musicians[i];
      if (!this.instruments.has(instr)) {
        this.instruments.set(instr, []);
      }
      this.instruments.get(instr)?.push(i);
    }
  }

  // ===========================================================================
  // Rendering
  // ===========================================================================

  public render() {
    this.vp.clear();
    this.drawRoomAndStage();

    if (this.option.tasteHeatmapInstrument === undefined) {
      let maxScore = Number.MIN_SAFE_INTEGER;
      let minScore = Number.MAX_SAFE_INTEGER;
      this.problem.attendees.forEach((_, i) => {
        const score = this.evalResult?.attendees.at(i)?.score!;
        maxScore = Math.max(maxScore, score);
        minScore = Math.min(minScore, score);
      });
      this.problem.attendees.forEach((a, i) =>
        this.drawAttendeeWithHeat(
          a,
          this.evalResult?.attendees.at(i)?.score!,
          maxScore,
          minScore,
        ),
      );
    } else {
      let maxTaste = Number.MIN_SAFE_INTEGER;
      let minTaste = Number.MAX_SAFE_INTEGER;
      const instr = this.option.tasteHeatmapInstrument;
      this.problem.attendees.forEach((a) => {
        const taste = a.tastes[instr];
        maxTaste = Math.max(maxTaste, taste);
        minTaste = Math.min(minTaste, taste);
      });

      this.problem.attendees.forEach((a) =>
        this.drawAttendeeWithHeat(a, a.tastes[instr], maxTaste, minTaste),
      );
    }

    this.problem.pillars.forEach((p) => this.drawPillar(p));
    if (this.solution) {
      if (this.option.scoreHeatmapMusicians) {
        let maxScore = Number.MIN_SAFE_INTEGER;
        let minScore = Number.MAX_SAFE_INTEGER;
        this.solution.placements.forEach((_, i) => {
          const score = this.evalResult?.musicians.at(i)?.score!;
          maxScore = Math.max(maxScore, score);
          minScore = Math.min(minScore, score);
        });
        this.solution.placements.forEach((m, i) =>
          this.drawMusicianWithHeat(m, i, maxScore, minScore),
        );
      } else {
        this.solution.placements.forEach((m, i) =>
          this.drawMusicianNormal(m, i),
        );
      }
    }
    this.vp.drawCursorPos();
  }

  private drawRoomAndStage() {
    this.vp.drawRect({
      pXY: [
        this.problem.stage_bottom_left[0],
        this.problem.stage_bottom_left[1],
      ],
      pWidth: this.problem.stage_width,
      pHeight: this.problem.stage_height,
      fillStyle: "#cbd5e1",
    });
    this.vp.drawRect({
      pXY: [0, 0],
      pWidth: this.problem.room_width,
      pHeight: this.problem.room_height,
      strokeStyle: "blue",
      lineWidth: 10,
    });
  }

  private drawAttendeeWithHeat(
    attendee: Attendee,
    value: number,
    maxValue: number,
    minValue: number,
  ) {
    let color: tinycolor.Instance;
    if (value > 0) {
      color = tinycolor({
        // Red
        h: 0,
        s: (value / maxValue) * 100,
        v: 100,
      });
    } else {
      color = tinycolor({
        // Blue
        h: 240,
        s: (value / minValue) * 100,
        v: 100,
      });
    }

    this.vp.drawCircle({
      pXY: [attendee.x, attendee.y],
      cRadius: ATTENDEE_RADIUS,
      fillStyle: color.toRgbString(),
    });
  }

  private drawMusicianNormal(musician: Musician, index: number) {
    const col = tinycolor({
      h: (this.problem.musicians[index] / this.instruments.size) * 360,
      s: 100,
      v: 100,
    });

    this.vp.drawCircle({
      pXY: [musician.x, musician.y],
      pRadius: MUSICIAN_RADIUS,
      fillStyle: col.toRgbString(),
    });
  }

  private drawMusicianWithHeat(
    musician: Musician,
    index: number,
    maxScore: number,
    minScore: number,
  ) {
    const score = this.evalResult?.musicians.at(index)?.score!;
    let color: tinycolor.Instance;
    if (score == 0) {
      color = tinycolor("#ffffff");
    } else if (score > 0) {
      color = tinycolor({
        // Red
        h: 0,
        s: (score / maxScore) * 100,
        v: 100,
      });
    } else {
      color = tinycolor({
        // Blue
        h: 240,
        s: (score / minScore) * 100,
        v: 100,
      });
    }

    this.vp.drawCircle({
      pXY: [musician.x, musician.y],
      pRadius: MUSICIAN_RADIUS,
      fillStyle: color.toRgbString(),
    });
  }

  private drawPillar(pillar: Pillar) {
    this.vp.drawCircle({
      pXY: [pillar.center[0], pillar.center[1]],
      pRadius: pillar.radius,
      fillStyle: "#7f8791",
    });
  }

  // ===========================================================================
  // Event handling
  // ===========================================================================

  public addEventListeners(canvas: HTMLCanvasElement): () => void {
    const mousedownEvent = (e: MouseEvent) => this.mousedownEvent(canvas, e);
    const mouseleaveEvent = this.mouseleaveEvent.bind(this);
    const mousemoveEvent = (e: MouseEvent) => this.mousemoveEvent(canvas, e);
    const mouseupEvent = this.mouseupEvent.bind(this);
    const wheelEvent = (e: WheelEvent) => this.wheelEvent(canvas, e);
    canvas.addEventListener("mousedown", mousedownEvent);
    canvas.addEventListener("mouseleave", mouseleaveEvent);
    canvas.addEventListener("mousemove", mousemoveEvent);
    canvas.addEventListener("mouseup", mouseupEvent);
    canvas.addEventListener("wheel", wheelEvent);
    return () => {
      canvas.removeEventListener("mousedown", mousedownEvent);
      canvas.removeEventListener("mouseleave", mouseleaveEvent);
      canvas.removeEventListener("mousemove", mousemoveEvent);
      canvas.removeEventListener("mouseup", mousedownEvent);
      canvas.removeEventListener("wheel", wheelEvent);
    };
  }

  private getMouseCCoord(
    canvas: HTMLCanvasElement,
    e: MouseEvent,
  ): [number, number] {
    const c = canvas.getBoundingClientRect();
    return [
      ((e.pageX - c.left) * canvas.width) / c.width,
      ((e.pageY - c.top) * canvas.height) / c.height,
    ];
  }

  private wheelEvent(canvas: HTMLCanvasElement, e: WheelEvent) {
    e.preventDefault();
    if (e.deltaY < 0) {
      this.vp.zoomWithMousePos(0.8, this.getMouseCCoord(canvas, e));
    } else {
      this.vp.zoomWithMousePos(1.2, this.getMouseCCoord(canvas, e));
    }
    return false;
  }

  private mousedownEvent(canvas: HTMLCanvasElement, e: MouseEvent) {
    this.dragStartCoord = this.getMouseCCoord(canvas, e);
  }

  private mouseupEvent() {
    this.dragStartCoord = undefined;
    this.vp.commitVpCenterMove();
  }

  private mousemoveEvent(canvas: HTMLCanvasElement, e: MouseEvent) {
    const current = this.getMouseCCoord(canvas, e);
    this.vp.setCursorPos(current);
    if (this.dragStartCoord) {
      this.vp.setVpCenterMove([
        current[0] - this.dragStartCoord[0],
        current[1] - this.dragStartCoord[1],
      ]);
    }
  }

  private mouseleaveEvent() {
    this.vp.setCursorPos(undefined);
  }
}
