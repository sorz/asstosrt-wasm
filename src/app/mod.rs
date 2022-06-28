mod drag;
mod form;

use self::drag::DragDropComponent;
use self::form::FormComponent;
use js_sys::Array;
use web_sys::{window, Blob, BlobPropertyBag, File, Url, Worker};
use yew::prelude::*;

fn load_worker() -> Worker {
    let origin = window()
        .expect("missing `window`")
        .location()
        .origin()
        .expect("missing `location.origin`");

    let script = Array::new();
    script.push(
        &format!(
            r#"
            importScripts("{origin}/worker.js");
            wasm_bindgen("{origin}/worker_bg.wasm");
            "#
        )
        .into(),
    );

    let mut blob_props = BlobPropertyBag::new();
    blob_props.type_("text/javascript");
    let blob = Blob::new_with_str_sequence_and_options(&script, &blob_props)
        .expect("failed to create blob");

    let url = Url::create_object_url_with_blob(&blob).expect("failed to create url from blob");
    Worker::new(&url).expect("failed to spawn worker")
}

#[function_component(App)]
pub fn app() -> Html {
    use_effect(|| {
        let worker = load_worker();
        // TODO: add event handlers
        move || worker.terminate()
    });

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
