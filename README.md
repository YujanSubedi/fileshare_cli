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
./target/debug/fileshare_cli ip_adress port_address filepath
```

```bash
cargo run ip_adress port_address filepath
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
