use web_sys::File;
use yew::prelude::*;

pub struct FormComponent {
    files_on_change: Callback<ChangeData>,
    props: FormProps,
}

pub enum FormMsg {
    Files(Vec<File>),
}

#[derive(Properties, Clone, PartialEq)]
pub struct FormProps {
    pub on_files: Callback<Vec<File>>,
}

impl Component for FormComponent {
    type Message = FormMsg;
    type Properties = FormProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        FormComponent {
            props,
            files_on_change: link.callback(|event: ChangeData| {
                let mut files = Vec::new();
                if let ChangeData::Files(list) = event {
                    for i in 0..list.length() {
                        if let Some(file) = list.get(i) {
                            files.push(file);
                        }
                    }
                }
                Self::Message::Files(files)
            }),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        if let Self::Message::Files(files) = msg {
            if !files.is_empty() {
                self.props.on_files.emit(files);
            }
        }
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        false
    }

    fn view(&self) -> Html {
        html! {
            <form>
                <h2>{ "Settings" }</h2>
                <details>
                    <summary>{ "Show encodings, lines, zip, etc." }</summary>
                    <p>
                        <label for="in-charset">{ "ASS Encoding" }</label>
                        <input type="text" id="in-charset" list="charsets" placeholder="auto detect" />
                    </p>
                    <p>
                        <label for="out-charset">{ "SRT Encoding" }</label>
                        <input type="text" id="out-charset" list="charsets" placeholder="no change" />
                    </p>
                    <p>
                        <label for="ignore-codec-err">{ "Ignore encoding errors" }</label>
                        <input type="checkbox" id="ignore-codec-err" />
                    </p>
                    <p>
                        <label for="conv-dict">{ "Chinese convert" }</label>
                        <select id="conv-dict">
                            <option value="" selected=true>{ "disabled" }</option>
                            <option value="t2s.txt">{ "to Simplified" }</option>
                            <option value="s2t.txt">{ "to Traditional" }</option>
                        </select>
                    </p>
                    <p>
                        <label for="lines">{ "Lines" }</label>
                        <select id="lines">
                            <option value="All" selected=true>{ "keep all" }</option>
                            <option value="First">{ "first line only" }</option>
                            <option value="Last">{ "last line only" }</option>
                        </select>
                    </p>
                    <p>
                        <label for="offset">{ "Offset seconds" }</label>
                        <input id="offset" type="number" placeholder="0.0" step="0.1" />
                    </p>
                    <p>
                        <label for="no-zip">{ "Don't archive files into single zip" }</label>
                        <input type="checkbox" id="no-zip" />
                    </p>
                </details>

                <h2>{ "Drop ASS/SSA Files" }</h2>
                <p>
                    { "Drag & drop to here; or " }
                    <input type="file" id="files" multiple=true onchange=&self.files_on_change />
                </p>
                <p>
                    { "Select/drop multiple files at once for bulk processing." }
                </p>

                <datalist id="charsets">
                <option label="Unicode (UTF-8)" value="utf-8" />
                <option label="Unicode (UTF-16)" value="utf-16" />
                <option label="Simplified Chinese (GB18030)" value="gb18030" />
                <option label="Traditional Chinese (Big5)" value="big5" />
                <option label="Japanese (Shift-JIS)" value="shift-jis" />
                </datalist>
            </form>
        }
    }
}
