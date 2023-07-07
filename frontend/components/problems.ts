import raw_problem1 from "../../problems/1.json";
import raw_problem2 from "../../problems/2.json";
import raw_problem3 from "../../problems/3.json";
import raw_problem4 from "../../problems/4.json";
import raw_problem5 from "../../problems/5.json";
import raw_problem6 from "../../problems/6.json";
import raw_problem7 from "../../problems/7.json";
import raw_problem8 from "../../problems/8.json";
import raw_problem9 from "../../problems/9.json";
import raw_problem10 from "../../problems/10.json";
import raw_problem11 from "../../problems/11.json";
import raw_problem12 from "../../problems/12.json";
import raw_problem13 from "../../problems/13.json";
import raw_problem14 from "../../problems/14.json";
import raw_problem15 from "../../problems/15.json";
import raw_problem16 from "../../problems/16.json";
import raw_problem17 from "../../problems/17.json";
import raw_problem18 from "../../problems/18.json";
import raw_problem19 from "../../problems/19.json";
import raw_problem20 from "../../problems/20.json";
import raw_problem21 from "../../problems/21.json";
import raw_problem22 from "../../problems/22.json";
import raw_problem23 from "../../problems/23.json";
import raw_problem24 from "../../problems/24.json";
import raw_problem25 from "../../problems/25.json";
import raw_problem26 from "../../problems/26.json";
import raw_problem27 from "../../problems/27.json";
import raw_problem28 from "../../problems/28.json";
import raw_problem29 from "../../problems/29.json";
import raw_problem30 from "../../problems/30.json";
import raw_problem31 from "../../problems/31.json";
import raw_problem32 from "../../problems/32.json";
import raw_problem33 from "../../problems/33.json";
import raw_problem34 from "../../problems/34.json";
import raw_problem35 from "../../problems/35.json";
import raw_problem36 from "../../problems/36.json";
import raw_problem37 from "../../problems/37.json";
import raw_problem38 from "../../problems/38.json";
import raw_problem39 from "../../problems/39.json";
import raw_problem40 from "../../problems/40.json";
import raw_problem41 from "../../problems/41.json";
import raw_problem42 from "../../problems/42.json";
import raw_problem43 from "../../problems/43.json";
import raw_problem44 from "../../problems/44.json";
import raw_problem45 from "../../problems/45.json";

export interface Attendee {
  x: number;
  y: number;
  tastes: number[];
}

export interface Pillar {}

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
  placements: Musician[];
}

export const problems = new Map<string, Problem>([
  ["problem1", raw_problem1 as Problem],
  ["problem2", raw_problem2 as Problem],
  ["problem3", raw_problem3 as Problem],
  ["problem4", raw_problem4 as Problem],
  ["problem5", raw_problem5 as Problem],
  ["problem6", raw_problem6 as Problem],
  ["problem7", raw_problem7 as Problem],
  ["problem8", raw_problem8 as Problem],
  ["problem9", raw_problem9 as Problem],
  ["problem10", raw_problem10 as Problem],
  ["problem11", raw_problem11 as Problem],
  ["problem12", raw_problem12 as Problem],
  ["problem13", raw_problem13 as Problem],
  ["problem14", raw_problem14 as Problem],
  ["problem15", raw_problem15 as Problem],
  ["problem16", raw_problem16 as Problem],
  ["problem17", raw_problem17 as Problem],
  ["problem18", raw_problem18 as Problem],
  ["problem19", raw_problem19 as Problem],
  ["problem20", raw_problem20 as Problem],
  ["problem21", raw_problem21 as Problem],
  ["problem22", raw_problem22 as Problem],
  ["problem23", raw_problem23 as Problem],
  ["problem24", raw_problem24 as Problem],
  ["problem25", raw_problem25 as Problem],
  ["problem26", raw_problem26 as Problem],
  ["problem27", raw_problem27 as Problem],
  ["problem28", raw_problem28 as Problem],
  ["problem29", raw_problem29 as Problem],
  ["problem30", raw_problem30 as Problem],
  ["problem31", raw_problem31 as Problem],
  ["problem32", raw_problem32 as Problem],
  ["problem33", raw_problem33 as Problem],
  ["problem34", raw_problem34 as Problem],
  ["problem35", raw_problem35 as Problem],
  ["problem36", raw_problem36 as Problem],
  ["problem37", raw_problem37 as Problem],
  ["problem38", raw_problem38 as Problem],
  ["problem39", raw_problem39 as Problem],
  ["problem40", raw_problem40 as Problem],
  ["problem41", raw_problem41 as Problem],
  ["problem42", raw_problem42 as Problem],
  ["problem43", raw_problem43 as Problem],
  ["problem44", raw_problem44 as Problem],
  ["problem45", raw_problem45 as Problem],
]);
