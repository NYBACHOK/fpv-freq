// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cell::RefCell, collections::HashSet, rc::Rc};

use anyhow::Context;
use slint::{Model, ModelRc, SharedString, ToSharedString, VecModel};

use crate::resolver::FpvConflictResolver;

slint::include_modules!();

mod logger;
mod resolver;

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
        }
    }
}

const RAW_JSON: &str = include_str!("../../frequency.json");

fn main() -> anyhow::Result<()> {
    logger::setup_logger();

    // Standard FPV safe channel separation is 40 MHz (covers 1 MHz to 37 MHz deltas)
    let resolver = FpvConflictResolver::new(40);
    let active_channel_ids = Rc::new(RefCell::new(HashSet::<String>::new()));

    let initial_items = serde_json::from_str::<Vec<ItemModel>>(RAW_JSON)
        .context("deserialize list of frequency")?
        .into_iter()
        .map(Item::from)
        .collect::<Vec<_>>();

    // Flatten all channels into a lookup list
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

        // Toggle active status
        if active_set.contains(&id) {
            active_set.remove(&id);
        } else {
            active_set.insert(id);
        }

        // Collect details for active channels
        let active_channels: Vec<(String, i32)> = all_channels
            .iter()
            .filter(|(ch_id, _)| active_set.contains(ch_id))
            .cloned()
            .collect();

        // Calculate incompatible channel IDs across ALL channels
        let incompatible_ids = resolver.find_incompatible_ids(&all_channels, &active_channels);

        // Update Slint VecModel rows dynamically
        for row in 0..model_clone.row_count() {
            if let Some(mut item) = model_clone.row_data(row) {
                let item_id = item.id.to_string();

                // Highlight channel if it is incompatible with active selection
                item.conflict = incompatible_ids.contains(&item_id);
                model_clone.set_row_data(row, item);
            }
        }

        // Update top status message
        if let Some(ui_app) = ui_weak.upgrade() {
            let status = format!(
                "Active Pilots: {} | Incompatible Channels: {}",
                active_set.len(),
                incompatible_ids.len()
            );
            ui_app.set_statusText(SharedString::from(status));
        }
    });

    ui.run()?;

    Ok(())
}
