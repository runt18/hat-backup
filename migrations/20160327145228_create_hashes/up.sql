CREATE TABLE IF NOT EXISTS hashes (
	id		INTEGER PRIMARY KEY,
	hash		BLOB,
        tag		INTEGER,
	height		INTEGER,
	childs		BLOB,
	blob_ref	BLOB
);

CREATE UNIQUE INDEX IF NOT EXISTS Hashes_UniqueHash ON hashes(hash);


CREATE TABLE IF NOT EXISTS gc_metadata (
	id		INTEGER PRIMARY KEY,
	hash_id		INTEGER,
	family_id	INTEGER,
	gc_int		INTEGER,
	gc_vec		BLOB
);

CREATE UNIQUE INDEX IF NOT EXISTS GcMetadata_UniqueHashFamily ON gc_metadata(hash_id, family_id);
