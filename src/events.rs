use prism::Context;
use prism::events::{ChildEvent, Event, Key, MouseButton, MouseEvent, MouseState};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Press {
    Pending {id: Uuid, position: Option<(f32, f32)>},
    Commit(Uuid),
    Claim(Uuid),
    Cancel(Uuid),
}

impl Event for Press {
    fn pass(&self, ctx: &mut Context, children: &mut [ChildEvent<'_, Self>]) {
        let Self::Pending {id, position} = *self else {
            for child in children {*child.event = Some(*self);}
            return;
        };
        let mouse = MouseEvent {position, state: MouseState::Pressed(MouseButton::Left)};
        let mut events = vec![None; children.len()];
        let mut slots: Vec<_> = children.iter().zip(&mut events)
            .map(|(child, event)| ChildEvent {area: child.area, event}).collect();
        mouse.pass(ctx, &mut slots);
        for (child, mouse) in children.iter_mut().zip(events) {
            *child.event = mouse.map(|mouse| Self::Pending {id, position: mouse.position});
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Button {Pressed(bool), Hover(bool), Disable(bool)}

impl Event for Button {
    fn pass(&self, _ctx: &mut Context, children: &mut [ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(*self);}
    }
}

#[derive(Debug, Clone)]
pub enum Selectable {Pressed(String, String), Selected(bool)}

impl Event for Selectable {
    fn pass(&self, _ctx: &mut Context, children: &mut [ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(self.clone());}
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Slider {Start(f32), Moved(f32)}

impl Event for Slider {
    fn pass(&self, _ctx: &mut Context, children: &mut [ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(*self);}
    }
}

#[derive(Debug, Clone)]
pub enum TextInput {Hover(bool), Focused(bool), Edited(Key)}

impl Event for TextInput {
    fn pass(&self, _ctx: &mut Context, children: &mut [ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(self.clone());}
    }
}

#[derive(Debug, Clone)]
pub enum NumericalInput {Delete, Digit(char), Char(char)}

impl Event for NumericalInput {
    fn pass(&self, _ctx: &mut Context, children: &mut [ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(self.clone());}
    }
}

#[derive(Debug, Clone)]
pub enum Scrollable {ScrollTo(f32)}

impl Event for Scrollable {
    fn pass(&self, _ctx: &mut Context, children: &mut [ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(self.clone());}
    }
}
