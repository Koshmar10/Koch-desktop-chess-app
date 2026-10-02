import { AlertTriangle, Target } from "lucide-react";
import type { GameSummary } from "../../../../api/bindings/GameSummary";
import type { MoveQuality } from "../../../../api/bindings/MoveQuality";
import type { PieceColor } from "../../../../api/bindings/PieceColor";
import type { SideAccuracy } from "../../../../api/bindings/SideAccuracy";
import { formatDatePlayed } from "../../../../api/gameSummary";
import { PieceAvatar } from "../../../../components/chessboard/PieceAvatar";
import PanelSection from "../PanelSection";
import MoveTimeline from "./MoveTimeline";

// The same king tile the play screen's player cards use, at roughly half
// the size — enough to read the side at a glance without the row turning
// into a card of its own.
const AVATAR_SIZE_PX = 26;

interface PlayerRowProps {
  color: PieceColor;
  name: string;
  elo: number;
  // The side the person sitting here played. Null for an import with no
  // recorded side, or a game the engine played against itself — in which
  // case neither row gets marked rather than one being guessed.
  isHuman: boolean;
  // This player's accuracy over the game. Null until it's analysed — or,
  // for an analysis made before both sides were graded, until it's
  // analysed again — in which case the row just doesn't show one.
  accuracy: SideAccuracy | null;
}

const PlayerRow = ({ color, name, elo, isHuman, accuracy }: PlayerRowProps) => (
  <div className="flex items-center gap-2">
    <PieceAvatar color={color} size={AVATAR_SIZE_PX} />
    <span className="truncate text-sm text-foreground/85">{name}</span>
    {isHuman && (
      <span className="shrink-0 rounded-sm bg-primary/30 px-1 text-xs text-foreground/70">
        you
      </span>
    )}
    <span className="ml-auto shrink-0 text-sm tabular-nums text-foreground/50">
      {elo}
    </span>
    {accuracy && (
      <span
        title={`Accuracy — average centipawn loss ${accuracy.average_centipawn_loss}`}
        className="flex shrink-0 items-center gap-1 text-sm font-semibold tabular-nums text-foreground/85"
      >
        <Target size={13} className="text-primary" />
        {accuracy.accuracy_percent.toFixed(1)}%
      </span>
    )}
  </div>
);

const Fact = ({ label, value }: { label: string; value: string }) => (
  <div className="grid grid-cols-[4.5rem_minmax(0,1fr)] items-baseline gap-2">
    <span className="text-xs text-foreground/45">{label}</span>
    <span className="text-sm text-foreground/80">{value}</span>
  </div>
);

const NoGame = () => (
  <div className="flex flex-1 flex-col items-center justify-center gap-2 px-2 text-center">
    <span className="text-sm text-foreground/50">No game loaded</span>
    <span className="text-xs text-foreground/35">
      Open a saved game from History to step through it here.
    </span>
  </div>
);

interface GamePanelProps {
  game: GameSummary | null;
  moves: string[];
  clocks?: (string | null)[];
  qualities?: (MoveQuality | null)[];
  whiteAccuracy: SideAccuracy | null;
  blackAccuracy: SideAccuracy | null;
  viewedPly: number;
  onSelectPly: (ply: number) => void;
}

/**
 * Who played, when, and how it ended.
 *
 * This is the one tab that can legitimately be empty — the analyzer's
 * sandbox mode has no game behind it — which is why it gets a real empty
 * state rather than rendering a card full of blanks.
 */
const GamePanel = ({
  game,
  moves,
  clocks,
  qualities,
  whiteAccuracy,
  blackAccuracy,
  viewedPly,
  onSelectPly,
}: GamePanelProps) => {
  if (game === null) return <NoGame />;

  return (
    // The header sections keep their natural height and the move list
    // takes what's left, scrolling on its own — so who played and how it
    // ended stay visible however far down the game you scrub.
    <div className="flex min-h-0 flex-1 flex-col gap-4">
      <PanelSection title="Players" divided>
        <PlayerRow
          color="white"
          name={game.white_player}
          elo={game.white_elo}
          isHuman={game.human_color === "white"}
          accuracy={whiteAccuracy}
        />
        <PlayerRow
          color="black"
          name={game.black_player}
          elo={game.black_elo}
          isHuman={game.human_color === "black"}
          accuracy={blackAccuracy}
        />
      </PanelSection>

      <PanelSection title="Game" divided>
        <Fact label="Result" value={game.result} />
        <Fact
          label="Played"
          value={formatDatePlayed(game.date_played) || "—"}
        />
        <Fact label="Opening" value={game.opening_name ?? "Out of book"} />
        <Fact label="Time" value={game.time_control ?? "—"} />
        <Fact label="Source" value={game.source} />
      </PanelSection>

      {game.partial_import && (
        <div className="flex shrink-0 items-start gap-2 rounded-md border-[1px] border-amber-500/40 bg-amber-500/10 px-2 py-1.5">
          <AlertTriangle size={14} className="mt-0.5 shrink-0 text-amber-500" />
          <span className="text-xs text-foreground/70">
            Imported movetext stopped part way — this is a prefix of the real
            game.
          </span>
        </div>
      )}

      <PanelSection title="Moves" fill>
        <MoveTimeline
          moves={moves}
          clocks={clocks}
          qualities={qualities}
          viewedPly={viewedPly}
          onSelectPly={onSelectPly}
        />
      </PanelSection>
    </div>
  );
};

export default GamePanel;
