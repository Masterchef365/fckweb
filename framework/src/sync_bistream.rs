use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Serialize};

use crate::{BiStream, ClientFramework};

pub struct BiStreamProxy<Rx, Tx> {
    tx: futures::channel::mpsc::UnboundedSender<Tx>,
    rx: std::sync::mpsc::Receiver<Rx>,
}

impl<Rx, Tx> BiStreamProxy<Rx, Tx>
where
    Rx: DeserializeOwned + Send + Sync + 'static,
    Tx: Serialize + Send + 'static,
{
    pub async fn new<F>(
        token: BiStream<Rx, Tx>,
        frame: ClientFramework,
        mut call_on_rx: F,
    ) -> Result<Self>
    where
        F: FnMut() + Send + 'static,
    {
        let (loop_tx, rx) = std::sync::mpsc::channel();
        let (tx, mut loop_rx) = futures::channel::mpsc::unbounded();

        let stream = frame.connect_bistream(token).await?;
        let (mut sink, mut stream) = stream.split();

        crate::spawn(async move {
            while let Some(msg) = stream.next().await.transpose()? {
                loop_tx.send(msg)?;
                call_on_rx();
            }
            Ok::<_, anyhow::Error>(())
        });

        crate::spawn(async move {
            while let Some(msg) = loop_rx.next().await {
                sink.send(msg).await?;
            }
            Ok::<_, anyhow::Error>(())
        });

        Ok(Self { tx, rx })
    }

    pub fn send(&mut self, val: Tx) {
        let mut tx = self.tx.clone();
        crate::spawn(async move {
            let _ = tx.send(val).await;
        });
    }

    pub fn recv_iter(&mut self) -> impl Iterator<Item = Rx> + '_ {
        self.rx.try_iter()
    }
}
