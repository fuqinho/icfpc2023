import tinycolor from "tinycolor2";
import { Attendee, Musician, Pillar, Problem, Solution } from "../problems";
import { Coord, Viewport, ViewportState } from "./viewport";
import type { CanvasRenderingContext2D as ServerCanvasContext2D } from "canvas";
import { EvaluationResult } from "../evaluation_result";

const ATTENDEE_RADIUS = 10;
const MUSICIAN_RADIUS = 5;

export class UpdateHoveredItemEvent extends Event {
  hoveredItem: HoveredItem | undefined;

  constructor(hoveredItem: HoveredItem | undefined) {
    super("updateHoveredItem");
    this.hoveredItem = hoveredItem;
  }
}

export interface RenderingOption {
  tasteHeatmapInstrument?: number;
  scoreHeatmapMusicians?: boolean;
}

export interface HoveredItem {
  kind: "attendee" | "musician";
  index: number;
}

export class Renderer {
  private readonly vp: Viewport;
  private readonly problem: Problem;
  private readonly solution: Solution | null;
  private readonly evalResult: EvaluationResult | null;
  private readonly option: RenderingOption;

  private readonly instruments: Map<number, number[]>;

  private dragStartCoord: [number, number] | undefined = undefined;
  private currentHoveredItem: HoveredItem | undefined = undefined;

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
      let maxScore = 0;
      this.problem.attendees.forEach((_, i) => {
        const score = this.evalResult?.attendees.at(i)?.score!;
        maxScore = Math.max(maxScore, Math.abs(score));
      });
      this.problem.attendees.forEach((a, i) =>
        this.drawAttendeeWithHeat(
          a,
          this.evalResult?.attendees.at(i)?.score!,
          maxScore,
        ),
      );
    } else {
      let maxTaste = 0;
      const instr = this.option.tasteHeatmapInstrument;
      this.problem.attendees.forEach((a) => {
        const taste = a.tastes[instr];
        maxTaste = Math.max(maxTaste, Math.abs(taste));
      });

      this.problem.attendees.forEach((a) =>
        this.drawAttendeeWithHeat(a, a.tastes[instr], maxTaste),
      );
    }

    this.problem.pillars.forEach((p) => this.drawPillar(p));
    if (this.solution) {
      if (this.option.scoreHeatmapMusicians) {
        let maxScore = 0;
        this.solution.placements.forEach((_, i) => {
          const score = this.evalResult?.musicians.at(i)?.score!;
          maxScore = Math.max(maxScore, Math.abs(score));
        });
        this.solution.placements.forEach((m, i) =>
          this.drawMusicianWithHeat(
            m,
            this.evalResult?.musicians.at(i)?.score!,
            maxScore,
          ),
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
  ) {
    let color: tinycolor.Instance;
    if (value > 0) {
      color = tinycolor.fromRatio({
        // Red
        h: 0,
        s: value / maxValue,
        v: 1,
      });
    } else {
      color = tinycolor({
        // Blue
        h: 240.0 / 360.0,
        s: Math.abs(value) / maxValue,
        v: 1,
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
    value: number,
    maxValue: number,
  ) {
    let color: tinycolor.Instance;
    if (value == 0) {
      color = tinycolor("#ffffff");
    } else if (value > 0) {
      color = tinycolor({
        // Red
        h: 0,
        s: value / maxValue,
        v: 1,
      });
    } else {
      color = tinycolor.fromRatio({
        // Blue
        h: 240.0 / 360.0,
        s: Math.abs(value) / maxValue,
        v: 1,
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

  private findClosestItem(): [number, HoveredItem | null] {
    const pPos = this.vp.getPCursorPos();
    if (!pPos) {
      return [-1, null];
    }
    let closestDist2 = -1;
    let closestItem = null;
    this.problem.attendees.forEach((a, i) => {
      const dist2 = distance2(pPos, [a.x, a.y]);
      if (closestDist2 == -1 || dist2 < closestDist2) {
        closestItem = { kind: "attendee", index: i };
        closestDist2 = dist2;
      }
    });
    this.solution?.placements.forEach((m, i) => {
      const dist2 = distance2(pPos, [m.x, m.y]);
      if (closestDist2 == -1 || dist2 < closestDist2) {
        closestItem = { kind: "musician", index: i };
        closestDist2 = dist2;
      }
    });
    return [closestDist2, closestItem];
  }

  private updateCurrentHover() {
    let [closestDist2, closestItem] = this.findClosestItem();
    const minRadius = Math.pow(this.vp.toProblemScale(100), 2);
    if (!closestItem || closestDist2 > minRadius) {
      if (this.currentHoveredItem) {
        this.currentHoveredItem = undefined;
        return true;
      }
      return false;
    }

    if (!this.currentHoveredItem) {
      this.currentHoveredItem = closestItem;
      return true;
    }
    if (
      this.currentHoveredItem.kind === closestItem.kind &&
      this.currentHoveredItem.index === closestItem.index
    ) {
      return false;
    }
    this.currentHoveredItem = closestItem;
    return true;
  }

  // ===========================================================================
  // Event handling
  // ===========================================================================

  public addEventListeners(canvas: HTMLCanvasElement): () => void {
    const mousedownEvent = (e: MouseEvent) => this.mousedownEvent(canvas, e);
    const mouseleaveEvent = () => this.mouseleaveEvent(canvas);
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
    if (this.updateCurrentHover()) {
      canvas.dispatchEvent(new UpdateHoveredItemEvent(this.currentHoveredItem));
    }
  }

  private mouseleaveEvent(canvas: HTMLCanvasElement) {
    this.vp.setCursorPos(undefined);
    if (this.updateCurrentHover()) {
      canvas.dispatchEvent(new UpdateHoveredItemEvent(this.currentHoveredItem));
    }
  }
}

function distance2(pos1: Coord, pos2: Coord) {
  return Math.pow(pos1[0] - pos2[0], 2) + Math.pow(pos1[1] - pos2[1], 2);
}
