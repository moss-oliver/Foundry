# Foundry

A reactive UI API, web framework, and virtual DOM implementation written in Rust.

##### API Example:

[Click here for an example.](example/src/lib.rs)

##### A brief description of each project:

foundry_core
> Library for maintaining a generic tree structure, diffing the tree structure and defining Component + State structs.

foundry_web
> Library that defines a context used to render to a website, and HTML-specific utilities.

foundry-macro-html
> A proc-macro that parses HTML and outputs valid Rust code at compile-time.

example
> A demo application.
