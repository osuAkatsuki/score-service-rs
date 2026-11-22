CREATE TABLE users (
    id INT NOT NULL PRIMARY KEY
) ENGINE = InnoDB;

CREATE TABLE scores (
    id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    beatmap_md5 CHAR(32) NOT NULL,
    userid INT NOT NULL,
    score INT NOT NULL,
    max_combo INT NOT NULL,
    full_combo TINYINT(1) NOT NULL,
    mods INT NOT NULL,
    `300_count` SMALLINT NOT NULL,
    `100_count` SMALLINT NOT NULL,
    `50_count` SMALLINT NOT NULL,
    katus_count SMALLINT NOT NULL,
    gekis_count SMALLINT NOT NULL,
    misses_count SMALLINT NOT NULL,
    time INT NOT NULL,
    play_mode TINYINT NOT NULL,
    completed TINYINT NOT NULL,
    accuracy FLOAT NOT NULL,
    pp FLOAT NOT NULL,
    checksum CHAR(32) UNIQUE,
    KEY (beatmap_md5, play_mode, completed, userid)
) ENGINE = InnoDB;

INSERT INTO users (id) VALUES (1);
