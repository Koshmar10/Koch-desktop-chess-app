-- Set on a PGN / chess.com import whose movetext stopped replaying part
-- way through (an illegal or unparsable move). The game and the moves that
-- did replay are still saved; the history card marks it so analysis over a
-- fragment isn't mistaken for a full game.
ALTER TABLE games ADD COLUMN partial_import INTEGER NOT NULL DEFAULT 0;
