use futures::{SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Serialize};

use crate::{BiStream, ClientFramework};

pub struct BiStreamProxy<Rx, Tx> {
    tx: tokio::sync::mpsc::Sender<Tx>,
    rx: std::sync::mpsc::Receiver<Rx>,
}

impl<Rx, Tx> BiStreamProxy<Rx, Tx>
where
    Rx: DeserializeOwned + Send + Sync + 'static,
    Tx: Serialize + Send + 'static,
{
    pub fn new<F>(token: BiStream<Rx, Tx>, frame: ClientFramework, mut call_on_rx: F) -> Self
    where
        F: FnMut() + Send + 'static,
    {
        let (loop_tx, rx) = std::sync::mpsc::channel();
        let (tx, mut loop_rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            let stream = frame.connect_bistream(token).await?;
            let (mut sink, mut stream) = stream.split();

            tokio::spawn(async move {
                while let Some(msg) = stream.next().await.transpose()? {
                    loop_tx.send(msg)?;
                    call_on_rx();
                }
                Ok::<_, anyhow::Error>(())
            });

            tokio::spawn(async move {
                while let Some(msg) = loop_rx.recv().await {
                    sink.send(msg).await?;
                }
                Ok::<_, anyhow::Error>(())
            });

            Ok::<_, anyhow::Error>(())
        });

        Self { tx, rx }
    }

    pub fn send(&mut self, val: Tx) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(val).await;
        });
    }

    pub fn recv_iter(&mut self) -> impl Iterator<Item = Rx> + '_ {
        self.rx.try_iter()
    }
}
