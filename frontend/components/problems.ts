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
