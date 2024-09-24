use std::{future::Future, marker::PhantomData, sync::{Arc, Mutex}};

use egui::Ui;
use poll_promise::Promise;

pub struct SimpleSpawner<T> {
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
