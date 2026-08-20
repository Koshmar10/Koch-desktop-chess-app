import blackBishop from "../../../assets/pieces/black_bishop.svg";
import blackKing from "../../../assets/pieces/black_king.svg";
import blackKnight from "../../../assets/pieces/black_knight.svg";
import blackPawn from "../../../assets/pieces/black_pawn.svg";
import blackQueen from "../../../assets/pieces/black_queen.svg";
import blackRook from "../../../assets/pieces/black_rook.svg";
import whiteBishop from "../../../assets/pieces/white_bishop.svg";
import whiteKing from "../../../assets/pieces/white_king.svg";
import whiteKnight from "../../../assets/pieces/white_knight.svg";
import whitePawn from "../../../assets/pieces/white_pawn.svg";
import whiteQueen from "../../../assets/pieces/white_queen.svg";
import whiteRook from "../../../assets/pieces/white_rook.svg";
import type { PieceColor, PieceKind } from "./types";

export const PIECE_IMAGES: Record<PieceColor, Record<PieceKind, string>> = {
  white: {
    pawn: whitePawn,
    knight: whiteKnight,
    bishop: whiteBishop,
    rook: whiteRook,
    queen: whiteQueen,
    king: whiteKing,
  },
  black: {
    pawn: blackPawn,
    knight: blackKnight,
    bishop: blackBishop,
    rook: blackRook,
    queen: blackQueen,
    king: blackKing,
  },
};
