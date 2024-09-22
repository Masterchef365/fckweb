use std::future::Future;

use anyhow::Result;
use common::MyServiceClient;
use egui::{DragValue, Ui};
use framework::tarpc::client::RpcError;
use poll_promise::Promise;
use quic_session::web_transport::{RecvStream, SendStream, Session};

pub struct TemplateApp {
    sess: Promise<Result<Session>>,
    received: Vec<String>,
    client: Option<Promise<MyServiceClient>>,

    a: u32,
    b: u32,
    result: Option<Promise<Result<u32, RpcError>>>,
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn<T: Send, F: Future<Output = T> + Send + 'static>(f: F) -> Promise<T> {
    Promise::spawn_async(f)
}

#[cfg(target_arch = "wasm32")]
fn spawn<T: Send, F: Future<Output = T> + 'static>(f: F) -> Promise<T> {
    Promise::spawn_local(f)
}

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

            Ok(sess)
        });

        Self {
            a: 69,
            b: 420,
            result: None,
            received: vec![],
            sess: conn,
            client: None,
        }
    }
}

fn connection_status(ui: &mut Ui, prom: &Promise<Result<Session>>) {
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
        egui::CentralPanel::default().show(ctx, |ui| {
            connection_status(ui, &self.sess);
            ui.add(DragValue::new(&mut self.a).prefix("a: "));
            ui.add(DragValue::new(&mut self.b).prefix("b: "));

            self.open_conn(ui).unwrap();

            if let Some(result) = self.result.as_ref().and_then(|res| res.ready()) {
                match result {
                    Ok(val) => ui.label(format!("Result: {val}")),
                    Err(e) => ui.label(format!("Error: {e:?}")),
                };
            }

        });
    }
}

impl TemplateApp {
    fn open_conn(&mut self, ui: &mut Ui) -> Result<()> {
        if let Some(Ok(sess)) = self.sess.ready_mut() {
            match &mut self.client {
                None => {
                    ui.label("Opening socket");

                    let mut sess = sess.clone();

                    self.client = Some(Promise::spawn_async(async move {
                        let socks = sess.open_bi().await.unwrap();
                        let channel = framework::io::webtransport_protocol(socks);
                        let newclient = MyServiceClient::new(Default::default(), channel);
                        tokio::task::spawn(newclient.dispatch);
                        newclient.client
                    }));
                }
                Some(client) => {
                    if let Some(client) = client.ready() {
                        let ctx = framework::tarpc::context::current();

                        if ui.button("Add").clicked() {


                            let client = client.clone();
                            let a = self.a;
                            let b = self.b;

                            self.result = Some(Promise::spawn_async(async move {
                                client.add(ctx, a, b).await
                            }));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}