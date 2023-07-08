import tinycolor from "tinycolor2";
import { Attendee, Musician, Pillar, Problem, Solution } from "../problems";
import { Viewport } from "./viewport";
import type { EvaluationResult } from "wasm";
import type { CanvasRenderingContext2D as ServerCanvasContext2D } from "canvas";

const ATTENDEE_RADIUS = 10;
const MUSICIAN_RADIUS = 5;

export interface RenderingOption {
  tasteHeatmapInstrument?: number;
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
    canvasWidth: number,
    canvasHeight: number,
    problem: Problem,
    solution: Solution | null,
    evalResult: EvaluationResult | null,
    option: RenderingOption,
  ) {
    this.vp = new Viewport(ctx, canvasWidth, canvasHeight, problem, solution);
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
      this.problem.attendees.forEach((a) => this.drawAttendeeNormal(a));
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
        this.drawAttendeeWithHeat(a, instr, maxTaste, minTaste),
      );
    }

    this.problem.pillars.forEach((p) => this.drawPillar(p));
    this.solution?.placements.forEach((m, i) => this.drawMusician(m, i));
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

  private drawAttendeeNormal(attendee: Attendee) {
    this.vp.drawCircle({
      pXY: [attendee.x, attendee.y],
      cRadius: ATTENDEE_RADIUS,
      strokeStyle: "red",
      fillStyle: "red",
    });
  }

  private drawAttendeeWithHeat(
    attendee: Attendee,
    instr: number,
    maxTaste: number,
    minTaste: number,
  ) {
    const taste = attendee.tastes[instr];
    let color: tinycolor.Instance;
    if (taste > 0) {
      color = tinycolor({
        // Red
        h: 0,
        s: (taste / maxTaste) * 100,
        v: 100,
      });
    } else {
      color = tinycolor({
        // Blue
        h: 240,
        s: (taste / minTaste) * 100,
        v: 100,
      });
    }

    this.vp.drawCircle({
      pXY: [attendee.x, attendee.y],
      cRadius: ATTENDEE_RADIUS,
      fillStyle: color.toRgbString(),
    });
  }

  private drawMusician(musician: Musician, index: number) {
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
    const wheelEvent = this.wheelEvent.bind(this);
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

  private wheelEvent(e: WheelEvent) {
    e.preventDefault();
    if (e.deltaY < 0) {
      this.vp.zoom(0.8);
    } else {
      this.vp.zoom(1.2);
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
