![alt text](img/logo.png "HoneyBee")

**honeybee** is a port of the original [i3status](https://github.com/i3/i3status) written in Rust. This was meant to be a learning exercise and for my personal use. It's still lacking lots of functionality and proper error handling / robustness, so use it at your own peril.

# Motivation

My main motivation is to practice Rust and use more advanced patterns and constructs. I use [i3](https://i3wm.org/) as a Window Manager, and while searching for a replacement of the default i3status I thought it could actually be a good idea to write my own. I've read the entire [source code](https://github.com/i3/i3status) of the original one (which is written in C) and used it as a starting point. Eventually, I came across [i3status-rust](https://github.com/greshake/i3status-rust), which I used as source of inspiration for some idiomatic ways of writing certain parts in Rust, namely proper error handling and macros.

# Getting started

You can download a pre-compiled version from the [releases](https://github.com/csixteen/honeybee/releases) page. Alternatively, if you have Rust [installed](https://rustup.rs/), you can clone the repository and run `cargo install`.

# Configuration

Once installed, you can edit the [sample configuration file](examples/config.toml) and copy it to one of the following locations:

1. `$XDG_CONFIG_HOME/honeybee/config.toml`
2. `$XDG_DATA_HOME/honeybee/config.toml`
3. `$HOME/.honeybee.toml`
4. `$XDG_DATA_DIRS/honeybee/config.toml`
5. `$XDG_CONFIG_DIRS/honeybee/config.toml`

One of the main differences between the original i3status and honeybee is that the latter uses [TOML](https://github.com/toml-lang/toml/) format.

## General configuration

At the top-level, you can define the following configuration variables:

| Variable          | Description                                                                                                         | Values                                                                                | Default   |
|-------------------|---------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------|-----------|
| `output_format`   | Defines the format string used for the output                                                                       | `i3bar`,<br/>`term`,<br/>`xmobar`,<br/>`lemonbar`,<br/>`dzen2`                        | `i3bar`   |
| `colors`          | Will disable all the colors if set to `false`                                                                       | `true`, `false`                                                                       | `true`    |
| `separator`       | Separator to use between modules. Set to empty string if you want to disable.                                       |                                                                                       |           |
| `color_separator` | Color used to paint the separator bar between modules.                                                              | Canonical RGB hexadecimal triplet (with no separator between colors), prefixed by `#` |           |
| `color_good`      | Color used to display good values.                                                                                  | Same as above                                                                         | `#00FF00` |
| `color_degraded`  | Color used to display degraded values.                                                                              | Same as above                                                                         | `#FFFF00` |
| `color_bad`       | Color used to display bad values.                                                                                   | Same as above                                                                         | `#FF0000` |
| `markup`          | When set to `pango`, you'll be able to use Pango markup and specify font, color, size, etc, in the format strings.  | `pango`, `none`                                                                       | `none`    |
| `update_interval` | Time, in seconds, that modules will sleep until they update their values. Can be overwritten on a per module basis. | Integer (represents seconds)                                                          | 10        |

## Modules

Just like in the original i3status, the basic idea of honeybee is that you can specify which modules should be used.

Modules are specified in the configuration TOML as an array:

```toml
[[module]]
module = "module_name"
property1 = "value1"
```

You can consult the [documentation](https://csixteen.github.io/honeybee/honeybee/modules/index.html) for details on how to configure the modules.

| Module          | Status             |
|-----------------|--------------------|
| Battery         | :construction:     |
| Load Average    | :heavy_check_mark: |
| Memory          | :heavy_check_mark: |
| Path exists     | :heavy_check_mark: |
| Run watch       | :heavy_check_mark: |
| Time            | :heavy_check_mark: |
| Timezone        | :heavy_check_mark: |
| CPU Temperature | :x:                |
| CPU usage       | :x:                |
| Date            | :x:                |
| Disk            | :x:                |
| Ethernet        | :x:                |
| File Contents   | :x:                |
| IPv4 Address    | :x:                |
| IPv6 Address    | :x:                |
| Volume          | :x:                |


# Command-line options

```shell
honeybee --help
```

# Roadmap

You can follow the [open issues](https://github.com/csixteen/honeybee/issues) to see what I'm planning to work on. The main objective is to port the original i3status completely. Later, I'll probably enhance it with extra features.

# References

- [bumblebee-status](https://github.com/tobi-wan-kenobi/bumblebee-status) - I came across this project when I started doing some research. I tried it out, it looks really nice, but it consumes way more resources than I wanted. I used it as an inspiration mostly for the name.
- [i3status](https://github.com/i3/i3status) - the original one.
- [i3status-rust](https://github.com/greshake/i3status-rust) - helped rewrite certain parts in more idiomatic Rust.
- [procfs](https://github.com/eminence/procfs/blob/master/src/meminfo.rs) - source of inspiration for the `memory` module. I decided to not use this crate because it maintains way more info in memory than I need.
- [tokio](https://tokio.rs/)
- [serde](https://serde.rs/)
- [clap](https://docs.rs/clap/latest/clap/index.html)
