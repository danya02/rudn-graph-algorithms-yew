use yew::prelude::*;

pub struct Columns {
}

#[derive(Clone, PartialEq, Properties)]
pub struct ColumnsProps {
        #[prop_or_default]
        pub children: Children,
}

impl Component for Columns {
    type Message = ();
    type Properties = ColumnsProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="row">
                { for ctx.props().children.iter().map(|child| html!{
                    <div class="col">
                        { child.clone() }
                    </div>
                }) }
            </div>
        }
    }
}