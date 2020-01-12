use proc_macro_hack::proc_macro_hack;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use js_sys::{Array, Date};
use web_sys::{Document, Element, HtmlElement, Window};

use foundry_core::{Context, DomNode, DomIntoIterator, ReconciliationNote};

use std::rc::Rc;

extern crate foundry_macro_html;

#[proc_macro_hack(fake_call_site)]
pub use foundry_macro_html::html;

#[derive(std::cmp::PartialEq, std::cmp::Eq, std::fmt::Debug)]
pub enum HtmlNodeType {
    Str(String),
    Tag(String)
}

pub struct HtmlNode {
    tag: String,
    props: Rc<Vec<(String, foundry_core::Value)>>,
    children: Vec<Box<dyn DomNode<HtmlNodeType>>>,
}

impl HtmlNode {
    pub fn new(tag: impl Into<String>, props: Vec<(&str, foundry_core::Value)>, children: Vec<Box<dyn DomNode<HtmlNodeType>>>) -> HtmlNode
    {
        HtmlNode {
            tag: tag.into(),
            props: Rc::new(props.into_iter().map(|s| (s.0.to_string(), s.1)).collect()),
            children,
        }
    }
    pub fn value_to_str(value: &foundry_core::Value) -> String {
        match value {
            foundry_core::Value::Str(s) => s.clone(),
            foundry_core::Value::Int(i) => i.to_string(),
            foundry_core::Value::Float(f) => f.to_string(),
            foundry_core::Value::Func(_) => "method_0".to_string() //TODO: Lookup a unique string name from a lookup table.
        }
    }
}

impl DomNode<HtmlNodeType> for HtmlNode {
    fn get_children<'a>(&'a self) -> Box<dyn ExactSizeIterator<Item = &'a Box<dyn DomNode<HtmlNodeType>>> + 'a> {
        Box::new(DomIntoIterator::into_iter(&self.children))
    }
    fn get_inner(&self) -> HtmlNodeType {
        //TODO: Consider removing this clone by using a smart pointer.
        HtmlNodeType::Tag(self.tag.clone())
    }
    fn get_params<'a>(&'a self) -> Option<Rc<Vec<(String, foundry_core::Value)>>> {
        Some(self.props.clone())
    }
}

impl DomNode<HtmlNodeType> for String {
    fn get_children<'a>(&'a self) -> Box<dyn ExactSizeIterator<Item= &'a Box<dyn DomNode<HtmlNodeType>>> + 'a> {
        Box::new(std::iter::empty())
    }
    fn get_inner(&self) -> HtmlNodeType {
        HtmlNodeType::Str(self.clone())
    }
    fn get_params<'a>(&'a self) -> Option<Rc<Vec<(String, foundry_core::Value)>>> {
        None
    }
}

pub struct WebContext {
    document: web_sys::Document,
    root_element: web_sys::Element,
    last_tree: Option<Box<dyn DomNode<HtmlNodeType>>>,
}

impl Context<HtmlNodeType> for WebContext {
    fn set_recent_tree(&mut self, tree: Option<Box<dyn DomNode<HtmlNodeType>>>) {
        self.last_tree = tree;
    }

    fn get_recent_tree(&self) -> Option<&Box<dyn DomNode<HtmlNodeType>>> {
        self.last_tree.as_ref()
    }

    fn commit_changes(&mut self, changes: Vec<ReconciliationNote<HtmlNodeType>>) {
        // vDom diff debugging:
        //console_log!("Foundry_web committing changes. Count: {}", changes.len());

        for change in changes {
            // vDom diff debugging:
            //console_log!("change: {:?}", change);

            let parent_ref: &web_sys::Node = &self.root_element;
            let mut parent = parent_ref.clone();
            for path_entry in change.path.iter() {
                parent = parent.child_nodes().item(*path_entry).expect("Real dom differs from vdom.");
            }

            match &change.operation {
                foundry_core::ReconciliationOperation::<HtmlNodeType>::Remove => {
                    parent.remove_child(&parent.child_nodes().item(change.index).unwrap()).unwrap();
                },
                foundry_core::ReconciliationOperation::<HtmlNodeType>::Add(node) => {
                    //TODO: remove unwraps
                    fn add_node(parent: web_sys::Node, change: &foundry_core::ReconciliationNote<HtmlNodeType>, node: &web_sys::Node) {
                        if change.index < parent.child_nodes().length() {
                            parent.insert_before(node, parent.child_nodes().item(change.index).as_ref()).unwrap();
                        } else {
                            parent.append_child(node).unwrap();
                        }
                    }

                    match &node.0 {
                        HtmlNodeType::Str(s) => {
                            let n = web_sys::Text::new_with_data(&s).unwrap();
                            add_node(parent, &change, &n);
                        },
                        HtmlNodeType::Tag(s) => {
                            let n = self.document.create_element(&s).unwrap();

                            for x in node.1.as_ref().unwrap().iter() {
                                let node = n.dyn_ref::<HtmlElement>().expect("#green-square be an `HtmlElement`");
                                match &x.1 {
                                    foundry_core::Value::Str(s) => {
                                        node.set_attribute(&x.0, s).unwrap();
                                    },
                                    foundry_core::Value::Int(_i) => {
                                        //TODO: implement.
                                    },
                                    foundry_core::Value::Float(_f) => {
                                        //TODO: implement.
                                    },
                                    foundry_core::Value::Func(func) => {
                                        let func_clone = func.clone();

                                        let a = Closure::wrap(Box::new(move || {
                                            func_clone();
                                        }) as Box<dyn FnMut()>);

                                        node.set_onclick(Some(a.as_ref().unchecked_ref()));
                                        a.forget(); //TODO: this should not forget! This will cause leaks!
                                    }
                                }
                            }

                            add_node(parent, &change, &n);
                        }
                    }
                }
            }
        }
    }
}

//TODO: This should not return a String.
pub fn create_context(element_id: &str) -> Result<WebContext, String> {
    let window = match web_sys::window() {
        Some(w) => w,
        None => { return Err("Should have a window in this context".to_string()); }
    };
    let document = match window.document() {
        Some(d) => d,
        None => { return Err("Window should have a document".to_string()); }
    };
    let root_element = match document.get_element_by_id(element_id) {
        Some(r) => r,
        None => { return Err("Cannot find root element".to_string()); }
    };

    Ok(WebContext{document, root_element, last_tree: None})
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
