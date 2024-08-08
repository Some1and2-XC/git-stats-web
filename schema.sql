PRAGMA foreign_keys = on;

CREATE TABLE IF NOT EXISTS Users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL UNIQUE,
    credits INTEGER DEFAULT 0 NOT NULL,
    date_created DATE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_accessed DATE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS Hosts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    host_name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS Repos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    owner_fk INTEGER NOT NULL,
    host_fk INTEGER NOT NULL,
    repo_name TEXT NOT NULL,
    FOREIGN KEY (owner_fk)
        REFERENCES Users(id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    FOREIGN KEY (host_fk)
        REFERENCES Hosts(id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS Commits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_fk INTEGER NOT NULL,
    user_fk INTEGER NOT NULL,
    committer_username TEXT NOT NULL,
    committer_email TEXT NOT NULL,
    timestamp DATE NOT NULL,
    date_created DATE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_modified DATE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    projected_start DATE, -- nullable on purpose
    set_start DATE,       -- nullable on purpose
    FOREIGN KEY (user_fk)
        REFERENCES Users(id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    FOREIGN KEY (repo_fk)
        REFERENCES Repos(id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);
