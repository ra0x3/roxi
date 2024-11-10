# roxi

<image src="https://i.imgur.com/ADlVxrr.png" height="200px" />

## Usage

### Install WireGuard

```sh
sudo -E sh scripts/wg.sh
```

## Dependencies

- `rustc 1.81.0`
- `wireguard-tools v1.0`
- `docker 20.10` (optional)

## How it works

### Centralized peer-to-peer setup

<image src="https://www.researchgate.net/publication/356245976/figure/fig3/AS:1093585697021954@1637742556713/The-centralized-peer-to-peer-P2P-system-A-peer-E-sends-a-message-to-the-central-server.ppm" height="300px" />

### Network Address Translator (NAT) Punching

<image src="https://www.researchgate.net/publication/228411948/figure/fig6/AS:301985531219968@1449010369011/New-method-of-UDP-multi-hole-punching.png" height="300px" />

### WireGuard Tunneling

<image src="https://www.procustodibus.com/images/blog/wireguard-topologies/site-to-site-complex.svg" height="400px" />

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
| `seed`    | Register as seed client             | ✔️           |
| `tinfo`   | Request tunnel info                 |             |
| `stinfo`  | Request stun info                   |             |
|`regateway`| Request a gateway                   |             |

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

#### Generate Wireguard keys

Mac OS

```sh
sudo wg genkey | tee /opt/homebrew/etc/wireguard/privatekey | wg pubkey | tee /opt/homebrew/etc/wireguard/publickey
```

Linux

```sh
sudo wg genkey | tee /etc/wireguard/privatekey | wg pubkey | tee /etc/wireguard/publickey
```

⚠️  Update Wireguard config with new keys

#### Start Wireguard on interface `wg0` using `wg-quick`

You can also use `boringtun` but `wg-quick` is much faster

```sh
sudo /opt/homebrew/bin/bash /opt/homebrew/bin/wg-quick up wg0
```
