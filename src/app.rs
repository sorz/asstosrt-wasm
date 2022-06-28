use crate::drag::DragDropComponent;
use crate::form::FormComponent;

use web_sys::File;
use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    let on_files = Callback::from(|files: Vec<File>| {
        log::debug!("got {} files", files.len());
    });
    html! {
        <>
            <h1>
                <abbr title="Advanced SubStation Alpha">{ "ASS" }</abbr>
                { " / " }
                <abbr title="SubStation Alpha">{ "SSA" }</abbr>
                { " to " }
                <abbr title="SubRip">{ "SRT" }</abbr>
                { " Subtitles Converter" }
            </h1>
            <p>
                { "Only support for those newer browsers. " }
                { "You can " }
                <a href="https://lab.sorz.org/tools/asstosrt/">
                    { "try this" }
                </a>
                { " if you have problem with it. "}
            </p>

            <DragDropComponent on_files={ &on_files }>
                <FormComponent on_files={ &on_files } />
            </DragDropComponent>

            <footer>
                <p>{ "Your file would NOT be uploaded to anywhere." }</p>
                <p>{ "Powered by " }
                    <a href="https://www.rust-lang.org/" title="The Rust Programming Language">
                        { "Rust" }
                    </a>
                    { " and a set of lovely open-source projects." }
                </p>
                <p>
                    { "Source code is avaiable on " }
                    <a href="https://github.com/sorz/asstosrt-wasm">{ "GitHub" }</a>
                    { "."}
                </p>
            </footer>
        </>
    }
}
