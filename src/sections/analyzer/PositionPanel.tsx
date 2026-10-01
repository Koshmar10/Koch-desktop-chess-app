import {
  ArrowDown,
  ArrowUp,
  Copy,
  Crosshair,
  DoorOpen,
  LandPlot,
  MoveRight,
  Shield,
  Unlink,
  Wind,
  type LucideIcon,
} from "lucide-react";
import type { KingSafety } from "../../api/bindings/KingSafety";
import type { PawnStructure } from "../../api/bindings/PawnStructure";
import type { PositionFindings } from "../../api/bindings/PositionFindings";
import { PieceAvatar } from "../../components/chessboard/PieceAvatar";
import { FILES, squareName } from "../../components/chessboard/lib/squareName";
import type { PlacedPiece } from "../../components/chessboard/lib/types";
import { pieceLabel } from "./panel/pieceLabel";

const SIDE_AVATAR_SIZE_PX = 26;
const PIECE_CHIP_SIZE_PX = 28;
// Wide enough that the pinned piece can sit on the shaft without covering
// either arrowhead.
const PIN_ARROW_WIDTH_PX = 62;

interface SectionProps {
  title: string;
  children: React.ReactNode;
}

const Section = ({ title, children }: SectionProps) => (
  <section className="flex flex-col gap-2">
    <h3 className="text-xs font-semibold tracking-wide text-foreground/60 uppercase">
      {title}
    </h3>
    {children}
  </section>
);

const Empty = ({ label }: { label: string }) => (
  <p className="text-xs text-foreground/40 italic">{label}</p>
);

/**
 * A piece as an icon plus the square it stands on.
 *
 * The square is the one piece of text that stays: it's identity, not
 * prose — with two bishops on the board, the icon alone doesn't say which
 * one a finding is about.
 */
const PieceChip = ({
  id,
  pieces,
  showSquare = true,
}: {
  id: number;
  pieces: PlacedPiece[];
  showSquare?: boolean;
}) => {
  const piece = pieces.find((p) => p.id === id);
  if (!piece) return null;

  return (
    <span className="flex shrink-0 flex-col items-center gap-0.5">
      <PieceAvatar
        color={piece.color}
        kind={piece.kind}
        size={PIECE_CHIP_SIZE_PX}
        title={pieceLabel(id, pieces) ?? undefined}
      />
      {showSquare && (
        <span className="font-mono text-xs leading-none text-foreground/45">
          {squareName(piece.square)}
        </span>
      )}
    </span>
  );
};

const SquareChip = ({ label }: { label: string }) => (
  <span className="flex h-5 min-w-5 items-center justify-center rounded-sm border-[1px] border-border/60 bg-card/40 px-1 font-mono text-xs text-foreground/65">
    {label}
  </span>
);

/** An icon standing in for a label, with the word kept as its tooltip. */
const IconLabel = ({
  icon: Icon,
  title,
}: {
  icon: LucideIcon;
  title: string;
}) => (
  <span title={title} className="flex shrink-0 text-foreground/40">
    <Icon size={14} />
  </span>
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
}

const KingSafetyRow = ({ safety, scaleMax, pieces }: KingSafetyRowProps) => (
  <div className="flex flex-col gap-1.5">
    <div className="flex items-center gap-2">
      <PieceAvatar color={safety.color} size={SIDE_AVATAR_SIZE_PX} />
      <span className="ml-auto text-sm font-semibold tabular-nums">
        {safety.score}
      </span>
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
const PinRow = ({
  pinnerId,
  pinnedId,
  targetId,
  pieces,
}: {
  pinnerId: number;
  pinnedId: number;
  targetId: number;
  pieces: PlacedPiece[];
}) => (
  <div className="flex items-center gap-1">
    <PieceChip id={pinnerId} pieces={pieces} />
    <span
      className="relative flex shrink-0 items-center justify-center"
      style={{ width: PIN_ARROW_WIDTH_PX }}
    >
      <MoveRight
        className="text-foreground/25"
        style={{ width: PIN_ARROW_WIDTH_PX, height: 20 }}
      />
      <span className="absolute">
        <PieceChip id={pinnedId} pieces={pieces} showSquare={false} />
      </span>
    </span>
    <PieceChip id={targetId} pieces={pieces} />
  </div>
);

const ForkRow = ({
  forkerId,
  forkedIds,
  pieces,
}: {
  forkerId: number;
  forkedIds: number[];
  pieces: PlacedPiece[];
}) => (
  <div className="flex flex-wrap items-center gap-1">
    <PieceChip id={forkerId} pieces={pieces} />
    <MoveRight size={20} className="shrink-0 text-foreground/25" />
    {forkedIds.map((id) => (
      <PieceChip key={id} id={id} pieces={pieces} />
    ))}
  </div>
);

const FindingRow = ({ children }: { children: React.ReactNode }) => (
  <div className="rounded-md border-[1px] border-border/60 bg-card/40 px-2 py-1.5">
    {children}
  </div>
);

interface PositionPanelProps {
  findings: PositionFindings;
  // Findings name pieces by id and nothing else, so the panel can't render
  // a single row without the position's pieces to resolve them against.
  pieces: PlacedPiece[];
}

const PositionPanel = ({ findings, pieces }: PositionPanelProps) => {
  // Guarded against a position where neither king is in any danger, which
  // would otherwise divide every gauge segment by zero.
  const safetyScale = Math.max(
    findings.white_king_safety.score,
    findings.black_king_safety.score,
    1,
  );

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto">
      <Section title="King safety">
        <KingSafetyRow
          safety={findings.white_king_safety}
          scaleMax={safetyScale}
          pieces={pieces}
        />
        <KingSafetyRow
          safety={findings.black_king_safety}
          scaleMax={safetyScale}
          pieces={pieces}
        />
      </Section>

      <Section title="Pawns">
        <PawnStructureRow
          structure={findings.white_pawn_structure}
          pieces={pieces}
        />
        <PawnStructureRow
          structure={findings.black_pawn_structure}
          pieces={pieces}
        />
      </Section>

      <Section title="Pins">
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
              />
            </FindingRow>
          ))
        )}
      </Section>

      <Section title="Forks">
        {findings.forks.length === 0 ? (
          <Empty label="No forks" />
        ) : (
          findings.forks.map((fork) => (
            <FindingRow key={fork.forker_id}>
              <ForkRow
                forkerId={fork.forker_id}
                forkedIds={fork.forked_ids}
                pieces={pieces}
              />
            </FindingRow>
          ))
        )}
      </Section>
    </div>
  );
};

export default PositionPanel;
