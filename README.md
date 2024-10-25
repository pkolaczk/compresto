# A command-line tool to benchmark performance of various compression algorithms

## Installation

```
cargo install compresto
```

## Usage
```
$ compresto benchmark-many my-data-file.db                      
lz4 -c -9: 89546338 => 42221447 (47.2 %), compression: 541.4 MB/s, decompression: 2510.1 MB/s
lz4 -c -8: 89546338 => 41926937 (46.8 %), compression: 644.2 MB/s, decompression: 3300.6 MB/s
lz4 -c -7: 89546338 => 41829209 (46.7 %), compression: 634.8 MB/s, decompression: 3286.0 MB/s
lz4 -c -6: 89546338 => 41258304 (46.1 %), compression: 635.0 MB/s, decompression: 3345.1 MB/s
lz4 -c -5: 89546338 => 39902031 (44.6 %), compression: 629.2 MB/s, decompression: 3310.6 MB/s
lz4 -c -4: 89546338 => 39266371 (43.9 %), compression: 622.9 MB/s, decompression: 3196.3 MB/s
lz4 -c -3: 89546338 => 38674191 (43.2 %), compression: 607.6 MB/s, decompression: 3252.5 MB/s
lz4 -c -2: 89546338 => 38591727 (43.1 %), compression: 617.7 MB/s, decompression: 3189.3 MB/s
lz4 -c -1: 89546338 => 38400035 (42.9 %), compression: 621.4 MB/s, decompression: 3121.4 MB/s
lz4 -c 1: 89546338 => 32528001 (36.3 %), compression: 406.6 MB/s, decompression: 3999.9 MB/s
lz4 -c 2: 89546338 => 32528001 (36.3 %), compression: 406.8 MB/s, decompression: 3974.9 MB/s
lz4 -c 3: 89546338 => 29550723 (33.0 %), compression: 209.7 MB/s, decompression: 3971.8 MB/s
lz4 -c 4: 89546338 => 29126576 (32.5 %), compression: 181.9 MB/s, decompression: 3960.8 MB/s
lz4 -c 5: 89546338 => 28920442 (32.3 %), compression: 157.6 MB/s, decompression: 3957.5 MB/s
lz4 -c 6: 89546338 => 28833768 (32.2 %), compression: 138.7 MB/s, decompression: 3942.6 MB/s
lz4 -c 7: 89546338 => 28804139 (32.2 %), compression: 124.0 MB/s, decompression: 3943.4 MB/s
lz4 -c 8: 89546338 => 28783943 (32.1 %), compression: 111.2 MB/s, decompression: 3970.5 MB/s
lz4 -c 9: 89546338 => 28768860 (32.1 %), compression: 97.0 MB/s, decompression: 3990.5 MB/s
lzav -c 0: 89546338 => 34367849 (38.4 %), compression: 597.9 MB/s, decompression: 2802.6 MB/s
lzav -c 1: 89546338 => 30857000 (34.5 %), compression: 156.5 MB/s, decompression: 2725.9 MB/s
snappy -c 0: 89546338 => 35324879 (39.4 %), compression: 667.4 MB/s, decompression: 1502.5 MB/s
zstd -c -7: 89546338 => 40787934 (45.5 %), compression: 580.3 MB/s, decompression: 1328.8 MB/s
zstd -c -6: 89546338 => 38659206 (43.2 %), compression: 551.3 MB/s, decompression: 1318.8 MB/s
zstd -c -5: 89546338 => 38251290 (42.7 %), compression: 537.0 MB/s, decompression: 1232.5 MB/s
zstd -c -4: 89546338 => 35493717 (39.6 %), compression: 496.8 MB/s, decompression: 1161.0 MB/s
zstd -c -3: 89546338 => 32054789 (35.8 %), compression: 492.1 MB/s, decompression: 1138.3 MB/s
zstd -c -2: 89546338 => 29628625 (33.1 %), compression: 474.2 MB/s, decompression: 1104.9 MB/s
zstd -c -1: 89546338 => 26859909 (30.0 %), compression: 438.9 MB/s, decompression: 1079.8 MB/s
zstd -c 1: 89546338 => 25560726 (28.5 %), compression: 390.4 MB/s, decompression: 902.5 MB/s
zstd -c 2: 89546338 => 25571060 (28.6 %), compression: 398.1 MB/s, decompression: 839.2 MB/s
zstd -c 3: 89546338 => 22860123 (25.5 %), compression: 387.0 MB/s, decompression: 879.6 MB/s
zstd -c 4: 89546338 => 21829331 (24.4 %), compression: 232.4 MB/s, decompression: 892.6 MB/s
zstd -c 5: 89546338 => 21740274 (24.3 %), compression: 161.9 MB/s, decompression: 938.6 MB/s
zstd -c 6: 89546338 => 21495048 (24.0 %), compression: 108.2 MB/s, decompression: 974.7 MB/s
zstd -c 7: 89546338 => 21311887 (23.8 %), compression: 69.5 MB/s, decompression: 998.7 MB/s
zstd -c 8: 89546338 => 21254569 (23.7 %), compression: 43.1 MB/s, decompression: 997.1 MB/s
zstd -c 9: 89546338 => 21271497 (23.8 %), compression: 31.3 MB/s, decompression: 1008.1 MB/s
zstd -c 10: 89546338 => 21267578 (23.8 %), compression: 28.9 MB/s, decompression: 1011.0 MB/s
zstd -c 11: 89546338 => 21183520 (23.7 %), compression: 20.1 MB/s, decompression: 899.7 MB/s
zstd -c 12: 89546338 => 20401135 (22.8 %), compression: 14.5 MB/s, decompression: 907.4 MB/s
brotli -c 1: 89546338 => 28709909 (32.1 %), compression: 269.0 MB/s, decompression: 413.6 MB/s
brotli -c 2: 89546338 => 24774480 (27.7 %), compression: 173.8 MB/s, decompression: 465.9 MB/s
brotli -c 3: 89546338 => 24042747 (26.8 %), compression: 140.8 MB/s, decompression: 481.0 MB/s
brotli -c 4: 89546338 => 23555811 (26.3 %), compression: 108.3 MB/s, decompression: 487.9 MB/s
brotli -c 5: 89546338 => 21255567 (23.7 %), compression: 74.2 MB/s, decompression: 490.8 MB/s
brotli -c 6: 89546338 => 21058638 (23.5 %), compression: 54.5 MB/s, decompression: 500.6 MB/s
brotli -c 7: 89546338 => 20947199 (23.4 %), compression: 31.1 MB/s, decompression: 505.3 MB/s
brotli -c 8: 89546338 => 20876549 (23.3 %), compression: 52.2 MB/s, decompression: 505.4 MB/s
```