use crate::context::{Context, ContextError, FromContext};
use chrono::{DateTime, Local};
use cjtoolkit_structured_validator::common::locale::{LocaleData, LocaleValue, ValidateErrorStore};
use cjtoolkit_structured_validator::common::validation_collector::AsValidateErrorStore;
use error_stack::{Report, ResultExt};
use poem::FromRequest;
use poem::i18n::{I18NArgs, Locale};
use std::sync::Arc;

impl FromContext for Locale {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        let req = ctx.req_result()?;
        Locale::from_request_without_body(req)
            .await
            .change_context(ContextError::RequestError)
    }
}

pub trait LocaleExtForData {
    fn get_translation(&self, locale: &Locale, original: String) -> String;
}

impl LocaleExtForData for LocaleData {
    fn get_translation(&self, locale: &Locale, original: String) -> String {
        if !self.args.is_empty() {
            let mut values = I18NArgs::default();
            for (key, value) in self.args.iter() {
                match value {
                    LocaleValue::String(string) => {
                        values = values.set::<String, String>(key.clone(), string.clone());
                    }
                    LocaleValue::Uint(unit) => {
                        values = values.set::<String, usize>(key.clone(), *unit);
                    }
                    LocaleValue::Int(int) => {
                        values = values.set::<String, isize>(key.clone(), *int);
                    }
                    LocaleValue::Float(float) => {
                        values = values.set::<String, f64>(key.clone(), *float);
                    }
                }
            }
            locale
                .text_with_args(self.name.clone(), values)
                .unwrap_or(original)
        } else {
            locale.text(self.name.clone()).unwrap_or(original)
        }
    }
}

pub trait LocaleExtForStore {
    fn as_translated_message(&self, locale: &Locale) -> Arc<[String]>;
}

impl LocaleExtForStore for ValidateErrorStore {
    fn as_translated_message(&self, locale: &Locale) -> Arc<[String]> {
        self.0
            .iter()
            .map(|e| e.1.get_locale_data().get_translation(locale, e.0.clone()))
            .collect()
    }
}

pub trait LocaleExtForResult: AsValidateErrorStore {
    fn as_translated_message(&self, locale: &Locale) -> Arc<[String]>;

    fn as_original_message(&self) -> Arc<[String]>;
}

impl<T, E> LocaleExtForResult for Result<T, E>
where
    for<'a> &'a E: Into<ValidateErrorStore>,
{
    fn as_translated_message(&self, locale: &Locale) -> Arc<[String]> {
        self.as_validate_store().as_translated_message(locale)
    }

    fn as_original_message(&self) -> Arc<[String]> {
        self.as_validate_store().as_original_message()
    }
}

pub trait LocaleExt {
    fn text_with_default(&self, name: &str, default: &str) -> String;
    fn text_with_default_args(&self, name: &str, default: &str, args: I18NArgs) -> String;
}

impl LocaleExt for Locale {
    fn text_with_default(&self, name: &str, default: &str) -> String {
        self.text(name).unwrap_or(default.to_string())
    }

    fn text_with_default_args(&self, name: &str, default: &str, args: I18NArgs) -> String {
        self.text_with_args(name, args)
            .unwrap_or(default.to_string())
    }
}

pub trait LocaleDateTimeExt {
    fn date_time_format(&self, date_time: impl Into<DateTime<Local>>) -> String;
}

impl LocaleDateTimeExt for Locale {
    fn date_time_format(&self, date_time: impl Into<DateTime<Local>>) -> String {
        let date_time = date_time.into();
        self.text_with_args("top-date-time", (("date", date_time.to_rfc3339()),))
            .unwrap_or(date_time.to_rfc3339())
    }
}
