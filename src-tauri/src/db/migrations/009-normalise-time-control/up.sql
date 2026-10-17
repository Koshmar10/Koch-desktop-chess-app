-- Early PGN imports stored the `[TimeControl]` tag as the PGN wrote it,
-- before the import learned to normalise it to the
-- "<initial_ms>+<increment_ms>" form every other row uses. The app now
-- reads that column strictly, so the old spellings have to go.
--
-- A bare number is the PGN's base time in seconds with no increment
-- ("600" is ten minutes): rewrite it in the current form.
UPDATE games
SET time_control = (CAST(time_control AS INTEGER) * 1000) || '+0'
WHERE time_control NOT LIKE '%+%'
  AND time_control <> ''
  AND time_control NOT GLOB '*[^0-9]*';

-- Anything else still without a "+" ("-", "?", "40/7200") was never a
-- usable clock. The import stores those as NULL today; so do these.
UPDATE games
SET time_control = NULL
WHERE time_control NOT LIKE '%+%';
