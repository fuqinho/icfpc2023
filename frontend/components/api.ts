"use client";

import axios from "axios";
import type { AxiosResponse } from "axios";
import useSWR from "swr";
import { Problem, Solution, problems, readProblem } from "./problems";
import { NO_BACKEND } from "./env";

const client = axios.create({
  baseURL: NO_BACKEND
    ? "fake://"
    : "https://icfpc2023-backend-uadsges7eq-an.a.run.app/",
});

export interface ProblemMetadata {
  id: number;
}

export interface SolutionMetadata {
  uuid: string;
  problem_id: number;
  created: string;
  solver: string;
  submission: SubmissionMetadata | null;
  evaluation: EvaluationMetadata | null;
}

export interface SubmissionMetadata {
  solution_uuid: string;
  id: string;
  state: "PROCESSING" | "FINISHED";
  accepted: boolean;
  score: number;
  error: string;
  created: string;
  updated: string;
}

export interface EvaluationMetadata {
  solution_uuid: string;
  accepted: boolean;
  score: number;
  error: string;
  created: string;
}

export interface Scoreboard {
  frozen: boolean;
  scoreboard: ScoreboardTeam[];
}

export interface ScoreboardTeam {
  username: string;
  score: number;
}

export function problemImage(problemID: number) {
  return `https://icfpc2023-backend-uadsges7eq-an.a.run.app/api/problems/${problemID}/image`;
}

export function solutionImage(solutionID: string) {
  return `https://icfpc2023-backend-uadsges7eq-an.a.run.app/api/solutions/${solutionID}/image`;
}

export function solutionURL(solutionID: string) {
  return `https://icfpc2023-backend-uadsges7eq-an.a.run.app/api/solutions/${solutionID}/spec`;
}

export function useProblemList() {
  const { data, error, isLoading } = useSWR<AxiosResponse<ProblemMetadata[]>>(
    {
      method: "get",
      url: "/api/problems",
    },
    client,
  );
  if (NO_BACKEND) {
    const problemMetadata = problems.map((_, i) => ({ id: i + 1 }));
    return { data: problemMetadata, error: null, isLoading: false };
  }
  return { data: data?.data, error, isLoading };
}

export function useProblemSpec(problemID: number | undefined) {
  const { data, error, isLoading } = useSWR<AxiosResponse<Problem>>(
    problemID
      ? {
          method: "get",
          url: `/api/problems/${problemID}/spec`,
        }
      : null,
    client,
  );
  if (NO_BACKEND) {
    return {
      data: problemID && readProblem(problemID),
      error: null,
      isLoading: false,
    };
  }
  return { data: data?.data, error, isLoading };
}

export function useBestSolutions() {
  const { data, error, isLoading } = useSWR<AxiosResponse<SolutionMetadata[]>>(
    {
      method: "get",
      url: `/api/solutions`,
    },
    client,
  );

  if (NO_BACKEND) {
    return { data: [], error: null, isLoading: false };
  }

  const allSolutions = data?.data ?? [];
  allSolutions.sort(
    (a, b) => (b.submission?.score ?? -1e100) - (a.submission?.score ?? -1e100),
  );

  const bestSolutions = new Map<number, SolutionMetadata>();
  for (const s of allSolutions) {
    if (!bestSolutions.has(s.problem_id)) {
      bestSolutions.set(s.problem_id, s);
    }
  }

  return { data: bestSolutions, error, isLoading };
}

export function useSolutions() {
  const { data, error, isLoading } = useSWR<AxiosResponse<SolutionMetadata[]>>(
    {
      method: "get",
      url: `/api/solutions`,
    },
    client,
  );

  if (NO_BACKEND) {
    return { data: [], error: null, isLoading: false };
  }

  return { data: data?.data, error, isLoading };
}

export function useKnownSolutions(problemID: number | undefined) {
  const { data, error, isLoading } = useSWR<AxiosResponse<SolutionMetadata[]>>(
    problemID
      ? {
          method: "get",
          url: `/api/problems/${problemID}/solutions`,
        }
      : null,
    client,
  );

  if (NO_BACKEND) {
    return { data: [], error: null, isLoading: false };
  }

  if (data?.data) {
    data.data.sort(
      (a, b) =>
        (b.submission?.score ?? -1e100) - (a.submission?.score ?? -1e100),
    );
  }
  return { data: data?.data, error, isLoading };
}

export function useMismatchedSolutions() {
  const { data, error, isLoading } = useSWR<AxiosResponse<SolutionMetadata[]>>(
    {
      method: "get",
      url: `/api/solutions-mismatched`,
    },
    client,
  );
  return { data: data?.data, error, isLoading };
}

export async function loadSolutionSpec(uuid: string): Promise<Solution> {
  const response = (await client.get(
    `api/solutions/${uuid}/spec`,
  )) as AxiosResponse<Solution>;
  return response.data;
}

export function useScoreboard() {
  const { data, error, isLoading } = useSWR<AxiosResponse<Scoreboard>>(
    "https://api.icfpcontest.com/scoreboard",
    client,
  );
  return { data: data?.data, error, isLoading };
}
