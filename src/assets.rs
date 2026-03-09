use rust_embed::Embed;

#[derive(Embed)]
#[folder = "assets/"]
#[include = "public/**/*"]
#[include = "templates/**/*"]
#[exclude = "*.DS_Store"]
pub struct Assets;
