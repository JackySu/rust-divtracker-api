-- Add migration script here
CREATE TABLE user_ids (
    id VARCHAR(64) PRIMARY KEY
);

CREATE TABLE user_names (
    user_id VARCHAR(64) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    ts TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES user_ids(id)
);

