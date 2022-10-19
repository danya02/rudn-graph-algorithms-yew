use std::{collections::HashMap, sync::Mutex};

use js_sys::{Reflect, JSON, Function};
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


    #[wasm_bindgen(method, js_name = "listen")]
    fn listen(this: &CytoscapeElements, event: &str, callback: &Closure<dyn FnMut(JsValue)>); // https://js.cytoscape.org/#eles.listen

}

pub struct GraphView {
    controller: CytoscapeController,
    prev_layout: Option<CytoscapeRunnableLayout>,
    _producer: Box<dyn Bridge<EventBus>>,

    nodes: usize,

    closures: Vec<Closure<dyn FnMut(JsValue)>>,
}

#[derive(Debug)]
pub enum Msg {
    /// Request to change the settings of the graph
    SettingMessage(GraphSettingsMessage),

    /// A node has been selected
    SelectNode(String),

    /// A node has been unselected
    UnselectNode(String),
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct Properties {
    pub class: String,
}

impl Drop for GraphView {
    fn drop(&mut self) {
        unsafe{
            GRAPH_VIEW_EVENT_LINK.linked = None;
        }
    }
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
                    "data": { "id": "1", "selectable": true }
                },
                { 
                    "data": { "id": "2", "selectable": true }
                },
                { 
                    "data": { "id": "1-2", "source": "1", "target": "2" }
                }
            ]
            "#).unwrap()
        );


        controller.set_selection_type("additive");

        controller.filter("node").selectify();
        //controller.filter("node").select();
        


       let mut ret = Self {
            controller: controller,
            prev_layout: None,
            _producer: crate::graph_settings_bus::EventBus::bridge(ctx.link().callback(|setting_msg|{
                log::info!("Received event: {:?}", setting_msg);
                Msg::SettingMessage(setting_msg)
            })),
            nodes: 2,
            closures: Vec::new(),
        };
        ret.setup_listeners(ctx);
        #[allow(mutable_transmutes)]
        unsafe {
            // !!!!! (un)SAFETY !!!!!
            // We are using std::mem::transmute here
            // to convert a &GraphView to a &mut GraphView!
            // This is very unsafe and certainly UB; do not do this!!
            // (see https://doc.rust-lang.org/nomicon/transmutes.html)
            // If you have any idea how to avoid this, please let me know
            // (Also, may not work in future versions of Rust or in release mode)
            // 
            // Closures passed to JS must be 'static,
            // but we need closures that keep a reference to the GraphView.
            // So, GraphViewEventLink will keep a reference to the GraphView
            // (of which there must be only one) 
            // and the closures will keep a reference to GraphViewEventLink.
            // 
            // We must take special care to ensure that when the GraphView is dropped,
            // the GraphViewEventLink is unlinked from the GraphView.
            // This is done in the Drop implementation of GraphView.
            let graph_view_ref = &ret;
            let graph_view_ref = std::mem::transmute::<&GraphView, &mut GraphView>(graph_view_ref);
            GRAPH_VIEW_EVENT_LINK.linked = Some(graph_view_ref);
        }
        ret
    }


    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::info!("Cytoscape controller received message: {:?}", msg);
        self.handle_message(msg);
        false
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


// A structure that will be used to send messages to associated components,
// which will exist in 'static.
struct GraphViewEventLink<'a> {
    linked: Option<&'a mut GraphView>,
}

impl<'a> GraphViewEventLink<'a> {
    pub fn send(&mut self, msg: Msg) {
        if let Some(linked) = &mut self.linked {
            log::debug!("Sending message through EventLink: {:?}", msg);
            linked.handle_message(msg);
        } else {
            log::error!("Tried to send message {:?} through EventLink, but it was not linked", msg);
        }
    }
}

static mut GRAPH_VIEW_EVENT_LINK: GraphViewEventLink = GraphViewEventLink {
    linked: None
};



impl GraphView {

    fn handle_message(&mut self, msg: Msg){
        match msg {
            Msg::SettingMessage(setting_msg) => {
                self.apply_settings_message(setting_msg);
            },
            Msg::SelectNode(node) => log::info!("Selected node: {:?}", node),
            Msg::UnselectNode(node) => log::info!("Unselected node: {:?}", node),
        }
    }


    /// Get a reference to a closure, or create it if it doesn't exist.
    /// This is necessary because the closure must be alive as long as the event listener is alive.
    /// 
    /// Returns a JsValue that can be passed to the `listen` method.
    /*fn get_closure(&mut self, key: &str, callback: Closure<dyn FnMut(JsValue)>) -> JsValue {
        if !self.closures.contains_key(key) {
            self.closures.insert(key.to_string(), callback);
        }

        // This leaks memory, but this is acceptable, because this should only get called a few times as the graph is initialized.
        self.closures.get(key).unwrap().clone().into_js_value()
    }*/

    fn keep_closure(&mut self, closure: Closure<dyn FnMut(JsValue)>) {
        log::debug!("Keeping closure {:?}", closure);
        self.closures.push(closure);
    }

    fn setup_listeners(&mut self, ctx: &Context<Self>) {
        
        // Create a callback on selecting a node
        let callback = |event: JsValue| {
            let object: js_sys::Object = event.into();
            web_sys::console::log_1(&object);
            let target: js_sys::Object = js_sys::Reflect::get(&object, &JsValue::from("target")).unwrap().into();
            web_sys::console::log_1(&target);
            // id = target.id();
            let id_func: Function = js_sys::Reflect::get(&target, &JsValue::from("id")).unwrap().into();
            let id: String = id_func.call0(&target).unwrap().as_string().unwrap();
            unsafe {
                GRAPH_VIEW_EVENT_LINK.send(Msg::SelectNode(id));
            }
        };

        let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut(JsValue)>);
        self.keep_closure(closure);
        self.controller.filter("node").listen("select", &self.closures.last().unwrap());
        
        // Create a callback on unselecting a node
        let callback = |event: JsValue| {
            let object: js_sys::Object = event.into();
            web_sys::console::log_1(&object);
            let target: js_sys::Object = js_sys::Reflect::get(&object, &JsValue::from("target")).unwrap().into();
            web_sys::console::log_1(&target);
            // id = target.id();
            let id_func: Function = js_sys::Reflect::get(&target, &JsValue::from("id")).unwrap().into();
            let id: String = id_func.call0(&target).unwrap().as_string().unwrap();
            unsafe {
                GRAPH_VIEW_EVENT_LINK.send(Msg::UnselectNode(id));
            }
        };

        let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut(JsValue)>);
        self.keep_closure(closure);
        self.controller.filter("node").listen("unselect", &self.closures.last().unwrap());
        

    }


    fn apply_settings_message(&mut self, setting_msg: GraphSettingsMessage) {
        match setting_msg {
            GraphSettingsMessage::Relayout => {
                self.randomize_layout();
            },
            GraphSettingsMessage::AddNode => {
                self.add_node();
            },
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


    fn add_node(&mut self) {
        self.nodes += 1;
        let node_id = format!("{}", self.nodes);
        let array = js_sys::Array::new();
        let node_obj = js_sys::Object::new();
        let node_data = js_sys::Object::new();
        let node_id = JsValue::from(node_id);
        js_sys::Reflect::set(&node_data, &JsValue::from("id"), &node_id).unwrap();
        js_sys::Reflect::set(&node_data, &JsValue::from("selectable"), &JsValue::from(true)).unwrap();
        
        js_sys::Reflect::set(&node_obj, &JsValue::from("data"), &JsValue::from(node_data)).unwrap();
        array.push(&node_obj);
        web_sys::console::log_1(&array);
        self.controller.add(&array);
    }
}