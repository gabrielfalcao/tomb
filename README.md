# ⚰Tomb - Password Manager powered by Rust and AES-256-CBC encryption.

[![CI](https://github.com/gabrielfalcao/tomb/actions/workflows/rust.yml/badge.svg)](https://github.com/gabrielfalcao/tomb/actions/workflows/rust.yml)


## Developing

### "Unit" Testing

```bash
cargo test
```

### "End-to-end" Testing

```bash
make test
```


### Building

```bash
cargo build --release
```


## Installing

```
cargo build --release
cp target/release/tomb /usr/local/bin/
```


## Command-line Usage


> Tomb File is located in `~/.tomb.yaml` by default.
> Customizable via:
> * environment variable `TOMB_FILE`
> * command-line option `--tomb-file` or `-t`

> Tomb Key is located in `~/.tomb.key` by default.
> Customizable via:
> * environment variable `TOMB_KEY`
> * command-line option `--key-filename` or `-k`


### Initialize your Tomb


```bash
tomb init
```

### Add secrets

tomb save personal/email/myuser@protonmail.com 'I <3 Nickelback'
tomb save personal/netflix/myuser@protonmail.com '123456'
tomb save personal/spotify/myuser@protonmail.com '987654'

```
