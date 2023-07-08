"use client";

import Link from "next/link";

export default function NavBar() {
  return (
    <div className="navbar bg-neutral text-neutral-content">
      <div className="flex-none">
        <Link className="btn btn-ghost normal-case text-xl" href="/">
          Spica [kari]
        </Link>
      </div>
      <div className="flex-1">
        <ul className="menu menu-horizontal px-1"></ul>
      </div>
    </div>
  );
}
