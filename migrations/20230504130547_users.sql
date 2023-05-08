-- Add migration script here
CREATE TABLE IF NOT EXISTS user_ids (
    id VARCHAR(64) PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS user_names (
    user_id VARCHAR(64) NOT NULL,
    name VARCHAR(255) NOT NULL,
    ts TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, name),
    FOREIGN KEY (user_id) REFERENCES user_ids(id)
);

