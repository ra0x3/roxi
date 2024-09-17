use crate::{
    wireguard::{WireGuardKey, WireGuardKeyPair},
    ProtoResult,
};
use std::{
    io::Write,
    process::{Command, Stdio},
};

pub fn wireguard_keypair() -> ProtoResult<WireGuardKeyPair> {
    let privkey = Command::new("wg").arg("genkey").output()?;
    let privkey = String::from_utf8(privkey.stdout)
        .unwrap()
        .trim()
        .to_string();
    Command::new("wg")
        .arg("pubkey")
        .stdin(Stdio::piped())
        .spawn()?
        .stdin
        .as_mut()
        .unwrap()
        .write_all(privkey.as_bytes())?;

    let pubkey = Command::new("wg").arg("pubkey").output()?;
    let pubkey = String::from_utf8(pubkey.stdout).unwrap().trim().to_string();

    let pubkey = WireGuardKey::from_public(pubkey);
    let privkey = WireGuardKey::from_private(privkey);

    Ok(WireGuardKeyPair::new(pubkey, privkey))
}
