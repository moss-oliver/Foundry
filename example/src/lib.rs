#[macro_use(html)]
extern crate foundry_web;
use wasm_bindgen::prelude::*;
use foundry_core::{State, Component, Event};
use foundry_web::HtmlNode;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    // Create web context.
    let context = foundry_web::create_context("content")?;

    // Define component state.
    struct HelloWorldState {
        title: String,
        clicks: i32
    }

    let hws = HelloWorldState {
        title: "Hello, world!".to_string(),
        clicks: 0,
    };

    let state = State::new(hws);

    let on_click_event = Event::new(&state, move |ci| {
        let mut state = ci.state.get_mut(); // Creates a mutable reference to the state.
        state.clicks += 1;
        if state.clicks > 6 {
            state.title = "Stop clicking the button!".to_string();
        }
        // When this code block ends, the mutable reference is deallocated,
        // and this component is marked for re-rendering.
    });

    // Create new component. We provide a state instance, and a closure that is executed when the state changes.
    let root = Component::from_state(state.clone(), move |ri| {
        let x = if ri.state.clicks > 3 {
            html!(
                <div>
                    You have clicked the button too many times!
                    <div>...</div>
                </div>
            )
        } else {
            html!(
                <div>
                    You haven't clicked the button enough.
                </div>
            )
        };

        // A macro that is evaluated and transformed to Rust code at compile time.
        html!(
            <div>
                <h2>
                    {&ri.state.title}
                </h2>
                {x}
                <div id="script">
                    <button id="green-square" onClick=@on_click_event >
                        <span>
                            Click me..!
                        </span>
                    </button>
                    
                    <p style="background-color: lime">
                        You've clicked the green square
                        <span id="num-clicks" >
                            {ri.state.clicks}
                        </span>
                        times
                    </p>
                </div>
            </div>
        )
    });

    root.bind_context(context);
    Ok(())
}
