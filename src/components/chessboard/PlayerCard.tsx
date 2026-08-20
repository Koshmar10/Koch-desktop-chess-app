import { ChessKing, Clock } from "lucide-react";
import { PIECE_IMAGES } from "./lib/pieceImages";
import type { PieceColor, PieceKind } from "./lib/types";

interface PlayerCardProps {
  display: boolean;
  color: PieceColor;
  player: string;
  clock?: string;
  isTurn?: boolean;
  piecesTaken?: [PieceKind, PieceColor][];
  materialDiff?: number;
}

const AVATAR_BG_CLASS: Record<PieceColor, string> = {
  white: "bg-gray-200 text-gray-900",
  black: "bg-gray-900 text-gray-100",
};

const BADGE_BG_CLASS: Record<PieceColor, string> = {
  white: "bg-white",
  black: "bg-black border-white/60 border-[1px]",
};

const CAPTURED_PIECE_FILTER: Record<PieceColor, string> = {
  white: "drop-shadow(0 0 2px black)",
  black: "drop-shadow(0 0 1px white)",
};

const CAPTURED_PIECE_SIZE = 28 * 0.9;
const CAPTURED_PIECE_OVERLAP = -12;
const CAPTURED_PIECE_BASE_Z_INDEX = 10;

interface PlayerAvatarProps {
  color: PieceColor;
}

const PlayerAvatar = ({ color }: PlayerAvatarProps) => {
  return (
    <div className={`relative flex items-center justify-center w-12 h-12 rounded-md shadow-inner ${AVATAR_BG_CLASS[color]}`}>
      <ChessKing className="w-8 h-8 opacity-80" />
      <div className={`absolute -bottom-1 -right-1 w-3 h-3 rounded-full border-2 border-background ${BADGE_BG_CLASS[color]}`} />
    </div>
  );
};

interface PlayerNameProps {
  player: string;
}

const PlayerName = ({ player }: PlayerNameProps) => {
  return (
    <span className="text-xl font-semibold text-foreground/90 tracking-tight leading-tight truncate max-w-[300px]">
      {player}
    </span>
  );
};

interface TakenPiecesProps {
  piecesTaken?: [PieceKind, PieceColor][];
  materialDiff?: number;
}

const TakenPieces = ({ piecesTaken, materialDiff }: TakenPiecesProps) => {
  return (
    <div className="flex flex-row">
      {piecesTaken && piecesTaken.map(([kind, pieceColor], idx) => (
        <img
          key={idx}
          src={PIECE_IMAGES[pieceColor][kind]}
          alt=""
          style={{
            zIndex: CAPTURED_PIECE_BASE_Z_INDEX + idx,
            width: CAPTURED_PIECE_SIZE,
            height: CAPTURED_PIECE_SIZE,
            marginLeft: idx === 0 ? 0 : CAPTURED_PIECE_OVERLAP,
            filter: CAPTURED_PIECE_FILTER[pieceColor],
          }}
          draggable={false}
        />
      ))}
      {materialDiff && materialDiff > 0 && (
        <span className="ml-2 px-2 py-0.5 text-white/90 font-semibold text-sm">
          +{materialDiff}
        </span>
      )}
    </div>
  );
};

export const PlayerCard = ({ display, player, color, clock, isTurn, piecesTaken, materialDiff }: PlayerCardProps) => {
  if (!display) return null;

  return (
    <div className="flex items-center gap-3 w-full px-3 py-2 rounded-lg bg-secondary/60 shadow-sm transition-colors border-[1px] border-border/80">
      <PlayerAvatar color={color} />

      <div className="flex-1 flex items-center justify-between">
        <div className="flex flex-col gap-1 justify-center min-w-0 py-0">
          <PlayerName player={player} />
          <TakenPieces piecesTaken={piecesTaken} materialDiff={materialDiff} />
        </div>

        <div className="flex flex-col items-end ml-3">
          {typeof clock !== "undefined" && (
            <div className={`flex items-center gap-2 ${isTurn ? "text-foreground" : "text-foreground/40"}`}>
              <Clock className="w-5 h-5 text-foreground/40" />
              <span className="font-mono text-2xl tracking-widest">{clock}</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
