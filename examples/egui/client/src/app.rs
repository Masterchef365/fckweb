use std::future::Future;

use anyhow::Result;
use common::MyServiceClient;
use egui::Ui;
use poll_promise::Promise;
use quic_session::web_transport::{RecvStream, SendStream, Session};

struct Connections {
    sess: Session,
}

pub struct TemplateApp {
    conn: Promise<Result<Connections>>,
    received: Vec<String>,
    client: Option<Promise<MyServiceClient>>,
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn<T: Send, F: Future<Output = T> + Send + 'static>(f: F) -> Promise<T> {
    Promise::spawn_async(f)
}

#[cfg(target_arch = "wasm32")]
fn spawn<T: Send, F: Future<Output = T> + 'static>(f: F) -> Promise<T> {
    Promise::spawn_local(f)
}

/// Don't worry about it
unsafe impl Send for Connections {}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = cc.egui_ctx.clone();

        let conn = spawn(async move {
            let url = url::Url::parse("https://127.0.0.1:9090/")?;
            let sess = quic_session::client_session(&url).await?;

            /*
            let (send, recv) = sess
                .open_bi()
                .await
                .map_err(|e| anyhow::format_err!("{e}"))?;
            */

            ctx.request_repaint();

            Ok(Connections { sess })
        });

        Self {
            received: vec![],
            conn,
            client: None,
        }
    }
}

fn connection_status(ui: &mut Ui, prom: &Promise<Result<Connections>>) {
    match prom.ready() {
        Some(Ok(_)) => {
            ui.label(format!("Connection open"));
        }
        Some(Err(e)) => {
            ui.label(format!("Error: {e:?}"));
        }
        None => {
            ui.label(format!("Connecting"));
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| connection_status(ui, &self.conn));
    }
}

impl TemplateApp {
    fn open_conn(&mut self, ui: &mut Ui) -> Result<()> {
        if let Some(Ok(conn)) = self.conn.ready_mut() {
            match &mut self.client {
                None => {
                    self.client = Some(Promise::spawn_async(async {
                        let socks = conn.sess.accept_bi().await.unwrap();
                        let channel = framework::webtransport_transport_protocol(socks);
                        let newclient = MyServiceClient::new(Default::default(), channel);
                        tokio::task::spawn(newclient.dispatch);
                        newclient.client
                    }));
                }
                Some(_) => (),
            }
        }

        Ok(())
    }
}
