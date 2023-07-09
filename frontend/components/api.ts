"use client";

import axios from "axios";
import type { AxiosResponse } from "axios";
import useSWR from "swr";
import { Problem, Solution } from "./problems";

const client = axios.create({
  baseURL: "https://icfpc2023-backend-uadsges7eq-an.a.run.app/",
});

export interface ProblemMetadata {
  id: number;
}

export interface SolutionMetadata {
  uuid: string;
  problem_id: number;
  created: string;
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
