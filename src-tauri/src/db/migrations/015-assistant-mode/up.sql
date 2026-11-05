-- Which model the assistant answers with, and how long it thinks: 'fast'
-- or 'deep'. NULL on rows saved before the setting existed, read as fast.
ALTER TABLE app_settings ADD COLUMN assistant_mode TEXT;
