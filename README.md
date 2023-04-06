# honeybee
This is a port of the original [i3status](https://github.com/i3/i3status) written in Rust. This was meant to be a learning exercise and for my personal use. It's still lacking lots of functionality and proper error handling / robustness, so use it at your own peril. On the other hand, contributions are welcome!

# Motivation

My main motivation is to practice Rust and use more advanced patterns and constructs. I use i3 as a Window Manager, and while searching for a replacement of the default i3status I thought it could actually be a good idea to write my own. I've read the entire [source code](https://github.com/i3/i3status) of the original one (which is written in C) and used it as a starting point. Eventually, I came across [i3status-rust](https://github.com/greshake/i3status-rust), which I used as source of inspiration for some idimatic ways of writing certain parts in Rust, namely proper error handling and macros.

Hopefully, soon I'll write an extensive companion blog post.

# Running and testing

For now, you'll have to clone the repository. Then it's business as usual:

```shell
cargo run -- --help
```

If you want to run the (few) tests that exist:
```shell
cargo test
```

# How to install

```shell
cargo install
```

# Roadmap

You can follow the [open issues](https://github.com/csixteen/honeybee/issues) to see what I'm planning to work on. The main objective is to port the original i3status completely. Later, I'll probably enhance it with extra features.

# References

- [i3status](https://github.com/i3/i3status) - the original one.
- [i3status-rust](https://github.com/greshake/i3status-rust) - helped rewrite certain parts in more idiomatic Rust.
- [procfs](https://github.com/eminence/procfs/blob/master/src/meminfo.rs) - source of inspiration for the `memory` module. I decided to not use this crate because it maintains way more info in memory that I need.
- [tokio](https://tokio.rs/)
- [serde](https://serde.rs/)
- [clap](https://docs.rs/clap/latest/clap/index.html)
