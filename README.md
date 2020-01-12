# Foundry

A reactive UI API, web framework, and virtual DOM implementation written in Rust with WebAssembly as a build target.

##### A brief description of each project:

foundry_core
> Library for maintaining a generic tree structure, diffing the tree structure and defining Component + State structs.

foundry_web
> Library that defines a context used to render to a website, and HTML-specific utilities.

foundry-macro-html
> A proc-macro that parses HTML and outputs valid Rust code at compile-time.

example
> A demo application.

##### API Example:
``` rust
#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
	  // Create web context.
    let context = match foundry_web::create_context("content")?;

	  // Define component state.
    struct HelloWorldState {
        clicks: i32
    }
    let hws = HelloWorldState {
        clicks: 0,
    };
    let state = Rc::new(State::new(hws));

    // Create new component. We provude a state instance, and a closure that is executed when the state changes.
    let root = Component::from_state(state.clone(), move |s| {
        let state_clone = state.clone();

        let x: Box<HtmlNode>;
        if s.clicks > 3 {
            x = html!(<div>"You have clicked the button too many times!"</div>);
        } else {
            x = html!(<div>"You haven't clicked the button enough."</div>);
        }

        // A macro that is evaluated and transformed to Rust code at compile time.
        html!(<div>
            {x}
            <div>
                <div onClick={
                    let state_callback = state_clone.clone();
                    let mut state_ref = state_callback.get_mut(); // Creates a mutable reference to the state.
                    state_ref.clicks += 1;
                    // When this code block ends, the mut reference is deallocated,
                    // and this component is marked for re-rendering.
                }>
                    <span>"Click me!"</span>
                </div>
                <p>
                    "You've clicked the text "
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
```
