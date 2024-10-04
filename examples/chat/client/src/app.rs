use std::{
    fmt::{Debug, Display},
    future::Future,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use chat_common::{ChatServiceClient, MessageMetaData};
use egui::{Color32, DragValue, Grid, RichText, Ui};
use egui_shortcuts::SimpleSpawner;
use framework::{BiStreamProxy, ClientFramework};
use poll_promise::Promise;
use std::sync::mpsc::Receiver;

#[derive(Clone)]
struct Connection {
    frame: ClientFramework,
    client: ChatServiceClient,
}

pub struct TemplateApp {
    sess: Promise<Result<Connection>>,
    msg_edit: String,
    username: String,
    color: Color32,
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

        Self {
            sess,
            color: Color32::WHITE,
            msg_edit: "".into(),
            username: "my_username".into(),
        }
    }
}

fn connection_status<T: Send, E: Debug + Send>(ui: &mut Ui, prom: &Promise<Result<T, E>>) {
    match prom.ready() {
        None => ui.label("Connecting"),
        Some(Ok(_)) => ui.label("Connection open"),
        Some(Err(e)) => ui.label(format!("Error: {e:?}")),
    };
}

struct ChatSession {
    stream: BiStreamProxy<MessageMetaData, MessageMetaData>,
    received: Vec<MessageMetaData>,
}

impl ChatSession {
    pub fn new(stream: BiStreamProxy<MessageMetaData, MessageMetaData>) -> Self {
        Self {
            stream,
            received: vec![],
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.username);
                ui.color_edit_button_srgba(&mut self.color);
            });

            connection_status(ui, &self.sess);

            if let Some(Ok(sess)) = self.sess.ready_mut() {
                let rooms_spawner = SimpleSpawner::new("rooms_spawner");
                let chat_spawner = SimpleSpawner::new("chat_spawner");

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

                                        let egui_ctx = ui.ctx().clone();
                                        let name = name.clone();
                                        let frame = sess.frame.clone();
                                        chat_spawner.spawn(ui, async move {
                                            let stream = client_clone.chat(ctx, name).await??;
                                            let stream =
                                                BiStreamProxy::new(stream, frame, move || {
                                                    egui_ctx.request_repaint()
                                                });
                                            let chat_sess = ChatSession::new(stream);
                                            Ok::<_, anyhow::Error>(chat_sess)
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

                chat_spawner.show(ui, |ui, result| match result {
                    Ok(chat_sess) => {
                        ui.strong("Connected to chat");

                        for msg in chat_sess.stream.recv_iter() {
                            chat_sess.received.push(msg);
                        }

                        for msg in &chat_sess.received {
                            ui.horizontal(|ui| {
                                let [r, g, b] = msg.user_color;
                                ui.label(
                                    RichText::new(&msg.username).color(Color32::from_rgb(r, g, b)),
                                );
                                ui.label(&msg.msg);
                            });
                        }

                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.msg_edit);
                            if ui.button("Submit").clicked() {
                                let msg = MessageMetaData {
                                    msg: self.msg_edit.clone(),
                                    username: self.username.clone(),
                                    user_color: [0xff; 3],
                                };
                                chat_sess.stream.send(msg);
                                self.msg_edit = "".into();
                            }
                        });
                    }
                    Err(e) => {
                        ui.label(format!("Error: {e:?}"));
                    }
                });
            }
        });
    }
}
