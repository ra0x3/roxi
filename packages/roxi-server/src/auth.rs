use roxi_types::SharedKey;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Authentication {
    shared_key: SharedKey,
}
