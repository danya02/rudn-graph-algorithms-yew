use yew::prelude::*;
use wasm_bindgen::prelude::*;
use yew::virtual_dom::VNode;
use crate::graph_settings_bus::EventBus;
use yew_agent::{Bridge, Bridged};
use crate::views::graph_settings::GraphSettingsMessage;


#[wasm_bindgen()]
extern "C" {
    pub type CytoscapeController;

    #[wasm_bindgen(js_name = "cytoscape")]
    pub fn make_cytoscape_controller(options: &JsValue) -> CytoscapeController;

    type CytoscapeRunnableLayout;

    // Create a layout object from its options
    #[wasm_bindgen(method)]
    fn layout(this: &CytoscapeController, options: &JsValue) -> CytoscapeRunnableLayout; // https://js.cytoscape.org/#cy.layout

    #[wasm_bindgen(method)]
    fn run(this: &CytoscapeRunnableLayout);

    #[wasm_bindgen(method)]
    fn stop(this: &CytoscapeRunnableLayout);


    #[wasm_bindgen(method)]
    fn mount(this: &CytoscapeController, container: &JsValue);

    #[wasm_bindgen(method)]
    fn add(this: &CytoscapeController, elements: &JsValue); // https://js.cytoscape.org/#cy.add
    
    #[wasm_bindgen(method, js_name = "selectionType")]
    fn get_selection_type(this: &CytoscapeController) -> String; // https://js.cytoscape.org/#cy.selectionType

    #[wasm_bindgen(method, js_name = "selectionType")]
    fn set_selection_type(this: &CytoscapeController, selection_type: &str); // https://js.cytoscape.org/#cy.selectionType

    type CytoscapeElements;

    #[wasm_bindgen(method, js_name = "$")]
    fn filter_js(this: &CytoscapeController, selector: &str) -> JsValue; // https://js.cytoscape.org/#cy.$

    #[wasm_bindgen(method, js_name = "$")]
    fn filter(this: &CytoscapeController, selector: &str) -> CytoscapeElements; // https://js.cytoscape.org/#cy.$

    #[wasm_bindgen(method, js_name = "selectify")]
    fn selectify(this: &CytoscapeElements); // https://js.cytoscape.org/#eles.selectify

    #[wasm_bindgen(method, js_name = "unselectify")]
    fn unselectify(this: &CytoscapeElements); // https://js.cytoscape.org/#eles.unselectify

    #[wasm_bindgen(method, js_name = "select")]
    fn select(this: &CytoscapeElements); // https://js.cytoscape.org/#eles.select

    #[wasm_bindgen(method, js_name = "unselect")]
    fn unselect(this: &CytoscapeElements); // https://js.cytoscape.org/#eles.unselect



}

pub struct GraphView {
    controller: CytoscapeController,
    prev_layout: Option<CytoscapeRunnableLayout>,
    _producer: Box<dyn Bridge<EventBus>>,
}

#[derive(Debug)]
pub enum Msg {
    /// Request to relayout the graph
    SettingMessage(GraphSettingsMessage),
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct Properties {
    pub class: String,
}

impl Component for GraphView {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        log::info!("Create cytoscape controller");

        let options = js_sys::Object::new();

        let style = js_sys::JSON::parse(include_str!("style.json")).unwrap();
        js_sys::Reflect::set(&options, &JsValue::from("style"), &JsValue::from(style)).unwrap();
        

        let controller = make_cytoscape_controller(&options);

        controller.add(
            &js_sys::JSON::parse(r#"
            [
                {
                    "data": { "id": "a", "selectable": true }
                },
                { 
                    "data": { "id": "b", "selectable": true }
                },
                { 
                    "data": { "id": "ab", "source": "a", "target": "b" }
                }
            ]
            "#).unwrap()
        );


        controller.set_selection_type("additive");

        controller.filter("node").selectify();
        //controller.filter("node").select();
        


        Self {
            controller: controller,
            prev_layout: None,
            _producer: crate::graph_settings_bus::EventBus::bridge(ctx.link().callback(|setting_msg|{
                log::info!("Received event: {:?}", setting_msg);
                Msg::SettingMessage(setting_msg)
        })),
        }
    }


    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::info!("Cytoscape controller received message: {:?}", msg);

        match msg {
            Msg::SettingMessage(setting_msg) => {
                self.apply_settings_message(setting_msg);
                false
            }
        }
    }


    fn view(&self, _ctx: &Context<Self>) -> Html {
        log::info!("Cytoscape controller is viewed (should only happen once)");

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let container = document.create_element("div").unwrap();
        container.set_attribute("id", "cy").unwrap();
        self.controller.mount(&container.clone());
        let vref = VNode::VRef(container.into());
        html!{
            <div class={_ctx.props().class.clone()}>
                {vref}
            </div>
        }

    }
    
}


impl GraphView {

    fn apply_settings_message(&mut self, setting_msg: GraphSettingsMessage) {
        match setting_msg {
            GraphSettingsMessage::Relayout => {
                self.randomize_layout();
            }
        }
    }


    fn randomize_layout(&mut self) {
        if let Some(layout) = &self.prev_layout {
            layout.stop();
        }
        let new_layout = self.controller.layout(
            &js_sys::JSON::parse(r#"
            {
                "name": "cose",
                "animate": true,
                "randomize": false,
                "maxSimulationTime": 1500,
                "animationDuration": 1000,
                "fit": true,
                "animationThreshold": 10
            }
            "#).unwrap()
        );
        new_layout.run();
        self.prev_layout = Some(new_layout);
    }
}