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

pub struct GraphView {
    graph: Graph,
    graph_html: web_sys::Node,
    refresh_function: Function,
}

#[wasm_bindgen(module="/src/graph_controller.js")]
extern "C" {
    #[wasm_bindgen(js_name="ForceGraph")]
    fn create_force_graph() -> Box<[JsValue]>;  // Expected: first value is an HTML component,
                                                // second is the refresh function.
}

impl Component for GraphView {
    type Message = ();
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut graph = Graph::new();
        graph.nodes.push(Node { id: 0, name: "A".to_string(), color: "red".to_string() });
        let force_graph_items = create_force_graph();
        let mut unboxed_force_graph_items = force_graph_items.to_vec();
        let refresh = unboxed_force_graph_items.pop().expect("Second value should be a refresh function.");
        let force_graph = unboxed_force_graph_items.pop().expect("First value should be an HTML component.");
        let refresh: Function = refresh.dyn_into().expect("Second value should be a refresh function.");
        Self { 
            graph,
            graph_html: force_graph.into(),
            refresh_function: refresh,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        let new_id = self.graph.nodes.len();
        self.graph.nodes.push(Node { id: new_id, name: "A".to_string(), color: "red".to_string() });
        self.graph.links.push(
            Link {
                from: self.graph.nodes[0].clone(),
                to: self.graph.nodes[new_id].clone(),
                
            }
        );
        self.refresh_function.call1(&JsValue::NULL, &self.graph.to_jsvalue()).expect("Refresh function should be callable.");
    false
    }


    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let graph_component = VNode::VRef(self.graph_html.clone());

        html!{
            <div>
                { graph_component }
                <button onclick={link.callback(|_| ())}>{ "+1" }</button>

            </div>
        }
    }
    
}