DROP TABLE IF EXISTS approved;

CREATE TABLE approved (
  id SERIAL PRIMARY KEY,
  image_data BYTEA NOT NULL,
  attribution TEXT
);

DROP TABLE IF EXISTS submitted;

CREATE TABLE submitted (
  id SERIAL PRIMARY KEY,
  image_data BYTEA NOT NULL,
  attribution TEXT
);

DROP TABLE IF EXISTS removed;

CREATE TABLE removed (
  id SERIAL PRIMARY KEY,
  image_data BYTEA NOT NULL,
  attribution TEXT,
  reason TEXT
);