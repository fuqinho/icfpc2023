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
  return { data: data?.data, error, isLoading };
}

export async function loadSolutionSpec(uuid: string): Promise<Solution> {
  const response = (await client.get(
    `api/solutions/${uuid}/spec`,
  )) as AxiosResponse<Solution>;
  return response.data;
}
