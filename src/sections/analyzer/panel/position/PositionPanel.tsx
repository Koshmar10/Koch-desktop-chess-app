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
import type { KingDanger } from "../../../../api/bindings/KingDanger";
import type { KingSafety } from "../../../../api/bindings/KingSafety";
import type { PawnStructure } from "../../../../api/bindings/PawnStructure";
import type { PositionFindings } from "../../../../api/bindings/PositionFindings";
import { PieceAvatar } from "../../../../components/chessboard/PieceAvatar";
import {
  FILES,
  squareName,
} from "../../../../components/chessboard/lib/squareName";
import type { PlacedPiece } from "../../../../components/chessboard/lib/types";
import EyeIndicator from "../EyeIndicator";
import InteractiveRow from "../InteractiveRow";
import PanelSection from "../PanelSection";
import { PieceChip, SquareChip } from "../chips";
import {
  HOVER_ROW_CLASS,
  maskControls,
  type RowControls,
} from "../rowControls";
import { maskId, type PawnFlagKey } from "../../overlays/masks";
import type { MaskSelection } from "../../types";
import { controlColor, controlCounts, type ControlSide } from "./controlTint";

const SIDE_AVATAR_SIZE_PX = 26;
// The pawn-weakness tiles carry a badge on their corner, so they get a
// little more room than a plain piece chip for both to read clearly.
const PAWN_FLAG_SIZE_PX = 32;
// Wide enough that the pinned piece can sit on the shaft without covering
// either arrowhead.
const PIN_ARROW_WIDTH_PX = 76;

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

interface KingSafetyRowProps {
  safety: KingSafety;
  pieces: PlacedPiece[];
  controls: RowControls;
}

const KingSafetyRow = ({ safety, pieces, controls }: KingSafetyRowProps) => {
  const [shield, storm, attack] = PENALTY_PARTS;
  // Past the full-scale point the segments shrink to fit rather than spill
  // out of the bar; the bar being full already says "critical".
  const scale = Math.max(safety.score, KING_DANGER_FULL_SCALE);

  return (
    <InteractiveRow
      controls={controls}
      className={`flex flex-col gap-1.5 py-1 ${HOVER_ROW_CLASS}`}
    >
      <div className="flex items-center gap-2">
        <PieceAvatar color={safety.color} size={SIDE_AVATAR_SIZE_PX} />
        <DangerBadge safety={safety} />
        <span className="ml-auto">
          <EyeIndicator state={controls.state} />
        </span>
      </div>

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

      {/* What's causing each penalty, rather than how many points it is.
          A factor that isn't contributing doesn't get a line. */}
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
    </InteractiveRow>
  );
};

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
  controlsFor,
}: {
  structure: PawnStructure;
  pieces: PlacedPiece[];
  // Built by the panel, which knows what's on the board — one per
  // weakness, so you can light up the isolated pawns without the
  // backward ones coming too.
  controlsFor: (flag: PawnFlagKey, title: string) => RowControls;
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
      <div className="flex items-center gap-2">
        <span
          title="Pawn islands"
          className="ml-auto flex items-center gap-1 text-xs tabular-nums text-foreground/50"
        >
          <LandPlot size={13} className="rotate-90" />
          {structure.pawn_islands.length}
        </span>
      </div>

      {flags.length === 0 ? (
        <span className="text-xs text-foreground/40 italic">nothing weak</span>
      ) : (
        flags.map(({ key, title, overlay, tone, ids }) => {
          const controls = controlsFor(key, title);
          return (
            <InteractiveRow
              key={key}
              controls={controls}
              className={`flex flex-wrap items-center gap-1.5 ${HOVER_ROW_CLASS}`}
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
              <span className="ml-auto">
                <EyeIndicator state={controls.state} />
              </span>
            </InteractiveRow>
          );
        })
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
 */
const StretchArrow = ({ width }: { width: number }) => (
  <span className="flex items-center text-foreground/25" style={{ width }}>
    <span className="h-0.5 flex-1 rounded-full bg-current" />
    {/* Pulled left over the line's end so the shaft runs into the tip
        rather than stopping short of it. */}
    <ChevronRight size={16} strokeWidth={2.5} className="-ml-2 shrink-0" />
  </span>
);

/**
 * The pinner, an arrow to what it's pinning against, and the pinned piece
 * riding the shaft — the geometry of the pin, drawn rather than spelled
 * out. The rider carries no square label so it doesn't outgrow the arrow;
 * its tooltip names it.
 */
const PinRow = ({
  pinnerId,
  pinnedId,
  targetId,
  pieces,
  eye,
}: {
  pinnerId: number;
  pinnedId: number;
  targetId: number;
  pieces: PlacedPiece[];
  eye: ReactNode;
}) => (
  <div className="flex items-center gap-1">
    <PieceChip id={pinnerId} pieces={pieces} />
    <span className="relative flex shrink-0 items-start ml-1 relative">
      <StretchArrow width={PIN_ARROW_WIDTH_PX} />
      <span className="absolute left-1/4 bottom-[-6px]">
        <PieceChip id={pinnedId} pieces={pieces} showSquare={false} />
      </span>
    </span>
    <PieceChip id={targetId} pieces={pieces} />
    <span className="ml-auto">{eye}</span>
  </div>
);

const ForkRow = ({
  forkerId,
  forkedIds,
  pieces,
  eye,
}: {
  forkerId: number;
  forkedIds: number[];
  pieces: PlacedPiece[];
  eye: ReactNode;
}) => (
  <div className="flex flex-wrap items-center gap-1">
    <PieceChip id={forkerId} pieces={pieces} />
    <MoveRight size={20} className="shrink-0 text-foreground/25" />
    {forkedIds.map((id) => (
      <PieceChip key={id} id={id} pieces={pieces} />
    ))}
    <span className="ml-auto">{eye}</span>
  </div>
);

// Extra room at the bottom for the square names that hang below each
// tile — they're positioned outside the chips' boxes, so the row has to
// make space for them itself.
const FindingRow = ({
  controls,
  children,
}: {
  controls: RowControls;
  children: ReactNode;
}) => (
  <InteractiveRow
    controls={controls}
    className="rounded-md border-[1px] border-border/60 bg-card/40 px-2 pt-1.5 pb-4 hover:bg-card/80"
  >
    {children}
  </InteractiveRow>
);

const PinFinding = ({
  pin,
  pieces,
  controls,
}: {
  pin: PositionFindings["pins"][number];
  pieces: PlacedPiece[];
  controls: RowControls;
}) => (
  <FindingRow controls={controls}>
    <PinRow
      pinnerId={pin.pinner_piece_id}
      pinnedId={pin.pinned_piece_id}
      targetId={pin.pin_target_id}
      pieces={pieces}
      eye={<EyeIndicator state={controls.state} />}
    />
  </FindingRow>
);

const ForkFinding = ({
  fork,
  pieces,
  controls,
}: {
  fork: PositionFindings["forks"][number];
  pieces: PlacedPiece[];
  controls: RowControls;
}) => (
  <FindingRow controls={controls}>
    <ForkRow
      forkerId={fork.forker_id}
      forkedIds={fork.forked_ids}
      pieces={pieces}
      eye={<EyeIndicator state={controls.state} />}
    />
  </FindingRow>
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

const ControlSection = ({
  findings,
  controls,
}: {
  findings: PositionFindings;
  controls: RowControls;
}) => {
  const counts = controlCounts(findings);

  return (
    <PanelSection title="Control">
      <InteractiveRow
        controls={controls}
        className={`flex items-center gap-4 py-1 ${HOVER_ROW_CLASS}`}
      >
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
        <span className="ml-auto">
          <EyeIndicator state={controls.state} />
        </span>
      </InteractiveRow>
    </PanelSection>
  );
};

interface PositionPanelProps {
  // Null while a game is still loading, or if it failed to.
  findings: PositionFindings | null;
  // Findings name pieces by id and nothing else, so the panel can't render
  // a single row without the position's pieces to resolve them against.
  pieces: PlacedPiece[];
  // What's on the board, owned above this panel because the board is a
  // sibling of it, not a child.
  maskSelection: MaskSelection;
}

const PositionPanel = ({
  findings,
  pieces,
  maskSelection,
}: PositionPanelProps) => {
  if (findings === null) {
    return <Empty label="No position loaded" />;
  }

  // Every row is the same control over a different mask id. The rows
  // only name what they toggle — what it draws is built from the same
  // findings in `overlays/masks.ts`, for the board.
  const controlsFor = (id: string, label: string) =>
    maskControls(id, label, maskSelection);

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto px-1">
      {/* First because it's the widest view — the whole board — ahead of
          the sections about individual pieces. */}
      <ControlSection
        findings={findings}
        controls={controlsFor(maskId.control, "square control")}
      />

      <PanelSection title="King safety">
        <KingSafetyRow
          safety={findings.white_king_safety}
          pieces={pieces}
          controls={controlsFor(
            maskId.king("white"),
            "White's king and its attackers",
          )}
        />
        <KingSafetyRow
          safety={findings.black_king_safety}
          pieces={pieces}
          controls={controlsFor(
            maskId.king("black"),
            "Black's king and its attackers",
          )}
        />
      </PanelSection>

      <PanelSection title="Pawns">
        <PawnStructureRow
          structure={findings.white_pawn_structure}
          pieces={pieces}
          controlsFor={(flag, title) =>
            controlsFor(
              maskId.pawns("white", flag),
              `White's ${title.toLowerCase()} pawns`,
            )
          }
        />
        <PawnStructureRow
          structure={findings.black_pawn_structure}
          pieces={pieces}
          controlsFor={(flag, title) =>
            controlsFor(
              maskId.pawns("black", flag),
              `Black's ${title.toLowerCase()} pawns`,
            )
          }
        />
      </PanelSection>

      <PanelSection title="Pins">
        {findings.pins.length === 0 ? (
          <Empty label="No pins" />
        ) : (
          findings.pins.map((pin) => (
            <PinFinding
              key={maskId.pin(pin)}
              pin={pin}
              pieces={pieces}
              controls={controlsFor(maskId.pin(pin), "this pin")}
            />
          ))
        )}
      </PanelSection>

      <PanelSection title="Forks">
        {findings.forks.length === 0 ? (
          <Empty label="No forks" />
        ) : (
          findings.forks.map((fork) => (
            <ForkFinding
              key={maskId.fork(fork)}
              fork={fork}
              pieces={pieces}
              controls={controlsFor(maskId.fork(fork), "this fork")}
            />
          ))
        )}
      </PanelSection>
    </div>
  );
};

export default PositionPanel;
