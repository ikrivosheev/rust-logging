CREATE TABLE IF NOT EXISTS logging.logs (
    `level` Enum('debug' = 0, 'info' = 1, 'warning' = 2, 'error' = 3),
    time UInt64,
    message String
) 
ENGINE = MergeTree()
PARTITION BY toYYYYMM(FROM_UNIXTIME(time)) 
ORDER BY (time, level);

