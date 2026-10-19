# PTSD Overview

PTSD is a Rust crate that provides a foundational layer between **Prism** and higher-level UI design systems. It is responsible for handling **navigation**, **user interactions**, and **theming**, allowing design systems to focus on defining visual components and layout patterns.

Rather than acting as a UI framework itself, PTSD supplies common infrastructure that design systems can build upon. This separation allows UI frameworks to remain focused on presentation while delegating behavioral and structural concerns to PTSD.

PTSD sits beneath design systems such as **Pelican UI** and above **Prism**. Prism provides drawable and layout traits, drawing instructions, input events, and application interfaces. PTSD supplies the concrete UI layouts, containers, and interactions. Prism is PTSD's only RAMP-stack dependency.

Together these layers form the following structure:

```text
Application
    ↓
Design System (e.g. Pelican UI)
    ↓
PTSD
    ↓
Prism
```

## Scope

PTSD provides:

* Navigation infrastructure
* Interaction systems
* Theme resource management
* Layouts: `Stack`, `Row`, `Column`, and `Wrap`
* Display containers: `Bin`, `Opt`, `EitherOr`, and `Enum`
* Input emitters and UI events

Styled components remain in higher-level design systems such as Pelican UI.

The former Prism helpers now live in `ptsd::layout`, `ptsd::display`, `ptsd::emitters`, and `ptsd::events`. Drawable implementations use Prism's `SizeRange`, `RangeTree`, and `LayoutTree`. Concrete component handlers use `prism::events::OnEvent<E>` with `#[on_event]` above each implementation.

Components use `#[derive(Component)]`. Generic emitters use `#[on_dyn_event]` and define an inherent `on_event` method to receive boxed events directly. `Opt` transfers its child between a visible `Option<D>` and a `#[skip]` hidden `Option<D>`, retaining its state while hidden.

Prism applies window insets before laying out the root component. PTSD interfaces do not add those insets again, and their constructors no longer require a `Context`.

## Interactions

PTSD defines reusable emitters that translate Prism input events into UI events, and interaction types that update the drawable components in response.

These interaction layers translate raw input events into meaningful UI behaviors.

The core interaction types currently include:

* **Button** — handles press and activation behaviors
* **Selectable** — manages selectable or toggleable UI elements
* **Slider** — supports continuous value selection
* **Text Input** — manages editable text fields

By centralizing these behaviors, PTSD ensures consistent interaction logic across all components built on top of it.

## Theming

PTSD provides a flexible **Theme** object used to store shared visual resources for an application.

The theme system is intentionally designed to be highly flexible. While PTSD defines the structure of the theme, it is expected that higher-level design systems narrow and structure its usage to fit their own visual design requirements.

A theme contains several resource collections:

* **ColorResources** — a collection of colors as `Color`
* **IconResources** — a collection of icons as `Arc<RgbaImage>`
* **FontResources** — a collection of fonts and font sizes

These resources allow UI systems to maintain consistent visual styling across an application.

## Design Goals

PTSD is designed to provide shared infrastructure without imposing design decisions on the UI systems that use it.

Key goals include:

* **Separation of concerns** between UI rendering and UI behavior
* **Reusable interaction logic** across multiple design systems
* **Flexible theme management** adaptable to different visual frameworks
* **Minimal assumptions** about the structure of higher-level UI systems

Because of this approach, PTSD can support a variety of UI frameworks built on top of Prism.

## Platform Support

* Linux
* Windows
* macOS
* Android
* iOS

## Examples

Example design systems built on top of PTSD can be found in the ecosystem, including **Pelican UI**, which demonstrates how PTSD can be used to build structured UI frameworks with consistent interactions and navigation.

Developers exploring PTSD are encouraged to review these examples to understand how navigation, interactions, and theming are typically integrated into a full UI system.

## Discord

https://discord.gg/53ERRpS4S4

Join the Discord server to ask questions, discuss development, or share projects.
