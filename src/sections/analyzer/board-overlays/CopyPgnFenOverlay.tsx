interface CopyPgnFenOverlayProps {
  // The whole game, and the position on screen.
  pgn: string;
  fen: string;
}

const CopyPgnFenOverlay = ({ pgn, fen }: CopyPgnFenOverlayProps) => {
  return (
    <div className="flex flex-col items-center justify-start gap-4 p-4 bg-card rounded-lg w-[50%] h-[72%]">
      <h1 className="text-card-foreground/90">Copy PGN / FEN of game</h1>

      <div className="flex flex-col gap-1">
        <label
          htmlFor="fen-textarea"
          className="text-sm text-card-foreground/80 ml-1"
        >
          FEN
        </label>
        <textarea
          className="bg-secondary/30 rounded-md text-sm p-1"
          name="Fen"
          id="fen-textarea"
          cols={35}
          rows={1}
          value={fen}
          readOnly
          spellCheck={false}
          // Selects it all on focus, so one click and a copy takes the lot.
          onFocus={(e) => e.currentTarget.select()}
        ></textarea>
      </div>

      <div className="flex flex-col gap-1">
        <label
          htmlFor="pgn-textarea"
          className="text-sm text-card-foreground/80 ml-1"
        >
          PGN
        </label>
        <textarea
          className="bg-secondary/30 rounded-md text-sm p-1"
          name="Pgn"
          id="pgn-textarea"
          cols={35}
          rows={14}
          value={pgn}
          readOnly
          spellCheck={false}
          onFocus={(e) => e.currentTarget.select()}
        ></textarea>
      </div>
    </div>
  );
};

export default CopyPgnFenOverlay;
