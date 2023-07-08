# BWT construction in small space

This is a Rust implementation of the BWT construction algorithm in small space,
described in Algorithm 11.8 of the book:
[Compact Data Structures - A Practical Approach](https://users.dcc.uchile.cl/~gnavarro/CDSbook/),
Gonzalo Navarro, 2016.

## Documentation

https://docs.rs/small-bwt/

## Command line tool

`tools` provides a command line tool to construct the BWT of a file.

```shell
$ cargo run --release -p tools -- -i english.50MB -o english.50MB.bwt -t
```

## Benchmarks

`benches` provides benchmarks on the time performance for English texts
extracted from [Pizza&Chili Corpus](http://pizzachili.dcc.uchile.cl/texts.html).

```shell
$ cargo bench
```

## Licensing

This library is licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

`benches/english.10MB.zst` is extracted from [Pizza&Chili Corpus](http://pizzachili.dcc.uchile.cl/texts.html) and follows [LGPL License](https://www.gnu.org/licenses/lgpl-3.0.html).
