CREATE TABLE IF NOT EXISTS payments (
    correlation_id BLOB(16) PRIMARY KEY,
    amount INTEGER NOT NULL,           
    processor BOOLEAN NOT NULL,
    processed_at INTEGER NOT NULL       
) WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_payments_processed_at_processor ON payments (processor, processed_at);

CREATE TABLE IF NOT EXISTS processor_status (
    name TEXT PRIMARY KEY,
    is_failing BOOLEAN NOT NULL, 
    min_response_time INTEGER NOT NULL,
    last_checked_at INTEGER NOT NULL
);

INSERT OR IGNORE INTO processor_status (name, is_failing, min_response_time, last_checked_at)
VALUES
    ('default', FALSE, 0, 0),
    ('fallback', FALSE, 0, 0);