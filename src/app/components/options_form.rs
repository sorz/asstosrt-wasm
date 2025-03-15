use leptos::prelude::*;
use leptos_i18n::{t, t_string};
use reactive_stores::Store;

use crate::{ChineseConvertion, LineStrip, Options, OptionsStoreFields, app::i18n::use_i18n};

#[component]
pub(crate) fn OptionsForm(options: Store<Options>) -> impl IntoView {
    let i18n = use_i18n();
    let offset_string = RwSignal::new("".to_string());

    view! {
        <datalist id="charsets">
            <option label=move || t_string!(i18n, opt_charset_utf8) value="UTF-8" />
            <option label=move || t_string!(i18n, opt_charset_gb) value="GB18030" />
            <option label=move || t_string!(i18n, opt_charset_big5) value="Big5" />
            <option label=move || t_string!(i18n, opt_charset_jis) value="Shift_JIS" />
        </datalist>
        <label for="in-charset">{t!(i18n, opt_ass_encoding_label)}</label>
        <input
            type="text"
            id="in-charset"
            list="charsets"
            placeholder=move || t_string!(i18n, opt_ass_encoding_placeholder)
            prop:value=move || options.ass_charset().read().clone()
            on:change:target=move |ev| {
                *options.ass_charset().write() = ev.target().value().trim().to_string();
            }
        />

        <label for="out-charset">{t!(i18n, opt_srt_encoding_label)}</label>
        <input
            type="text"
            id="out-charset"
            list="charsets"
            placeholder=move || t_string!(i18n, opt_srt_encoding_placeholder)
            bind:value=options.srt_charset()
        />

        <label for="conv-dict">{t!(i18n, opt_chinese_convert_label)}</label>
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
            <option value=ChineseConvertion::Keep>{t!(i18n, opt_chinese_convert_no)}</option>
            <option value=ChineseConvertion::ToSimplified>
                {t!(i18n, opt_chinese_convert_t2s)}
            </option>
            <option value=ChineseConvertion::ToTraditional>
                {t!(i18n, opt_chinese_convert_st2)}
            </option>
        </select>

        <label for="lines">{t!(i18n, opt_lines_label)}</label>
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
            <option value=LineStrip::KeepAll>{t!(i18n, opt_lines_all)}</option>
            <option value=LineStrip::KeepFirst>{t!(i18n, opt_lines_first)}</option>
            <option value=LineStrip::KeepLast>{t!(i18n, opt_lines_last)}</option>
        </select>

        <label for="offset">{t!(i18n, opt_offset_label)}</label>
        <input
            type="text"
            id="offset"
            placeholder="0.00"
            pattern=r"-?\d*\.?\d{0,3}"
            bind:value=offset_string
            on:blur=move |_| {
                let offset: f32 = offset_string.get().parse().unwrap_or_default();
                options.offset_millis().set((offset / 1000.0).round() as i32);
            }
        />

        <label class="checkbox">
            <input type="checkbox" id="no-zip" bind:value=options.no_zip() />
            {t!(i18n, opt_no_zip_label)}
        </label>
    }
}
