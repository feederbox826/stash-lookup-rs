CREATE TABLE tags (
    uuid BLOB PRIMARY KEY,
    category BLOB
);

CREATE TABLE studios (
    uuid BLOB PRIMARY KEY,
    parent BLOB
);

-- name/alias table
-- 0 primary 1 alias
CREATE TABLE tag_names (
    tag_uuid BLOB NOT NULL,
    name TEXT NOT NULL,
    role INTEGER NOT NULL,
    PRIMARY KEY (tag_uuid, name),
    FOREIGN KEY (tag_uuid) REFERENCES tags(uuid)
);

CREATE TABLE studio_names (
    studio_uuid BLOB NOT NULL,
    name TEXT NOT NULL,
    role INTEGER NOT NULL,
    PRIMARY KEY (studio_uuid, name),
    FOREIGN KEY (studio_uuid) REFERENCES studios(uuid)
);

CREATE TABLE performer_names (
    uuid BLOB NOT NULL,
    name TEXT NOT NULL,
    role INTEGER NOT NULL,
    PRIMARY KEY (uuid, name)
);

CREATE INDEX idx_tags_uuid ON tags(uuid);
CREATE INDEX idx_tag_name_uuid ON tag_names(tag_uuid);
CREATE INDEX idx_tag_name_name ON tag_names(LOWER(TRIM(name)));
CREATE INDEX idx_studios_uuid ON studios(uuid);
CREATE INDEX idx_studio_name_uuid ON studio_names(studio_uuid);
CREATE INDEX idx_studio_name_name ON studio_names(LOWER(TRIM(name)));
CREATE INDEX idx_performer_name_uuid ON performer_names(uuid);
CREATE INDEX idx_performer_name_name ON performer_names(LOWER(TRIM(name)));
