use core::slice;
use std::iter::Peekable;

use crate::{
    color::Hsla,
    geometry::{Bounds, Corners, Edges, ScaledPixels},
};

pub(crate) type DrawOrder = u32;

pub(crate) struct Scene {
    pub quads: Vec<Quad>,
    pub monochrome_sprites: Vec<MonochromeSprite>,
}

impl Scene {
    pub(crate) fn batches(&self) -> impl IntoIterator<Item = PrimitiveBatch> {
        BatchIterator {
            quads: &self.quads,
            quads_start: 0,
            quads_iter: self.quads.iter().peekable(),
            monochrome_sprites: &self.monochrome_sprites,
            monochrome_sprites_start: 0,
            monochrome_sprites_iter: self.monochrome_sprites.iter().peekable(),
        }
    }
}

#[derive(Clone, Copy, Default, Ord, PartialEq, Eq, PartialOrd)]
pub(crate) enum PrimitiveKind {
    #[default]
    Quad,
    MonochromeSprite,
}

#[derive(Clone, Ord, PartialEq, Eq, PartialOrd)]
pub(crate) enum Primitive {
    Quad(Quad),
    MonochromeSprite(MonochromeSprite),
}

struct BatchIterator<'a> {
    quads: &'a [Quad],
    quads_start: usize,
    quads_iter: Peekable<slice::Iter<'a, Quad>>,
    monochrome_sprites: &'a [MonochromeSprite],
    monochrome_sprites_start: usize,
    monochrome_sprites_iter: Peekable<slice::Iter<'a, MonochromeSprite>>,
}

impl<'a> Iterator for BatchIterator<'a> {
    type Item = PrimitiveBatch<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut orders_and_kinds = [
            (self.quads_iter.peek().map(|q| q.order), PrimitiveKind::Quad),
            (
                self.monochrome_sprites_iter.peek().map(|s| s.order),
                PrimitiveKind::MonochromeSprite,
            ),
        ];

        orders_and_kinds.sort_by_key(|(order, kind)| (order.unwrap_or(u32::MAX), *kind));

        let first = orders_and_kinds[0];
        let second = orders_and_kinds[1];
        let (batch_kind, max_order_and_kind) = if first.0.is_some() {
            (first.1, (second.0.unwrap_or(u32::MAX), second.1))
        } else {
            return None;
        };

        match batch_kind {
            PrimitiveKind::Quad => {
                let quads_start = self.quads_start;
                let mut quads_end = self.quads_start + 1;
                self.quads_iter.next();
                while self
                    .quads_iter
                    .next_if(|quad| (quad.order, batch_kind) < max_order_and_kind)
                    .is_some()
                {
                    quads_end += 1;
                }
                self.quads_start = quads_end;
                Some(PrimitiveBatch::Quads(&self.quads[quads_start..quads_end]))
            }
            PrimitiveKind::MonochromeSprite => todo!(),
        }
    }
}

pub(crate) enum PrimitiveBatch<'a> {
    Quads(&'a [Quad]),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub(crate) struct Quad {
    pub order: DrawOrder,
    pub pad: u32, // align to 8 bytes
    pub bounds: Bounds<ScaledPixels>,
    pub background: Hsla,
    pub border_color: Hsla,
    pub corner_radii: Corners<ScaledPixels>,
    pub border_widths: Edges<ScaledPixels>,
}

impl Ord for Quad {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order.cmp(&other.order)
    }
}

impl PartialOrd for Quad {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl From<Quad> for Primitive {
    fn from(quad: Quad) -> Self {
        Primitive::Quad(quad)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct MonochromeSprite {
    pub order: DrawOrder,
    pub bounds: Bounds<ScaledPixels>,
    pub color: Hsla,
}

impl Ord for MonochromeSprite {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order.cmp(&other.order)
    }
}

impl PartialOrd for MonochromeSprite {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl From<MonochromeSprite> for Primitive {
    fn from(sprite: MonochromeSprite) -> Self {
        Primitive::MonochromeSprite(sprite)
    }
}
