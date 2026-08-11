// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Context;
use slint::{ModelRc, ToSharedString, VecModel};

slint::include_modules!();

mod logger;

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
        }
    }
}

const RAW_JSON: &str = include_str!("../../frequency.json");

fn main() -> anyhow::Result<()> {
    logger::setup_logger();

    let model = serde_json::from_str::<Vec<ItemModel>>(RAW_JSON)
        .context("deserialize list of frequency")?
        .into_iter()
        .map(Item::from)
        .collect::<VecModel<_>>();

    let ui = AppWindow::new()?;

    ui.set_model(ModelRc::new(model));

    ui.run()?;

    Ok(())
}
