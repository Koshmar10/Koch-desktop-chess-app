-- The engine's best move in the position this move was played from, in
-- UCI: what it would have played instead. NULL until analysis runs, and
-- for games analysed before this column existed.
ALTER TABLE game_moves ADD COLUMN best_uci TEXT;
