# Emergent, a Visual Testrunner for Rust

The emergent project is an attempt to create a visual testrunner for Rust.

"Visual" in the sense that not only the test-results can be shown but also the output of the tests can be visualized as vector graphics, drawings, animations, and may be even more.

The "vision" of this project is to build a basis for developing applications that are simulatable and testable in their visual and internal representation at any time in any state.

Furthermore, the testrunner should be able to create new testcases by interacting directly with the application under test.

## Building & Running Tests

Currently, there is not a lot to see. 

On Windows, `cargo run -- emergent-layout` should - with LLVM installed, and a decent Vulkan driver, and a lot of luck - compile everything, power up the testrunner, and visualize some early results of the layout engine.

It does that by starting the testrunner, which starts `cargo watch` internally, which in turn runs `cargo test` on the `emergent-layout` package, captures its results, and visualizes them. From now on, changes in the `emergent-layout` packages are detected and the visualizations are updated automatically.

## Strategy

My strategy is to ...

- make a **graphics library** with a GPU backend and high quality perspective anti-aliasing available to the Rust ecosystem. A first attempt is to [interface with Google's Skia library](https://github.com/rust-skia/rust-skia). Later, if mature, [Pathfinder](https://github.com/servo/pathfinder) and [Skribo](https://github.com/linebender/skribo) may be used as a replacement.
- create a decent abstraction library for **drawings** and **layout**. While there are modern attempts like [Piet](https://github.com/linebender/piet), [Stretch](https://github.com/vislyhq/stretch), and [Druid](https://github.com/xi-editor/druid). I feel that the focus of these projects don't fit: Piet is focused on a per platform implementations, which I would like to see unified, Stretch puts all layout under the 2D Flexbox doctrine, which seems rather un-flexible, and Druid combines UI widgets and hierarchy with layout, which makes the layout engine unusable for vector drawings.
  My goals for a drawing library is a complete, fast, and compact serializable representation with a minum set of external functional dependencies, like text measurements and path combinators, for example.
  And for the layout engine, it should be built from one-dimensional combinators and scale up to three or four dimensions while providing a simplified set of combinators to create 2D layouts.
- create an **application component system** that looks like a combination of TEA and React. While React focuses on UI components, TEA focuses on having one single application state. I think by layering multiple TEAs, an optimal combination of both worlds is possible. Conceptually, this is probably the hardest part to realize.
- create an **interpolation** layer, that enables **animations**. This should work similar to the DOM diffing algorithms that enable incremental updates, but also produce animations that are independent of layout hierarchies and placement.
- use or create a **gesture recognition** library.
- specify and create text protocol based I/O interfaces and **simulators** for **operating system functionality**, so that all desktop and mobile operating systems look similar to the application and interfacing with them does not depend on complex FFI APIs.

All these components are developed _very carefully_ in lock-step with the testrunner. Strictly adhering to the the [first principle](https://en.wikipedia.org/wiki/First_principle) that a component and all its functionality _must_ be fully visualizable, simulatable, reproducible, and as a result, testable.

## History and Context

I've had the vision of live programming for a long time and dived deep into languages, frameworks, built countless prototypes, visited conferences, but never felt that I or the live programming community was able to realize what I've imagined.

Years ago while working on a an Visual Studio extension that executed F# code live and rendered the result into the editor, I realized that focusing on live programming - while seemingly motivating at first - is doomed to fail when attempted in isolation.

I think that live programming does not make sense except for a [good demo](https://www.youtube.com/watch?v=PUv66718DII), because developers spend most of the time refactoring. This is because the creation of new features is the trivial part of programming, but modifying an environment that supports all existing features _while_ enabling new features is the complex part.

The live programming research community answers this problem with creating specifically suited live programming languages or environments, and some of the researchers even created several over the years.

Somehow, all that investment does not seem to lead to solution that is usable. And I think I know why. From my point of view, live programming is merely a by-product of a larger solution to a problem that is much more pressing, and that is live testing.

This project should enable live testing up until the point we can test _any_ imagined aspect of the software in development. The result will be much more than live programming ever attempted, it will be an accessible representation of the application in any state at any time. A god view into the multiverse of the application under test that can be navigated, extended, tested, and compared with previous snapshots. All that without being constrained to a specific programming language or IDE.

To realize that, I think we need to push only _one_ recently developed idea to its extreme.

Basically it is event sourcing und unidirectional data flow that makes all of it possible, React and Flux were the first popular concepts that tried to project interaction to the input - output model of simple console applications _and_ simplified state handling at the same time. And that lead to The Elm Architecture, which disrupted MVC and is currently the pinnacle of application logic design.

If all input to an application can be serialized and the application's state and side-effects captured in full, it is possible to put the application into a sandbox, provide environments to it, and simulate its results in form of its state and visual output.

## Copyright & License 

(c) 2019 Armin Sander

MIT

