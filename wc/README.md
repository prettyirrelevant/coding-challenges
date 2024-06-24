# ccwc - Custom Word Count

`ccwc` is a Go reimplementation of the Unix `wc` (word count) utility. It provides identical functionality to count bytes, lines, words, and characters in a file.

## Features

- [x] Count bytes in a file (`-c`)
- [x] Count lines in a file (`-l`)
- [x] Count words in a file (`-w`)
- [x] Count characters in a file (`-m`)

## Usage

To use `ccwc`, compile and run it as follows:

```sh
go build

./ccwc [options] [filename]
```

If no filename is provided, `ccwc` reads from standard input.

### Options

- `-c`: Count bytes
- `-l`: Count lines
- `-w`: Count words
- `-m`: Count characters

If no options are specified, `ccwc` defaults to `-c -l -w` (count bytes, lines, and words).

For detailed usage information, refer to the original `wc` command:

```sh
man wc
```

## Performance

Benchmarks were conducted using `hyperfine` on a MacBook Pro M1 8GB, comparing `ccwc` against the original `wc` tool.

> [!NOTE]
> These benchmarks may not provide an entirely accurate depiction of performance across all scenarios or systems. However, they made me feel good about the implementation, so I'm including them here.

### Benchmark Command

```sh
hyperfine --prepare 'sudo purge' "./ccwc <flags> test.txt" "wc <flags> test.txt"
```

### Results Summary

#### All Flags Set

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `./ccwc -l -c -w -m test.txt` | 75.1 ± 29.8 | 55.6 | 167.9 | 1.00 |
| `wc -l -c -w -m test.txt` | 76.9 ± 31.6 | 48.7 | 149.0 | 1.02 ± 0.58 |

#### Count Bytes Only

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `./ccwc -c test.txt` | 76.3 ± 36.6 | 18.3 | 165.5 | 1.00 |
| `wc -c test.txt` | 80.4 ± 39.9 | 35.9 | 158.6 | 1.05 ± 0.73 |

#### Count Lines Only

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `./ccwc -l test.txt` | 74.3 ± 32.5 | 19.9 | 146.9 | 1.00 |
| `wc -l test.txt` | 88.5 ± 46.7 | 5.8 | 157.7 | 1.19 ± 0.82 |

#### Count Words Only

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `./ccwc -w test.txt` | 88.8 ± 35.6 | 50.2 | 143.8 | 1.00 |
| `wc -w test.txt` | 106.8 ± 64.1 | 38.3 | 223.6 | 1.20 ± 0.87 |

#### Count Characters Only

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `./ccwc -m test.txt` | 98.0 ± 46.5 | 55.8 | 201.5 | 1.00 |
| `wc -m test.txt` | 130.1 ± 70.3 | 47.1 | 231.7 | 1.33 ± 0.95 |

#### No Flags Set

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `./ccwc test.txt` | 92.4 ± 38.2 | 54.9 | 179.4 | 1.00 |
| `wc test.txt` | 130.1 ± 55.7 | 43.8 | 208.7 | 1.41 ± 0.84 |
