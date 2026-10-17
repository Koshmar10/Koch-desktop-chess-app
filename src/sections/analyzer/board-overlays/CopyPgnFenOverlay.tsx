import DialogCard, { NARROW_DIALOG_PX } from "../../../components/DialogCard";
import NotationField from "./NotationField";

const FEN_ROWS = 2;
const PGN_ROWS = 14;

interface CopyPgnFenOverlayProps {
  // The whole game, and the position on screen.
  pgn: string;
  fen: string;
}

const CopyPgnFenOverlay = ({ pgn, fen }: CopyPgnFenOverlayProps) => (
  <DialogCard title="Copy PGN / FEN" widthPx={NARROW_DIALOG_PX}>
    <NotationField id="fen-textarea" label="FEN" value={fen} rows={FEN_ROWS} />
    <NotationField id="pgn-textarea" label="PGN" value={pgn} rows={PGN_ROWS} />
  </DialogCard>
);

export default CopyPgnFenOverlay;
