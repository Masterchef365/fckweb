use std::future::Future;

use anyhow::Result;
use common::MyServiceClient;
use egui::{DragValue, Ui};
use framework::{tarpc::client::RpcError, Framework};
use poll_promise::Promise;

struct Connection {
    frame: Framework,
    client: MyServiceClient,
}

pub struct TemplateApp {
    sess: Promise<Result<Connection>>,
    //subconn: Option<Promise<MyOtherServiceClient>>,

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

        let sess = spawn(async move {
            let url = url::Url::parse("https://127.0.0.1:9090/")?;
            let sess = quic_session::client_session(&url).await?;
            let (frame, channel) = Framework::new(sess).await?;

            let newclient = MyServiceClient::new(Default::default(), channel);
            tokio::task::spawn(newclient.dispatch);
            let client = newclient.client;

            /*
            let (send, recv) = sess
                .open_bi()
                .await
                .map_err(|e| anyhow::format_err!("{e}"))?;
            */

            ctx.request_repaint();

            Ok(Connection {
                frame,
                client,
            })
        });

        Self {
            sess,
            a: 69,
            b: 420,
            result: None,
        }
    }
}

fn connection_status<T: Send>(ui: &mut Ui, prom: &Promise<Result<T>>) {
    match prom.ready() {
        None => {
            ui.label(format!("Connecting"));
        }
        Some(Ok(_)) => {
            ui.label(format!("Connection open"));
        }
        Some(Err(e)) => {
            ui.label(format!("Error: {e:?}"));
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            connection_status(ui, &self.sess);
            if let Some(Ok(sess)) = self.sess.ready_mut() {
                ui.add(DragValue::new(&mut self.a).prefix("a: "));
                ui.add(DragValue::new(&mut self.b).prefix("b: "));

                if ui.button("Add").clicked() {

                    let ctx = framework::tarpc::context::current();
                    let client_clone = sess.client.clone();
                    let a = self.a;
                    let b = self.b;

                    self.result = Some(Promise::spawn_async(async move {
                        client_clone.add(ctx, a, b).await
                    }));
                }

                if let Some(result) = self.result.as_ref().and_then(|res| res.ready()) {
                    match result {
                        Ok(val) => ui.label(format!("Result: {val}")),
                        Err(e) => ui.label(format!("Error: {e:?}")),
                    };
                }
            }

        });
    }
}
