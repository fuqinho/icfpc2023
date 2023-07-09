import { EvaluationResult } from "@/components/evaluation_result";
import { formatNumber } from "@/components/number_format";
import { Problem, Solution } from "@/components/problems";
import { HoveredItem, RenderingOption } from "@/components/visualizer/renderer";
import { orderBy } from "natural-orderby";
import { useMemo, useState } from "react";
import tinycolor from "tinycolor2";
import { VisualizerElement } from "./Visualizer";

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
    const attendee = problem.attendees[hoveredItem.index];
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
                {formatNumber(evalResult.attendees[hoveredItem.index].score)}
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
        </div>
      </div>
    );
  }

  const showDetailedScore = lockedItem?.kind === "attendee";
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
        Hovered Item (奏者 {hoveredItem.index}, 座標 ({pos?.x}, {pos?.y}))
      </h2>

      <div className="stats">
        {evalResult ? (
          <div className="stat">
            <div className="stat-title">スコア</div>
            <div className="stat-value">
              {formatNumber(evalResult.musicians[hoveredItem.index].score)}
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
                {formatNumber(evalResult.attendees[hoveredItem.index].score)}
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
              {formatNumber(evalResult.musicians[hoveredItem.index].score)}
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

function Instruments({
  problem,
  evalResult,
}: {
  problem: Problem;
  evalResult: EvaluationResult | null;
}) {
  const [instrumentsPage, setInstrumentsPage] = useState(1);
  const [order, setOrder] = useState("by-instr");
  const instruments = useMemo(() => {
    const instruments = new Map<number, number[]>();
    for (let i = 0; i < problem.musicians.length; i++) {
      const instr = problem.musicians[i];
      if (!instruments.has(instr)) {
        instruments.set(instr, []);
      }
      instruments.get(instr)?.push(i);
    }
    return instruments;
  }, [problem]);

  let instrumentsKeys = Array.from(instruments.keys());
  switch (order) {
    case "by-instr":
      instrumentsKeys = orderBy(instrumentsKeys, [(v) => v], ["asc"]);
      break;
    case "by-num-musicians-desc":
      instrumentsKeys = orderBy(
        instrumentsKeys,
        [(v) => instruments.get(v)?.length, (v) => v],
        ["desc", "asc"],
      );
      break;
    case "by-num-musicians-asc":
      instrumentsKeys = orderBy(
        instrumentsKeys,
        [(v) => instruments.get(v)?.length, (v) => v],
        ["asc", "asc"],
      );
      break;
    case "by-score-desc":
      instrumentsKeys = orderBy(
        instrumentsKeys,
        [(v) => evalResult?.instruments?.at(v)?.score, (v) => v],
        ["desc", "asc"],
      );
      break;
    case "by-score-asc":
      instrumentsKeys = orderBy(
        instrumentsKeys,
        [(v) => evalResult?.instruments?.at(v)?.score, (v) => v],
        ["asc", "asc"],
      );
      break;
  }

  const instrumentsCurrentPage = instrumentsKeys.slice(
    (instrumentsPage - 1) * 10,
    instrumentsPage * 10,
  );

  return (
    <div className="w-full">
      <h2 className="text-xl">楽器 ({instruments.size})</h2>
      <select
        className="select select-bordered select-sm m-2"
        onChange={(e) => setOrder(e.target.value)}
        value={order}
      >
        <option value="by-instr">楽器番号順</option>
        <option value="by-num-musicians-desc">奏者が多い順</option>
        <option value="by-num-musicians-asc">奏者が少ない順</option>
        <option value="by-score-desc">スコアの高い順</option>
        <option value="by-score-asc">スコアの低い順</option>
      </select>
      <table className="table w-auto">
        <thead>
          <tr>
            <th></th>
            <th></th>
            <th>奏者の人数</th>
            <th>スコア</th>
          </tr>
        </thead>
        <tbody>
          {instrumentsCurrentPage.map((instr) => {
            const col = tinycolor({
              h: (instr / instruments.size) * 360,
              s: 100,
              v: 100,
            });
            return (
              <tr key={instr}>
                <td className="w-8">{instr} </td>
                <td
                  className="w-16"
                  style={{ backgroundColor: col.toHex8String() }}
                >
                  &nbsp;
                </td>
                <td>{instruments.get(instr)?.length}人</td>
                <td className="font-mono text-right">
                  {formatNumber(evalResult?.instruments.at(instr)?.score)}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      <div className="my-4 mx-auto w-fit">
        <div className="join">
          <button
            className="join-item btn"
            onClick={() => setInstrumentsPage((p) => Math.max(1, p - 1))}
          >
            «
          </button>
          <button className="join-item btn">Page {instrumentsPage}</button>
          <button
            className="join-item btn"
            onClick={() =>
              setInstrumentsPage((p) =>
                Math.min(
                  Math.floor(instrumentsKeys.length / 10) +
                    (instrumentsKeys.length % 10 == 0 ? 0 : 1),
                  p + 1,
                ),
              )
            }
          >
            »
          </button>
        </div>
      </div>
    </div>
  );
}

function Musicians({
  problem,
  evalResult,
}: {
  problem: Problem;
  evalResult: EvaluationResult | null;
}) {
  const [page, setPage] = useState(1);
  const [order, setOrder] = useState("by-musician");

  const instruments = useMemo(() => {
    const instruments = new Map<number, number[]>();
    for (let i = 0; i < problem.musicians.length; i++) {
      const instr = problem.musicians[i];
      if (!instruments.has(instr)) {
        instruments.set(instr, []);
      }
      instruments.get(instr)?.push(i);
    }
    return instruments;
  }, [problem]);

  let musicianKeys = Array.from(problem.musicians.keys());
  switch (order) {
    case "by-musician":
      musicianKeys = orderBy(musicianKeys, [(v) => v], ["asc"]);
      break;
    case "by-instr":
      musicianKeys = orderBy(
        musicianKeys,
        [(v) => problem.musicians.at(v), (v) => v],
        ["asc", "asc"],
      );
      break;
    case "by-score-desc":
      musicianKeys = orderBy(
        musicianKeys,
        [(v) => evalResult?.musicians?.at(v)?.score, (v) => v],
        ["desc", "asc"],
      );
      break;
    case "by-score-asc":
      musicianKeys = orderBy(
        musicianKeys,
        [(v) => evalResult?.musicians?.at(v)?.score, (v) => v],
        ["asc", "asc"],
      );
      break;
  }

  const currentPage = musicianKeys.slice((page - 1) * 10, page * 10);
  return (
    <div>
      <h2 className="text-xl">奏者 ({problem.musicians.length})</h2>
      <select
        className="select select-bordered select-sm m-2"
        onChange={(e) => setOrder(e.target.value)}
        value={order}
      >
        <option value="by-musician">奏者番号順</option>
        <option value="by-instr">楽器番号順</option>
        <option value="by-score-desc">スコアの高い順</option>
        <option value="by-score-asc">スコアの低い順</option>
      </select>
      <table className="table w-auto">
        <thead>
          <tr>
            <th></th>
            <th>楽器</th>
            <th></th>
            <th>スコア</th>
          </tr>
        </thead>
        <tbody>
          {currentPage.map((m) => {
            const instr = problem.musicians[m];
            const col = tinycolor({
              h: (instr / instruments.size) * 360,
              s: 100,
              v: 100,
            });
            return (
              <tr key={m}>
                <td className="w-8">{m}</td>
                <td className="w-8">{instr}</td>
                <td
                  className="w-16"
                  style={{ backgroundColor: col.toHex8String() }}
                >
                  &nbsp;
                </td>
                <td className="font-mono text-right">
                  {formatNumber(evalResult?.musicians.at(m)?.score)}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      <div className="my-4 mx-auto w-fit">
        <div className="join">
          <button
            className="join-item btn"
            onClick={() => setPage((p) => Math.max(1, p - 1))}
          >
            «
          </button>
          <button className="join-item btn">Page {page}</button>
          <button
            className="join-item btn"
            onClick={() =>
              setPage((p) =>
                Math.min(
                  Math.floor(problem.musicians.length / 10) +
                    (problem.musicians.length % 10 == 0 ? 0 : 1),
                  p + 1,
                ),
              )
            }
          >
            »
          </button>
        </div>
      </div>
    </div>
  );
}

function ProblemInfo({
  problem,
  evalResult,
  solution,
  rawSolution,
  setRawSolution,
  parseError,
  visualizer,
  option,
  setOption,
}: {
  problem: Problem;
  evalResult: EvaluationResult | null;
  solution: Solution | null;
  rawSolution: string;
  setRawSolution: (s: string) => void;
  parseError: any;
  visualizer: VisualizerElement | null;
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
    <div className="overflow-x-auto space-y-4">
      <div className="stats">
        <div className="stat">
          <div className="stat-title">観客</div>
          <div className="stat-value">
            {formatNumber(problem.attendees.length)}
          </div>
        </div>
        {evalResult ? (
          <div className="stat">
            <div className="stat-title">スコア</div>
            <div className="stat-value">{formatNumber(evalResult.score)}</div>
          </div>
        ) : null}
        <div className="stat">
          <div className="stat-title">解答</div>
          <div className="stat-value">
            <textarea
              placeholder="Solution"
              className="textarea textarea-bordered font-mono"
              onChange={(e) => setRawSolution(e.target.value)}
              value={rawSolution}
            ></textarea>
          </div>
          <div className="stat-actions">
            <button
              className="btn btn-xs"
              onClick={async () => {
                setRawSolution(await navigator.clipboard.readText());
              }}
            >
              クリップボードからコピー
            </button>
          </div>
        </div>
      </div>
      <pre>
        <code>{parseError ? `${parseError}` : null}</code>
      </pre>
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
      <div className="flex">
        <div className="w-1/2">
          <Instruments problem={problem} evalResult={evalResult} />
        </div>
        <div className="divider divider-horizontal"></div>
        <div className="w-1/2">
          <Musicians problem={problem} evalResult={evalResult} />
        </div>
      </div>
    </div>
  );
}

export default function VisualizerControl({
  visualizer,
  problem,
  evalResult,
  solution,
  option,
  setOption,
  rawSolution,
  setRawSolution,
  parseError,
}: {
  visualizer: VisualizerElement | null;
  problem: Problem;
  evalResult: EvaluationResult | null;
  solution: Solution | null;
  option: RenderingOption;
  setOption: (fn: (option: RenderingOption) => RenderingOption) => void;
  rawSolution: string;
  setRawSolution: (s: string) => void;
  parseError: any;
}) {
  const instruments = Array.from(new Set(problem.musicians)).sort(
    (a, b) => a - b,
  );

  return (
    <div className="w-full">
      <ProblemInfo
        problem={problem}
        evalResult={evalResult}
        solution={solution}
        rawSolution={rawSolution}
        setRawSolution={setRawSolution}
        parseError={parseError}
        visualizer={visualizer}
        option={option}
        setOption={setOption}
      />

      <div className="divider"></div>

      <div>
        <h2 className="text-xl">コントロール</h2>
        <div className="form-control w-full max-w-xs">
          <label className="label">
            <span className="label-text">Tasteをヒートマップ表示</span>
          </label>
          <select
            className="select select-bordered"
            onChange={(e) => {
              if (e.target.value === "Pick one") {
                setOption((o) => {
                  return {
                    ...o,
                    attendeeHeatmapByScore: undefined,
                    attendeeHeatmapByTasteWithThisInstrument: undefined,
                  };
                });
              } else {
                setOption((o) => {
                  return {
                    ...o,
                    attendeeHeatmapByScore: undefined,
                    attendeeHeatmapByTasteWithThisInstrument: parseInt(
                      e.target.value,
                    ),
                  };
                });
              }
            }}
            value={
              option.attendeeHeatmapByTasteWithThisInstrument === undefined
                ? "Pick one"
                : option.attendeeHeatmapByTasteWithThisInstrument
            }
          >
            <option>Pick one</option>
            {instruments.map((instr) => {
              return <option key={instr}>{instr}</option>;
            })}
          </select>

          <label className="label">
            <span className="label-text-alt">
              赤(Taste最大)→白(0)→青(Taste最低)
            </span>
          </label>
        </div>

        <div className="form-control w-full max-w-xs">
          <label className="label cursor-pointer">
            <span className="label-text">
              奏者の寄与スコアでヒートマップ表示
            </span>
            <input
              type="checkbox"
              className="checkbox"
              checked={option.musicianHeatmapByScore ?? false}
              onChange={(e) => {
                setOption((o) => {
                  return { ...o, musicianHeatmapByScore: e.target.checked };
                });
              }}
            />
          </label>
          <label className="label cursor-pointer">
            <span className="label-text">
              観客の寄与スコアでヒートマップ表示
            </span>
            <input
              type="checkbox"
              className="checkbox"
              checked={option.attendeeHeatmapByScore ?? false}
              onChange={(e) => {
                setOption((o) => {
                  return {
                    ...o,
                    attendeeHeatmapByScore: e.target.checked,
                    attendeeHeatmapByTasteWithThisInstrument: undefined,
                  };
                });
              }}
            />
          </label>
        </div>
      </div>
    </div>
  );
}
