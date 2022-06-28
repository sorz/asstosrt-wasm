use web_sys::File;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct DragDropProps {
    pub on_files: Callback<Vec<File>>,
    pub children: Children,
}

#[function_component(DragDropComponent)]
pub fn drag_drop_component(props: &DragDropProps) -> Html {
    let on_files = props.on_files.clone();
    let on_drop = move |event: DragEvent| {
        event.prevent_default();
        let mut files = Vec::new();
        if let Some(data) = event.data_transfer() {
            let items = data.items();
            for i in 0..items.length() {
                let item = items.get(i).unwrap();
                if item.kind() == "file" {
                    if let Ok(Some(file)) = item.get_as_file() {
                        files.push(file);
                    }
                }
            }
        }
        on_files.emit(files);
    };
    let on_drag_over = |event: DragEvent| {
        event.prevent_default();
    };
    let on_drag_end = |event: DragEvent| {
        if let Some(data) = event.data_transfer() {
            let _ = data.clear_data();
        }
    };

    html! {
        <div ondrop={ on_drop } ondragover={ on_drag_over } ondragend={ on_drag_end }>
            { for props.children.iter() }
        </div>
    }
}
