# A command-line tool to benchmark performance of various compression algorithms

## Installation

```
cargo install compresto
```

## Usage
```
$ compresto benchmark-many my-data-file.db                      
lz4 -c 1: 89546338 => 36469599 (40.7 %), compression: 500.3 MB/s, decompression: 2012.9 MB/s
lz4 -c 2: 89546338 => 32588127 (36.4 %), compression: 363.3 MB/s, decompression: 2079.8 MB/s
lz4 -c 3: 89546338 => 29610849 (33.1 %), compression: 196.3 MB/s, decompression: 2085.9 MB/s
lz4 -c 4: 89546338 => 29186702 (32.6 %), compression: 172.0 MB/s, decompression: 2107.8 MB/s
lz4 -c 5: 89546338 => 28980568 (32.4 %), compression: 148.5 MB/s, decompression: 2041.0 MB/s
lz4 -c 6: 89546338 => 28893894 (32.3 %), compression: 129.3 MB/s, decompression: 2052.5 MB/s
lz4 -c 7: 89546338 => 28864265 (32.2 %), compression: 116.7 MB/s, decompression: 2051.1 MB/s
lz4 -c 8: 89546338 => 28844069 (32.2 %), compression: 104.6 MB/s, decompression: 2080.8 MB/s
lz4 -c 9: 89546338 => 28828986 (32.2 %), compression: 92.8 MB/s, decompression: 2061.7 MB/s
snappy -c 0: 89546338 => 35379539 (39.5 %), compression: 503.6 MB/s, decompression: 1073.2 MB/s
zstd -c -7: 89546338 => 42104638 (47.0 %), compression: 567.1 MB/s, decompression: 1540.0 MB/s
zstd -c -6: 89546338 => 39925560 (44.6 %), compression: 541.7 MB/s, decompression: 1511.0 MB/s
zstd -c -5: 89546338 => 39063211 (43.6 %), compression: 507.7 MB/s, decompression: 1360.1 MB/s
zstd -c -4: 89546338 => 37257137 (41.6 %), compression: 504.2 MB/s, decompression: 1311.8 MB/s
zstd -c -3: 89546338 => 33355740 (37.2 %), compression: 463.2 MB/s, decompression: 1263.5 MB/s
zstd -c -2: 89546338 => 30451624 (34.0 %), compression: 463.0 MB/s, decompression: 1232.8 MB/s
zstd -c -1: 89546338 => 27335385 (30.5 %), compression: 415.9 MB/s, decompression: 1166.7 MB/s
zstd -c 1: 89546338 => 25142634 (28.1 %), compression: 381.0 MB/s, decompression: 960.4 MB/s
zstd -c 2: 89546338 => 25016687 (27.9 %), compression: 269.7 MB/s, decompression: 924.0 MB/s
zstd -c 3: 89546338 => 23389191 (26.1 %), compression: 199.2 MB/s, decompression: 930.3 MB/s
zstd -c 4: 89546338 => 23385840 (26.1 %), compression: 129.0 MB/s, decompression: 906.6 MB/s
zstd -c 5: 89546338 => 22260445 (24.9 %), compression: 79.2 MB/s, decompression: 897.4 MB/s
zstd -c 6: 89546338 => 22144788 (24.7 %), compression: 62.2 MB/s, decompression: 990.1 MB/s
zstd -c 7: 89546338 => 21977755 (24.5 %), compression: 42.3 MB/s, decompression: 989.6 MB/s
zstd -c 8: 89546338 => 21907444 (24.5 %), compression: 38.8 MB/s, decompression: 1024.5 MB/s
zstd -c 9: 89546338 => 21907212 (24.5 %), compression: 70.9 MB/s, decompression: 975.0 MB/s
zstd -c 10: 89546338 => 21799350 (24.3 %), compression: 14.6 MB/s, decompression: 1014.5 MB/s
zstd -c 11: 89546338 => 21729419 (24.3 %), compression: 14.6 MB/s, decompression: 1022.3 MB/s
zstd -c 12: 89546338 => 21729415 (24.3 %), compression: 7.7 MB/s, decompression: 1023.0 MB/s
brotli -c 1: 89546338 => 28673712 (32.0 %), compression: 231.2 MB/s, decompression: 402.6 MB/s
brotli -c 2: 89546338 => 24737627 (27.6 %), compression: 166.2 MB/s, decompression: 448.3 MB/s
brotli -c 3: 89546338 => 24005938 (26.8 %), compression: 133.8 MB/s, decompression: 466.8 MB/s
brotli -c 4: 89546338 => 23518948 (26.3 %), compression: 107.6 MB/s, decompression: 480.6 MB/s
brotli -c 5: 89546338 => 21304743 (23.8 %), compression: 67.1 MB/s, decompression: 481.6 MB/s
brotli -c 6: 89546338 => 21112346 (23.6 %), compression: 58.1 MB/s, decompression: 500.9 MB/s
brotli -c 7: 89546338 => 21019355 (23.5 %), compression: 51.0 MB/s, decompression: 485.2 MB/s
brotli -c 8: 89546338 => 20943694 (23.4 %), compression: 44.9 MB/s, decompression: 492.8 MB/s
```