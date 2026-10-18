CREATE TABLE rooms (
    code TEXT PRIMARY KEY NOT NULL,
    data TEXT NOT NULL
);

CREATE TABLE replays (
    id TEXT PRIMARY KEY NOT NULL,
    room_code TEXT NOT NULL REFERENCES rooms(code),
    data TEXT NOT NULL
);
