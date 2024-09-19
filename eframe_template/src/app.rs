use anyhow::Result;
use poll_promise::Promise;
use quic_session::web_transport::{RecvStream, SendStream, Session};

struct Connections {
    sess: Session,
    recv: RecvStream,
    send: SendStream,
}

pub struct TemplateApp {
    conn: Promise<Result<Connections>>,
    received: Vec<String>,
}



impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = cc.egui_ctx.clone();

        let conn = Promise::spawn_async(async move {
            let url = url::Url::parse("https://127.0.0.1:9090/")?;
            let mut sess = quic_session::client_session(&url).await?;

            let (send, recv) = sess.open_bi().await?;

            ctx.request_repaint();

            Ok(Connections {
                send,
                recv,
                sess,
            })
        });

        Self {
            received: vec![],
            conn,
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.conn.ready_mut() {
                Some(Ok(conn)) => {
                    ui.label(format!("Connection open"));
                },
                Some(Err(e)) => {
                    ui.label(format!("Error: {e:?}"));
                },
                None => {
                    ui.label(format!("Connecting"));
                }
            }
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
