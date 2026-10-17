import { useState } from "react";
import { ClipboardPaste } from "lucide-react";
import DialogCard, { NARROW_DIALOG_PX } from "../../../components/DialogCard";
import { TooltipButton } from "../../../components/TooltipButton";
import NotationField from "./NotationField";

const ICON_SIZE = 18;
const NOTATION_ROWS = 14;

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
    <DialogCard title="Import a FEN or PGN" widthPx={NARROW_DIALOG_PX}>
      <NotationField
        id="import-textarea"
        label="FEN or PGN"
        value={notation}
        rows={NOTATION_ROWS}
        onChange={setNotation}
        autoFocus
      />

      {error && <p className="text-sm text-destructive">{error}</p>}

      <div className="flex justify-end">
        <TooltipButton
          icon={<ClipboardPaste size={ICON_SIZE} />}
          tooltip="Load into the sandbox"
          onClick={load}
          disabled={isLoading || notation.trim() === ""}
        />
      </div>
    </DialogCard>
  );
};

export default ImportPgnFenOverlay;
