use web_sys::File;
use yew::prelude::*;

pub struct DragDropComponent {
    on_drag_over: Callback<DragEvent>,
    on_drag_end: Callback<DragEvent>,
    on_drop: Callback<DragEvent>,
    props: DragDropProps,
}

pub enum DragDropMsg {
    Drop(Vec<File>),
    DragOver,
    DragEnd,
}

#[derive(Properties, Clone, PartialEq)]
pub struct DragDropProps {
    pub on_drop: Callback<Vec<File>>,
    pub children: Children,
}

impl Component for DragDropComponent {
    type Message = DragDropMsg;
    type Properties = DragDropProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        DragDropComponent {
            props,
            on_drag_over: link.callback(|event: DragEvent| {
                event.prevent_default();
                Self::Message::DragOver
            }),
            on_drag_end: link.callback(|event: DragEvent| {
                if let Some(data) = event.data_transfer() {
                    let _ = data.clear_data();
                }
                Self::Message::DragEnd
            }),
            on_drop: link.callback(|event: DragEvent| {
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
                Self::Message::Drop(files)
            }),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        if let Self::Message::Drop(files) = msg {
            if !files.is_empty() {
                self.props.on_drop.emit(files);
            }
        }
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let rerender = props.children != self.props.children;
        self.props = props;
        rerender
    }

    fn view(&self) -> Html {
        html! {
            <div ondrop=&self.on_drop ondragover=&self.on_drag_over ondragend=&self.on_drag_end>
                { for self.props.children.iter() }
            </div>
        }
    }
}
