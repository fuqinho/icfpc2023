import problem1 from "../../problems/1.json";
import problem2 from "../../problems/2.json";
import problem3 from "../../problems/3.json";
import problem4 from "../../problems/4.json";
import problem5 from "../../problems/5.json";
import problem6 from "../../problems/6.json";
import problem7 from "../../problems/7.json";
import problem8 from "../../problems/8.json";
import problem9 from "../../problems/9.json";
import problem10 from "../../problems/10.json";
import problem11 from "../../problems/11.json";
import problem12 from "../../problems/12.json";
import problem13 from "../../problems/13.json";
import problem14 from "../../problems/14.json";
import problem15 from "../../problems/15.json";
import problem16 from "../../problems/16.json";
import problem17 from "../../problems/17.json";
import problem18 from "../../problems/18.json";
import problem19 from "../../problems/19.json";
import problem20 from "../../problems/20.json";
import problem21 from "../../problems/21.json";
import problem22 from "../../problems/22.json";
import problem23 from "../../problems/23.json";
import problem24 from "../../problems/24.json";
import problem25 from "../../problems/25.json";
import problem26 from "../../problems/26.json";
import problem27 from "../../problems/27.json";
import problem28 from "../../problems/28.json";
import problem29 from "../../problems/29.json";
import problem30 from "../../problems/30.json";
import problem31 from "../../problems/31.json";
import problem32 from "../../problems/32.json";
import problem33 from "../../problems/33.json";
import problem34 from "../../problems/34.json";
import problem35 from "../../problems/35.json";
import problem36 from "../../problems/36.json";
import problem37 from "../../problems/37.json";
import problem38 from "../../problems/38.json";
import problem39 from "../../problems/39.json";
import problem40 from "../../problems/40.json";
import problem41 from "../../problems/41.json";
import problem42 from "../../problems/42.json";
import problem43 from "../../problems/43.json";
import problem44 from "../../problems/44.json";
import problem45 from "../../problems/45.json";
import problem46 from "../../problems/46.json";
import problem47 from "../../problems/47.json";
import problem48 from "../../problems/48.json";
import problem49 from "../../problems/49.json";
import problem50 from "../../problems/50.json";
import problem51 from "../../problems/51.json";
import problem52 from "../../problems/52.json";
import problem53 from "../../problems/53.json";
import problem54 from "../../problems/54.json";
import problem55 from "../../problems/55.json";
import problem56 from "../../problems/56.json";
import problem57 from "../../problems/57.json";
import problem58 from "../../problems/58.json";
import problem59 from "../../problems/59.json";
import problem60 from "../../problems/60.json";
import problem61 from "../../problems/61.json";
import problem62 from "../../problems/62.json";
import problem63 from "../../problems/63.json";
import problem64 from "../../problems/64.json";
import problem65 from "../../problems/65.json";
import problem66 from "../../problems/66.json";
import problem67 from "../../problems/67.json";
import problem68 from "../../problems/68.json";
import problem69 from "../../problems/69.json";
import problem70 from "../../problems/70.json";
import problem71 from "../../problems/71.json";
import problem72 from "../../problems/72.json";
import problem73 from "../../problems/73.json";
import problem74 from "../../problems/74.json";
import problem75 from "../../problems/75.json";
import problem76 from "../../problems/76.json";
import problem77 from "../../problems/77.json";
import problem78 from "../../problems/78.json";
import problem79 from "../../problems/79.json";
import problem80 from "../../problems/80.json";
import problem81 from "../../problems/81.json";
import problem82 from "../../problems/82.json";
import problem83 from "../../problems/83.json";
import problem84 from "../../problems/84.json";
import problem85 from "../../problems/85.json";
import problem86 from "../../problems/86.json";
import problem87 from "../../problems/87.json";
import problem88 from "../../problems/88.json";
import problem89 from "../../problems/89.json";
import problem90 from "../../problems/90.json";

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

export const problems = [
  problem1,
  problem2,
  problem3,
  problem4,
  problem5,
  problem6,
  problem7,
  problem8,
  problem9,
  problem10,
  problem11,
  problem12,
  problem13,
  problem14,
  problem15,
  problem16,
  problem17,
  problem18,
  problem19,
  problem20,
  problem21,
  problem22,
  problem23,
  problem24,
  problem25,
  problem26,
  problem27,
  problem28,
  problem29,
  problem30,
  problem31,
  problem32,
  problem33,
  problem34,
  problem35,
  problem36,
  problem37,
  problem38,
  problem39,
  problem40,
  problem41,
  problem42,
  problem43,
  problem44,
  problem45,
  problem46,
  problem47,
  problem48,
  problem49,
  problem50,
  problem51,
  problem52,
  problem53,
  problem54,
  problem55,
  problem56,
  problem57,
  problem58,
  problem59,
  problem60,
  problem61,
  problem62,
  problem63,
  problem64,
  problem65,
  problem66,
  problem67,
  problem68,
  problem69,
  problem70,
  problem71,
  problem72,
  problem73,
  problem74,
  problem75,
  problem76,
  problem77,
  problem78,
  problem79,
  problem80,
  problem81,
  problem82,
  problem83,
  problem84,
  problem85,
  problem86,
  problem87,
  problem88,
  problem89,
  problem90,
];

export function readProblem(problemID: number): Problem {
  return problems[problemID - 1];
}
