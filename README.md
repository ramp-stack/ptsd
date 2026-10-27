# PTSD Overview

## Frontend development setup

Clone `orange`, `chk`, `pelican_ui`, and `ptsd` into sibling directories and update
their default branches together. Frontend dependencies use these local checkouts;
backend dependencies use published crates. Follow the
[Orange setup instructions](https://github.com/ramp-stack/orange#frontend-setup)
for Cargo Piston, the shared team identity, and build troubleshooting.

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
* Display containers: `Bin`, `Opt`, `EitherOr`, `Enum`, and `Dismissible`
* Input emitters and UI events

Styled components remain in higher-level design systems such as Pelican UI.

The former Prism helpers now live in `ptsd::layout`, `ptsd::display`, `ptsd::emitters`, and
`ptsd::events`. Drawable implementations use mutable `request_width()`, `request_height(width)`,
and `layout(size)` methods. Requests return `(minimum, maximum)`; layouts retain their direct
children's measurements and final placement produces a `LayoutTree`. Construct stacks with
`Stack::new(...)` or the named convenience constructors. Concrete component handlers use
`prism::events::OnEvent<E>` with `#[on_event]` above each implementation.

Components use `#[derive(Component)]`. Generic emitters list their event types in `#[events(...)]`
above the component and use `#[on_event]` on each typed implementation. Concrete handlers need no
event list. `Opt` transfers its child between a visible `Option<D>` and a `#[skip]` hidden `Option<D>`,
retaining its state while hidden.

Prism applies window insets before laying out the root component. PTSD interfaces do not add those insets again, and their constructors no longer require a `Context`.

## Mobile keyboards

`Interface::mobile(navigator, body, keyboard)` accepts `Option<Box<dyn Drawable>>` for the
keyboard. Pass `None` to use the native keyboard or `Some(Box::new(keyboard))` to supply a
custom keyboard. Construction does not request keyboard presentation. `InputField` requests it on
focus through `ShowKeyboard::Show {dismiss_padding}`; `ShowKeyboard::Hide` and navigation dismiss it.
`InputField::keyboard_dismiss_padding` configures that distance in logical pixels, defaulting to zero.
On iOS it extends the native swipe-dismiss region above the keyboard to cover an app toolbar.

Hardware owns one shared native keyboard handle. `ctx.hardware.keyboard()` clones that handle
without opening the keyboard; `show(dismiss_padding)` and `hide()` control its presentation.
The interface calls those methods on focus, explicit hide, and navigation. Visibility changes
preserve input focus, and repeated focus requests can reopen the keyboard after a swipe dismissal.
Custom keyboard visibility continues to emit `CustomKeyboardVisibilityChanged`.

Ramp forwards Maverick's available window size into `prism::Window`, including native keyboard
animation frames. The mobile interface uses that space directly without another reduction for
the native keyboard. Custom keyboards remain ordinary children in the mobile column and occupy
their allocated height. The navigator is hidden while a custom or native keyboard is visible.
Native visibility is read from `ctx.hardware.keyboard().visible()`, including floating keyboards.
It remains true during opening, closing, and partial swipe dismissal while any part of the keyboard
is visible, and becomes false only when the native keyboard is fully hidden.
The interface refreshes navigator visibility through typed `WindowEvent::Resized`,
`ShowKeyboard`, and navigation handlers. Native visibility changes also emit `Resized` when window
dimensions stay unchanged. Window updates reach the context before those handlers run, and the
interface forwards their events to its children.

The body receives the remaining layout space. A messaging composer anchored at its bottom
therefore follows the keyboard. Wrap form content in `emitters::Scrollable` to allow manual
scrolling. Keep fixed headers, composers, and toolbars outside that scrollable.

### Scrolling

`Scrollable::new(content)` anchors scrolling at the start; `Scrollable::end(content)` anchors
it at the end. Both respond to scroll gestures and `ScrollTo`, clamping the position to the
content bounds as the viewport changes. Focusing or editing an input does not reposition the
scrollable or add extra space around the content.

On windows marked `ctx.window.is_mobile`, scrollables replace a left-button press with
`events::Press::Pending {id, position}`. A stationary press requests a commit after 150
milliseconds; a quick release requests a commit before the release is forwarded. Moving more
than 12 logical pixels before a control claims the press cancels it and begins scrolling.
A commit over unused space has no claimant, so holding there still allows a later drag to
scroll. This uses the window's mobile flag, not a per-event touch-device classification.

Built-in buttons, selectable controls, sliders, and input emitters understand `Press`.
Custom controls inside a scrollable should handle `Pending`, `Commit(id)`, and `Cancel(id)`:
retain the pending identity and local position, perform the action only on a matching commit,
emit `Press::Claim(id)` through `ctx.emit` when accepting that commit, and clear pending/held
state on cancellation. The reusable `emitters::Press` helper captures a pending press and
accepts its commit at most once. Only `Claim` reserves the gesture for a control. Pending
events follow pointer hit testing; commit and cancel events reach descendants so a control
can match its captured identity even after layout moves. Controls needing immediate drag
ownership, such as sliders, can accept their captured press during `Pending`, emit `Claim`,
and begin their drag immediately. Keep raw mouse handling for use outside a scrollable and
for desktop input.

Native text focus and selection do not participate in this arbitration. Native text adapters
currently forward an immediate press/release pair, so gestures starting over native text may
commit before the physical release. The protocol does not defer native editor behavior.

## Dismissible panels

`display::Dismissible<D>` wraps any drawable and handles dragging, velocity, settling, and
layout. Use `DismissEdge::Bottom` for a keyboard or bottom sheet, and `DismissEdge::Top` for
a notification popover that swipes upward. Styling and the action taken after dismissal
remain with the owner.

The owner calls `observe` once for each event, before routing it to children. Supply the
panel's resting rectangle in the same stable coordinate space as the event, or `None` while hidden.
Convert moving local event coordinates to window coordinates before observing them.
Drags start within a 12-pixel strip at the panel's content-facing edge: the top of a bottom
panel, or the bottom of a top panel. The optional final argument adds a handle rectangle,
such as an adjacent toolbar, in the same coordinate space. Gestures starting elsewhere stay
with the content, so grids inside the panel can still scroll. Movement up to 8 logical pixels
leaves the panel at rest. Beyond that threshold, movement toward dismissal takes over the
gesture, with the threshold subtracted from its offset so the panel starts moving smoothly.
Buttons inside a handle should activate on release so a drag can cancel their pending action.

```rust,ignore
use prism::drawable::{Area, Component};
use ptsd::display::{Dismissible, DismissEdge};

let mut banner = Dismissible::new(notification, DismissEdge::Top);
banner.reserve_space(true); // An overlay keeps its layout slot while sliding.

// In the parent's event handler, after layout and before forwarding the event:
let area = Area {offset: (0.0, 0.0), size: banner.size()};
banner.observe(&mut *event, showing.then_some(area), None);

// The owner's listener awaits the banner, then runs its callback before advancing:
let callback = Component::listen(&mut banner).await;
callback(&mut banner, ctx);
if banner.advance() {
    showing = false;
    // Hide the banner or emit the application's notification-dismissed event.
}
```

By default the panel's occupied height shrinks as it moves, allowing neighboring content to
expand. `reserve_space(true)` keeps that space unchanged. Align bottom panels to the bottom
of their parent and top panels to the top. The resting rectangle excludes the panel's own drag
offset, but follows parent layout changes. Resizing preserves the drag and recomputes its offset
from the last pointer position, so parent movement is not added to finger movement a second time.
`observe` neutralizes captured mouse events, cancelling child input during a drag. The owner
includes the panel in its `listen` selection, runs the winning callback, then calls `advance`.
`advance` returns `true` once dismissal finishes and explicitly renews its timer while settling.
Short drags and reversals return to rest. Unpressed wheel input and trailing touch momentum
do not begin a drag. Call `reset` when replacing, showing, or navigating away from a panel;
hidden observation and focus loss also reset it. `is_dismissing` includes a possible drag, an
active drag, and settling, so the owner can retain the panel until its gesture completes.

Call `dismiss` to animate the remaining distance without a drag. It preserves an active or
possible gesture and repeated calls do not restart an existing dismissal. While the resting
edge moves in either direction, settling pauses and follows that edge. A 32-millisecond quiet
interval leaves room for the next resize frame before the panel resumes its own 0.22-second
slide. This follows observed layout movement without assuming a platform animation duration.

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
* **IconResources** — a collection of validated SVG icons as `prism::drawable::Svg`
* **FontResources** — a collection of fonts and font sizes

These resources allow UI systems to maintain consistent visual styling across an application.

`Assets::get_image(path)` and `get_asset_image(name)` return backend raster
`Image` handles. `get_svg(path)` and `get_asset_svg(name)` return validated `Svg` drawables.
Each collection retains shared content handles; cloned assets and themes reuse them. Raster pixels
decode lazily, and display resizing reuses their GPU texture. SVGs rasterize at their physical
display size. `IconResources` loads SVG files and returns `Svg` values through `get`.

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
