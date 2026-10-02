-- Each side's own accuracy, now that every move of a game is graded, not
-- just the human's. Nullable: analyses saved before this have no per-side
-- numbers, and NULL reads as "re-analyse to get them" where a 0 would
-- read as a perfect game.
--
-- `accuracy_percent` / `average_centipawn_loss` stay as they are: the
-- human's side, which "your average accuracy" across games reads.
ALTER TABLE analysis ADD COLUMN white_accuracy_percent REAL;
ALTER TABLE analysis ADD COLUMN white_average_centipawn_loss INTEGER;
ALTER TABLE analysis ADD COLUMN black_accuracy_percent REAL;
ALTER TABLE analysis ADD COLUMN black_average_centipawn_loss INTEGER;
