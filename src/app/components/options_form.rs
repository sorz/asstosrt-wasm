use leptos::prelude::*;
use reactive_stores::Store;

use crate::{ChineseConvertion, LineStrip, Options, OptionsStoreFields};

#[component]
pub(crate) fn OptionsForm(options: Store<Options>) -> impl IntoView {
    let offset_string = RwSignal::new("".to_string());

    view! {
        <datalist id="charsets">
            <option label="Unicode (UTF-8)" value="utf-8" />
            <option label="Unicode (UTF-16)" value="utf-16" />
            <option label="Simplified Chinese (GB18030)" value="gb18030" />
            <option label="Traditional Chinese (Big5)" value="big5" />
            <option label="Japanese (Shift-JIS)" value="shift-jis" />
        </datalist>
        <p>
            <label>
                ASS Encoding
                <input
                    type="text"
                    id="in-charset"
                    list="charsets"
                    placeholder="Auto detect"
                    prop:value=move || options.ass_charset().read().clone()
                    on:change:target=move |ev| {
                        *options.ass_charset().write() = ev.target().value().trim().to_string();
                    }
                />
            </label>
        </p>
        <p>
            <label>
                SRT Encoding
                <input
                    type="text"
                    id="out-charset"
                    list="charsets"
                    placeholder="Same as ASS file"
                    bind:value=options.srt_charset()
                />
            </label>
        </p>
        <p>
            <label>
                Ignore encoding errors
                <input
                    type="checkbox"
                    id="ignore-codec-err"
                    bind:value=options.ignore_charset_error()
                />
            </label>
        </p>
        <p>
            <label>
                Chinese convert
                <select
                    id="conv-dict"
                    prop:value=move || {
                        let value: &str = options.chinese_convertion().get().into();
                        value
                    }
                    on:change:target=move |ev| {
                        let value = ev.target().value().parse().unwrap();
                        options.chinese_convertion().set(value);
                    }
                >
                    <option value=ChineseConvertion::Keep>Disabled</option>
                    <option value=ChineseConvertion::ToSimplified>To Simplified</option>
                    <option value=ChineseConvertion::ToTraditional>To Traditional</option>
                </select>
            </label>
        </p>
        <p>
            <label>
                Lines
                <select
                    id="lines"
                    prop:value=move || {
                        let value: &str = options.line_strip().get().into();
                        value
                    }
                    on:change:target=move |ev| {
                        let value = ev.target().value().parse().unwrap();
                        options.line_strip().set(value);
                    }
                >
                    <option value=LineStrip::KeepAll>Keep all</option>
                    <option value=LineStrip::KeepFirst>First line only</option>
                    <option value=LineStrip::KeepLast>Last line only</option>
                </select>
            </label>
        </p>
        <p>
            <label>
                Offset seconds
                <input
                    type="text"
                    id="offset"
                    placeholder="0.00"
                    pattern=r"-?\d*\.?\d{0,3}"
                    bind:value=offset_string
                    on:blur=move |_| {
                        let offset = offset_string.get().parse().unwrap_or_default();
                        options.offset_secs().set(offset);
                    }
                />
            </label>
        </p>
        <p>
            <label>
                Do not zip files <input type="checkbox" id="no-zip" bind:value=options.no_zip() />
            </label>
        </p>
    }
}
