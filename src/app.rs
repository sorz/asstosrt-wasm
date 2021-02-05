use yew::prelude::*;

pub struct App {}

pub enum Msg {}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        App {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
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
            </>
        }
    }
}
