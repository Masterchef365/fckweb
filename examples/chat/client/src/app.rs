use std::{
    fmt::{Debug, Display},
    future::Future,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use chat_common::ChatServiceClient;
use egui::{DragValue, Grid, Ui};
use egui_shortcuts::SimpleSpawner;
use framework::{tarpc::client::RpcError, ClientFramework};
use poll_promise::Promise;

#[derive(Clone)]
struct Connection {
    frame: ClientFramework,
    client: ChatServiceClient,
}

pub struct TemplateApp {
    sess: Promise<Result<Connection>>,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let egui_ctx = cc.egui_ctx.clone();

        let sess = Promise::spawn_async(async move {
            // Get framework and channel
            let url = url::Url::parse("https://127.0.0.1:9090/")?;
            let sess =
                quic_session::client_session(&url, chat_common::CERTIFICATE.to_vec()).await?;
            let (frame, channel) = ClientFramework::new(sess).await?;

            // Get root client
            let newclient = ChatServiceClient::new(Default::default(), channel);
            tokio::spawn(newclient.dispatch);
            let client = newclient.client;

            egui_ctx.request_repaint();

            Ok(Connection { frame, client })
        });

        Self { sess }
    }
}

fn connection_status<T: Send, E: Debug + Send>(ui: &mut Ui, prom: &Promise<Result<T, E>>) {
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
                let rooms_spawner = SimpleSpawner::new("rooms_spawner");
                let chat_spawner = SimpleSpawner::new("rooms_spawner");

                if ui.button("Get rooms").clicked() {
                    let ctx = framework::tarpc::context::current();
                    let client_clone = sess.client.clone();

                    rooms_spawner.spawn(ui, async move { client_clone.get_rooms(ctx).await });
                }

                rooms_spawner.show(ui, |ui, result| {
                    match result {
                        Ok(val) => {
                            for (name, desc) in val {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{name} {}", desc.long_desc));

                                    if ui.button("Connect").clicked() {
                                        let ctx = framework::tarpc::context::current();
                                        let client_clone = sess.client.clone();

                                        rooms_spawner.reset(ui);

                                        let name = name.clone();
                                        chat_spawner.spawn(ui, async move {
                                            client_clone.chat(ctx, name).await
                                        });
                                    }
                                });
                            }
                        }
                        Err(e) => {
                            ui.label(format!("Error: {e:?}"));
                        }
                    };
                });

                chat_spawner.show(ui, |ui, result| {
                    match result {
                        Ok(Ok(stream)) => {
                            //stream
                        },
                        Err(e) => {
                            ui.label(format!("Error: {e:?}"));
                        },
                        Ok(Err(e)) => {
                            ui.label(format!("Error: {e:?}"));
                        }
                    }
                });
            }
        });
    }
}
