use crate::common::embed::AssetLocale;
use poem::error::I18NError;
use poem::i18n::I18NResources;

fn list_of_alias() -> Vec<(String, String)> {
    vec![("en-US".to_string(), "en-GB".to_string())]
}

pub fn build_locale_resources() -> Result<I18NResources, I18NError> {
    let locale_map = AssetLocale::locale_map();
    let mut resources = I18NResources::builder();
    for (locale, context) in &locale_map {
        resources = resources.add_ftl(locale, context)
    }
    for (alias, locale) in list_of_alias() {
        if let Some(locale_data) = locale_map.get(locale.as_str()) {
            resources = resources.add_ftl(alias, locale_data);
        }
    }
    resources.build()
}
