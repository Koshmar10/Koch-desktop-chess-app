-- Which 'best' grades were 'brilliant' or 'great' isn't recorded, so going
-- back calls them all 'great', the closer of the two.
UPDATE game_moves SET quality = 'great' WHERE quality = 'best';
UPDATE game_moves SET quality = NULL WHERE quality = 'book';
