-- 'brilliant' used to mean "lost nothing", and 'great' "lost under 15
-- centipawns": both are what 'best' means now. Real brilliancies need a
-- re-analysis to be found.
UPDATE game_moves SET quality = 'best' WHERE quality IN ('brilliant', 'great');
