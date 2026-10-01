import {
  ArrowDown,
  ArrowUp,
  ChevronRight,
  Copy,
  Crosshair,
  DoorOpen,
  LandPlot,
  MoveRight,
  Scale,
  Shield,
  Unlink,
  Wind,
  type LucideIcon,
} from "lucide-react";
import type { ReactNode } from "react";
import type { KingSafety } from "../../../../api/bindings/KingSafety";
import type { PawnStructure } from "../../../../api/bindings/PawnStructure";
import type { PositionFindings } from "../../../../api/bindings/PositionFindings";
import { PieceAvatar } from "../../../../components/chessboard/PieceAvatar";
import {
  FILES,
  squareName,
} from "../../../../components/chessboard/lib/squareName";
import type { Square } from "../../../../api/bindings/Square";
import { TONE_ARROW_COLOR } from "../../../../components/chessboard/lib/highlightTones";
import type {
  ArrowData,
  HighlightTone,
  PlacedPiece,
  SquareHighlight,
} from "../../../../components/chessboard/lib/types";
import EyeToggle from "../EyeToggle";
import PanelSection from "../PanelSection";
import { IconLabel, PIECE_CHIP_SIZE_PX, PieceChip, SquareChip } from "../chips";
import { controlColor, controlCounts, type ControlSide } from "./controlTint";

const SIDE_AVATAR_SIZE_PX = 26;
// Wide enough that the pinned piece can sit on the shaft without covering
// either arrowhead.
const PIN_ARROW_WIDTH_PX = 76;

const squareOf = (id: number, pieces: PlacedPiece[]): Square | undefined =>
  pieces.find((p) => p.id === id)?.square;

/**
 * An arrow between two pieces, or nothing if either has left the board —
 * which happens whenever the findings and the rendered position are a
 * ply apart.
 */
const arrowBetween = (
  fromId: number,
  toId: number,
  pieces: PlacedPiece[],
  tone: HighlightTone,
): ArrowData[] => {
  const from = squareOf(fromId, pieces);
  const to = squareOf(toId, pieces);
  if (!from || !to) return [];
  return [
    {
      from: [from.rank, from.file],
      to: [to.rank, to.file],
      color: TONE_ARROW_COLOR[tone],
      type: "finding",
    },
  ];
};

/** Where the named pieces stand, skipping ids this position doesn't hold. */
const squaresOf = (ids: number[], pieces: PlacedPiece[]): Square[] =>
  ids
    .map((id) => pieces.find((p) => p.id === id)?.square)
    .filter((square): square is Square => square !== undefined);

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
    title: "Shield damage",
    icon: Shield,
    bar: "bg-primary",
    text: "text-primary",
  },
  {
    key: "storm_penalty",
    title: "Pawn storm",
    icon: Wind,
    bar: "bg-amber-500",
    text: "text-amber-500",
  },
  {
    key: "attack_penalty",
    title: "Pieces attacking the king",
    icon: Crosshair,
    bar: "bg-destructive",
    text: "text-destructive",
  },
] as const;

interface KingSafetyRowProps {
  safety: KingSafety;
  // Both colors' gauges are drawn against the same scale, so the
  // comparison between them is the one your eye makes first.
  scaleMax: number;
  pieces: PlacedPiece[];
  toggle: ReactNode;
}

const KingSafetyRow = ({
  safety,
  scaleMax,
  pieces,
  toggle,
}: KingSafetyRowProps) => (
  <div className="flex flex-col gap-1.5">
    <div className="flex items-center gap-2">
      <PieceAvatar color={safety.color} size={SIDE_AVATAR_SIZE_PX} />
      <span className="ml-auto text-sm font-semibold tabular-nums">
        {safety.score}
      </span>
      {toggle}
    </div>

    <div className="flex h-1.5 w-full overflow-hidden rounded-full bg-foreground/10">
      {PENALTY_PARTS.map(({ key, bar, title }) => (
        <div
          key={key}
          title={title}
          className={bar}
          style={{ width: `${(safety[key] / scaleMax) * 100}%` }}
        />
      ))}
    </div>

    <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
      {PENALTY_PARTS.filter(({ key }) => safety[key] > 0).map(
        ({ key, title, icon: Icon, text }) => (
          <span
            key={key}
            title={title}
            className={`flex items-center gap-1 text-xs tabular-nums ${text}`}
          >
            <Icon size={13} />
            {safety[key]}
          </span>
        ),
      )}
    </div>

    {safety.missing_shield_files.length > 0 && (
      <div className="flex flex-wrap items-center gap-1">
        <IconLabel icon={DoorOpen} title="Files with no shield pawn" />
        {safety.missing_shield_files.map((file) => (
          <SquareChip key={file} label={FILES[file]} />
        ))}
      </div>
    )}

    {safety.attacking_piece_ids.length > 0 && (
      <div className="flex flex-wrap items-center gap-1.5">
        <IconLabel icon={Crosshair} title="Attacking this king" />
        {safety.attacking_piece_ids.map((id) => (
          <PieceChip key={id} id={id} pieces={pieces} showSquare={false} />
        ))}
      </div>
    )}
  </div>
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
    size={PIECE_CHIP_SIZE_PX}
    title={title}
    badge={
      <span
        className={`absolute -right-1 -bottom-1 rounded-full bg-background p-[1px] ${tone}`}
      >
        <Overlay size={10} strokeWidth={3} />
      </span>
    }
  />
);

const PawnStructureRow = ({
  structure,
  pieces,
  renderToggle,
}: {
  structure: PawnStructure;
  pieces: PlacedPiece[];
  // Built by the panel, which owns which highlights are on — one per
  // weakness, so you can light up the isolated pawns without the
  // backward ones coming too.
  renderToggle: (flagKey: string, ids: number[], title: string) => ReactNode;
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
        flags.map(({ key, title, overlay, tone, ids }) => (
          <div key={key} className="flex flex-wrap items-center gap-1.5">
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
            {renderToggle(key, [...ids], title)}
          </div>
        ))
      )}
    </div>
  );
};

// ------------------------------------------------------- pins & forks

/**
 * The pinner, an arrow to what it's pinning against, and the pinned piece
 * riding the shaft — the geometry of the pin, drawn rather than spelled
 * out. The rider carries no square label so it doesn't outgrow the arrow;
 * its tooltip names it.
 */
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

const PinRow = ({
  pinnerId,
  pinnedId,
  targetId,
  pieces,
  toggle,
}: {
  pinnerId: number;
  pinnedId: number;
  targetId: number;
  pieces: PlacedPiece[];
  toggle: ReactNode;
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
    <span className="ml-auto">{toggle}</span>
  </div>
);

const ForkRow = ({
  forkerId,
  forkedIds,
  pieces,
  toggle,
}: {
  forkerId: number;
  forkedIds: number[];
  pieces: PlacedPiece[];
  toggle: ReactNode;
}) => (
  <div className="flex flex-wrap items-center gap-1">
    <PieceChip id={forkerId} pieces={pieces} />
    <MoveRight size={20} className="shrink-0 text-foreground/25" />
    {forkedIds.map((id) => (
      <PieceChip key={id} id={id} pieces={pieces} />
    ))}
    <span className="ml-auto">{toggle}</span>
  </div>
);

// Extra room at the bottom for the square names that hang below each
// tile — they're positioned outside the chips' boxes, so the row has to
// make space for them itself.
const FindingRow = ({ children }: { children: React.ReactNode }) => (
  <div className="rounded-md border-[1px] border-border/60 bg-card/40 px-2 pt-1.5 pb-4">
    {children}
  </div>
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
  shown,
  onToggle,
}: {
  findings: PositionFindings;
  shown: boolean;
  onToggle: () => void;
}) => {
  const counts = controlCounts(findings);

  return (
    <PanelSection title="Control">
      <div className="flex items-center gap-4">
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
          <EyeToggle shown={shown} onToggle={onToggle} label="square control" />
        </span>
      </div>
    </PanelSection>
  );
};

interface PositionPanelProps {
  findings: PositionFindings;
  // Findings name pieces by id and nothing else, so the panel can't render
  // a single row without the position's pieces to resolve them against.
  pieces: PlacedPiece[];
  // The highlights currently drawn on the board, owned above this panel
  // because the board is a sibling of it, not a child.
  highlights: SquareHighlight[];
  onToggleHighlight: (highlight: SquareHighlight) => void;
  // A whole-board overlay rather than a highlight: it has no single
  // finding to toggle, so it's an on/off owned with the other overlays.
  showControl: boolean;
  onToggleControl: () => void;
}

const PositionPanel = ({
  findings,
  pieces,
  highlights,
  onToggleHighlight,
  showControl,
  onToggleControl,
}: PositionPanelProps) => {
  // Guarded against a position where neither king is in any danger, which
  // would otherwise divide every gauge segment by zero.
  const safetyScale = Math.max(
    findings.white_king_safety.score,
    findings.black_king_safety.score,
    1,
  );

  // Every row's eye is the same control over a different set of squares,
  // so it's built once here rather than each row knowing how highlights
  // are stored.
  const toggleFor = (highlight: SquareHighlight, label: string) => (
    <EyeToggle
      shown={highlights.some((shown) => shown.id === highlight.id)}
      onToggle={() => onToggleHighlight(highlight)}
      label={label}
    />
  );

  const kingHighlight = (safety: PositionFindings["white_king_safety"]) => {
    const king = pieces.find(
      (p) => p.kind === "king" && p.color === safety.color,
    );
    return {
      id: `king:${safety.color}`,
      tone: "warning" as const,
      squares: [
        ...(king ? [king.square] : []),
        ...squaresOf(safety.attacking_piece_ids, pieces),
      ],
      // One arrow per attacker, all converging on the king — which is
      // what "attack_penalty 14" means, drawn.
      arrows: king
        ? safety.attacking_piece_ids.flatMap((id) =>
            arrowBetween(id, king.id, pieces, "warning"),
          )
        : [],
    };
  };

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto px-1">
      {/* First because it's the widest view — the whole board — ahead of
          the sections about individual pieces. */}
      <ControlSection
        findings={findings}
        shown={showControl}
        onToggle={onToggleControl}
      />

      <PanelSection title="King safety">
        <KingSafetyRow
          safety={findings.white_king_safety}
          scaleMax={safetyScale}
          pieces={pieces}
          toggle={toggleFor(
            kingHighlight(findings.white_king_safety),
            "White's king and its attackers",
          )}
        />
        <KingSafetyRow
          safety={findings.black_king_safety}
          scaleMax={safetyScale}
          pieces={pieces}
          toggle={toggleFor(
            kingHighlight(findings.black_king_safety),
            "Black's king and its attackers",
          )}
        />
      </PanelSection>

      <PanelSection title="Pawns">
        <PawnStructureRow
          structure={findings.white_pawn_structure}
          pieces={pieces}
          renderToggle={(flagKey, ids, title) =>
            toggleFor(
              {
                id: `pawns:white:${flagKey}`,
                tone: flagKey === "passed_pawn_ids" ? "good" : "warning",
                squares: squaresOf(ids, pieces),
              },
              `White's ${title.toLowerCase()} pawns`,
            )
          }
        />
        <PawnStructureRow
          structure={findings.black_pawn_structure}
          pieces={pieces}
          renderToggle={(flagKey, ids, title) =>
            toggleFor(
              {
                id: `pawns:black:${flagKey}`,
                tone: flagKey === "passed_pawn_ids" ? "good" : "warning",
                squares: squaresOf(ids, pieces),
              },
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
            <FindingRow key={`${pin.pinner_piece_id}-${pin.pinned_piece_id}`}>
              <PinRow
                pinnerId={pin.pinner_piece_id}
                pinnedId={pin.pinned_piece_id}
                targetId={pin.pin_target_id}
                pieces={pieces}
                toggle={toggleFor(
                  {
                    id: `pin:${pin.pinner_piece_id}-${pin.pinned_piece_id}`,
                    tone: "danger",
                    // The ray the pin runs along, plus the pinner at its
                    // root — `squares` starts past it.
                    // Only the three pieces, not `pin.squares` — the
                    // ray's empty squares are what the arrow already
                    // traces, and lighting both says the same thing
                    // twice while burying the pieces that matter.
                    squares: squaresOf(
                      [
                        pin.pinner_piece_id,
                        pin.pinned_piece_id,
                        pin.pin_target_id,
                      ],
                      pieces,
                    ),
                    // Pinner to target, the line the pinned piece can't
                    // step off — the same geometry the row draws.
                    arrows: arrowBetween(
                      pin.pinner_piece_id,
                      pin.pin_target_id,
                      pieces,
                      "danger",
                    ),
                  },
                  "this pin",
                )}
              />
            </FindingRow>
          ))
        )}
      </PanelSection>

      <PanelSection title="Forks">
        {findings.forks.length === 0 ? (
          <Empty label="No forks" />
        ) : (
          findings.forks.map((fork) => (
            <FindingRow key={fork.forker_id}>
              <ForkRow
                forkerId={fork.forker_id}
                forkedIds={fork.forked_ids}
                pieces={pieces}
                toggle={toggleFor(
                  {
                    id: `fork:${fork.forker_id}`,
                    tone: "danger",
                    squares: squaresOf(
                      [fork.forker_id, ...fork.forked_ids],
                      pieces,
                    ),
                    arrows: fork.forked_ids.flatMap((victimId) =>
                      arrowBetween(fork.forker_id, victimId, pieces, "danger"),
                    ),
                  },
                  "this fork",
                )}
              />
            </FindingRow>
          ))
        )}
      </PanelSection>
    </div>
  );
};

export default PositionPanel;
