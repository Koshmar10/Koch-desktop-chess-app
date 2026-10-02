import type { PieceView } from "../../../api/bindings/PieceView";
import type { Square } from "../../../api/bindings/Square";

export type { PieceColor } from "../../../api/bindings/PieceColor";
export type { PieceType as PieceKind } from "../../../api/bindings/PieceType";
export type { PieceView as PlacedPiece } from "../../../api/bindings/PieceView";

export interface ArrowData {
  from: [number, number];
  to: [number, number];
  color?: string;
  type: "engine" | "user" | "ghost" | "finding";
}

export type HighlightTone = "info" | "danger" | "warning" | "good";

/**
 * One square's fill in an overlay. The colour arrives resolved, so the
 * board never has to know whether it marks a pin's victim or a square
 * White holds by two attackers.
 */
export interface SquareMark {
  square: Square;
  color: string;
}

/**
 * Everything drawn on the board for one reason — a pin, a fork, a king
 * and its attackers, the whole board's square control, an engine move.
 * Deliberately generic: the board renders squares, arrows and pieces and
 * knows nothing about pins or forks, so a new kind of finding needs no
 * change here (KOCH-15).
 *
 * Plain data rather than something that renders itself, so a mask can be
 * built and tested without React, and described to the AI layer as
 * "what's on the board right now".
 */
export interface OverlayMask {
  /**
   * The *reason*, not the squares — "pin:29-13", "control". Piece ids
   * hold for a whole replay, so one id finds the same finding on every
   * ply it exists on.
   */
  id: string;
  /** Paint order: a higher priority draws over a lower one. */
  priority: number;
  squares?: SquareMark[];
  /**
   * For findings that have a direction as well as a location — a pin
   * runs *from* its pinner *to* its target, and lighting squares says
   * nothing about which way.
   */
  arrows?: ArrowData[];
  /**
   * Pieces drawn translucent where a move would put them. Typed now so
   * the shape is settled, but nothing builds or draws them yet.
   */
  ghostPieces?: PieceView[];
}
