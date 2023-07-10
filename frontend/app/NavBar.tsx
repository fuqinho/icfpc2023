"use client";

import { DateTime } from "luxon";
import Link from "next/link";
import { useEffect, useState } from "react";

const DEADLINE = DateTime.fromISO("2023-07-10T12:00:00Z");

export default function NavBar() {
  const [hour, setHour] = useState(0);
  const [min, setMin] = useState(0);

  useEffect(() => {
    const refreshClock = () => {
      const diff = DEADLINE.diffNow(["minute", "hours"]);
      setHour(diff.hours);
      setMin(Math.floor(diff.minutes));
    };
    refreshClock();
    const timerId = setInterval(refreshClock, 1000);
    return () => clearInterval(timerId);
  }, [hour, min]);

  return (
    <div className="navbar bg-neutral text-neutral-content">
      <div className="flex-none">
        <Link
          className="btn btn-ghost normal-case text-xl hover:text-white hover:underline focus:text-white"
          href="/"
        >
          Spica [kari]
        </Link>
      </div>
      <div className="flex-1">
        <ul className="menu menu-horizontal px-1">
          <li>
            <Link
              className="hover:text-white hover:underline focus:text-white"
              href="/"
            >
              Problems
            </Link>
          </li>
          <li>
            <Link
              className="hover:text-white hover:underline focus:text-white"
              href="/mismatched-solutions"
            >
              Mismatched solutions
            </Link>
          </li>
          <li>
            <Link
              className="hover:text-white hover:underline focus:text-white"
              href="/matrix"
            >
              点数マトリックス
            </Link>
          </li>
        </ul>
      </div>

      <div>
        {hour > 0 || min > 0 ? (
          <div className="font-mono">
            <p>〆切 {DEADLINE.toFormat("ccc HH:mm")}</p>
            <div className="text-2xl">
              あと {hour}時間 {min}分
            </div>
          </div>
        ) : null}
        <ul className="menu menu-horizontal px-1">
          <li>
            <Link
              className="hover:text-white hover:underline focus:text-white"
              href="https://www.icfpcontest.com/scoreboard"
              target="_blank"
            >
              公式
            </Link>
          </li>
        </ul>
      </div>
    </div>
  );
}
