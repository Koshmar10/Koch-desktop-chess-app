import { useEffect, useState } from "react";
import { Microscope, Play as PlayIcon, X } from "lucide-react";
import { LoadingSpinner } from "../../components/LoadingSpinner";
import PostGameStats from "./PostGameStats";
import { useGameContext } from "./GameContext";
import type { GameResult } from "../../api/bindings/GameResult";
import type { PieceColor } from "../../api/bindings/PieceColor";

// Win/lose/draw is real, from `game.state.result` + `humanColor` below.
// *Why* the game ended (checkmate vs. resignation vs. timeout) isn't in
// that message — `GameStateView` only carries the result, not the
// `TerminationReason`, so there's nothing to phrase that part from yet.
const getResultMessage = (
  result: GameResult,
  humanColor: PieceColor | null,
): string => {
  if (result === "Draw") return "Draw";
  const humanWon =
    (result === "WhiteWin" && humanColor === "white") ||
    (result === "BlackWin" && humanColor === "black");
  return humanWon ? "You Win" : "You Lose";
};

// Mock, deliberately — no rating system exists yet.
const MOCK_ELO_DELTA = 18;

const ELO_COUNT_UP_MS = 1000;

// Counts from 0 up to `target` once, on mount / whenever `target` changes.
const useCountUp = (target: number, durationMs: number) => {
  const [value, setValue] = useState(0);

  useEffect(() => {
    let frame: number;
    const start = performance.now();

    const tick = (now: number) => {
      const progress = Math.min((now - start) / durationMs, 1);
      setValue(Math.round(target * progress));
      if (progress < 1) frame = requestAnimationFrame(tick);
    };

    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [target, durationMs]);

  return value;
};

const GameResultCard = () => {
  const { analysis, game, humanColor, startGame, dismissResult } = useGameContext();
  const isGain = MOCK_ELO_DELTA >= 0;
  // Must run unconditionally (Rules of Hooks) — the `!game` guard below has
  // to come after every hook call, not before.
  const eloDelta = useCountUp(MOCK_ELO_DELTA, ELO_COUNT_UP_MS);

  // Only mounted (by `Play.tsx`) once `game` is non-null and finished — this
  // just narrows the type for what follows.
  if (!game) return null;

  return (
    <div className="relative h-[80%] flex flex-col items-center gap-4 py-6 px-8 rounded-xl bg-card border-[1px] border-border/80 shadow-lg">
      <button
        type="button"
        aria-label="Close"
        onClick={dismissResult}
        className="absolute top-3 right-3 p-1 rounded-md text-foreground/40 hover:text-foreground/80 hover:bg-primary/10 transition-colors"
      >
        <X size={18} />
      </button>

      <div className="flex flex-col items-center gap-1">
        <h2 className="text-2xl font-bold text-foreground/90 tracking-tight text-center">
          {getResultMessage(game.state.result, humanColor)}
        </h2>
        <span
          className={`text-xl font-semibold ${isGain ? "text-primary" : "text-destructive"}`}
        >
          {isGain ? "+" : ""}
          {eloDelta} Elo
        </span>
      </div>

      {analysis && game ? (
        <PostGameStats analysis={analysis} moveHistory={game.state.move_history} />
      ) : (
        <LoadingSpinner message="Analyzing game…" />
      )}

      <div className="flex items-center gap-3 mt-auto">
        <button
          type="button"
          onClick={startGame}
          className="flex items-center justify-center gap-2 w-34 px-2 py-2 rounded-md border-[1px] border-primary/40 text-foreground/90 text-md hover:bg-primary/10 transition-colors"
        >
          <PlayIcon size={16} />
          New Game
        </button>
        <button
          type="button"
          className="flex items-center justify-center gap-2 w-34 px-2 py-2 rounded-md border-[1px] border-primary/40 text-foreground/90 text-md hover:bg-primary/10 transition-colors"
        >
          <Microscope size={16} />
          Analyze
        </button>
      </div>
    </div>
  );
};

export default GameResultCard;
