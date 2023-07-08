"use client";

import axios from "axios";
import type { AxiosResponse } from "axios";
import useSWR from "swr";
import { Problem } from "./problems";

const client = axios.create({
  baseURL: "https://icfpc2023-backend-uadsges7eq-an.a.run.app/",
});

export interface ProblemListEntry {
  id: number;
}

export function useProblemList() {
  const { data, error, isLoading } = useSWR<AxiosResponse<ProblemListEntry[]>>(
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
