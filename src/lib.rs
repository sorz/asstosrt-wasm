use leptos::prelude::*;
use reactive_stores::Store;
use serde::{Deserialize, Serialize};
use strum::{EnumString, IntoStaticStr};

pub mod app;
pub mod worker;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, EnumString, IntoStaticStr,
)]
pub enum ChineseConvertion {
    #[default]
    Keep,
    ToSimplified,
    ToTraditional,
}

impl IntoAttributeValue for ChineseConvertion {
    type Output = &'static str;

    fn into_attribute_value(self) -> Self::Output {
        self.into()
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, EnumString, IntoStaticStr,
)]
pub enum LineStrip {
    #[default]
    KeepAll,
    KeepFirst,
    KeepLast,
}

impl IntoAttributeValue for LineStrip {
    type Output = &'static str;

    fn into_attribute_value(self) -> Self::Output {
        self.into()
    }
}

#[derive(Debug, Clone, Store, Default, Serialize, Deserialize)]
pub struct Options {
    pub ass_charset: String,
    pub srt_charset: String,
    pub ignore_charset_error: bool,
    pub chinese_convertion: ChineseConvertion,
    pub line_strip: LineStrip,
    pub offset_secs: f32,
    pub no_zip: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerMessage {
    Ready,
}
