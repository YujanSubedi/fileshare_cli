# File share cli

## build

```bash
cargo build
```

## Add binary to path

```bash
[ -d "$HOME/.local/bin" ] || mkdir -p ~/.local/bin
cargo build && cp ./target/debug/fileshare_cli ~/.local/bin/
PATH="${PATH}":"${HOME}/.local/bin"
```

Add the following on ~/.bash_profile or ~/.zprofile

```txt
export PATH="${PATH}":"${HOME}/.local/bin"
```

------

## Server

> Command:

```bash
./target/debug/fileshare_cli filepath
```

```bash
cargo run filepath
```

------
Specify port for http and tcp

```bash
./target/debug/fileshare_cli filepath http_port tcp_port
```

```bash
cargo run filepath http_port tcp_port
```
------

## Client
>
> Command:

```bash
./target/debug/fileshare_cli ip_adress port_address
```

```bash
cargo run ip_adress port_address
```

------

## Todo

- Secure connnection
- Multiple file and folder
