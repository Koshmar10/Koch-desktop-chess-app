import { LoaderCircle } from "lucide-react";
import { evalBarWhiteFraction, formatEvalScore } from "../../board/evalBar";
import type { EngineSnapshot, EngineStatus, PvLine } from "../../types";

interface StatusPillProps {
  status: EngineStatus;
  onToggle: () => void;
}

const StatusPill = ({ status, onToggle }: StatusPillProps) => {
  if (status === "loading") {
    return (
      <span className="inline-flex items-center gap-2 rounded-md border border-border bg-card px-2 py-1 text-xs text-foreground/60">
        <LoaderCircle size={12} className="animate-spin" />
      </span>
    );
  }

  const isRunning = status === "running";
  const pillCls = isRunning
    ? "border-green-500/60 bg-green-700/30 text-green-500"
    : "border-destructive/60 bg-destructive/20 text-destructive";

  return (
    <button
      type="button"
      onClick={onToggle}
      title={isRunning ? "Stop engine" : "Start engine"}
      className={`inline-flex cursor-pointer items-center gap-2 rounded-md border px-2 py-1 ${pillCls}`}
    >
      <span className="relative flex h-2 w-2">
        {isRunning && (
          <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-green-400 opacity-75" />
        )}
        <span
          className={`relative inline-flex h-2 w-2 rounded-full ${isRunning ? "bg-green-400" : "bg-destructive"}`}
        />
      </span>
    </button>
  );
};

const PvLineRow = ({ line }: { line: PvLine }) => {
  // The one place that decides what a score's sign means is
  // `evalBarWhiteFraction`; reusing it here keeps the chip and the bar
  // from ever disagreeing about who a position favours.
  const favoursWhite = evalBarWhiteFraction(line.score) >= 0.5;
  const chipCls = favoursWhite
    ? "bg-neutral-100 text-neutral-900"
    : "bg-neutral-900 text-neutral-100";
  const moves = line.moves.join(" ");

  return (
    <div className="flex flex-row items-center gap-2">
      <span
        className={`w-13 shrink-0 rounded px-1 text-right font-mono tabular-nums ${chipCls}`}
      >
        {formatEvalScore(line.score)}
      </span>
      <span className="truncate text-foreground/80" title={moves}>
        {moves}
      </span>
    </div>
  );
};

interface EngineLinesProps {
  // Null until the first `info` line arrives for the current position.
  snapshot: EngineSnapshot | null;
  status: EngineStatus;
  onToggleStatus: () => void;
}

const EngineLines = ({
  snapshot,
  status,
  onToggleStatus,
}: EngineLinesProps) => {
  const hasLines = snapshot !== null && snapshot.lines.length > 0;

  return (
    <div className="flex min-h-0 w-full flex-1 flex-col gap-2">
      {/* No "Engine" heading — the tab above already says so. */}
      <div className="flex w-full items-center justify-between">
        <span className="text-sm text-foreground/40 tabular-nums">
          {snapshot === null ? "" : `depth ${snapshot.depth}`}
        </span>
        <StatusPill status={status} onToggle={onToggleStatus} />
      </div>

      <div className="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto text-sm">
        {hasLines ? (
          // Rendered in arrival order. Stockfish ranks multipv lines
          // best-first already and every score is White-relative, so
          // sorting by score here picks the *worst* line whenever Black
          // is to move — the old app's bug, see KOCH-HANDOFF.md §3.
          snapshot.lines.map((line) => (
            <PvLineRow key={line.multipv} line={line} />
          ))
        ) : (
          <span className="text-foreground/40 italic">No lines yet</span>
        )}
      </div>
    </div>
  );
};

export default EngineLines;
