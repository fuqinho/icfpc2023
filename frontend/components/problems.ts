import problem1 from "../../problems/1.json";

export interface Attendee {
  x: number;
  y: number;
  tastes: number[];
}

export interface Pillar {
  center: number[];
  radius: number;
}

export interface Problem {
  room_width: number;
  room_height: number;
  stage_width: number;
  stage_height: number;
  stage_bottom_left: number[];
  musicians: number[];
  attendees: Attendee[];
  pillars: Pillar[];
}

export interface Musician {
  x: number;
  y: number;
}

export interface Solution {
  problem_id: number;
  placements: Musician[];
}

export const problems = [problem1];

export function readProblem(problemID: number): Problem {
  return problems[problemID - 1];
}
