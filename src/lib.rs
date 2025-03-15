#![feature(closure_lifetime_binder)]
use std::borrow::Cow;

use leptos::prelude::*;
use reactive_stores::Store;
use serde::{Deserialize, Serialize};
use strum::{EnumString, IntoStaticStr};
use web_sys::File;
use worker::{ConvertError, ConvertMeta};

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

impl ChineseConvertion {
    pub fn dict_name(&self) -> Option<&'static str> {
        match self {
            Self::Keep => None,
            Self::ToSimplified => Some("t2s.txt"),
            Self::ToTraditional => Some("s2t.txt"),
        }
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

impl LineStrip {
    fn strip<'a>(&self, text: Cow<'a, str>) -> Cow<'a, str> {
        match (self, text) {
            (Self::KeepAll, text) => text,
            (Self::KeepFirst, Cow::Borrowed(text)) => {
                Cow::Borrowed(text.lines().next().unwrap_or(text))
            }
            (Self::KeepFirst, Cow::Owned(text)) => {
                Cow::Owned(text.lines().next().map(ToString::to_string).unwrap_or(text))
            }
            (Self::KeepLast, Cow::Borrowed(text)) => {
                Cow::Borrowed(text.lines().last().unwrap_or(text))
            }
            (Self::KeepLast, Cow::Owned(text)) => {
                Cow::Owned(text.lines().last().map(ToString::to_string).unwrap_or(text))
            }
        }
    }
}

#[derive(Debug, Clone, Store, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Options {
    pub ass_charset: String,
    pub srt_charset: String,
    pub chinese_convertion: ChineseConvertion,
    pub line_strip: LineStrip,
    pub offset_millis: i32,
    pub no_zip: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WorkerMessage {
    WorkerReady,
    TaskDone(Result<TaskResult, ConvertError>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    file_url: String,
    meta: ConvertMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    pub options: Options,
    pub files: Vec<FileWrap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWrap(#[serde(with = "serde_wasm_bindgen::preserve")] pub File);

#[test]
fn test_line_strip() {
    let empty = Cow::Borrowed("");
    let one_line = Cow::Borrowed("1");
    let two_lines = Cow::Borrowed("1\n2");

    assert!(LineStrip::KeepAll.strip(empty.clone()).is_empty());
    assert!(LineStrip::KeepFirst.strip(empty.clone()).is_empty());
    assert!(LineStrip::KeepLast.strip(empty.clone()).is_empty());

    assert_eq!(LineStrip::KeepAll.strip(one_line.clone()), "1");
    assert_eq!(LineStrip::KeepFirst.strip(one_line.clone()), "1");
    assert_eq!(LineStrip::KeepLast.strip(one_line.clone()), "1");

    assert_eq!(LineStrip::KeepAll.strip(two_lines.clone()), "1\n2");
    assert_eq!(LineStrip::KeepFirst.strip(two_lines.clone()), "1");
    assert_eq!(LineStrip::KeepLast.strip(two_lines.clone()), "2");

    assert!(matches!(
        LineStrip::KeepAll.strip(Cow::Owned(String::new())),
        Cow::Owned(_)
    ));
}
