export interface MusicianStat {
  score: number;
}

export interface InstrumentStat {
  score: number;
}

export interface AttendeeStat {
  score: number;
}

export interface EvaluationResult {
  score: number;
  musicians: MusicianStat[];
  instruments: InstrumentStat[];
  attendees: AttendeeStat[];

  detailed_musicians: number[];
  detailed_attendees: number[];
  detailed_instruments: number[];
}
