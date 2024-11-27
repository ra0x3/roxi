# roxi

<image src="https://i.imgur.com/ADlVxrr.png" height="200px" />

## Usage

### Install WireGuard

This will install WireGuard, including keys and configs with sensible default
settings.

```sh
sudo -E sh scripts/wg.sh
```

## Dependencies

- `rustc 1.81.0`
- `wireguard-tools v1.0`
- `docker 20.10` (optional)
- `bash 4.1+ (5.2.37)`
- `brew 4.4.0`

## How it works

Roxi is a sort've centralized peer-to-peer network where a centralized server is
used to handle most of the business logic, and individual WireGuard
nodes/clients do the actual heavy lifting VPN logic.

### Centralized peer-to-peer setup

<image src="https://berty.tech/faq/2-decentralized-distributed/centralized_decentralized_distributed.svg" height="300px" />

### Network Address Translator (NAT) Punching

<image src="https://niekdeschipper.com/networksbehindNAT.svg" height="300px" />

### WireGuard Tunneling

<image src="https://www.procustodibus.com/images/blog/wireguard-topologies/site-to-site-complex.svg" height="400px" />

### Process

Here's a snapshot of the process of two clients connecting via the Roxi
protocol. In this example ClientB is merely a seed peer - ClientA is the peer
using actual VPN functionality.

```text
ClientA                        Server                        ClientB
   |                              |                              |
   |---------- Ping ------------->|                              |
   |                              |                              |
   |<--------- Pong --------------|                              |
   |                              |                              |
   |---- Authentication --------->|                              |
   |                              |                              |
   |<------ Auth Confirm ---------|                              |
   |                              |                              |
   |                              |                              |
   |                              |<---------- Ping -------------|
   |                              |                              |
   |                              |----------- Pong ------------>|
   |                              |                              |
   |                              |<--- Authentication ----------|
   |                              |                              |
   |                              |--- Auth Confirm ------------>|
   |                              |                              |
   |                              |                              |
   |                              |<----------- Seed ------------|
   |                              |                              |
   |                              |---- Seed Ack --------------->|
   |                              |                              |
   |                              |                              |
   |--- Stun Info Req ----------->|                              |
   |                              |                              |
   |<--- ClientB Info ------------|                              |
   |                              |                              |
   |--- NAT Punch Req ------------------------------------------>|
   |                                                             |
   |<------- NAT Punch Success ----------------------------------|
   |                                                             |
   |--- Peer Tunnel Init Req ----------------------------------->|
   |                                                             |
   |<-- Peer WireGuard Info -------------------------------------|
   |                                                             |
   |--- Peer Tunnel Req ---------------------------------------->|
   |                                                             |
   |<-------- Tunnel Established --------------------------------|
   |                                                             |
```

## Installation

The following script should install all prerequisites and dependencies

> ⚠️ This script doesn't exist yet but servers as a placeholder, please follow the
> other installation instructions below.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://install.roxi.app | sh
```

### Install WireGuard

This will install WireGuard, including keys and configs with sensible default
settings.

```sh
sudo -E sh scripts/wg.sh
```

## Commands

| **Command**              | **Description**             | **Example**                     | **Implemented** |
|--------------------------|-----------------------------|---------------------------------|-----------------|
| Start development server | Run the development server  | `roxi serve -c server.yaml`     | ✔️               |
| Ping                     | Send a ping request         | `roxi ping -c client.yaml`| ✔️    |
| Authenticate             | Authenticate the client     | `roxi auth -c client.yaml`| ✔️    |
| Quick           | Run `wg-quick` commands       | `roxi quick -c client.yaml`| ✔️    |
| Seed Stun Info           | Seed STUN information       | `roxi stun -c client.yaml`| ✔️    |
| Seed Connection          | Initialize seed connection  | `roxi seed -c client.yaml`| ✔️    |
| Test Hello Command       | Test hello command          | `roxi hello`              | ✔️    |
| Start Gateway Server     | Start client gateway server | `roxi gateway -c client.yaml`|       ✔️  |
| Create tunnel          | Create a tunnel through a gateway | `roxi tunnel -c client.yaml`|           |

