use std::sync::{Arc, Mutex};

use prism::drawable::{Area, Layout, SizeRange};

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

    pub fn adjust_request(&self, request: SizeRange) -> SizeRange {
        let wp = self.0+self.2;
        let hp = self.1+self.3;
        SizeRange {
            min_width: request.min_width + wp, max_width: request.max_width + wp,
            min_height: request.min_height + hp, max_height: request.max_height + hp,
        }
    }
}

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

#[derive(Debug, Clone, Copy)]
pub struct DefaultStack;
impl Layout for DefaultStack {
    fn size_range(&self, children: Vec<SizeRange>) -> SizeRange {
        children.into_iter().reduce(|c, o| SizeRange {
            min_width: c.min_width.max(o.min_width), max_width: c.max_width.max(o.max_width),
            min_height: c.min_height.max(o.min_height), max_height: c.max_height.max(o.max_height),
        }).unwrap()
    }

    fn layout(&self, size: (f32, f32), children: Vec<SizeRange>) -> Vec<Area> {
        children.into_iter().map(|c| Area{offset: (0.0, 0.0), size: c.clamp(size)}).collect()
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

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Row(f32, Offset, Size, Padding);

impl Row {
    pub fn new(spacing: f32, offset: Offset, size: Size, padding: Padding) -> Self {
        Row(spacing, offset, size, padding)
    }

    pub fn center(spacing: f32) -> Self {
        Row::new(spacing, Offset::Center, Size::Fit, Padding::default())
    }

    pub fn start(spacing: f32) -> Self {
        Row::new(spacing, Offset::Start, Size::Fit, Padding::default())
    }

    pub fn end(spacing: f32) -> Self {
        Row::new(spacing, Offset::End, Size::Fit, Padding::default())
    }

    pub fn padding(&mut self) -> &mut Padding {&mut self.3}
}

impl Layout for Row {
    fn size_range(&self, children: Vec<SizeRange>) -> SizeRange {
        let (widths, heights): (Vec<_>, Vec<_>) = children.into_iter().map(|i|
            ((i.min_width, i.max_width), (i.min_height, i.max_height))
        ).unzip();
        let spacing = self.0 * (widths.len() - 1) as f32;
        let width = Size::add(widths);
        let height = self.2.get(heights, Size::max);
        self.3.adjust_request(SizeRange {
            min_width: width.0 + spacing, max_width: width.1 + spacing,
            min_height: height.0, max_height: height.1,
        })
    }

    fn layout(&self, row_size: (f32, f32), children: Vec<SizeRange>) -> Vec<Area> {
        let row_size = self.3.adjust_size(row_size);
        let widths = UniformExpand::get(
            children.iter().map(|i| (i.min_width, i.max_width)).collect(), row_size.0, self.0
        );
        let mut offset = 0.0;
        children.into_iter().zip(widths).map(|(i, width)| {
            let size = i.clamp((width, row_size.1));
            let off = self.3.adjust_offset((offset, self.1.get(row_size.1, size.1)));
            if size.0 > 0.0 {offset += size.0+self.0;}
            Area{offset: off, size}
        }).collect()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Column(f32, Offset, Size, Padding, Option<Arc<Mutex<f32>>>, ScrollAnchor);

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
            scroll.is_some().then_some(Arc::new(Mutex::new(0.0))), scroll.unwrap_or_default()
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
    fn size_range(&self, children: Vec<SizeRange>) -> SizeRange {
        if children.is_empty() { return SizeRange::default() };
        let (widths, heights): (Vec<_>, Vec<_>) = children.into_iter().map(|i|
            ((i.min_width, i.max_width), (i.min_height, i.max_height))
        ).unzip();
        let visible = heights.iter().filter(|h| h.1 > 0.0).count();
        let spacing = self.0 * visible.saturating_sub(1) as f32;
        let width = self.2.get(widths, Size::max);
        let height = Size::add(heights);
        self.3.adjust_request(SizeRange {
            min_width: if self.4.is_some() {0.0} else {width.0}, max_width: width.1,
            min_height: (if self.4.is_some() {0.0} else {height.0}) + spacing,
            max_height: height.1 + spacing,
        })
    }

    fn layout(&self, col_size: (f32, f32), mut children: Vec<SizeRange>) -> Vec<Area> {
        let col_size = self.3.adjust_size(col_size);

        let ranges: Vec<_> = children.iter().map(|c| (c.min_height, c.max_height)).collect();
        let visible: Vec<usize> = (0..ranges.len()).filter(|&i| ranges[i].1 > 0.0).collect();
        let expanded = UniformExpand::get(
            visible.iter().map(|&i| ranges[i]).collect(), col_size.1, self.0
        );
        let mut heights = vec![0.0; ranges.len()];
        visible.iter().zip(expanded).for_each(|(&i, h)| heights[i] = h);

        let spacing = self.0 * visible.len().saturating_sub(1) as f32;
        let content_height = heights.iter().sum::<f32>() + spacing;

        let mut offset = 0.0;

        let max_scroll = (content_height - col_size.1).max(0.0);
        let scroll = self.4.as_ref().map(|s| {
            let mut v = s.lock().unwrap();
            *v = v.clamp(0.0, max_scroll);
            *v
        }).unwrap_or(0.0);

        let is_end = self.5 == ScrollAnchor::End;
        if is_end { children.reverse(); }

        let mut areas: Vec<_> = children.into_iter().zip(heights).map(|(child, h)| {
            let size = child.clamp((col_size.0, h));
            let off_y = offset;
            if child.max_height > 0.0 { offset += size.1 + self.0; }
            let n = if is_end {col_size.1 - h + scroll} else {off_y - scroll};
            Area { offset: self.3.adjust_offset((self.1.get(col_size.0, size.0), n)), size }
        }).collect();

        if is_end { areas.reverse(); }
        areas
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Stack(pub Offset, pub Offset, pub Size, pub Size, pub Padding);

impl Stack {
    pub fn new(x_offset: Offset, y_offset: Offset, x_size: Size, y_size: Size, padding: Padding) -> Self {
        Stack(x_offset, y_offset, x_size, y_size, padding)
    }

    pub fn center() -> Self {
        Stack(Offset::Center, Offset::Center, Size::Fit, Size::Fit, Padding::default())
    }

    pub fn start() -> Self {
        Stack(Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding::default())
    }

    pub fn end() -> Self {
        Stack(Offset::End, Offset::End, Size::Fit, Size::Fit, Padding::default())
    }

    pub fn fill() -> Self {
        Stack(Offset::Center, Offset::Center, Size::Fill, Size::Fill, Padding::default())
    }
}

impl Layout for Stack {
    fn size_range(&self, children: Vec<SizeRange>) -> SizeRange {
        let (widths, heights): (Vec<_>, Vec<_>) = children.into_iter().map(|r|
            ((r.min_width, r.max_width), (r.min_height, r.max_height))
        ).unzip();
        let width = self.2.get(widths, Size::max);
        let height = self.3.get(heights, Size::max);
        self.4.adjust_request(SizeRange {
            min_width: width.0, max_width: width.1, min_height: height.0, max_height: height.1,
        })
    }

    fn layout(&self, stack_size: (f32, f32), children: Vec<SizeRange>) -> Vec<Area> {
        let stack_size = self.4.adjust_size(stack_size);
        children.into_iter().map(|i| {
            let size = i.clamp(stack_size);
            let offset = (self.0.get(stack_size.0, size.0), self.1.get(stack_size.1, size.1));
            Area{offset: self.4.adjust_offset(offset), size}
        }).collect()
    }
}

#[derive(Debug, Clone)]
pub struct Wrap(pub f32, pub f32, pub Offset, pub Offset, pub Padding, Arc<Mutex<f32>>);

impl Wrap {
    pub fn new(w_spacing: f32, h_spacing: f32) -> Self {
        Wrap(
            w_spacing, h_spacing, Offset::Center, Offset::Center,
            Padding::default(), Arc::new(Mutex::new(0.0))
        )
    }

    pub fn start(w_spacing: f32, h_spacing: f32) -> Self {
        Wrap(
            w_spacing, h_spacing, Offset::Start, Offset::Center,
            Padding::default(), Arc::new(Mutex::new(0.0))
        )
    }

    pub fn end(w_spacing: f32, h_spacing: f32) -> Self {
        Wrap(
            w_spacing, h_spacing, Offset::End, Offset::Center,
            Padding::default(), Arc::new(Mutex::new(0.0))
        )
    }

    pub fn center(w_spacing: f32, h_spacing: f32) -> Self {
        Wrap::new(w_spacing, h_spacing)
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
    fn size_range(&self, children: Vec<SizeRange>) -> SizeRange {
        let available_width = *self.5.lock().unwrap();

        let left = self.4.1;
        let right = self.4.2;
        let top = self.4.0;
        let bottom = self.4.3;

        if children.is_empty() {
            let w = left + right;
            let h = top + bottom;
            return SizeRange::fixed((w, h));
        }

        let widest_child = children.iter().map(|c| c.min_width).fold(0.0_f32, f32::max);
        let tallest_child = children.iter().map(|c| c.min_height).fold(0.0_f32, f32::max);

        let min_width = widest_child + left + right;

        if available_width <= 0.0 {
            let min_height = tallest_child + top + bottom;
            return SizeRange {min_width, min_height, max_width: f32::MAX, max_height: f32::MAX};
        }

        let content_width = (available_width - left - right).max(0.0);

        let mut line_w = 0.0_f32;
        let mut line_h = 0.0_f32;
        let mut total_h = top;
        let mut has_any = false;

        for child in &children {
            let w = child.min_width;
            let h = child.min_height;

            let projected_w = if line_w == 0.0 { w } else { line_w + self.0 + w };

            if line_w > 0.0 && projected_w > content_width {
                total_h += line_h + self.1;

                line_w = w;
                line_h = h;
            } else {
                line_w = projected_w;
                line_h = line_h.max(h);
            }

            has_any = true;
        }

        if has_any {
            total_h += line_h;
        }

        let min_height = total_h + bottom;
        let max_width = available_width.max(min_width);
        let max_height = min_height;

        SizeRange {min_width, min_height, max_width, max_height}
    }

    fn layout(&self, maximum_size: (f32, f32), children: Vec<SizeRange>) -> Vec<Area> {
        let left = self.4.1;
        let right = self.4.2;
        let top = self.4.0;

        let content_width = (maximum_size.0 - left - right).max(0.0);

        if let Ok(mut stored) = self.5.lock() {
            *stored = maximum_size.0;
        }

        if children.is_empty() {
            return Vec::new();
        }

        let mut lines: Vec<Vec<Item>> = Vec::new();
        let mut line_widths: Vec<f32> = Vec::new();
        let mut line_heights: Vec<f32> = Vec::new();

        let mut current_line: Vec<Item> = Vec::new();
        let mut current_w = 0.0_f32;
        let mut current_h = 0.0_f32;

        for child in children {
            let item = Item {
                w: child.min_width,
                h: child.min_height,
            };

            let projected_w = if current_line.is_empty() {
                item.w
            } else {
                current_w + self.0 + item.w
            };

            if !current_line.is_empty() && projected_w > content_width {
                lines.push(current_line);
                line_widths.push(current_w);
                line_heights.push(current_h);

                current_line = vec![item];
                current_w = item.w;
                current_h = item.h;
            } else {
                if current_line.is_empty() {
                    current_w = item.w;
                } else {
                    current_w += self.0 + item.w;
                }
                current_h = current_h.max(item.h);
                current_line.push(item);
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
            line_widths.push(current_w);
            line_heights.push(current_h);
        }

        let mut areas = Vec::new();
        let mut y = top;

        for ((line, &line_w), &line_h) in lines.iter().zip(&line_widths).zip(&line_heights) {
            let extra_x = (content_width - line_w).max(0.0);

            let start_x = match self.2 {
                Offset::Start => left,
                Offset::End => left + extra_x,
                Offset::Center => left + extra_x / 2.0,
                Offset::Static(x) => left + x,
            };

            let mut x = start_x;

            for item in line {
                let child_y = match self.3 {
                    Offset::Start => y,
                    Offset::End => y + (line_h - item.h).max(0.0),
                    Offset::Center => y + (line_h - item.h).max(0.0) / 2.0,
                    Offset::Static(v) => y + v,
                };

                areas.push(Area {
                    offset: (x, child_y),
                    size: (item.w, item.h),
                });

                x += item.w + self.0;
            }

            y += line_h + self.1;
        }

        areas
    }
}

type FitFunc = fn(Vec<(f32, f32)>) -> (f32, f32);

#[derive(Clone, Copy)]
struct Item {w: f32, h: f32}
