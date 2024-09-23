use std::{
    fmt::{Debug, Display},
    future::Future,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use common::{MyOtherServiceClient, MyServiceClient};
use egui::{DragValue, Ui};
use framework::{tarpc::client::RpcError, ClientFramework};
use poll_promise::Promise;

#[derive(Clone)]
struct Connection {
    frame: ClientFramework,
    client: MyServiceClient,
}

pub struct TemplateApp {
    sess: Promise<Result<Connection>>,
    other_client: Option<Promise<Result<MyOtherServiceClient>>>,
    a: u32,
    b: u32,
    subtract_result: Option<Promise<Result<u32, RpcError>>>,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let egui_ctx = cc.egui_ctx.clone();

        let sess = Promise::spawn_async(async move {
            // Get framework and channel
            let url = url::Url::parse("https://127.0.0.1:9090/")?;
            let sess = quic_session::client_session(&url, common::CERTIFICATE.to_vec()).await?;
            let (frame, channel) = ClientFramework::new(sess).await?;

            // Get root client
            let newclient = MyServiceClient::new(Default::default(), channel);
            tokio::spawn(newclient.dispatch);
            let client = newclient.client;

            egui_ctx.request_repaint();

            Ok(Connection { frame, client })
        });

        Self {
            sess,
            a: 420,
            b: 69,
            subtract_result: None,
            other_client: None,
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

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            connection_status(ui, &self.sess);

            if let Some(Ok(sess)) = self.sess.ready_mut() {
                // Adding
                ui.add(DragValue::new(&mut self.a).prefix("a: "));
                ui.add(DragValue::new(&mut self.b).prefix("b: "));

                let spawner = SimpleSpawner::new("adder_id");

                if ui.button("Add").clicked() {
                    let ctx = framework::tarpc::context::current();
                    let client_clone = sess.client.clone();
                    let a = self.a;
                    let b = self.b;

                    spawner.spawn(ui, async move { client_clone.add(ctx, a, b).await });
                }

                spawner.show(ui, |ui, result| {
                    match result {
                        Ok(val) => ui.label(format!("Subtract result: {val}")),
                        Err(e) => ui.label(format!("Error: {e:?}")),
                    };
                });

                ui.strong("Subtraction");

                if let Some(prom) = self.other_client.as_mut() {
                    connection_status(ui, prom);

                    let spawner = SimpleSpawner::new("subtractor_id");

                    if let Some(Ok(other_client)) = prom.ready_mut() {
                        // Subtracting
                        if ui.button("Subtract").clicked() {
                            let ctx = framework::tarpc::context::current();
                            let client_clone = other_client.clone();
                            let a = self.a;
                            let b = self.b;

                            spawner.spawn(ui, async move { client_clone.subtract(ctx, a, b).await });
                        }

                        spawner.show(ui, |ui, result| {
                            match result {
                                Ok(val) => ui.label(format!("Subtract result: {val}")),
                                Err(e) => ui.label(format!("Error: {e:?}")),
                            };
                        });
                    }
                } else {
                    if ui.button("Connect to subtractor").clicked() {
                        let sess = sess.clone();
                        self.other_client = Some(Promise::spawn_async(async move {
                            // Call a method on that client, yielding another service!
                            let ctx = framework::tarpc::context::current();
                            let subservice = sess.client.get_sub(ctx).await?;
                            let other_channel = sess.frame.connect_subservice(subservice).await?;
                            let newclient =
                                MyOtherServiceClient::new(Default::default(), other_channel);
                            tokio::task::spawn(newclient.dispatch);
                            Ok(newclient.client)
                        }));
                    }
                }
            }
        });
    }
}

struct SimpleSpawner<T> {
    id: egui::Id,
    _phantom: PhantomData<T>,
}

impl<T: Send + 'static> SimpleSpawner<T> {
    pub fn new(id: impl Into<egui::Id>) -> Self {
        Self {
            id: id.into(),
            _phantom: PhantomData,
        }
    }

    /// Spawns the task, requesting repaint on finish. Saves to temporary memory.
    pub fn spawn<F>(&self, ui: &mut Ui, f: F)
    where
        F: Future<Output = T> + Send + 'static,
    {
        let ctx = ui.ctx().clone();

        let id = self.id;
        ui.ctx().memory_mut(move |w| {
            w.data.insert_temp(
                id,
                Some(Arc::new(Mutex::new(Promise::spawn_async(async move {
                    let ret = f.await;
                    ctx.request_repaint();
                    ret
                })))),
            );
        });
    }

    pub fn show(&self, ui: &mut Ui, f: impl FnOnce(&mut Ui, &mut T)) {
        if ui
            .ctx()
            .memory(|w| w.data.get_temp::<Option<Arc<Mutex<Promise<T>>>>>(self.id))
            .is_none()
        {
            ui.label("Value not set.");
        } else {
            let val = ui.ctx().memory_mut(|w| {
                w.data
                    .get_temp_mut_or_default::<Option<Arc<Mutex<Promise<T>>>>>(self.id)
                    .clone()
                    .unwrap()
            });

            let mut lck = val.lock().unwrap();

            if let Some(ready) = lck.ready_mut() {
                f(ui, ready)
            } else {
                ui.label("Working ...");
            }
        }
    }
}
