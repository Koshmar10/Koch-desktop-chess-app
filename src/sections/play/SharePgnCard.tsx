import { useEffect, useRef, useState } from "react";
import { Check, Copy } from "lucide-react";
import { PrimaryButton } from "../../components/buttons";
import { NOTATION_TEXTAREA_CLASS } from "../../components/controlStyles";
import DialogCard, { WIDE_DIALOG_PX } from "../../components/DialogCard";
import { useGameContext } from "./GameContext";
import { buildLiveGamePgn } from "./buildPgn";

const ICON_SIZE = 16;
const PGN_ROWS = 14;

const SharePgnCard = () => {
  const { game, closeSharePgn } = useGameContext();
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  // Tracks which PGN the "Copied" state belongs to, rather than a plain
  // boolean reset in an effect — a fresh PGN (e.g. re-opening after more
  // moves were played) naturally reads as "not copied" with no reset needed.
  const [copiedFor, setCopiedFor] = useState<string | null>(null);
  // Hooks run unconditionally above the `!game` guard below, so this falls
  // back to "" rather than skipping the effect/render on a null game.
  const pgn = game ? buildLiveGamePgn(game) : "";
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
    <DialogCard
      title="Share game"
      onClose={closeSharePgn}
      widthPx={WIDE_DIALOG_PX}
    >
      <textarea
        ref={textareaRef}
        value={pgn}
        readOnly
        rows={PGN_ROWS}
        onFocus={(e) => e.currentTarget.select()}
        spellCheck={false}
        className={NOTATION_TEXTAREA_CLASS}
      />

      <div className="flex justify-end">
        <PrimaryButton onClick={copy}>
          {copied ? <Check size={ICON_SIZE} /> : <Copy size={ICON_SIZE} />}
          {copied ? "Copied" : "Copy"}
        </PrimaryButton>
      </div>
    </DialogCard>
  );
};

export default SharePgnCard;
