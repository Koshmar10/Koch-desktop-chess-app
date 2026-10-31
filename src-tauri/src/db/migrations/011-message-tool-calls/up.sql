-- The tools the assistant called while writing an answer, in order. Kept for
-- the record and the chat's "how was this answered" detail; not sent back to
-- the model in later turns.
CREATE TABLE message_tool_calls (
  message_id INTEGER NOT NULL REFERENCES messages (message_id),
  position INTEGER NOT NULL,        -- order within the answer, from 0
  name TEXT NOT NULL,
  arguments TEXT NOT NULL,          -- JSON, as the model sent it
  output TEXT NOT NULL,             -- what the tool answered
  PRIMARY KEY (message_id, position)
);
