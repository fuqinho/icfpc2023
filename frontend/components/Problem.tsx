"use client"

import { useEffect, useState } from "react";
import type * as wasm from "wasm";

export default function Problem() {
    const [problem, setProblem] = useState<wasm.RawProblem | null>(null);

    useEffect(() => {
        (async () => {
            const wasm = await import("wasm");

            const problem = wasm.RawProblem.from_json(`
            {
                "room_width": 2000.0,
                "room_height": 5000.0,
                "stage_width": 1000.0,
                "stage_height": 200.0,
                "stage_bottom_left": [500.0, 0.0],
                "musicians": [0, 1, 0],
                "attendees": [
                { "x": 100.0, "y": 500.0, "tastes": [1000.0, -1000.0
                ] },
                { "x": 200.0, "y": 1000.0, "tastes": [200.0, 200.0]
                },
                { "x": 1100.0, "y": 800.0, "tastes": [800.0, 1500.0]
                }
                ]
            }`);
            setProblem(problem);
        })();
    })

    if (!problem) {
        return <div>loading...</div>
    }

    return <div>Room {problem.room_width()} {problem.room_height()}</div>
}
