#[macro_use(html)]
extern crate foundry_web;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use foundry_core::{State, Component, Value::*};
use foundry_web::{HtmlNode};

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let context = match foundry_web::create_context("content") {
        Ok(c) => c,
        Err(s) => { return Err(s.into()) }
    };

    #[derive(Debug)]
    struct HelloWorldState {
        val: i32,
        text: String,
        clicks: i32
    }

    let hws = HelloWorldState {
        val: 20,
        text: "Hello, world!".to_string(),
        clicks: 0,
    };

    let state = Rc::new(State::new(hws));
    let root = Component::from_state(state.clone(), move |s| {
        let state_clone = state.clone();

        let x :Box<HtmlNode>;
        if s.clicks > 3
        {
            x = html!(
                <div>
                    "You have clicked the button too many times!"
                    <div>"..."</div>
                </div>
            );
        } else {
            x = html!(
                <div>
                    "You haven't clicked the button enough."
                </div>
            );
        }

        html!(<div>
            {x}
            <div id="script">
                <div id="green-square" style="background-color: green" onClick={
                    let state_callback = state_clone.clone();
                    let mut st = state_callback.get_mut();
                    st.clicks += 1;
                }>
                    <span>
                        "Click me!"
                    </span>
                </div>
                <p>
                    "You've clicked the green square "
                    <span id="num-clicks">
                        {Box::new(s.clicks.to_string())}
                    </span>
                    " times"
                </p>
            </div>
        </div>)
    });
    root.bind_context(context);

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
