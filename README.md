# roxi

## Development

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
