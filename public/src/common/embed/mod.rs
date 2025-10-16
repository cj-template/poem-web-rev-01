use poem::endpoint::{EmbeddedFileEndpoint, EmbeddedFilesEndpoint};
use rust_embed::Embed;
use shared::embed::EmbedAsString;
use std::collections::HashMap;

pub const EMBED_PATH: &'static str = "/assets/";

#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/asset/embed/"]
#[cfg_attr(not(debug_assertions), include = "**.min.{css,js}")]
#[cfg_attr(not(debug_assertions), include = "favicon/**")]
pub struct Asset;

pub type AssetFileEndPoint = EmbeddedFileEndpoint<Asset>;
pub type AssetFilesEndPoint = EmbeddedFilesEndpoint<Asset>;

#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/asset/embed_hidden/"]
#[exclude = "assets/**"]
pub struct AssetHidden;

#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/asset/embed_locale/"]
pub struct AssetLocale;

impl AssetLocale {
    pub fn locale_map() -> HashMap<String, String> {
        let mut map = HashMap::new();
        for value in Self::iter() {
            let locale = value.split("/").next().unwrap_or_default();
            let mut str = map
                .get(locale)
                .map(|v: &String| v.to_string())
                .unwrap_or_default();
            let file = Self::get(String::from(value.clone()).as_str());
            str.push_str(&file.as_string());
            str.push('\n');
            map.insert(locale.to_string(), str);
        }
        map
    }
}
