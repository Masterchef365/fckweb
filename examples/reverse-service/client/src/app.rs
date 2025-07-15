use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;
use egui::Ui;
use egui_shortcuts::SimpleSpawner;
use egui_shortcuts::{spawn_promise, Promise};
use framework::futures::lock::Mutex as FuturesMutex;
use framework::futures::StreamExt;
use framework::tarpc::server::{BaseChannel, Channel};
use framework::{tarpc, ClientFramework};
use reverse_common::{MyOtherService, MyServiceClient};

#[derive(Clone)]
struct Connection {
    frame: ClientFramework,
    client: MyServiceClient,
}

pub struct TemplateApp {
    sess: Promise<Result<Connection>>,
    offer: SimpleSpawner<Result<()>>,
    log: Arc<FuturesMutex<Vec<String>>>,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let egui_ctx = cc.egui_ctx.clone();

        let sess = spawn_promise(async move {
            // Get framework and channel
            let url = url::Url::parse("https://127.0.0.1:9090/")?;
            let sess = quic_session::client_session_selfsigned(
                &url,
                reverse_common::CERTIFICATE.to_vec(),
                reverse_common::CERTIFICATE_HASHES.to_vec(),
            )
            .await?;
            let (frame, channel) = ClientFramework::new(sess).await?;

            // Get root client
            let newclient = MyServiceClient::new(Default::default(), channel);
            framework::spawn(newclient.dispatch);
            let client = newclient.client;

            egui_ctx.request_repaint();

            Ok(Connection { frame, client })
        });

        Self {
            sess,
            log: Default::default(),
            offer: SimpleSpawner::new("offer"),
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
                let conn = sess.clone();
                let log = self.log.clone();
                if ui.button("Offer service").clicked() {
                    self.offer.spawn(ui, async move {
                        let ctx = tarpc::context::current();
                        let (token, channelfuture) = conn.frame.accept_reverse_subservice();
                        conn.client.offer(ctx, token).await?;

                        framework::spawn(async move {
                            let transport = BaseChannel::with_defaults(channelfuture.await?);

                            let server = MyOtherServiceServer { log };
                            let executor = transport.execute(MyOtherService::serve(server));

                            framework::spawn(executor.for_each(|response| async move {
                                framework::spawn(response);
                            }));

                            Ok::<_, anyhow::Error>(())
                        });

                        Ok(())
                    });
                }
            }

            self.offer.show(ui, |ui, result| {
                match result {
                    Ok(()) => {
                        ui.strong(format!("Connected."));
                        if let Some(lck) = self.log.try_lock() {
                            for entry in lck.iter() {
                                ui.label(entry);
                            }
                        }
                    }
                    Err(e) => {
                        ui.label(format!("Error: {e:?}"));
                    }
                };
            });
        });
    }
}

#[derive(Clone)]
struct MyOtherServiceServer {
    log: Arc<FuturesMutex<Vec<String>>>,
}

impl MyOtherService for MyOtherServiceServer {
    async fn subtract(self, _context: tarpc::context::Context, a: u32, b: u32) -> u32 {
        let result = a.saturating_sub(b);
        self.log.lock().await.push(format!("{a} - {b} = {result}"));
        result
    }
}
