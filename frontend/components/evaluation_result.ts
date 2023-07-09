export interface EvaluationResult {
  score: number;
  musicians: number[];
  instruments: number[];
  attendees: number[];

  detailed_musicians: number[];
  detailed_attendees: number[];
  detailed_instruments: number[];
}
