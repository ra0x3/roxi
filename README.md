# roxi

<image src="#" height="200px" />

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

Install Wireguard

```sh
HOMEBREW_NO_AUTO_UPDATE=1 brew install wireguard-tools
```

Install bash 4+

```sh
HOMEBREW_NO_AUTO_UPDATE=1 brew install bash
```

Copy Wireguard server conf over to config

```sh
cp wg0.conf /opt/homebrew/etc/wireguard/
```

Generate Wireguard keys

```sh
sudo wg genkey | sudo tee /etc/wireguard/privatekey | sudo wg pubkey | sudo tee /etc/wireguard/publickey
```

*Update Wireguard config with new keys*

Start Wireguard

```sh
sudo /opt/homebrew/bin/bash /opt/homebrew/bin/wg-quick up wg0
```
