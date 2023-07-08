import { Problem } from "@/components/problems";
import { RenderingOption } from "@/components/visualizer/renderer";

function ProblemInfo({ problem }: { problem: Problem }) {
  const instruments = new Map<number, number[]>();
  for (let i = 0; i < problem.musicians.length; i++) {
    const instr = problem.musicians[i];
    if (!instruments.has(instr)) {
      instruments.set(instr, []);
    }
    instruments.get(instr)?.push(i);
  }
  return (
    <div>
      <div className="overflow-x-auto">
        <table className="table">
          <tbody>
            <tr>
              <td>観客</td>
              <td>{problem.attendees.length}</td>
            </tr>
            <tr>
              <td>奏者の数</td>
              <td>{problem.musicians.length}</td>
            </tr>
            <tr>
              <td>楽器の種類</td>
              <td>{instruments.size}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  );
}

export default function VisualizerControl({
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
    <div>
      <ProblemInfo problem={problem} />
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
  );
}
