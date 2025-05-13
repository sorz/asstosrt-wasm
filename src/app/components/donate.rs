use std::time::Duration;

use leptos::prelude::*;
use leptos_i18n::t;
use serde::{Deserialize, Serialize};
use web_sys::MouseEvent;
use web_time::SystemTime;

use crate::{
    app::i18n::use_i18n,
    storage::{self, Key},
};

const DONATE_LINK_STRIPE: &str = "https://donate.stripe.com/bIY4hlbfi5K80fe3cc";

fn open_stripe_page(ev: MouseEvent) {
    if window()
        .open_with_url_and_target_and_features(DONATE_LINK_STRIPE, "stripe", "popup,width=480")
        .is_ok()
    {
        ev.prevent_default();
    }
}

#[component]
pub(crate) fn DonateLink() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <a href=DONATE_LINK_STRIPE on:click=open_stripe_page>
            {t!(i18n, footer_donate)}
        </a>
    }
}

#[component]
pub(crate) fn DonateBanner() -> impl IntoView {
    let i18n = use_i18n();
    let (show, set_show) = signal(HideDonateUntil::load_from_storage().show());
    let (removing, set_removing) = signal(false);

    let on_donate = move |ev: MouseEvent| {
        open_stripe_page(ev);
        set_removing(true);
        HideDonateUntil::next_month().save_to_storage();
    };
    let on_hide = move |_: MouseEvent| {
        set_removing(true);
        HideDonateUntil::next_week().save_to_storage();
    };
    let banner = move || {
        view! {
            <li
                class="donate"
                class:removing=move || removing()
                on:animationend=move |ev| {
                    if ev.animation_name() == "fade-out" {
                        set_show(false);
                    }
                }
            >
                <p class="title">{t!(i18n, donate_banner_title)}</p>
                <p>{t!(i18n, donate_banner_content)}</p>
                <p class="buttons">
                    <a class="btn" href=DONATE_LINK_STRIPE on:click=on_donate>
                        {t!(i18n, footer_donate)}
                    </a>
                    <button on:click=on_hide>{t!(i18n, donate_banner_hide)}</button>
                </p>
            </li>
        }
    };
    move || Some(banner).take_if(|_| show())
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
struct HideDonateUntil(Option<SystemTime>);

impl HideDonateUntil {
    fn next_month() -> Self {
        Self(Some(
            SystemTime::now() + Duration::from_secs(3600 * 24 * 30),
        ))
    }

    fn next_week() -> Self {
        Self(Some(SystemTime::now() + Duration::from_secs(3600 * 24 * 7)))
    }

    fn show(&self) -> bool {
        // true if elapsed
        self.0.map(|t| t.elapsed().is_ok()).unwrap_or(true)
    }

    fn load_from_storage() -> Self {
        storage::get_from_json(Key::HideDonateUntil).unwrap_or_default()
    }

    fn save_to_storage(&self) {
        let _ = storage::set(
            Key::HideDonateUntil,
            serde_json::to_string(self).expect("failed to serialize"),
        );
    }
}
