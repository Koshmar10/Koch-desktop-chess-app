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
 * A set of squares lit on the board for one reason — a pin's ray, a
 * fork's victims, the pawns of one weakness. Deliberately generic: the
 * board renders squares and a tone and knows nothing about pins or
 * forks, so a new kind of finding needs no change here (KOCH-15).
 *
 * `id` identifies the *reason*, not the squares, so toggling one on and
 * off survives the squares changing underneath it when the viewed ply
 * moves.
 */
export interface SquareHighlight {
  id: string;
  squares: Square[];
  tone: HighlightTone;
  /**
   * Drawn alongside the squares, for findings that have a direction as
   * well as a location — a pin runs *from* its pinner *to* its target,
   * and lighting five squares says nothing about which way.
   */
  arrows?: ArrowData[];
}

/**
 * One square's colour in a full-board tint — square control, for now.
 * Unlike `SquareHighlight`, each square carries its own colour: a tint
 * shades every square by its own value rather than lighting a set of
 * squares for a single reason. The colour arrives resolved, so the board
 * never has to know what the underlying numbers meant.
 */
export interface SquareTint {
  square: Square;
  color: string;
}
