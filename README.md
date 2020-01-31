# Emergent, a Visual Testrunner for Rust

The emergent project is an attempt to create a visual testrunner for Rust.

"Visual" in the sense that not only the test-results can be shown but also the output of the tests can be visualized as vector graphics, drawings, animations, and may be even more.

The "vision" of this project is to build a basis for developing applications that are simulatable and testable in their visual and internal representation at any time in any state.

Furthermore, the testrunner should be able to create new testcases by interacting directly with the application under test.

## Building & Running Tests

So far emergent is not in a state that it can be used to test other packages besides emergent itself, but if you are curious and up for a rough ride, follow the instructions below to get a first look at what this is about.

### Prerequisites

Emergent runs with [Vulkan](https://en.wikipedia.org/wiki/Vulkan_(API)) graphic drivers only. **On Windows**, they are most likely available already, **on Linux** [this article on linuxconfig.org](https://linuxconfig.org/install-and-test-vulkan-on-linux) might get you started, and **on macOS** with Metal support, [install the Vulkan SDK](https://vulkan.lunarg.com/sdk/home) for Mac and configure MoltenVK by setting the `DYLD_LIBRARY_PATH`, `VK_LAYER_PATH`, and `VK_ICD_FILENAMES` environment variables as described in `Documentation/getting_started_macos.html`.

Furthermore, the compilation steps need an [LLVM](https://llvm.org/) installation. **On Linux** or **on macOS** LLVM should be available, **on Windows** LLVM can be installed with [Chocolatey](https://chocolatey.org/):

```bash
choco install llvm
```

### Building & Running Tests

Clone the repository, cd into it, and then check out the submodules with

```bash
git submodule update --init
```

**on Windows** [Ninja](https://github.com/ninja-build/ninja) is needed to compile [shaderc-sys](https://crates.io/crates/shaderc-sys)

```bash
choco install ninja
```

and then compile & run emergent with

```bash
cargo run
```

This should - with LLVM installed, and a decent Vulkan driver, and a bit of luck - compile everything, power up the testrunner, and visualize some early results of some of the emergent library test cases.

It does that by starting the testrunner, which starts `cargo watch` internally, which in turn runs `cargo test` on the emergent library, captures its results, and visualizes them. From now on, changes are detected and the visualizations are updated automatically.

## Plan

My plan is to ...

- make a **graphics library** with a GPU backend and high quality perspective anti-aliasing available to the Rust ecosystem. A first attempt is to [interface with Google's Skia library](https://github.com/rust-skia/rust-skia). Later, if mature, [Pathfinder](https://github.com/servo/pathfinder) and [Skribo](https://github.com/linebender/skribo) may be used as a replacement.
- create a decent abstraction library for **drawings** and **layout**. While there are modern attempts like [Piet](https://github.com/linebender/piet), [Stretch](https://github.com/vislyhq/stretch), and [Druid](https://github.com/xi-editor/druid). I feel that the focus of these projects don't fit: Piet is focused on a per platform implementations, which I would like to see unified, Stretch puts all layout under the 2D Flexbox doctrine, which seems rather un-flexible, and Druid combines UI widgets and hierarchy with layout, which makes the layout engine unusable for vector drawings.
  My goals for a drawing library is a complete, fast, and compact serializable representation with a minimum set of external functional dependencies, like text measurements and path combinators, for example.
  And for the layout engine, it should be built from one-dimensional combinators and scale up to three or four dimensions while providing a simplified set of combinators to create 2D layouts.
- create an **application component system** that looks like a combination of TEA and React. While React focuses on UI components, TEA focuses on having one single application state. I think by layering multiple TEAs, an optimal combination of both worlds is possible. Conceptually, this is probably the hardest part to realize.
- create an **interpolation** layer, that enables **animations**. This should work similar to the DOM diffing algorithms that enable incremental updates, but also produce animations that are independent of layout hierarchies and placement.
- use or create a **gesture recognition** library.
- specify and create text protocol based I/O interfaces and **simulators** for **operating system functionality**, so that all desktop and mobile operating systems look similar to the application and interfacing with them does not depend on complex FFI APIs.

All these components are developed _very carefully_ in lock-step with the testrunner. Strictly adhering to the the [first principle](https://en.wikipedia.org/wiki/First_principle) that a component and all its functionality _must_ be fully visualizable, simulatable, reproducible, and as a result, testable.

## History, Context, and Vision

I've had the vision of live programming for a long time and dived deep into languages, frameworks, built countless prototypes, visited conferences, but never felt that I or the live programming community was able to realize what I've imagined.

Years ago while working on a an Visual Studio extension that executed F# code live and rendered the result into the editor, I realized that focusing on live programming - while seemingly motivating at first - is doomed to fail when attempted in isolation.

I now think that live programming does not make sense except for a [good demo](https://www.youtube.com/watch?v=PUv66718DII), because developers spend most of the time refactoring. This is because the creation of new features is the trivial part of programming, but modifying an environment that supports all existing features _while_ enabling new features is the complex part.

The live programming research community answers this problem with creating specifically suited live programming languages or environments, and some of the researchers even created several over the years.

Somehow, all that investment does not seem to lead to solution that is usable. And I think I know why. From my point of view, live programming is merely a by-product of a larger solution to a problem that is much more pressing, and that is live testing.

This project should enable live testing up until the point we can test _any_ imagined aspect of the software in development. The result will be much more than live programming ever attempted, it will be an accessible representation of the application in any state at any time. A timeless god view into the multiverse of the application under test that can be navigated, extended, tested, and compared with previous snapshots.

To realize that, I think we need to push only _one_ recently developed concept a bit further.

Basically it is event sourcing und unidirectional data flow that makes all of it possible, React and Flux were the first popular concepts that tried to map interaction to the input - output model of simple console applications _and_ simplified state handling at the same time. And that lead to The Elm Architecture, which is finally disrupting MVC _and_ puts itself at the pinnacle of application logic design.

If all input to an application can be serialized and the application's state and side-effects captured in full, it is possible to put the application into a sandbox, provide environments to it, and simulate its results in form of its state and visual output.

Of course all that is a rather idealistic goal, but I think that we can learn a lot by just trying.

## Copyright & License 

(c) 2020 Armin Sander

MIT

