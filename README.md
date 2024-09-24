# roxi

<image src="#" height="200px" />

## Dependencies

- `rustc 1.78.0`
- `boringtun 0.5.2`
- `wireguard-tools v1.0`
- `docker 20.10` (optional)

## Installation

The following script should install all prerequisites and dependencies

```sh
curl --proto '=https' --tlsv1.2 -sSf https://install.roxi.app | sh
```

## Commands List

| Command   | Description                         | Implemented |
|-----------|-------------------------------------|-------------|
| `hello`   | Test hello command                  | ✔️           |
| `serve`   | Start Roxi server                   | ✔️           |
| `connect` | Connect to Roxi server              |             |
| `ping`    | Ping Roxi server                    | ✔️           |
| `auth`    | Authenticate against Roxi server    | ✔️           |
| `stun`    | Send public IP to STUN server       | ✔️           |
| `gateway` | Start client gateway server         |             |

## Development

### Commands

Start development server

```sh
cargo run --bin roxi -- serve --config server.yaml
```

#### Ping

```sh
cargo run --bin roxi -- ping --config client.yaml
```

#### Auth

```sh
cargo run --bin roxi -- auth --config client.yaml
```

#### Stun

```sh
cargo run --bin roxi -- stun --config client.yaml
```

### Testing TUN interface on Mac

#### Install Wireguard

Mac OS

```sh
HOMEBREW_NO_AUTO_UPDATE=1 brew install wireguard-tools
```

Linux

```sh
sudo apt update && sudo apt install -y wireguard
```

#### Install bash 4+

Mac OS

```sh
HOMEBREW_NO_AUTO_UPDATE=1 brew install bash
```

Linux

```sh
sudo apt-get update && sudo apt-get install -y bash
```

#### Update WireGuard config

Copy Wireguard server conf over to config

Mac OS

```sh
cp wg0.conf /opt/homebrew/etc/wireguard/
```

Generate Wireguard keys

```sh
sudo wg genkey | sudo tee /etc/wireguard/privatekey | sudo wg pubkey | sudo tee /etc/wireguard/publickey
```

⚠️  Update Wireguard config with new keys

#### Start Wireguard on interface `wg0` using `wg-quick`

You can also use `boringtun` but `wg-quick` is much faster

```sh
sudo /opt/homebrew/bin/bash /opt/homebrew/bin/wg-quick up wg0
```
