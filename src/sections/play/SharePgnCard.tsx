import { useEffect, useRef, useState } from "react";
import { Check, Copy, X } from "lucide-react";
import { useGameContext } from "./GameContext";
import { buildPgn } from "./buildPgn";

interface CloseButtonProps {
  onClick: () => void;
}

const CloseButton = ({ onClick }: CloseButtonProps) => (
  <button
    type="button"
    aria-label="Close"
    onClick={onClick}
    className="absolute top-3 right-3 p-1 rounded-md text-foreground/40 hover:text-foreground/80 hover:bg-primary/10 transition-colors"
  >
    <X size={18} />
  </button>
);

const SharePgnCard = () => {
  const { game, closeSharePgn } = useGameContext();
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  // Tracks which PGN the "Copied" state belongs to, rather than a plain
  // boolean reset in an effect — a fresh PGN (e.g. re-opening after more
  // moves were played) naturally reads as "not copied" with no reset needed.
  const [copiedFor, setCopiedFor] = useState<string | null>(null);
  // Hooks run unconditionally above the `!game` guard below, so this falls
  // back to "" rather than skipping the effect/render on a null game.
  const pgn = game ? buildPgn(game) : "";
  const copied = copiedFor === pgn;

  useEffect(() => {
    textareaRef.current?.select();
  }, [pgn]);

  if (!game) return null;

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(pgn);
    } catch {
      // Clipboard API unavailable/denied — fall back to the classic
      // select-then-copy, which works off a plain DOM selection.
      textareaRef.current?.select();
      document.execCommand("copy");
    }
    setCopiedFor(pgn);
  };

  return (
    <div className="relative flex w-[min(480px,92vw)] flex-col gap-4 rounded-xl bg-card border-[1px] border-border/80 p-6 shadow-lg">
      <CloseButton onClick={closeSharePgn} />

      <h2 className="text-xl font-medium text-foreground/90">Share game</h2>

      <textarea
        ref={textareaRef}
        value={pgn}
        readOnly
        onFocus={(e) => e.currentTarget.select()}
        spellCheck={false}
        className="h-56 w-full resize-none rounded-md border border-border bg-input/30 p-2 font-mono text-xs text-foreground outline-none focus:border-primary"
      />

      <button
        type="button"
        onClick={copy}
        className="flex items-center justify-center gap-2 self-end rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90"
      >
        {copied ? <Check size={16} /> : <Copy size={16} />}
        {copied ? "Copied" : "Copy"}
      </button>
    </div>
  );
};

export default SharePgnCard;
