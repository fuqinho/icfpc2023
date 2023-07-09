import { EvaluationResult } from "@/components/evaluation_result";
import { formatNumber } from "@/components/number_format";
import { Problem, Solution } from "@/components/problems";
import { HoveredItem, RenderingOption } from "@/components/visualizer/renderer";
import { useMemo, useState } from "react";
import tinycolor from "tinycolor2";
import { VisualizerElement } from "./Visualizer";

export default function HoverInfo({
  visualizer,
  problem,
  evalResult,
  solution,
  option,
  setOption,
}: {
  visualizer: VisualizerElement | null;
  problem: Problem;
  evalResult: EvaluationResult | null;
  solution: Solution | null;
  option: RenderingOption;
  setOption: (fn: (option: RenderingOption) => RenderingOption) => void;
}) {
  const [hoveredItem, setHoveredItem] = useState<HoveredItem | undefined>(
    undefined,
  );
  visualizer?.onUpdateHoveredItemEvent((e) => setHoveredItem(e.hoveredItem));
  visualizer?.onClickHoveredItemEvent((e) => {
    setOption((o) => {
      if (
        o.lockedItem?.index === e.hoveredItem.index &&
        o.lockedItem?.kind === e.hoveredItem.kind
      ) {
        return {
          ...o,
          lockedItem: undefined,
        };
      }
      return {
        ...o,
        lockedItem: e.hoveredItem,
      };
    });
  });
  return (
    <div>
      {option.lockedItem ? (
        <LockedItemData
          hoveredItem={option.lockedItem}
          problem={problem}
          solution={solution}
          evalResult={evalResult}
          setOption={setOption}
        />
      ) : null}
      {hoveredItem ? (
        <HoveredItemData
          lockedItem={option.lockedItem}
          hoveredItem={hoveredItem}
          problem={problem}
          solution={solution}
          evalResult={evalResult}
        />
      ) : null}
    </div>
  );
}

function HoveredItemData({
  lockedItem,
  hoveredItem,
  problem,
  solution,
  evalResult,
}: {
  lockedItem?: HoveredItem;
  hoveredItem: HoveredItem;
  problem: Problem;
  solution: Solution | null;
  evalResult: EvaluationResult | null;
}) {
  const instrumentSize = useMemo(() => {
    return new Set(problem.musicians).size;
  }, [problem]);
  if (hoveredItem.kind === "attendee") {
    const showDetailedScore = lockedItem?.kind === "musician";
    const lockedInstr =
      lockedItem?.kind === "musician" ? problem.musicians[lockedItem.index] : 0;
    const attendee = problem.attendees[hoveredItem.index];
    const taste = showDetailedScore ? attendee.tastes[lockedInstr] : 0;
    return (
      <div className="w-full">
        <h2 className="text-xl">
          Hovered Item (観客 {hoveredItem.index}, 座標 ({attendee.x},{" "}
          {attendee.y}))
        </h2>

        <div className="stats">
          {evalResult ? (
            <div className="stat">
              <div className="stat-title">スコア</div>
              <div className="stat-value">
                {formatNumber(evalResult.attendees[hoveredItem.index])}
              </div>
            </div>
          ) : null}
          {showDetailedScore && evalResult ? (
            <div className="stat">
              <div className="stat-title">Locked Item からのスコア</div>
              <div className="stat-value">
                {formatNumber(evalResult.detailed_attendees[hoveredItem.index])}
              </div>
            </div>
          ) : null}
          {showDetailedScore && evalResult ? (
            <div className="stat">
              <div className="stat-title">Locked Item のTaste</div>
              <div className="stat-value">{taste}</div>
            </div>
          ) : null}
        </div>
      </div>
    );
  }

  const showDetailedScore = lockedItem?.kind === "attendee";
  const instr = problem.musicians[hoveredItem.index];
  const taste = showDetailedScore
    ? problem.attendees[lockedItem.index].tastes[instr]
    : 0;
  const pos = solution?.placements[hoveredItem.index];
  const col = tinycolor({
    h: (instr / instrumentSize) * 360,
    s: 100,
    v: 100,
  });
  return (
    <div className="w-full">
      <h2 className="text-xl">
        Hovered Item (奏者 {hoveredItem.index}, 座標 ({pos?.x}, {pos?.y}))
      </h2>

      <div className="stats">
        {evalResult ? (
          <div className="stat">
            <div className="stat-title">スコア</div>
            <div className="stat-value">
              {formatNumber(evalResult.musicians[hoveredItem.index])}
            </div>
          </div>
        ) : null}
        <div className="stat">
          <div className="stat-title">楽器</div>
          <div className="stat-value">
            {instr}
            <div
              className="w-16"
              style={{ backgroundColor: col.toHex8String() }}
            >
              &nbsp;
            </div>
          </div>
        </div>
        {showDetailedScore && evalResult ? (
          <div className="stat">
            <div className="stat-title">Locked Item へのスコア</div>
            <div className="stat-value">
              {formatNumber(evalResult.detailed_musicians[hoveredItem.index])}
            </div>
          </div>
        ) : null}
        {showDetailedScore && evalResult ? (
          <div className="stat">
            <div className="stat-title">Locked Item のTaste</div>
            <div className="stat-value">{taste}</div>
          </div>
        ) : null}
      </div>
    </div>
  );
}

function LockedItemData({
  hoveredItem,
  problem,
  solution,
  evalResult,
  setOption,
}: {
  hoveredItem: HoveredItem;
  problem: Problem;
  solution: Solution | null;
  evalResult: EvaluationResult | null;
  setOption: (fn: (option: RenderingOption) => RenderingOption) => void;
}) {
  const instrumentSize = useMemo(() => {
    return new Set(problem.musicians).size;
  }, [problem]);
  const unlock = () =>
    setOption((o) => {
      return { ...o, lockedItem: undefined };
    });

  if (hoveredItem.kind === "attendee") {
    const attendee = problem.attendees[hoveredItem.index];
    return (
      <div className="w-full">
        <h2 className="text-xl">
          Locked Item (観客 {hoveredItem.index}, 座標 ({attendee.x},{" "}
          {attendee.y}))
          <button className="btn btn-sm ml-2" onClick={unlock}>
            Unlock
          </button>
        </h2>

        <div className="stats">
          {evalResult ? (
            <div className="stat">
              <div className="stat-title">スコア</div>
              <div className="stat-value">
                {formatNumber(evalResult.attendees[hoveredItem.index])}
              </div>
            </div>
          ) : null}
        </div>
      </div>
    );
  }

  const instr = problem.musicians[hoveredItem.index];
  const pos = solution?.placements[hoveredItem.index];
  const col = tinycolor({
    h: (instr / instrumentSize) * 360,
    s: 100,
    v: 100,
  });
  return (
    <div className="w-full">
      <h2 className="text-xl">
        Locked Item (奏者 {hoveredItem.index}, 座標 ({pos?.x}, {pos?.y}))
        <button className="btn btn-sm ml-2" onClick={unlock}>
          Unlock
        </button>
      </h2>

      <div className="stats">
        {evalResult ? (
          <div className="stat">
            <div className="stat-title">スコア</div>
            <div className="stat-value">
              {formatNumber(evalResult.musicians[hoveredItem.index])}
            </div>
          </div>
        ) : null}
        <div className="stat">
          <div className="stat-title">楽器</div>
          <div className="stat-value">
            {instr}
            <div
              className="w-16"
              style={{ backgroundColor: col.toHex8String() }}
            >
              &nbsp;
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
