use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};
use tarpc::Transport;
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};
use tokio_util::codec::{Decoder, LengthDelimitedCodec};

use futures::{SinkExt, StreamExt};
use web_transport::{RecvStream, SendStream};

/*
/// Internal type representing the identity of a connection between client and server
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct BiStream(usize);
*/

const BUFFER_SIZE: usize = 4096; // Chosen arbitrarily!
const MAX_READ_BYTES: usize = 4096; // Chosen arbitrarily!

/*
/// This is the type used to provide connectivity to an alternate tarpc connection
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TarpcBiStream(BiStream);
*/

/// Converts a webtransport bidirectional connection into a DuplexStream
/// Warning: spawns tasks underneath
pub fn webtransport_futures_bridge((mut tx, mut rx): (SendStream, RecvStream)) -> DuplexStream {
    let (proxy, ret) = tokio::io::duplex(BUFFER_SIZE);

    let (mut readhalf, mut writehalf) = tokio::io::split(proxy);

    tokio::spawn(async move {
        loop {
            let mut buf = vec![0_u8; BUFFER_SIZE];

            let n_bytes_read = readhalf.read(&mut buf).await?;
            buf.truncate(n_bytes_read);

            tx.write(&buf).await?;
        }

        #[allow(unreachable_code)]
        Ok::<_, FrameworkError>(())
    });

    tokio::spawn(async move {
        loop {
            if let Some(bytes) = rx.read(MAX_READ_BYTES).await? {
                writehalf.write(bytes.as_ref()).await?;
            }
        }

        #[allow(unreachable_code)]
        Ok::<_, FrameworkError>(())
    });

    ret
}

pub fn webtransport_protocol<Rx: DeserializeOwned, Tx: Serialize>(
    socks: (SendStream, RecvStream),
) -> impl Transport<Tx, Rx, Error = FrameworkError> {
    let duplex = webtransport_futures_bridge(socks);

    LengthDelimitedCodec::default()
        .framed(duplex)
        .sink_map_err(FrameworkError::from)
        .with(|obj: Tx| async move { Ok(Bytes::from(encode(&obj)?)) })
        .map(|frame| Ok(decode(&frame?)?))
}

#[derive(thiserror::Error, Debug)]
pub enum FrameworkError {
    #[error("Serialization")]
    Bincode(#[from] bincode::Error),

    #[error("Websocket")]
    WebSocket(#[from] web_transport::Error),

    #[error("Duplex IO")]
    Io(#[from] std::io::Error),
}

/// The encoding function for all data. Mostly for internal use, exposed here for debugging
/// potential
pub fn encode<T: Serialize>(value: &T) -> bincode::Result<Vec<u8>> {
    serde_json::to_writer_pretty(std::io::stdout(), value).unwrap();
    bincode::serialize(value)
}

/// The dencoding function for all data. Mostly for internal use, exposed here for debugging
/// potential
pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> bincode::Result<T> {
    bincode::deserialize(bytes)
}
