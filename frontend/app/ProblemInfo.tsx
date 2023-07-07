"use client";

import { Problem } from "@/components/problems";

export default function ProblemInfo({ problem }: { problem: Problem }) {
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
