import { useState } from "react";
import { ClipboardPaste } from "lucide-react";
import { TooltipButton } from "../../../components/TooltipButton";

const ICON_SIZE = 18;

interface ImportPgnFenOverlayProps {
  // Loads a pasted FEN or PGN into the sandbox. Rejects with a message to
  // show if it reads as neither.
  onLoad: (notation: string) => Promise<void>;
}

const ImportPgnFenOverlay = ({ onLoad }: ImportPgnFenOverlayProps) => {
  const [notation, setNotation] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  // On success the overlay closes, so only a failure has anything to show.
  const load = async () => {
    setIsLoading(true);
    setError(null);
    try {
      await onLoad(notation);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="flex flex-col items-center justify-start gap-4 p-4 bg-card rounded-lg w-[50%] h-[72%]">
      <h1 className="text-card-foreground/90">Import a FEN or PGN</h1>

      <div className="flex flex-col gap-1">
        <label
          htmlFor="import-textarea"
          className="text-sm text-card-foreground/80 ml-1"
        >
          FEN or PGN
        </label>
        <textarea
          className="bg-secondary/30 rounded-md text-sm p-1"
          id="import-textarea"
          cols={35}
          rows={14}
          value={notation}
          onChange={(e) => setNotation(e.target.value)}
          spellCheck={false}
          autoFocus
        ></textarea>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      <TooltipButton
        icon={<ClipboardPaste size={ICON_SIZE} />}
        tooltip="Load into the sandbox"
        onClick={load}
        disabled={isLoading || notation.trim() === ""}
      />
    </div>
  );
};

export default ImportPgnFenOverlay;
