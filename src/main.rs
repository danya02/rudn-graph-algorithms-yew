use yew::prelude::*;
use yew_router::prelude::*;
mod views;
mod graph_settings_bus;
use crate::views::graph_view::GraphView;
use crate::views::navbar::Navbar;
use crate::views::graph_settings::GraphSettingsModule;
use crate::views::not_found::NotFoundView;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(Home)]
fn home() -> Html {
    html! {
        <>
            <Navbar/>

            <div class={"container"}>

                <h1> {"Hello World!"} </h1>
                <div>
                    <GraphView class="mb-3" />
                    <GraphSettingsModule />
                </div>

            </div>
        </>
    }
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => html!{
            <Home />
        },
        Route::NotFound => html! { 
            <NotFoundView />
        },
    }
}

#[function_component(App)]
pub fn app() -> Html {
        html! {
            <BrowserRouter>
                <Switch<Route> render={Switch::render(switch)} />
            </BrowserRouter>
        }
    }

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
    
}
