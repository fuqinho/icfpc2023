
import type * as wasm from "wasm";

export default function Problem({ problem }: { problem: wasm.RawProblem }) {
    return <div>Hello {problem}</div>
}
