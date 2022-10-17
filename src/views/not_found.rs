use yew::prelude::*;

pub struct NotFoundView {
}

impl Component for NotFoundView {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="d-flex h-100 text-center text-bg-dark d-flex w-100 p-3 mx-auto flex-column">
                <div class="cover-container d-flex w-100 h-100 p-3 mx-auto flex-column">
                    <header class="mb-auto"></header>
                    <main class="px-3">
                        <h1>{"404"}</h1>
                        <p class="lead">{"No such page"}</p>
                        <p>{"This URL is not a valid page on this app."}</p>
                        <p>{"Perhaps you'd like to return to the"}</p>
                        <p><a class="btn btn-lg btn-secondary fw-bold border-white bg-white" style="color: #333;" href="/">{ "home page" }</a></p>
                    </main>
                    <footer class="mt-auto text-white-50"></footer>
                </div>
            </div>
        }
    }
}