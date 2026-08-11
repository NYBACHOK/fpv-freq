mod logger;
mod resolver;

use std::{cell::RefCell, collections::HashSet, rc::Rc};

use anyhow::Context;
use slint::{Model, ModelRc, SharedString, ToSharedString, VecModel};

use crate::resolver::FpvConflictResolver;

slint::include_modules!();

#[derive(Debug, serde::Deserialize)]
pub struct ItemModel {
    id: String,
    channel: String,
    frequency: u32,
}

impl From<ItemModel> for Item {
    fn from(
        ItemModel {
            id,
            channel,
            frequency,
        }: ItemModel,
    ) -> Self {
        Self {
            frequency: frequency as i32,
            channel: channel.to_shared_string(),
            id: id.to_shared_string(),
            conflict: false,
            selected: false,
        }
    }
}

const RAW_JSON: &str = include_str!("../../frequency.json");

pub fn run() -> anyhow::Result<()> {
    logger::setup_logger();

    let resolver = FpvConflictResolver::new(40); // 40 MHz safe separation
    let active_channel_ids = Rc::new(RefCell::new(HashSet::<String>::new()));

    let initial_items = serde_json::from_str::<Vec<ItemModel>>(RAW_JSON)
        .context("deserialize list of frequency")?
        .into_iter()
        .map(Item::from)
        .collect::<Vec<_>>();

    let all_channels: Vec<(String, i32)> = initial_items
        .iter()
        .map(|Item { id, frequency, .. }| (id.to_string(), *frequency))
        .collect();

    let ui = AppWindow::new()?;

    let model = Rc::new(VecModel::from(initial_items.clone()));
    ui.set_model(ModelRc::from(model.clone()));

    let active_ids_clone = active_channel_ids.clone();
    let model_clone = model.clone();
    let ui_weak = ui.as_weak();

    ui.on_item_clicked(move |clicked_item| {
        let id = clicked_item.id.to_string();
        let mut active_set = active_ids_clone.borrow_mut();

        // RUST SAFEGUARD: Ignore clicks on conflicting items if not already selected
        if clicked_item.conflict && !active_set.contains(&id) {
            if let Some(ui_app) = ui_weak.upgrade() {
                ui_app.set_statusText(SharedString::from(format!(
                    "Cannot select {} - frequency conflict detected!",
                    clicked_item.channel
                )));
            }
            return;
        }

        // Toggle selection
        if active_set.contains(&id) {
            active_set.remove(&id);
        } else {
            active_set.insert(id);
        }

        // Active channels snapshot
        let active_channels: Vec<(String, i32)> = all_channels
            .iter()
            .filter(|(ch_id, _)| active_set.contains(ch_id))
            .cloned()
            .collect();

        // Calculate incompatible channel IDs across all available channels
        let incompatible_ids = resolver.find_incompatible_ids(&all_channels, &active_channels);

        // Update Slint VecModel rows
        for row in 0..model_clone.row_count() {
            if let Some(mut item) = model_clone.row_data(row) {
                let item_id = item.id.to_string();

                let is_selected = active_set.contains(&item_id);
                let is_incompatible = incompatible_ids.contains(&item_id);

                item.selected = is_selected;
                // An active item is never marked in conflict with itself
                item.conflict = !is_selected && is_incompatible;

                model_clone.set_row_data(row, item);
            }
        }

        // Status update
        if let Some(ui_app) = ui_weak.upgrade() {
            let status = format!(
                "Active Pilots: {} | Blocked Channels: {}",
                active_set.len(),
                incompatible_ids.len()
            );
            ui_app.set_statusText(SharedString::from(status));
        }
    });

    ui.run()?;

    Ok(())
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();
    run().expect("failed to run app");
}
