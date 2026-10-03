CREATE TABLE waitlist (
    email TEXT PRIMARY KEY COLLATE NOCASE NOT NULL
        CHECK (email = lower(trim(email)) AND length(email) BETWEEN 3 AND 254),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) WITHOUT ROWID;
