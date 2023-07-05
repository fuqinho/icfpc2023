'use client';

import type * as lifegame from "lifegame-wasm";
import { useState, useEffect } from "react";

function LifeGameCell({ cell }: { cell: boolean }) {
  return (
    <td style={{
      width: '8px',
      height: '8px',
      backgroundColor: cell ? 'black' : 'white',
    }} />
  );
}

function LifeGameRow({ row }: { row: boolean[] }) {
  return (
    <tr>
      { row.map((cell, i) => <LifeGameCell key={i} cell={cell} />) }
    </tr>
  );
}

function useLifeGameWorld(): boolean[][] {
  const [game, setGame] = useState<lifegame.LifeGameWasm | null>(null);

  useEffect(() => {
    (async () => {
      const lifegame = await import('lifegame-wasm');
      const game = new lifegame.LifeGameWasm(19, 19);
      for (let i = 5; i < 14; i++) {
        for (let j = 5; j < 14; j++) {
          game.set(i, j, true);
        }
      }
      for (let i = 7; i < 12; i++) {
        for (let j = 7; j < 12; j++) {
          game.set(i, j, false);
        }
      }
      game.set(5, 7, false);
      game.set(6, 7, false);
      game.set(7, 12, false);
      game.set(7, 13, false);
      game.set(11, 5, false);
      game.set(11, 6, false);
      game.set(12, 11, false);
      game.set(13, 11, false);
      setGame(game);
    })();
  }, []);

  let [world, setWorld] = useState<boolean[][]>([]);

  useEffect(() => {
    if (!game) {
      return;
    }

    const ticket = window.setInterval(() => {
      game.tick();
      const linearWorld = game.world() as boolean[];
      const [h, w] = Array.from(game.size());
      const world = [];
      for (let i = 0; i < h; i++) {
        world.push(linearWorld.slice(i * w, (i + 1) * w));
      }
      setWorld(world);
    }, 200);

    return () => {
      window.clearInterval(ticket);
    };
  }, [game]);

  return world;
}

export default function LifeGameBoard() {
  const world = useLifeGameWorld();

  return (
    <table className="lifegame">
      { world.map((row, i) => <LifeGameRow key={i} row={row} />) }
    </table>
  );
}
