import { EvaluationResult } from "@/components/evaluation_result";
import { formatNumber } from "@/components/number_format";
import { Problem } from "@/components/problems";
import { RenderingOption } from "@/components/visualizer/renderer";
import { orderBy } from "natural-orderby";
import { useMemo, useState } from "react";
import tinycolor from "tinycolor2";

function Instruments({
  problem,
  evalResult,
  option,
}: {
  problem: Problem;
  evalResult: EvaluationResult | null;
  option: RenderingOption;
}) {
  const [instrumentsPage, setInstrumentsPage] = useState(1);
  const [order, setOrder] = useState("by-instr");
  const hasLockedItem = option.lockedItem?.kind === "musician";

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
        [(v) => evalResult?.instruments?.at(v), (v) => v],
        ["desc", "asc"],
      );
      break;
    case "by-score-asc":
      instrumentsKeys = orderBy(
        instrumentsKeys,
        [(v) => evalResult?.instruments?.at(v), (v) => v],
        ["asc", "asc"],
      );
      break;
    case "by-locked-score-desc":
      if (hasLockedItem) {
        instrumentsKeys = orderBy(
          instrumentsKeys,
          [(v) => evalResult?.detailed_instruments?.at(v), (v) => v],
          ["desc", "asc"],
        );
      }
      break;
    case "by-locked-score-asc":
      if (hasLockedItem) {
        instrumentsKeys = orderBy(
          instrumentsKeys,
          [(v) => evalResult?.detailed_instruments?.at(v), (v) => v],
          ["asc", "asc"],
        );
      }
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
        {hasLockedItem ? (
          <>
            <option value="by-locked-score-desc">Lockedスコアの高い順</option>
            <option value="by-locked-score-asc">Lockedスコアの低い順</option>
          </>
        ) : null}
      </select>
      <table className="table w-auto">
        <thead>
          <tr>
            <th></th>
            <th></th>
            <th>奏者の人数</th>
            <th>スコア</th>
            {hasLockedItem ? <th>Lockedスコア</th> : null}
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
                  {formatNumber(evalResult?.instruments.at(instr))}
                </td>
                {hasLockedItem ? (
                  <td className="font-mono text-right">
                    {formatNumber(evalResult?.detailed_instruments.at(instr))}
                  </td>
                ) : null}
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
  option,
}: {
  problem: Problem;
  evalResult: EvaluationResult | null;
  option: RenderingOption;
}) {
  const [page, setPage] = useState(1);
  const [order, setOrder] = useState("by-musician");
  const hasLockedItem = option.lockedItem?.kind === "attendee";

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
        [(v) => evalResult?.musicians?.at(v), (v) => v],
        ["desc", "asc"],
      );
      break;
    case "by-score-asc":
      musicianKeys = orderBy(
        musicianKeys,
        [(v) => evalResult?.musicians?.at(v), (v) => v],
        ["asc", "asc"],
      );
      break;
    case "by-locked-score-desc":
      if (hasLockedItem) {
        musicianKeys = orderBy(
          musicianKeys,
          [(v) => evalResult?.detailed_musicians?.at(v), (v) => v],
          ["desc", "asc"],
        );
      }
      break;
    case "by-locked-score-asc":
      if (hasLockedItem) {
        musicianKeys = orderBy(
          musicianKeys,
          [(v) => evalResult?.detailed_musicians?.at(v), (v) => v],
          ["asc", "asc"],
        );
      }
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
        {hasLockedItem ? (
          <>
            <option value="by-locked-score-desc">Lockedスコアの高い順</option>
            <option value="by-locked-score-asc">Lockedスコアの低い順</option>
          </>
        ) : null}
      </select>
      <table className="table w-auto">
        <thead>
          <tr>
            <th></th>
            <th>楽器</th>
            <th></th>
            <th>スコア</th>
            {hasLockedItem ? <th>Lockedスコア</th> : null}
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
                  {formatNumber(evalResult?.musicians.at(m))}
                </td>
                {hasLockedItem ? (
                  <td className="font-mono text-right">
                    {formatNumber(evalResult?.detailed_musicians.at(m))}
                  </td>
                ) : null}
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

function Control({
  problem,
  option,
  setOption,
}: {
  problem: Problem;
  option: RenderingOption;
  setOption: (fn: (option: RenderingOption) => RenderingOption) => void;
}) {
  const instruments = Array.from(new Set(problem.musicians)).sort(
    (a, b) => a - b,
  );

  return (
    <div className="flex">
      <div className="form-control w-1/2">
        <div className="w-fit">
          <label className="label cursor-pointer space-x-2">
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
            <span className="label-text">
              奏者の寄与スコアでヒートマップ表示
            </span>
          </label>
          <label className="label cursor-pointer space-x-2">
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
            <span className="label-text">
              観客の寄与スコアでヒートマップ表示
            </span>
          </label>
          <label className="label cursor-pointer space-x-2 justify-normal">
            <input
              type="checkbox"
              className="checkbox"
              checked={option.useBipolarHeatmap ?? false}
              onChange={(e) => {
                setOption((o) => {
                  return {
                    ...o,
                    useBipolarHeatmap: e.target.checked,
                  };
                });
              }}
            />
            <span className="label-text">ヒートマップ表示を極端にする</span>
          </label>
        </div>
      </div>

      <div className="form-control w-1/2">
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
    </div>
  );
}

function ClipboardIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      className="w-6 h-6"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M9 12h3.75M9 15h3.75M9 18h3.75m3 .75H18a2.25 2.25 0 002.25-2.25V6.108c0-1.135-.845-2.098-1.976-2.192a48.424 48.424 0 00-1.123-.08m-5.801 0c-.065.21-.1.433-.1.664 0 .414.336.75.75.75h4.5a.75.75 0 00.75-.75 2.25 2.25 0 00-.1-.664m-5.8 0A2.251 2.251 0 0113.5 2.25H15c1.012 0 1.867.668 2.15 1.586m-5.8 0c-.376.023-.75.05-1.124.08C9.095 4.01 8.25 4.973 8.25 6.108V8.25m0 0H4.875c-.621 0-1.125.504-1.125 1.125v11.25c0 .621.504 1.125 1.125 1.125h9.75c.621 0 1.125-.504 1.125-1.125V9.375c0-.621-.504-1.125-1.125-1.125H8.25zM6.75 12h.008v.008H6.75V12zm0 3h.008v.008H6.75V15zm0 3h.008v.008H6.75V18z"
      />
    </svg>
  );
}

export default function VisualizerControl({
  problem,
  evalResult,
  option,
  setOption,
  rawSolution,
  setRawSolution,
  parseError,
}: {
  problem: Problem;
  evalResult: EvaluationResult | null;
  option: RenderingOption;
  setOption: (fn: (option: RenderingOption) => RenderingOption) => void;
  rawSolution: string;
  setRawSolution: (s: string) => void;
  parseError: any;
}) {
  return (
    <div className="overflow-x-auto space-y-4 w-full">
      <div className="flex justify-between items-center">
        <p className="text-4xl">
          スコア:
          <span className="font-extrabold pl-3">
            {formatNumber(evalResult?.score)}
          </span>
        </p>
        <div className="w-1/4 flex">
          <textarea
            placeholder="Solution"
            className="textarea textarea-bordered font-mono w-[16ch]"
            onChange={(e) => setRawSolution(e.target.value)}
            value={rawSolution}
          ></textarea>
          <button
            className="btn btn-xs"
            onClick={async () => {
              setRawSolution(await navigator.clipboard.readText());
            }}
          >
            <ClipboardIcon />
          </button>
        </div>
      </div>
      <pre>
        <code>{parseError ? `${parseError}` : null}</code>
      </pre>

      <div className="divider"></div>

      <Control option={option} setOption={setOption} problem={problem} />

      <div className="divider"></div>

      <div className="flex">
        <div className="w-1/2">
          <Instruments
            problem={problem}
            evalResult={evalResult}
            option={option}
          />
        </div>
        <div className="divider divider-horizontal"></div>
        <div className="w-1/2">
          <Musicians
            problem={problem}
            evalResult={evalResult}
            option={option}
          />
        </div>
      </div>
    </div>
  );
}
