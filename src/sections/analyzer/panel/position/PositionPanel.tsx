import {
  ArrowDown,
  ArrowUp,
  ChevronRight,
  Copy,
  Crosshair,
  LandPlot,
  MoveRight,
  Scale,
  Shield,
  ShieldAlert,
  ShieldCheck,
  ShieldX,
  Unlink,
  Wind,
  type LucideIcon,
} from "lucide-react";
import type { ReactNode } from "react";
import type { Fork } from "../../../../api/bindings/Fork";
import type { KingDanger } from "../../../../api/bindings/KingDanger";
import type { KingSafety } from "../../../../api/bindings/KingSafety";
import type { PawnStructure } from "../../../../api/bindings/PawnStructure";
import type { PieceColor } from "../../../../api/bindings/PieceColor";
import type { Pin } from "../../../../api/bindings/Pin";
import type { PositionFindings } from "../../../../api/bindings/PositionFindings";
import { PieceAvatar } from "../../../../components/chessboard/PieceAvatar";
import {
  FILES,
  squareName,
} from "../../../../components/chessboard/lib/squareName";
import type { PlacedPiece } from "../../../../components/chessboard/lib/types";
import { CardRow, PlainRow } from "../InteractiveRow";
import PanelSection from "../PanelSection";
import { PieceChip, SquareChip } from "../chips";
import { maskIdFor } from "../../overlays/masks";
import { controlColor, controlCounts, type ControlSide } from "./controlTint";

const SIDE_AVATAR_SIZE_PX = 26;
// The pawn-weakness tiles carry a badge on their corner, so they get a
// little more room than a plain piece chip for both to read clearly.
const PAWN_FLAG_SIZE_PX = 32;
// Wide enough that the pinned piece can sit on the shaft without covering
// either arrowhead.
const PIN_ARROW_WIDTH_PX = 76;

// For the rows' accessible names, e.g. "Show White's king and its
// attackers on the board".
const SIDE_NAME: Record<PieceColor, string> = {
  white: "White",
  black: "Black",
};

const Empty = ({ label }: { label: string }) => (
  <p className="text-xs text-foreground/40 italic">{label}</p>
);

// ---------------------------------------------------------------- king

// The three penalties are the point of showing king safety at all: a lone
// score says a king is in trouble, this says whether the trouble is a
// broken shield, an incoming pawn storm, or pieces already aimed at it.
// Each gauge segment and its chip share one color, so the bar needs no
// legend of its own.
const PENALTY_PARTS = [
  {
    key: "shield_penalty",
    title: "Pawn shield: missing files and advanced pawns",
    icon: Shield,
    bar: "bg-primary",
    text: "text-primary",
  },
  {
    key: "storm_penalty",
    title: "Pawn storm: enemy pawns advancing on the king",
    icon: Wind,
    bar: "bg-amber-500",
    text: "text-amber-500",
  },
  {
    key: "attack_penalty",
    title: "Attackers: pieces aimed at the king's zone",
    icon: Crosshair,
    bar: "bg-destructive",
    text: "text-destructive",
  },
] as const;

// The gauge fills completely at the score where a king becomes critical —
// koch-engine's `CRITICAL_FROM` (two open-file shield holes' worth). A
// fixed scale, so a full bar always means the same thing; it used to be
// scaled to whichever king was worse off, which drew a near-safe king's
// score of 6 as a full bar next to a 4.
const KING_DANGER_FULL_SCALE = 80;

const DANGER_BADGE: Record<
  KingDanger,
  { icon: LucideIcon; label: string; cls: string }
> = {
  Safe: {
    icon: ShieldCheck,
    label: "Safe",
    cls: "border-emerald-500/40 bg-emerald-500/10 text-emerald-600 dark:text-emerald-400",
  },
  Uneasy: {
    icon: Shield,
    label: "Uneasy",
    cls: "border-amber-500/40 bg-amber-500/10 text-amber-600 dark:text-amber-400",
  },
  Exposed: {
    icon: ShieldAlert,
    label: "Exposed",
    cls: "border-orange-500/40 bg-orange-500/10 text-orange-600 dark:text-orange-400",
  },
  Critical: {
    icon: ShieldX,
    label: "Critical",
    cls: "border-destructive/50 bg-destructive/10 text-destructive",
  },
};

/**
 * The band, not the number. A raw score is an open-ended sum of three
 * penalties, so "22" means nothing without the formula — "uneasy" does.
 * The number and its breakdown stay one hover away for anyone who wants
 * them. The word is kept beside the icon: the four shield icons alone are
 * too alike to tell apart at a glance.
 */
const DangerBadge = ({ safety }: { safety: KingSafety }) => {
  const { icon: Icon, label, cls } = DANGER_BADGE[safety.danger];
  return (
    <span
      title={`${label} — score ${safety.score} (shield ${safety.shield_penalty}, storm ${safety.storm_penalty}, attackers ${safety.attack_penalty})`}
      className={`flex items-center gap-1 rounded-md border-[1px] px-1.5 py-0.5 text-xs font-semibold ${cls}`}
    >
      <Icon size={13} />
      {label}
    </span>
  );
};

/** One factor's causes, led by that factor's icon in its gauge colour. */
const CauseLine = ({
  part,
  children,
}: {
  part: (typeof PENALTY_PARTS)[number];
  children: ReactNode;
}) => (
  <div className="flex flex-wrap items-center gap-1.5">
    <span title={part.title} className={`flex shrink-0 ${part.text}`}>
      <part.icon size={14} />
    </span>
    {children}
  </div>
);

/** The three penalties as one bar, each segment in its factor's colour. */
const PenaltyGauge = ({ safety }: { safety: KingSafety }) => {
  // Past the full-scale point the segments shrink to fit rather than spill
  // out of the bar; the bar being full already says "critical".
  const scale = Math.max(safety.score, KING_DANGER_FULL_SCALE);

  return (
    <div className="flex h-1.5 w-full overflow-hidden rounded-full bg-foreground/10">
      {PENALTY_PARTS.map(({ key, bar, title }) => (
        <div
          key={key}
          title={title}
          className={bar}
          style={{ width: `${(safety[key] / scale) * 100}%` }}
        />
      ))}
    </div>
  );
};

/**
 * What's causing each penalty, rather than how many points it is. A
 * factor that isn't contributing doesn't get a line.
 */
const PenaltyCauses = ({
  safety,
  pieces,
}: {
  safety: KingSafety;
  pieces: PlacedPiece[];
}) => {
  const [shield, storm, attack] = PENALTY_PARTS;

  return (
    <>
      {safety.shield_penalty > 0 && (
        <CauseLine part={shield}>
          {safety.missing_shield_files.map((file) => (
            <SquareChip key={`file-${file}`} label={FILES[file]} />
          ))}
          {safety.advanced_shield_pawn_ids.map((id) => (
            <PieceChip key={id} id={id} pieces={pieces} showSquare={false} />
          ))}
        </CauseLine>
      )}
      {safety.storm_penalty > 0 && (
        <CauseLine part={storm}>
          {safety.storming_pawn_ids.map((id) => (
            <PieceChip key={id} id={id} pieces={pieces} showSquare={false} />
          ))}
        </CauseLine>
      )}
      {safety.attack_penalty > 0 && (
        <CauseLine part={attack}>
          {safety.attacking_piece_ids.map((id) => (
            <PieceChip key={id} id={id} pieces={pieces} showSquare={false} />
          ))}
        </CauseLine>
      )}
    </>
  );
};

const KingSafetyRow = ({
  safety,
  pieces,
}: {
  safety: KingSafety;
  pieces: PlacedPiece[];
}) => (
  <PlainRow
    maskId={maskIdFor.king(safety.color)}
    maskLabel={`${SIDE_NAME[safety.color]}'s king and its attackers`}
    details={
      <>
        <PenaltyGauge safety={safety} />
        <PenaltyCauses safety={safety} pieces={pieces} />
      </>
    }
  >
    <PieceAvatar color={safety.color} size={SIDE_AVATAR_SIZE_PX} />
    <DangerBadge safety={safety} />
  </PlainRow>
);

// --------------------------------------------------------------- pawns

// Each weakness is the pawn icon with a second icon overlaid on its
// corner, so the tile reads as "a pawn, but <this>" rather than needing
// the word next to it.
const PAWN_FLAGS = [
  {
    key: "passed_pawn_ids",
    title: "Passed",
    overlay: ArrowUp,
    tone: "text-emerald-600",
  },
  {
    key: "backward_pawn_ids",
    title: "Backward",
    overlay: ArrowDown,
    tone: "text-amber-600",
  },
  {
    key: "isolated_pawn_ids",
    title: "Isolated",
    overlay: Unlink,
    tone: "text-destructive",
  },
  {
    key: "doubled_pawn_ids",
    title: "Doubled",
    overlay: Copy,
    tone: "text-destructive",
  },
] as const;

const PawnFlagTile = ({
  color,
  overlay: Overlay,
  tone,
  title,
}: {
  color: PawnStructure["color"];
  overlay: LucideIcon;
  tone: string;
  title: string;
}) => (
  <PieceAvatar
    color={color}
    kind="pawn"
    size={PAWN_FLAG_SIZE_PX}
    title={title}
    badge={
      <span
        className={`absolute -right-1.5 -bottom-1.5 rounded-full bg-background p-0.5 ${tone}`}
      >
        <Overlay size={13} strokeWidth={3} />
      </span>
    }
  />
);

const PawnStructureRow = ({
  structure,
  pieces,
}: {
  structure: PawnStructure;
  pieces: PlacedPiece[];
}) => {
  const flags = PAWN_FLAGS.map((flag) => ({
    ...flag,
    ids: structure[flag.key],
  })).filter(({ ids }) => ids.length > 0);

  const squareOf = (id: number) => {
    const piece = pieces.find((p) => p.id === id);
    return piece ? squareName(piece.square) : null;
  };

  return (
    <div className="flex flex-row gap-1.5">
      <span
        title="Pawn islands"
        className="flex items-center gap-1 text-xs tabular-nums text-foreground/50"
      >
        <LandPlot size={13} className="rotate-90" />
        {structure.pawn_islands.length}
      </span>

      {flags.length === 0 ? (
        <span className="text-xs text-foreground/40 italic">nothing weak</span>
      ) : (
        // A row per weakness, so you can light up the isolated pawns
        // without the backward ones coming too.
        flags.map(({ key, title, overlay, tone, ids }) => (
          <PlainRow
            key={key}
            maskId={maskIdFor.pawns(structure.color, key)}
            maskLabel={`${SIDE_NAME[structure.color]}'s ${title.toLowerCase()} pawns`}
          >
            <PawnFlagTile
              color={structure.color}
              overlay={overlay}
              tone={tone}
              title={title}
            />
            {ids.map((id) => {
              const square = squareOf(id);
              return square && <SquareChip key={id} label={square} />;
            })}
          </PlainRow>
        ))
      )}
    </div>
  );
};

// ------------------------------------------------------- pins & forks

/**
 * An arrow that actually stretches to its width: a line plus a chevron
 * head. A lucide arrow icon can't do this — icons keep their square
 * aspect ratio, so widening one just pads a fixed-size arrow with empty
 * space, and the piece riding a pin covered nearly all of what was left.
 *
 * `rider` sits on the shaft — a pin's pinned piece, caught between the
 * pinner and what it's pinned against.
 */
const StretchArrow = ({
  width,
  rider,
}: {
  width: number;
  rider?: ReactNode;
}) => (
  <span className="relative flex shrink-0 items-center" style={{ width }}>
    <span className="h-0.5 flex-1 rounded-full bg-foreground/25" />
    {/* Pulled left over the line's end so the shaft runs into the tip
        rather than stopping short of it. */}
    <ChevronRight
      size={16}
      strokeWidth={2.5}
      className="-ml-2 shrink-0 text-foreground/25"
    />
    {rider && <span className="absolute bottom-[-6px] left-1/4">{rider}</span>}
  </span>
);

/**
 * The pinner, an arrow to what it's pinning against, and the pinned piece
 * riding the shaft — the geometry of the pin, drawn rather than spelled
 * out. The rider carries no square label so it doesn't outgrow the arrow;
 * its tooltip names it.
 */
const PinRow = ({ pin, pieces }: { pin: Pin; pieces: PlacedPiece[] }) => (
  <CardRow maskId={maskIdFor.pin(pin)} maskLabel="this pin" captioned>
    <PieceChip id={pin.pinner_piece_id} pieces={pieces} />
    <StretchArrow
      width={PIN_ARROW_WIDTH_PX}
      rider={
        <PieceChip
          id={pin.pinned_piece_id}
          pieces={pieces}
          showSquare={false}
        />
      }
    />
    <PieceChip id={pin.pin_target_id} pieces={pieces} />
  </CardRow>
);

const ForkRow = ({ fork, pieces }: { fork: Fork; pieces: PlacedPiece[] }) => (
  <CardRow maskId={maskIdFor.fork(fork)} maskLabel="this fork" captioned>
    <PieceChip id={fork.forker_id} pieces={pieces} />
    <MoveRight size={20} className="shrink-0 text-foreground/25" />
    {fork.forked_ids.map((id) => (
      <PieceChip key={id} id={id} pieces={pieces} />
    ))}
  </CardRow>
);

// ------------------------------------------------------------- control

/**
 * A count with a ring in its side's board-tint colour, so the summary is
 * also the legend: you learn that blue means White here, not from a key.
 */
const ControlCount = ({
  side,
  count,
  title,
  children,
}: {
  side: ControlSide;
  count: number;
  title: string;
  children: ReactNode;
}) => (
  <span title={title} className="flex items-center gap-1.5">
    <span
      className="rounded-md"
      style={{ boxShadow: `0 0 0 2px ${controlColor(side)}` }}
    >
      {children}
    </span>
    <span className="text-sm font-semibold tabular-nums">{count}</span>
  </span>
);

const ControlRow = ({ findings }: { findings: PositionFindings }) => {
  const counts = controlCounts(findings);

  return (
    <PlainRow maskId={maskIdFor.control} maskLabel="square control" gap="wide">
      <ControlCount
        side="white"
        count={counts.white}
        title="Squares White controls"
      >
        <PieceAvatar color="white" size={SIDE_AVATAR_SIZE_PX} />
      </ControlCount>
      <ControlCount
        side="black"
        count={counts.black}
        title="Squares Black controls"
      >
        <PieceAvatar color="black" size={SIDE_AVATAR_SIZE_PX} />
      </ControlCount>
      <ControlCount
        side="contested"
        count={counts.contested}
        title="Squares both sides cover equally"
      >
        <span
          className="flex items-center justify-center rounded-md bg-card/60 text-foreground/60"
          style={{ width: SIDE_AVATAR_SIZE_PX, height: SIDE_AVATAR_SIZE_PX }}
        >
          <Scale size={14} />
        </span>
      </ControlCount>
    </PlainRow>
  );
};

interface PositionPanelProps {
  // Null while a game is still loading, or if it failed to.
  findings: PositionFindings | null;
  // Findings name pieces by id and nothing else, so the panel can't render
  // a single row without the position's pieces to resolve them against.
  pieces: PlacedPiece[];
}

/**
 * Each row names the mask it toggles; what that mask draws is built from
 * the same findings in `overlays/masks.ts`, for the board.
 */
const PositionPanel = ({ findings, pieces }: PositionPanelProps) => {
  if (findings === null) {
    return <Empty label="No position loaded" />;
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto px-1">
      {/* First because it's the widest view — the whole board — ahead of
          the sections about individual pieces. */}
      <PanelSection title="Control">
        <ControlRow findings={findings} />
      </PanelSection>

      <PanelSection title="King safety">
        <KingSafetyRow safety={findings.white_king_safety} pieces={pieces} />
        <KingSafetyRow safety={findings.black_king_safety} pieces={pieces} />
      </PanelSection>

      <PanelSection title="Pawns">
        <PawnStructureRow
          structure={findings.white_pawn_structure}
          pieces={pieces}
        />
        <PawnStructureRow
          structure={findings.black_pawn_structure}
          pieces={pieces}
        />
      </PanelSection>

      <PanelSection title="Pins">
        {findings.pins.length === 0 ? (
          <Empty label="No pins" />
        ) : (
          findings.pins.map((pin) => (
            <PinRow key={maskIdFor.pin(pin)} pin={pin} pieces={pieces} />
          ))
        )}
      </PanelSection>

      <PanelSection title="Forks">
        {findings.forks.length === 0 ? (
          <Empty label="No forks" />
        ) : (
          findings.forks.map((fork) => (
            <ForkRow key={maskIdFor.fork(fork)} fork={fork} pieces={pieces} />
          ))
        )}
      </PanelSection>
    </div>
  );
};

export default PositionPanel;
