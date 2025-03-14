use leptos::prelude::*;
use web_sys::File;

#[component]
pub(crate) fn FileInput(#[prop(into)] on_files: UnsyncCallback<(Vec<File>,), ()>) -> impl IntoView {
    view! {
        <div
            class="drop-zone"
            on:dragover=|ev| ev.prevent_default()
            on:drop=move |ev| {
                ev.prevent_default();
                if let Some(data) = ev.data_transfer() {
                    let items = data.items();
                    let files: Result<Vec<_>, _> = (0..items.length())
                        .filter_map(|idx| items.get(idx))
                        .filter(|item| item.kind() == "file")
                        .filter_map(|item| item.get_as_file().transpose())
                        .collect();
                    match files {
                        Ok(files) => on_files.run((files,)),
                        Err(err) => log::warn!("failed to get input files: {:?}", err),
                    }
                }
            }
        >
            <p>Drag & drop your files here</p>
            <p>
                <input
                    type="file"
                    id="files"
                    accept=".ass, .ssa"
                    multiple
                    on:change:target=move |ev| {
                        if let Some(files) = ev.target().files() {
                            let files: Vec<_> = (0..files.length())
                                .filter_map(|idx| files.get(idx))
                                .collect();
                            if !files.is_empty() {
                                on_files.run((files,));
                            }
                        }
                        ev.target().set_value("");
                    }
                />
            </p>
            <p>Select/drop multiple files at once for bulk processing</p>
        </div>
    }
}
