use prism::drawable::{Area, Layout, SizeRange};

use std::sync::{Arc, Mutex};

type FitFunc = fn(Vec<(f32, f32)>) -> (f32, f32);

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum ScrollAnchor {
    #[default]
    Start,
    End,
}

pub trait CustomFunc: Fn(Vec<(f32, f32)>) -> (f32, f32) + 'static {
    fn clone_box(&self) -> Box<dyn CustomFunc>;
}

impl<F: Fn(Vec<(f32, f32)>) -> (f32, f32) + Clone + 'static> CustomFunc for F {
    fn clone_box(&self) -> Box<dyn CustomFunc> { Box::new(self.clone()) }
}

impl Clone for Box<dyn CustomFunc> {fn clone(&self) -> Self {self.as_ref().clone_box()}}

impl std::fmt::Debug for dyn CustomFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Page Builder...")
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum Offset {
    #[default]
    Start,
    Center,
    End,
    Static(f32)
}

impl Offset {
    pub fn get(&self, max_size: f32, item_size: f32) -> f32 {
        match self {
            Self::Start => 0.0,
            Self::Center => (max_size - item_size) / 2.0,
            Self::End => max_size - item_size,
            Self::Static(offset) => *offset,
        }
    }

    pub fn size(&self) -> Option<f32> {
        match self {
            Self::Start => Some(0.0),
            Self::Center | Self::End => None,
            Self::Static(offset) => Some(*offset),
        }
    }
}

#[derive(Default, Clone)]
pub enum Size {
    #[default]
    Fit,
    Fill,
    Static(f32),
    Custom(Box<dyn CustomFunc>),
}

impl PartialEq for Size {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Size::Fit, Size::Fit) => true,
            (Size::Fill, Size::Fill) => true,
            (Size::Static(a), Size::Static(b)) => a == b,
            (Size::Custom(_), Size::Custom(_)) => false,
            _ => false,
        }
    }
}

impl Size {
    pub fn custom(func: impl Fn(Vec<(f32, f32)>) -> (f32, f32) + Clone + 'static) -> Self {
        Size::Custom(Box::new(func))
    }

    pub fn get(&self, items: Vec<(f32, f32)>, fit: FitFunc) -> (f32, f32) {
        match self {
            Size::Fit => fit(items),
            Size::Fill => (items.iter().fold(f32::MIN, |a, b| a.max(b.0)), f32::MAX),
            Size::Static(s) => (*s, *s),
            Size::Custom(f) => f(items)
        }
    }

    pub fn max(items: Vec<(f32, f32)>) -> (f32, f32) {
        items.into_iter().reduce(|s, i| (s.0.max(i.0), s.1.max(i.1))).unwrap_or_default()
    }

    pub fn add(items: Vec<(f32, f32)>) -> (f32, f32) {
        items.into_iter().reduce(|s, i| (s.0+i.0, s.1+i.1)).unwrap_or_default()
    }
}

impl std::fmt::Debug for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Size::Fit => write!(f, "Size::Fit"),
            Size::Fill => write!(f, "Size::Fill"),
            Size::Static(val) => write!(f, "Size::Static({val})"),
            Size::Custom(_) => write!(f, "Size::Custom(<function>)"),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Padding(pub f32, pub f32, pub f32, pub f32);

impl Padding {
    pub fn new(p: f32) -> Self {Padding(p, p, p, p)}

    pub fn adjust_size(&self, size: (f32, f32)) -> (f32, f32) {
        let wp = self.0+self.2;
        let hp = self.1+self.3;
        (size.0-wp, size.1-hp)
    }

    pub fn adjust_offset(&self, offset: (f32, f32)) -> (f32, f32) {
        (offset.0+self.0, offset.1+self.1)
    }
}

#[derive(Debug, Default, Clone)]
pub struct DefaultStack(Vec<SizeRange>);
impl Layout for DefaultStack {
    fn request_width(&mut self, children: Vec<(f32, f32)>) -> (f32, f32) {
        self.0 = children.iter().map(|&(min_width, max_width)| SizeRange {
            min_width, max_width, ..SizeRange::default()
        }).collect();
        Size::max(children)
    }

    fn assign_width(&mut self, width: f32) -> Vec<f32> {
        self.0.iter_mut().map(|child| {
            let width = width.clamp(child.min_width, child.max_width);
            child.min_width = width;
            child.max_width = width;
            width
        }).collect()
    }

    fn request_height(&mut self, _width: f32, children: Vec<(f32, f32)>) -> (f32, f32) {
        for (child, &(min, max)) in self.0.iter_mut().zip(&children) {
            child.min_height = min;
            child.max_height = max;
        }
        Size::max(children)
    }

    fn layout(&mut self, size: (f32, f32)) -> Vec<Area> {
        self.0.iter().map(|child| Area {offset: (0.0, 0.0), size: child.clamp(size)}).collect()
    }
}

pub struct UniformExpand;

impl UniformExpand {
    pub fn get(sizes: Vec<(f32, f32)>, max_size: f32, spacing: f32) -> Vec<f32> {
        if sizes.is_empty() {return vec![];}
        let spacing = sizes.len().saturating_sub(1) as f32 * spacing;
        let min_size = sizes.iter().fold(0.0, |s, i| s + i.0) + spacing;

        let mut sizes = sizes;

        let mut free_space = (max_size - min_size).max(0.0);
        while free_space > 0.0 {
            let (min_exp, count, next) = sizes.iter().fold(
                (None, 0.0, free_space), |(mut me, mut c, mut ne), size| {
                    let min = size.0;
                    let max = size.1;
                    if min < max {
                        match me {
                            Some(w) if w < min => {ne = ne.min(min - w);},
                            Some(w) if w == min => {
                                ne = ne.min(max - min);
                                c += 1.0;
                            },
                            Some(w) if w > min => {
                                ne = ne.min(max - min).min(w - min);
                                me = Some(min);
                                c = 1.0;
                            },
                            _ => {
                                ne = ne.min(max - min);
                                me = Some(min);
                                c = 1.0;
                            }
                        }
                    }
                    (me, c, ne)
                }
            );

            if min_exp.is_none() { break; }
            let min_exp = min_exp.unwrap();

            let expand = (next * count).min(free_space);
            free_space -= expand;
            let expand = expand / count;

            sizes.iter_mut().for_each(|size| {
                if size.0 < size.1 && size.0 == min_exp {
                    size.0 += expand;
                }
            });
        }

        sizes.into_iter().map(|s| s.0).collect()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Row(f32, Offset, Size, Padding, Vec<SizeRange>);

impl PartialEq for Row {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2 && self.3 == other.3
    }
}

impl Row {
    pub fn new(spacing: f32, offset: Offset, size: Size, padding: Padding) -> Self {
        Row(spacing, offset, size, padding, vec![])
    }

    pub fn center(spacing: f32) -> Self {
        Row::new(spacing, Offset::Center, Size::Fit, Padding::default())
    }

    pub fn start(spacing: f32) -> Self {
        Row::new(spacing, Offset::Start, Size::Fit, Padding::default())
    }

    pub fn end(spacing: f32) -> Self {Row::new(spacing, Offset::End, Size::Fit, Padding::default())}

    pub fn padding(&mut self) -> &mut Padding {&mut self.3}
}

impl Layout for Row {
    fn request_width(&mut self, children: Vec<(f32, f32)>) -> (f32, f32) {
        let spacing = self.0 * children.len().saturating_sub(1) as f32;
        self.4 = children.iter().map(|&(min_width, max_width)| SizeRange {
            min_width, max_width, ..SizeRange::default()
        }).collect();
        let width = Size::add(children);
        let padding = self.3.0 + self.3.2 + spacing;
        (width.0 + padding, width.1 + padding)
    }

    fn assign_width(&mut self, width: f32) -> Vec<f32> {
        let widths = UniformExpand::get(
            self.4.iter().map(|child| (child.min_width, child.max_width)).collect(),
            (width - self.3.0 - self.3.2).max(0.0), self.0,
        );
        for (child, width) in self.4.iter_mut().zip(&widths) {
            child.min_width = *width;
            child.max_width = *width;
        }
        widths
    }

    fn request_height(&mut self, _width: f32, children: Vec<(f32, f32)>) -> (f32, f32) {
        for (child, &(min, max)) in self.4.iter_mut().zip(&children) {
            child.min_height = min;
            child.max_height = max;
        }
        let height = self.2.get(children, Size::max);
        let padding = self.3.1 + self.3.3;
        (height.0 + padding, height.1 + padding)
    }

    fn layout(&mut self, row_size: (f32, f32)) -> Vec<Area> {
        let row_size = self.3.adjust_size(row_size);
        let mut offset = 0.0;
        self.4.iter().map(|child| {
            let size = (child.min_width, row_size.1.clamp(child.min_height, child.max_height));
            let off = self.3.adjust_offset((offset, self.1.get(row_size.1, size.1)));
            if size.0 > 0.0 {offset += size.0+self.0;}
            Area{offset: off, size}
        }).collect()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Column(f32, Offset, Size, Padding, Option<Arc<Mutex<f32>>>, ScrollAnchor, Vec<SizeRange>);

impl PartialEq for Column {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
        && self.1 == other.1
        && self.2 == other.2
        && self.3 == other.3
        && self.5 == other.5
    }
}

impl Column {
    pub fn new(spacing: f32, offset: Offset, size: Size, padding: Padding, scroll: Option<ScrollAnchor>) -> Self {
        Column(
            spacing, offset, size, padding,
            scroll.is_some().then_some(Arc::new(Mutex::new(0.0))), scroll.unwrap_or_default(), vec![],
        )
    }

    pub fn center(spacing: f32) -> Self {
        Self::new(spacing, Offset::Center, Size::Fill, Padding::default(), None)
    }

    pub fn start(spacing: f32) -> Self {
        Self::new(spacing, Offset::Start, Size::Fill, Padding::default(), None)
    }

    pub fn end(spacing: f32) -> Self {
        Self::new(spacing, Offset::End, Size::Fill, Padding::default(), None)
    }

    pub fn padding(&mut self) -> &mut Padding {&mut self.3}

    pub fn adjust_scroll(&mut self, delta: f32) {
        if let Some(s) = &mut self.4 {
            match self.5 {
                ScrollAnchor::Start => **s.lock().as_mut().unwrap() += delta,
                ScrollAnchor::End => **s.lock().as_mut().unwrap() -= delta,
            }
        }
    }

    pub fn set_scroll(&mut self, val: f32) {
        if let Some(s) = &mut self.4 {**s.lock().as_mut().unwrap() = val;}
    }
}
impl Layout for Column {
    fn request_width(&mut self, children: Vec<(f32, f32)>) -> (f32, f32) {
        self.6 = children.iter().map(|&(min_width, max_width)| SizeRange {
            min_width, max_width, ..SizeRange::default()
        }).collect();
        if children.is_empty() {return (0.0, 0.0)}
        let width = self.2.get(children, Size::max);
        let padding = self.3.0 + self.3.2;
        ((if self.4.is_some() {0.0} else {width.0}) + padding, width.1 + padding)
    }

    fn assign_width(&mut self, width: f32) -> Vec<f32> {
        let width = (width - self.3.0 - self.3.2).max(0.0);
        self.6.iter_mut().map(|child| {
            let width = width.clamp(child.min_width, child.max_width);
            child.min_width = width;
            child.max_width = width;
            width
        }).collect()
    }

    fn request_height(&mut self, _width: f32, children: Vec<(f32, f32)>) -> (f32, f32) {
        if children.is_empty() {return (0.0, 0.0)}
        let visible = children.iter().filter(|child| child.1 > 0.0).count();
        let spacing = self.0 * visible.saturating_sub(1) as f32;
        for (child, &(min, max)) in self.6.iter_mut().zip(&children) {
            child.min_height = min;
            child.max_height = max;
        }
        let height = Size::add(children);
        let padding = self.3.1 + self.3.3 + spacing;
        ((if self.4.is_some() {0.0} else {height.0}) + padding, height.1 + padding)
    }

    fn layout(&mut self, col_size: (f32, f32)) -> Vec<Area> {
        let col_size = self.3.adjust_size(col_size);
        let col_size = (col_size.0.max(0.0), col_size.1.max(0.0));
        let ranges: Vec<_> = self.6.iter().map(|child| (child.min_height, child.max_height)).collect();
        let visible: Vec<usize> = (0..ranges.len()).filter(|&i| ranges[i].1 > 0.0).collect();
        let expanded = UniformExpand::get(
            visible.iter().map(|&i| ranges[i]).collect(), col_size.1, self.0
        );
        let mut heights = vec![0.0; ranges.len()];
        visible.iter().zip(expanded).for_each(|(&i, h)| heights[i] = h);

        let spacing = self.0 * visible.len().saturating_sub(1) as f32;
        let content_height = heights.iter().sum::<f32>() + spacing;

        let max_scroll = (content_height - col_size.1).max(0.0);
        let scroll = self.4.as_ref().map(|s| {
            let mut v = s.lock().unwrap();
            *v = v.clamp(0.0, max_scroll);
            *v
        }).unwrap_or(0.0);

        let mut offset = match self.5 {
            ScrollAnchor::Start => -scroll,
            ScrollAnchor::End => col_size.1 - content_height + scroll,
        };
        self.6.iter().zip(heights).map(|(child, height)| {
            let size = (child.min_width, height);
            let off = self.3.adjust_offset((self.1.get(col_size.0, size.0), offset));
            if child.max_height > 0.0 {offset += height + self.0;}
            Area {offset: off, size}
        }).collect()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Stack(pub Offset, pub Offset, pub Size, pub Size, pub Padding, Vec<SizeRange>);

impl PartialEq for Stack {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
            && self.3 == other.3 && self.4 == other.4
    }
}

impl Stack {
    pub fn new(x_offset: Offset, y_offset: Offset, x_size: Size, y_size: Size, padding: Padding) -> Self {
        Stack(x_offset, y_offset, x_size, y_size, padding, vec![])
    }

    pub fn center() -> Self {
        Self::new(Offset::Center, Offset::Center, Size::Fit, Size::Fit, Padding::default())
    }

    pub fn start() -> Self {
        Self::new(Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding::default())
    }

    pub fn end() -> Self {Self::new(Offset::End, Offset::End, Size::Fit, Size::Fit, Padding::default())}

    pub fn fill() -> Self {
        Self::new(Offset::Center, Offset::Center, Size::Fill, Size::Fill, Padding::default())
    }
}

impl Layout for Stack {
    fn request_width(&mut self, children: Vec<(f32, f32)>) -> (f32, f32) {
        self.5 = children.iter().map(|&(min_width, max_width)| SizeRange {
            min_width, max_width, ..SizeRange::default()
        }).collect();
        let width = self.2.get(children, Size::max);
        let padding = self.4.0 + self.4.2;
        (width.0 + padding, width.1 + padding)
    }

    fn assign_width(&mut self, width: f32) -> Vec<f32> {
        let width = (width - self.4.0 - self.4.2).max(0.0);
        self.5.iter_mut().map(|child| {
            let width = width.clamp(child.min_width, child.max_width);
            child.min_width = width;
            child.max_width = width;
            width
        }).collect()
    }

    fn request_height(&mut self, _width: f32, children: Vec<(f32, f32)>) -> (f32, f32) {
        for (child, &(min, max)) in self.5.iter_mut().zip(&children) {
            child.min_height = min;
            child.max_height = max;
        }
        let height = self.3.get(children, Size::max);
        let padding = self.4.1 + self.4.3;
        (height.0 + padding, height.1 + padding)
    }

    fn layout(&mut self, stack_size: (f32, f32)) -> Vec<Area> {
        let stack_size = self.4.adjust_size(stack_size);
        self.5.iter().map(|child| {
            let size = child.clamp(stack_size);
            let offset = (self.0.get(stack_size.0, size.0), self.1.get(stack_size.1, size.1));
            Area{offset: self.4.adjust_offset(offset), size}
        }).collect()
    }
}

/// Lays out a track, its filled portion, and a handle in that order.
#[derive(Debug, Clone)]
pub struct Slider {
    pub value: f32,
    stack: Stack,
    width: f32,
    handle_width: f32,
}

impl Slider {
    pub fn new(value: f32, height: Size) -> Self {
        Self {
            value, stack: Stack::new(
                Offset::Start, Offset::Center, Size::Fit, height, Padding::default(),
            ), width: 0.0, handle_width: 0.0,
        }
    }

    pub fn position(&self, x: f32) -> f32 {x / self.width}
}

impl Layout for Slider {
    fn request_width(&mut self, children: Vec<(f32, f32)>) -> (f32, f32) {
        self.handle_width = children[2].0;
        self.stack.request_width(children)
    }

    fn assign_width(&mut self, width: f32) -> Vec<f32> {
        let mut widths = self.stack.assign_width(width);
        widths[1] = width * self.value.clamp(0.0, 1.0);
        widths[2] = self.handle_width;
        widths
    }

    fn request_height(&mut self, width: f32, children: Vec<(f32, f32)>) -> (f32, f32) {
        self.stack.request_height(width, children)
    }

    fn layout(&mut self, size: (f32, f32)) -> Vec<Area> {
        self.width = size.0;
        let x = size.0 * self.value.clamp(0.0, 1.0);
        let mut areas = self.stack.layout(size);
        areas[1].size.0 = x;
        areas[2].size.0 = self.handle_width;
        areas[2].offset.0 = (x - self.handle_width / 2.0)
            .clamp(0.0, (size.0 - self.handle_width).max(0.0));
        areas
    }
}

#[derive(Debug, Clone)]
pub struct Wrap(pub f32, pub f32, pub Offset, pub Offset, pub Padding, Vec<SizeRange>);

impl Wrap {
    pub fn new(w_spacing: f32, h_spacing: f32) -> Self {
        Wrap(
            w_spacing, h_spacing, Offset::Center, Offset::Center,
            Padding::default(), vec![],
        )
    }

    pub fn start(w_spacing: f32, h_spacing: f32) -> Self {
        Wrap(
            w_spacing, h_spacing, Offset::Start, Offset::Center,
            Padding::default(), vec![],
        )
    }

    pub fn end(w_spacing: f32, h_spacing: f32) -> Self {
        Wrap(
            w_spacing, h_spacing, Offset::End, Offset::Center,
            Padding::default(), vec![],
        )
    }

    pub fn center(w_spacing: f32, h_spacing: f32) -> Self {Wrap::new(w_spacing, h_spacing)}

    fn lines(&self, width: f32) -> Vec<(Vec<Item>, f32, f32)> {
        let width = (width - self.4.0 - self.4.2).max(0.0);
        let mut lines = Vec::new();
        let mut items = Vec::new();
        let mut line_width = 0.0;
        let mut line_height = 0.0_f32;
        for child in &self.5 {
            let item = Item {w: child.min_width, h: child.min_height};
            let next_width = if items.is_empty() {item.w} else {line_width + self.0 + item.w};
            if !items.is_empty() && next_width > width {
                lines.push((items, line_width, line_height));
                items = vec![item];
                line_width = item.w;
                line_height = item.h;
            } else {
                items.push(item);
                line_width = next_width;
                line_height = line_height.max(item.h);
            }
        }
        if !items.is_empty() {lines.push((items, line_width, line_height));}
        lines
    }
}

impl PartialEq for Wrap {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
        && self.1 == other.1
        && self.2 == other.2
        && self.3 == other.3
        && self.4 == other.4
    }
}

impl Layout for Wrap {
    fn request_width(&mut self, children: Vec<(f32, f32)>) -> (f32, f32) {
        let padding = self.4.0 + self.4.2;
        let width = children.iter().map(|child| child.0).fold(0.0_f32, f32::max) + padding;
        self.5 = children.into_iter().map(|(min_width, max_width)| SizeRange {
            min_width, max_width, ..SizeRange::default()
        }).collect();
        (width, if self.5.is_empty() {width} else {f32::MAX})
    }

    fn assign_width(&mut self, _width: f32) -> Vec<f32> {
        self.5.iter_mut().map(|child| {
            child.max_width = child.min_width;
            child.min_width
        }).collect()
    }

    fn request_height(&mut self, width: f32, children: Vec<(f32, f32)>) -> (f32, f32) {
        for (child, (min, max)) in self.5.iter_mut().zip(children) {
            child.min_height = min;
            child.max_height = max;
        }
        let lines = self.lines(width);
        let height = lines.iter().map(|(_, _, height)| height).sum::<f32>()
            + self.1 * lines.len().saturating_sub(1) as f32 + self.4.1 + self.4.3;
        (height, height)
    }

    fn layout(&mut self, size: (f32, f32)) -> Vec<Area> {
        let content_width = (size.0 - self.4.0 - self.4.2).max(0.0);
        let mut areas = Vec::new();
        let mut y = self.4.1;
        for (line, line_w, line_h) in self.lines(size.0) {
            let extra_x = (content_width - line_w).max(0.0);
            let mut x = self.4.0 + match self.2 {
                Offset::Start => 0.0,
                Offset::End => extra_x,
                Offset::Center => extra_x / 2.0,
                Offset::Static(x) => x,
            };
            for item in line {
                let child_y = match self.3 {
                    Offset::Start => y,
                    Offset::End => y + (line_h - item.h).max(0.0),
                    Offset::Center => y + (line_h - item.h).max(0.0) / 2.0,
                    Offset::Static(v) => y + v,
                };

                areas.push(Area {offset: (x, child_y), size: (item.w, item.h)});
                x += item.w + self.0;
            }
            y += line_h + self.1;
        }
        areas
    }
}

#[derive(Clone, Copy)]
struct Item {
    w: f32,
    h: f32,
}
