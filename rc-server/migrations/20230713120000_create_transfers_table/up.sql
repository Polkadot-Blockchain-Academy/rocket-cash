CREATE TABLE transfers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sender INTEGER NOT NULL,
    receiver INTEGER NOT NULL,
    amount INTEGER NOT NULL,
    transferred_on TEXT NOT NULL
);