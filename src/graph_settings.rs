use serde::{Deserialize, Serialize};
use crate::graph_settings_bus::{EventBus, Request};
use yew::prelude::*;
use yew_agent::{Dispatched, Dispatcher};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GraphSettingsMessage {
    Clicked,
}

pub struct Producer {
    event_bus: Dispatcher<EventBus>,
}

impl Component for Producer {
    type Message = GraphSettingsMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            event_bus: EventBus::dispatcher(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.event_bus
        .send(Request::EventBusMsg(msg));
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <button onclick={ctx.link().callback(|_| GraphSettingsMessage::Clicked)}>
                { "Do relayout" }
            </button>
        }
    }
}