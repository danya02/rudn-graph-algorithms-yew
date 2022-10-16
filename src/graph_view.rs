use yew::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Function;
use yew::virtual_dom::VNode;

#[derive(Eq, PartialEq, Clone)]
pub struct Node {
    id: usize,
    name: String,
    color: String,
}

impl Node {
    fn to_jsvalue(&self) -> JsValue {
        let obj = js_sys::Object::new();
        let id = JsValue::from(self.id);
        let name = JsValue::from(self.name.clone());
        js_sys::Reflect::set(&obj, &JsValue::from("id"), &id).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from("name"), &name).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from("color"), &JsValue::from(self.color.clone())).unwrap();
        obj.into()
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct Link {
    from: Node,
    to: Node,
}

impl Link {
    fn to_jsvalue(&self) -> JsValue {
        let obj = js_sys::Object::new();
        let from = self.from.to_jsvalue();
        let to = self.to.to_jsvalue();
        js_sys::Reflect::set(&obj, &JsValue::from("source"), &from).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from("target"), &to).unwrap();
        obj.into()
    }
}

pub struct Graph {
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            links: vec![],
        }
    }

    pub fn to_jsvalue(&self) -> JsValue {
        let obj = js_sys::Object::new();
        let nodes = js_sys::Array::new();
        let links = js_sys::Array::new();
        for node in &self.nodes {
            nodes.push(&node.to_jsvalue());
        }
        for link in &self.links {
            links.push(&link.to_jsvalue());
        }
        js_sys::Reflect::set(&obj, &JsValue::from("nodes"), &nodes).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from("links"), &links).unwrap();
        JsValue::from(obj)
    }
}


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
}

pub struct GraphView {
    controller: CytoscapeController,
    prev_layout: Option<CytoscapeRunnableLayout>,
}


impl Component for GraphView {
    type Message = ();
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        log::info!("Create cytoscape controller");

        let options = js_sys::Object::new();

        let style = js_sys::JSON::parse(r##"
            [
                {
                    "selector": "node",
                    "style": {
                        "background-color": "#666",
                        "label": "data(id)"
                    }
                },
            
                {
                    "selector": "edge",
                    "style": {
                        "width": 3,
                        "line-color": "#ccc",
                        "target-arrow-color": "#ccc",
                        "target-arrow-shape": "triangle",
                        "curve-style": "bezier"
                    }
                }
            ]
        "##).unwrap();
        js_sys::Reflect::set(&options, &JsValue::from("style"), &style).unwrap();
        

        let controller = make_cytoscape_controller(&options);

        controller.add(
            &js_sys::JSON::parse(r#"
            [
                {
                    "data": { "id": "a" }
                },
                { 
                    "data": { "id": "b" }
                },
                { 
                    "data": { "id": "ab", "source": "a", "target": "b" }
                }
            ]
            "#).unwrap()
        );


        Self {
            controller: controller,
            prev_layout: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::info!("Cytoscape controller received message: {:?}", msg);
        if let Some(layout) = &self.prev_layout {
            layout.stop();
        }
        self.controller.layout(
            &js_sys::JSON::parse(r#"
            {
                "name": "cose",
                "animate": true,
                "randomize": true,
                "maxSimulationTime": 1500,
                "animationDuration": 1000,
                "fit": true
            }
            "#).unwrap()
        ).run();

        false
    }


    fn view(&self, ctx: &Context<Self>) -> Html {
        log::info!("Cytoscape controller is viewed (should only happen once)");

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let container = document.create_element("div").unwrap();
        container.set_attribute("id", "cy").unwrap();
        self.controller.mount(&container.clone());
        let vref = VNode::VRef(container.into());
        html!{
            <div>
                {vref}
                <button onclick={ ctx.link().callback(|_| ()) }>{"Run layout"}</button>
            </div>
        }

    }
    
}