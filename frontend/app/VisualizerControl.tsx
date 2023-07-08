import { Problem } from "@/components/problems";
import { RenderingOption } from "@/components/visualizer/renderer";
import { orderBy } from "natural-orderby";
import { useState } from "react";
import tinycolor from "tinycolor2";
import type { EvaluationResult } from "wasm";

const numberFormat = new Intl.NumberFormat();

function formatNumber(n?: number) {
  if (!n) {
    return "";
  }
  return numberFormat.format(n);
}

function Instruments({ instruments }: { instruments: Map<number, number[]> }) {
  const [instrumentsPage, setInstrumentsPage] = useState(1);
  const [order, setOrder] = useState("by-instr");
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
  }

  const instrumentsCurrentPage = instrumentsKeys.slice(
    (instrumentsPage - 1) * 10,
    instrumentsPage * 10,
  );

  return (
    <div>
      <h2 className="text-xl">楽器</h2>
      <select
        className="select select-bordered select-sm m-2"
        onChange={(e) => setOrder(e.target.value)}
        value={order}
      >
        <option value="by-instr">楽器番号順</option>
        <option value="by-num-musicians-desc">奏者が多い順</option>
        <option value="by-num-musicians-asc">奏者が少ない順</option>
      </select>
      <table className="table">
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
                  className="w-32"
                  style={{ backgroundColor: col.toHex8String() }}
                >
                  &nbsp;
                </td>
                <td>奏者 {instruments.get(instr)?.length}人</td>
              </tr>
            );
          })}
        </tbody>
      </table>
      <div className="join">
        <button
          className="join-item btn"
          onClick={() => setInstrumentsPage((p) => Math.max(1, p - 1))}
        >
          «
        </button>
        <button className="join-item btn btn-disabled">
          Page {instrumentsPage}
        </button>
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
  );
}

function ProblemInfo({
  problem,
  evalResult,
}: {
  problem: Problem;
  evalResult: EvaluationResult | null;
}) {
  const instruments = new Map<number, number[]>();
  for (let i = 0; i < problem.musicians.length; i++) {
    const instr = problem.musicians[i];
    if (!instruments.has(instr)) {
      instruments.set(instr, []);
    }
    instruments.get(instr)?.push(i);
  }
  return (
    <div className="overflow-x-auto space-y-4">
      <div>
        <h2 className="text-xl">問題情報</h2>
        <table className="table">
          <tbody>
            <tr>
              <td>観客</td>
              <td>{formatNumber(problem.attendees.length)}</td>
            </tr>
            <tr>
              <td>奏者の数</td>
              <td>{formatNumber(problem.musicians.length)}</td>
            </tr>
            <tr>
              <td>楽器の種類</td>
              <td>{formatNumber(instruments.size)}</td>
            </tr>
            {evalResult ? (
              <tr>
                <td>スコア</td>
                <td>{formatNumber(evalResult.score)}</td>
              </tr>
            ) : null}
          </tbody>
        </table>
      </div>
      <Instruments instruments={instruments} />
    </div>
  );
}

export default function VisualizerControl({
  problem,
  evalResult,
  option,
  setOption,
}: {
  problem: Problem;
  evalResult: EvaluationResult | null;
  option: RenderingOption;
  setOption: (fn: (option: RenderingOption) => RenderingOption) => void;
}) {
  const instruments = Array.from(new Set(problem.musicians)).sort(
    (a, b) => a - b,
  );

  return (
    <div className="w-full">
      <ProblemInfo problem={problem} evalResult={evalResult} />

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
                  return { ...o, tasteHeatmapInstrument: undefined };
                });
              } else {
                setOption((o) => {
                  return {
                    ...o,
                    tasteHeatmapInstrument: parseInt(e.target.value),
                  };
                });
              }
            }}
            value={
              option.tasteHeatmapInstrument === undefined
                ? "Pick one"
                : option.tasteHeatmapInstrument
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
      </div>
    </div>
  );
}
