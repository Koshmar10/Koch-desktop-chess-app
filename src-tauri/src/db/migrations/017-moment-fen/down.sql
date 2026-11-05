-- Back to the shape of 016. The moments and their ids stay; only the
-- position goes, which 016's code never read.
ALTER TABLE game_moments DROP COLUMN fen_before;
