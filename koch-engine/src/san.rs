use crate::board::{Board, PieceMove, BLACK_BACK_RANK, BOARD_SIZE, WHITE_BACK_RANK};
use crate::castling::{is_castle_move, KINGSIDE, QUEENSIDE};
use crate::direction::pawn_forward;
use crate::move_gen::MoveError;
use crate::piece::{ChessPiece, PieceColor, PieceType};
use crate::square::Square;

fn coord_to_algebraic(square: Square) -> String {
    let file = (b'a' + square.file as u8) as char;
    format!("{}{}", file, BOARD_SIZE - square.rank)
}

fn piece_letter(kind: PieceType) -> &'static str {
    match kind {
        PieceType::King => "K",
        PieceType::Queen => "Q",
        PieceType::Rook => "R",
        PieceType::Bishop => "B",
        PieceType::Knight => "N",
        PieceType::Pawn => "",
    }
}

fn promotion_letter(kind: PieceType) -> &'static str {
    match kind {
        PieceType::Rook => "R",
        PieceType::Bishop => "B",
        PieceType::Knight => "N",
        _ => "Q",
    }
}

impl Board {
    /// Standard Algebraic Notation for the move `from` -> `to`, including the
    /// trailing `+`/`#` for check/checkmate. Call this on the board *before*
    /// the move is applied — `self` is read as the pre-move position.
    pub fn compute_san_for_move(
        &self,
        from: Square,
        to: Square,
        promotion: Option<PieceType>,
        is_capture: bool,
    ) -> Result<String, MoveError> {
        let moving = self.squares[from.rank][from.file].ok_or(MoveError::IllegalMove)?;

        let mut san = self.castle_san(&moving, from, to).unwrap_or_else(|| {
            if moving.kind == PieceType::Pawn {
                self.pawn_san(from, to, is_capture, promotion)
            } else {
                self.piece_san(&moving, from, to, is_capture)
            }
        });

        self.append_check_or_mate_suffix(&mut san, &moving, from, to, promotion);

        Ok(san)
    }

    fn castle_san(&self, moving: &ChessPiece, from: Square, to: Square) -> Option<String> {
        if moving.kind != PieceType::King || !is_castle_move(from, to) {
            return None;
        }
        if to.file == KINGSIDE.king_destination_file {
            Some("O-O".to_string())
        } else {
            Some("O-O-O".to_string())
        }
    }

    fn pawn_san(
        &self,
        from: Square,
        to: Square,
        is_capture: bool,
        promotion: Option<PieceType>,
    ) -> String {
        let dest = coord_to_algebraic(to);
        let mut san = if is_capture {
            format!("{}x{}", (b'a' + from.file as u8) as char, dest)
        } else {
            dest
        };
        if let Some(kind) = promotion {
            san.push('=');
            san.push_str(promotion_letter(kind));
        }
        san
    }

    fn piece_san(&self, moving: &ChessPiece, from: Square, to: Square, is_capture: bool) -> String {
        format!(
            "{}{}{}{}",
            piece_letter(moving.kind),
            self.disambiguate(moving, from, to),
            if is_capture { "x" } else { "" },
            coord_to_algebraic(to)
        )
    }

    /// The file/rank/both prefix needed to tell `moving` apart from any other
    /// piece of the same kind and color that could also legally reach `to` —
    /// e.g. "Nbd7" when two knights could both go to d7. Empty if `moving` is
    /// the only piece of its kind that can make this move.
    fn disambiguate(&self, moving: &ChessPiece, from: Square, to: Square) -> String {
        let others_that_reach_to: Vec<Square> = self
            .squares
            .iter()
            .flatten()
            .flatten()
            .filter(|p| p.position != from && p.kind == moving.kind && p.color == moving.color)
            .filter(|p| {
                self.legal_moves.get(&p.id).is_some_and(|moves| {
                    moves.quiet_moves.contains(&to) || moves.capture_moves.contains(&to)
                })
            })
            .map(|p| p.position)
            .collect();

        if others_that_reach_to.is_empty() {
            return String::new();
        }

        let file_char = (b'a' + from.file as u8) as char;
        let rank_char = (BOARD_SIZE - from.rank).to_string();
        let shares_a_file = others_that_reach_to.iter().any(|sq| sq.file == from.file);

        let disambiguator = if shares_a_file {
            rank_char.clone()
        } else {
            file_char.to_string()
        };

        if others_that_reach_to.len() > 1 && disambiguator.chars().count() == 1 {
            format!("{}{}", file_char, rank_char)
        } else {
            disambiguator
        }
    }

    /// Plays `from` -> `to` on a scratch clone (applied by hand, not via
    /// `move_piece` — calling that would recompute SAN and recurse forever)
    /// and appends '#' or '+' to `san` if the opponent ends up in checkmate
    /// or check.
    fn append_check_or_mate_suffix(
        &self,
        san: &mut String,
        moving: &ChessPiece,
        from: Square,
        to: Square,
        promotion: Option<PieceType>,
    ) {
        let opponent = moving.color.opposite();
        let mut sim = self.clone();

        let is_en_passant = moving.kind == PieceType::Pawn
            && from.file != to.file
            && sim.squares[to.rank][to.file].is_none()
            && sim.en_passant_target == Some(to);
        if is_en_passant {
            let (forward_dr, _) = pawn_forward(moving.color).step();
            let captured_rank = (to.rank as i8 - forward_dr) as usize;
            sim.squares[captured_rank][to.file] = None;
        }

        sim.squares[from.rank][from.file] = None;

        let mut moved = *moving;
        moved.position = to;
        if let Some(kind) = promotion {
            if Board::pawn_reaches_promotion_rank(moving.color, to.rank) {
                moved.kind = kind;
            }
        }

        if moving.kind == PieceType::King && is_castle_move(from, to) {
            let side = if to.file == KINGSIDE.king_destination_file {
                &KINGSIDE
            } else {
                &QUEENSIDE
            };
            let rook = sim.squares[from.rank][side.rook_file].take();
            sim.squares[from.rank][side.rook_destination_file] = rook;
        }

        sim.squares[to.rank][to.file] = Some(moved);
        sim.turn = opponent;

        if sim.is_checkmate() {
            san.push('#');
        } else if sim.is_in_check(opponent) {
            san.push('+');
        }
    }

    /// Resolves one SAN token ("Nbd2", "exd8=Q+", "O-O") against the side to
    /// move in the current position, into the `PieceMove` its own
    /// `move_piece` takes the fields of. `Err(IllegalMove)` if the token is
    /// malformed, matches no legal move, or — after applying any file/rank
    /// disambiguator it carries — still matches more than one.
    ///
    /// Refreshes the legal-move cache itself, so callers replaying a game
    /// don't have to interleave `refresh_legal_moves` between plies.
    pub fn san_to_move(&mut self, san: &str) -> Result<PieceMove, MoveError> {
        self.refresh_legal_moves();
        let parsed = parse_san(san).ok_or(MoveError::IllegalMove)?;

        if let Some(side) = parsed.castle {
            return self.resolve_castle(side);
        }

        let origins: Vec<Square> = self
            .squares
            .iter()
            .flatten()
            .flatten()
            .filter(|piece| piece.color == self.turn && piece.kind == parsed.kind)
            .filter(|piece| parsed.from_file.is_none_or(|f| piece.position.file == f))
            .filter(|piece| parsed.from_rank.is_none_or(|r| piece.position.rank == r))
            .filter(|piece| {
                let Some(moves) = self.legal_moves.get(&piece.id) else {
                    return false;
                };
                let is_capture = moves.capture_moves.contains(&parsed.dest);
                let reaches = is_capture || moves.quiet_moves.contains(&parsed.dest);
                let promotes = parsed.kind == PieceType::Pawn
                    && Board::pawn_reaches_promotion_rank(piece.color, parsed.dest.rank);
                reaches
                    && (!parsed.is_capture || is_capture)
                    && promotes == parsed.promotion.is_some()
            })
            .map(|piece| piece.position)
            .collect();

        match origins.as_slice() {
            [from] => Ok(PieceMove {
                from: *from,
                to: parsed.dest,
                promotion: parsed.promotion,
            }),
            _ => Err(MoveError::IllegalMove),
        }
    }

    fn resolve_castle(&self, side: CastleSide) -> Result<PieceMove, MoveError> {
        let back_rank = match self.turn {
            PieceColor::White => WHITE_BACK_RANK,
            PieceColor::Black => BLACK_BACK_RANK,
        };
        let king = self.squares[back_rank]
            .iter()
            .flatten()
            .find(|piece| piece.kind == PieceType::King && piece.color == self.turn)
            .copied()
            .ok_or(MoveError::IllegalMove)?;

        let dest_file = match side {
            CastleSide::King => KINGSIDE.king_destination_file,
            CastleSide::Queen => QUEENSIDE.king_destination_file,
        };
        let dest = Square::new(back_rank, dest_file);

        let is_legal = self
            .legal_moves
            .get(&king.id)
            .is_some_and(|moves| moves.quiet_moves.contains(&dest));
        if is_legal {
            Ok(PieceMove {
                from: king.position,
                to: dest,
                promotion: None,
            })
        } else {
            Err(MoveError::IllegalMove)
        }
    }
}

enum CastleSide {
    King,
    Queen,
}

/// A SAN token split into what's needed to locate its move on the board:
/// the mover's kind, whatever file/rank disambiguator was written, the
/// destination square, and the promotion piece.
struct ParsedSan {
    kind: PieceType,
    from_file: Option<usize>,
    from_rank: Option<usize>,
    dest: Square,
    promotion: Option<PieceType>,
    is_capture: bool,
    castle: Option<CastleSide>,
}

fn piece_type_from_letter(ch: char) -> Option<PieceType> {
    match ch {
        'K' => Some(PieceType::King),
        'Q' => Some(PieceType::Queen),
        'R' => Some(PieceType::Rook),
        'B' => Some(PieceType::Bishop),
        'N' => Some(PieceType::Knight),
        _ => None,
    }
}

fn square_from_str(s: &str) -> Option<Square> {
    let mut chars = s.chars();
    let file = chars.next()?;
    let rank = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    let file = ('a'..='h')
        .contains(&file)
        .then(|| file as usize - 'a' as usize)?;
    let rank_digit = rank.to_digit(10)?;
    (1..=BOARD_SIZE as u32)
        .contains(&rank_digit)
        .then(|| Square::new(BOARD_SIZE - rank_digit as usize, file))
}

fn parse_san(raw: &str) -> Option<ParsedSan> {
    let compact: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    let core = compact
        .replace("e.p.", "")
        .trim_end_matches(['+', '#', '!', '?'])
        .to_string();

    let castle_form = core.replace('0', "O");
    let castle = match castle_form.as_str() {
        "O-O" => Some(CastleSide::King),
        "O-O-O" => Some(CastleSide::Queen),
        _ => None,
    };
    if let Some(side) = castle {
        return Some(ParsedSan {
            kind: PieceType::King,
            from_file: None,
            from_rank: None,
            dest: Square::new(0, 0),
            promotion: None,
            is_capture: false,
            castle: Some(side),
        });
    }

    let mut body = core;
    let mut promotion = None;
    if let Some(eq) = body.find('=') {
        promotion = Some(piece_type_from_letter(body[eq + 1..].chars().next()?)?);
        body.truncate(eq);
    } else {
        let bytes = body.as_bytes();
        if bytes.len() >= 3 {
            let last = bytes[bytes.len() - 1] as char;
            let rank = bytes[bytes.len() - 2] as char;
            let file = bytes[bytes.len() - 3] as char;
            if matches!(rank, '1' | '8')
                && ('a'..='h').contains(&file)
                && matches!(last, 'Q' | 'R' | 'B' | 'N')
            {
                promotion = piece_type_from_letter(last);
                body.pop();
            }
        }
    }

    let is_capture = body.contains('x');
    body.retain(|c| c != 'x');

    let (kind, rest) = match body.chars().next() {
        Some(c) if piece_type_from_letter(c).is_some() => {
            (piece_type_from_letter(c).unwrap(), &body[1..])
        }
        _ => (PieceType::Pawn, body.as_str()),
    };
    if rest.len() < 2 {
        return None;
    }

    let dest = square_from_str(&rest[rest.len() - 2..])?;

    let mut from_file = None;
    let mut from_rank = None;
    for ch in rest[..rest.len() - 2].chars() {
        if ('a'..='h').contains(&ch) {
            from_file = Some(ch as usize - 'a' as usize);
        } else if ('1'..='8').contains(&ch) {
            from_rank = Some(BOARD_SIZE - (ch as usize - '0' as usize));
        } else {
            return None;
        }
    }

    Some(ParsedSan {
        kind,
        from_file,
        from_rank,
        dest,
        promotion,
        is_capture,
        castle: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fen::FenString;

    fn board_from(fen: &str) -> Board {
        let mut board = Board::from(&FenString::try_from(fen).unwrap());
        board.refresh_legal_moves();
        board
    }

    #[test]
    fn simple_pawn_advance() {
        let board = board_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let san = board
            .compute_san_for_move(Square::new(6, 4), Square::new(4, 4), None, false)
            .unwrap();

        assert_eq!(san, "e4");
    }

    #[test]
    fn pawn_capture_includes_origin_file() {
        let board = board_from("8/8/8/3p4/4P3/8/8/4K2k w - - 0 1");
        let san = board
            .compute_san_for_move(Square::new(4, 4), Square::new(3, 3), None, true)
            .unwrap();

        assert_eq!(san, "exd5");
    }

    #[test]
    fn kingside_castle_is_o_dash_o() {
        let board = board_from("4k3/8/8/8/8/8/8/4K2R w K - 0 1");
        let san = board
            .compute_san_for_move(Square::new(7, 4), Square::new(7, 6), None, false)
            .unwrap();

        assert_eq!(san, "O-O");
    }

    #[test]
    fn check_gets_a_plus_suffix() {
        let board = board_from("4k3/8/8/8/8/8/8/3R3K w - - 0 1");
        let san = board
            .compute_san_for_move(Square::new(7, 3), Square::new(0, 3), None, false)
            .unwrap();

        assert_eq!(san, "Rd8+");
    }

    #[test]
    fn disambiguates_between_two_knights() {
        let board = board_from("4k3/8/8/8/8/8/8/1N1N3K w - - 0 1");
        // Both knights (b1 and d1) can reach c3; moving the b1 knight there
        // needs the file to disambiguate, since b1 and d1 don't share a file.
        let san = board
            .compute_san_for_move(Square::new(7, 1), Square::new(5, 2), None, false)
            .unwrap();

        assert_eq!(san, "Nbc3");
    }

    /// Shorthand for the `PieceMove` a test expects back.
    fn pm(from: Square, to: Square, promotion: Option<PieceType>) -> PieceMove {
        PieceMove {
            from,
            to,
            promotion,
        }
    }

    #[test]
    fn san_to_move_resolves_a_pawn_push() {
        let mut board = board_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        let mv = board.san_to_move("e4").unwrap();

        assert_eq!(mv, pm(Square::new(6, 4), Square::new(4, 4), None));
    }

    #[test]
    fn san_to_move_ignores_redundant_disambiguation() {
        let mut board = board_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        // Only the g1 knight can reach f3, so "Ngf3" over-specifies — it
        // should still resolve to the same move as "Nf3".
        assert_eq!(board.san_to_move("Ngf3"), board.san_to_move("Nf3"));
        assert_eq!(
            board.san_to_move("Nf3").unwrap(),
            pm(Square::new(7, 6), Square::new(5, 5), None)
        );
    }

    #[test]
    fn san_to_move_uses_the_file_disambiguator() {
        let mut board = board_from("4k3/8/8/8/8/5N2/8/1N2K3 w - - 0 1");

        let from_b1 = board.san_to_move("Nbd2").unwrap();
        let from_f3 = board.san_to_move("Nfd2").unwrap();

        assert_eq!(from_b1, pm(Square::new(7, 1), Square::new(6, 3), None));
        assert_eq!(from_f3, pm(Square::new(5, 5), Square::new(6, 3), None));
    }

    #[test]
    fn san_to_move_rejects_an_ambiguous_token() {
        let mut board = board_from("4k3/8/8/8/8/5N2/8/1N2K3 w - - 0 1");

        assert_eq!(board.san_to_move("Nd2"), Err(MoveError::IllegalMove));
    }

    #[test]
    fn san_to_move_resolves_a_capture_and_strips_the_check_suffix() {
        let mut board = board_from("6k1/8/8/8/8/8/8/4RK2 w - - 0 1");

        let mv = board.san_to_move("Re8+").unwrap();

        assert_eq!(mv, pm(Square::new(7, 4), Square::new(0, 4), None));
    }

    #[test]
    fn san_to_move_resolves_castling_both_sides() {
        let mut kingside = board_from("4k3/8/8/8/8/8/8/4K2R w K - 0 1");
        assert_eq!(
            kingside.san_to_move("O-O").unwrap(),
            pm(Square::new(7, 4), Square::new(7, 6), None)
        );

        let mut queenside = board_from("r3k3/8/8/8/8/8/8/R3K3 w Q - 0 1");
        assert_eq!(
            queenside.san_to_move("0-0-0").unwrap(),
            pm(Square::new(7, 4), Square::new(7, 2), None)
        );
    }

    #[test]
    fn san_to_move_reads_the_promotion_piece() {
        let mut board = board_from("7k/4P3/8/8/8/8/8/4K3 w - - 0 1");

        assert_eq!(
            board.san_to_move("e8=Q").unwrap(),
            pm(Square::new(1, 4), Square::new(0, 4), Some(PieceType::Queen))
        );
        assert_eq!(
            board.san_to_move("e8=N").unwrap().promotion,
            Some(PieceType::Knight)
        );
        // Same move written without the '=' sign.
        assert_eq!(
            board.san_to_move("e8Q").unwrap().promotion,
            Some(PieceType::Queen)
        );
    }

    #[test]
    fn san_to_move_rejects_an_illegal_move() {
        let mut board = board_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        assert_eq!(board.san_to_move("e5"), Err(MoveError::IllegalMove));
    }
}
