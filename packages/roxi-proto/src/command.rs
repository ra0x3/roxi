use crate::{
    wireguard::{WireGuardKey, WireGuardKeyPair},
    ProtoError, ProtoResult,
};
use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
    process::{Command, Stdio},
};

pub fn reload_wireguard(interface: &str) -> io::Result<()> {
    tracing::info!("Reloading WireGuard on interface: {interface}");
    let output = Command::new("wg-quick")
        .arg("down")
        .arg(interface)
        .output()?;

    if !output.status.success() {
        tracing::error!("Failed to bring down WireGuard interface: {interface}");
        io::stdout().write_all(&output.stderr)?;
    }

    let output = Command::new("wg-quick").arg("up").arg(interface).output()?;

    if !output.status.success() {
        tracing::error!("Failed to bring up WireGuard interface: {interface}");
        io::stdout().write_all(&output.stderr)?;
    }

    tracing::info!("WireGuard reloaded successfully on interface: {interface}");

    Ok(())
}

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

pub fn derive_wireguard_pubkey(privkey: &mut WireGuardKey) -> ProtoResult<WireGuardKey> {
    let mut output = Command::new("wg")
        .arg("pubkey")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = output.stdin.as_mut() {
        let _ = stdin.write_all(privkey.as_bytes());
    }

    let output = output.wait_with_output()?;
    if !output.status.success() {
        tracing::error!(
            "Failed to generate public key: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(ProtoError::Io(io::Error::new(
            io::ErrorKind::Other,
            "Failed to generate public key",
        )));
    }

    let pubkey = String::from_utf8(output.stdout)?.trim().to_string();

    Ok(WireGuardKey::from_public(pubkey))
}

pub fn cat_wireguard_key<P: AsRef<Path>>(p: P) -> ProtoResult<WireGuardKey> {
    let p = p.as_ref();
    let mut f = File::open(p)?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;
    let key = content.trim().to_string();
    if p.ends_with("publickey") {
        return Ok(WireGuardKey::from_public(key));
    }
    Ok(WireGuardKey::from_private(key))
}
