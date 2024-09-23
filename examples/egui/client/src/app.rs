use std::future::Future;

use anyhow::Result;
use common::{MyOtherServiceClient, MyServiceClient};
use egui::{DragValue, Ui};
use framework::{tarpc::client::RpcError, Framework};
use poll_promise::Promise;

struct Connection {
    frame: Framework,
    client: MyServiceClient,
    other_client: MyOtherServiceClient,
}

pub struct TemplateApp {
    sess: Promise<Result<Connection>>,
    a: u32,
    b: u32,
    add_result: Option<Promise<Result<u32, RpcError>>>,
    subtract_result: Option<Promise<Result<u32, RpcError>>>,
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
        let egui_ctx = cc.egui_ctx.clone();

        let sess = spawn(async move {
            // Get framework and channel
            let url = url::Url::parse("https://127.0.0.1:9090/")?;
            let sess = quic_session::client_session(&url).await?;
            let (frame, channel) = Framework::new(sess).await?;

            // Get root client
            let newclient = MyServiceClient::new(Default::default(), channel);
            tokio::task::spawn(newclient.dispatch);
            let client = newclient.client;

            // Call a method on that client, yielding another service!
            let ctx = framework::tarpc::context::current();
            let subservice = client.get_sub(ctx).await?;
            let other_channel = frame.connect_subservice(subservice).await?;
            let newclient = MyOtherServiceClient::new(Default::default(), other_channel);
            tokio::task::spawn(newclient.dispatch);
            let other_client = newclient.client;

            /*
            let (send, recv) = sess
                .open_bi()
                .await
                .map_err(|e| anyhow::format_err!("{e}"))?;
            */

            egui_ctx.request_repaint();

            Ok(Connection { frame, client, other_client })
        });

        Self {
            sess,
            a: 69,
            b: 420,
            add_result: None,
            subtract_result: None,
        }
    }
}

fn connection_status<T: Send>(ui: &mut Ui, prom: &Promise<Result<T>>) {
    match prom.ready() {
        None => ui.label("Connecting"),
        Some(Ok(_)) => ui.label("Connection open"),
        Some(Err(e)) => ui.label(format!("Error: {e:?}")),
    };
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            connection_status(ui, &self.sess);

            if let Some(Ok(sess)) = self.sess.ready_mut() {
                // Adding
                ui.add(DragValue::new(&mut self.a).prefix("a: "));
                ui.add(DragValue::new(&mut self.b).prefix("b: "));

                if ui.button("Add").clicked() {
                    let ctx = framework::tarpc::context::current();
                    let client_clone = sess.client.clone();
                    let a = self.a;
                    let b = self.b;

                    self.add_result = Some(Promise::spawn_async(async move {
                        client_clone.add(ctx, a, b).await
                    }));
                }

                if let Some(result) = self.add_result.as_ref().and_then(|res| res.ready()) {
                    match result {
                        Ok(val) => ui.label(format!("Result: {val}")),
                        Err(e) => ui.label(format!("Error: {e:?}")),
                    };
                }

                // Subtracting
                if ui.button("Subtract").clicked() {
                    let ctx = framework::tarpc::context::current();
                    let client_clone = sess.other_client.clone();
                    let a = self.a;
                    let b = self.b;

                    self.add_result = Some(Promise::spawn_async(async move {
                        client_clone.subtract(ctx, a, b).await
                    }));
                }

                if let Some(result) = self.add_result.as_ref().and_then(|res| res.ready()) {
                    match result {
                        Ok(val) => ui.label(format!("Result: {val}")),
                        Err(e) => ui.label(format!("Error: {e:?}")),
                    };
                }

            }
        });
    }
}
