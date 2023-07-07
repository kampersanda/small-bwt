# BWT construction in small space

This is a Rust implementation of the BWT construction algorithm in small space,
described in Algorithm 11.8 of the book:
[Compact Data Structures - A Practical Approach](https://users.dcc.uchile.cl/~gnavarro/CDSbook/),
Gonzalo Navarro, 2016.

## Documentation

https://docs.rs/small-bwt/

## Command line tool

```
$ cargo run --release -p tools -- -i ~/data/pizzachili/text/english.50MB -o bwt.50MB.txt -t
```

## Licensing

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
