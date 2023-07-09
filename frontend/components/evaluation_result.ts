export interface MusicianStat {
  score: number;
}

export interface InstrumentStat {
  score: number;
}

export interface EvaluationResult {
  score: number;
  musicians: MusicianStat[];
  instruments: InstrumentStat[];
}