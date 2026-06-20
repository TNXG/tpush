use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../app/panel/dist/"]
pub struct PanelAssets;
