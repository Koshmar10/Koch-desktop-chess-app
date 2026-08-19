-- Dev-only seed data for time_logs. NOT a migration — never registered with
-- rusqlite_migration, never ships to real users.
--
-- Applied automatically: db::connection::init() embeds this file (debug builds
-- only, via #[cfg(debug_assertions)]) and runs it once, the first time it finds
-- time_logs empty. No manual step needed for a normal `cargo tauri dev` run.
--
-- To force a reseed (e.g. after wiping your dev db), delete the existing rows
-- first, then relaunch the app — or run this file directly:
--   sqlite3 ~/.local/share/com.petru.koch/koch.db "DELETE FROM time_logs;"
--   sqlite3 ~/.local/share/com.petru.koch/koch.db < dev-seed.sql

INSERT INTO time_logs (connect_at, disconnect_at) VALUES
    ('2026-08-05T09:15:00.000Z', '2026-08-05T10:02:00.000Z'),
    ('2026-08-05T20:30:00.000Z', '2026-08-05T21:10:00.000Z'),
    ('2026-08-07T14:00:00.000Z', '2026-08-07T16:45:00.000Z'),
    ('2026-08-09T08:05:00.000Z', '2026-08-09T08:20:00.000Z'),
    -- crosses midnight: exercises the day-boundary edge case
    ('2026-08-10T23:59:00.000Z', '2026-08-11T01:02:00.000Z'),
    ('2026-08-12T12:00:00.000Z', '2026-08-12T12:30:00.000Z'),
    ('2026-08-13T18:00:00.000Z', '2026-08-13T19:20:00.000Z'),
    ('2026-08-14T07:45:00.000Z', '2026-08-14T08:00:00.000Z'),
    ('2026-08-15T21:00:00.000Z', '2026-08-15T23:30:00.000Z'),
    ('2026-08-16T10:00:00.000Z', '2026-08-16T10:45:00.000Z'),
    -- disconnect_at left NULL: exercises the crash/force-quit edge case
    ('2026-08-17T09:00:00.000Z', NULL);
