import { ChessKing, Clock, Loader2 } from "lucide-react";
import { PIECE_IMAGES } from "./lib/pieceImages";
import type { PieceColor, PieceKind } from "./lib/types";
import type { PlayerInfo } from "../../api/bindings/PlayerInfo";

interface PlayerCardProps {
  display: boolean;
  color: PieceColor;
  playerInfo: PlayerInfo;
  clock?: string;
  isTurn?: boolean;
  // True specifically while it's this player's turn *and* they're the
  // engine, not the human — the "waiting on Stockfish" signal, distinct
  // from isTurn (which is also true on the human's own turn).
  isThinking?: boolean;
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

// Inverted from the badge's own color — a white circle spins with a black
// loader and vice versa, so it reads against the (unchanged) badge/king color.
const SPINNER_COLOR_CLASS: Record<PieceColor, string> = {
  white: "black",
  black: "white",
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
  isThinking?: boolean;
}

const PlayerAvatar = ({ color, isThinking }: PlayerAvatarProps) => {
  return (
    <div
      className={`relative flex items-center justify-center w-12 h-12 rounded-md shadow-inner ${AVATAR_BG_CLASS[color]}`}
    >
      <ChessKing className="w-8 h-8 opacity-80" />
      <div
        className={`absolute -bottom-1 -right-1 rounded-full border-2 border-background flex items-center justify-center transition-all w-4 h-4 ${BADGE_BG_CLASS[color]}`
        }
      >
        {isThinking && (
          <Loader2
            size={10}
            color={SPINNER_COLOR_CLASS[color]}
            className={`animate-spin`}
          />
        )}
      </div>
    </div >
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
      {piecesTaken &&
        piecesTaken.map(([kind, pieceColor], idx) => (
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

export const PlayerCard = ({
  display,
  playerInfo,
  color,
  clock,
  isTurn,
  isThinking,
  piecesTaken,
  materialDiff,
}: PlayerCardProps) => {
  if (!display) return null;

  return (
    <div
      className={`flex items-center gap-3 w-full px-3 py-2 rounded-lg bg-secondary/60 shadow-sm transition-all h-20 ${isTurn ? "opacity-100" : "opacity-50"
        }`}
    >
      <PlayerAvatar color={color} isThinking={isThinking} />

      <div className="flex-1 flex items-center justify-between h-full py-1">
        <div className="flex flex-col gap-1 justify-start h-full min-w-0 py-0">
          <PlayerName player={playerInfo.name} />
          <TakenPieces piecesTaken={piecesTaken} materialDiff={materialDiff} />
        </div>

        <div className="flex flex-col items-end justify-start h-full mt-2 ml-3">
          {typeof clock !== "undefined" && (
            <div
              className={`flex items-center gap-2 transition-colors duration-150 ${isTurn ? "text-foreground" : "text-foreground/40"}`}
            >
              <Clock className="w-5 h-5 text-foreground/40" />
              <span className="font-mono text-2xl tracking-widest">
                {clock}
              </span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
