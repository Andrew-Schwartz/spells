use std::hash::Hash;

use iced::{Point, Rectangle, Size};
use iced_native::{Clipboard, Element, Event, Hasher, Layout, layout, Widget};
use iced_native::event::Status;
use iced_native::layout::{Limits, Node};

use crate::Length;

pub struct Slots<Renderer: self::Renderer> {
    pub max: u32,
    pub current: u32,
    pub width: Length,
    // pub height: Length,
    pub padding: u16,
    pub style: Renderer::Style,
}

impl<Message, Renderer: self::Renderer> Widget<Message, Renderer> for Slots<Renderer> {
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Units(Renderer::DEFAULT_HEIGHT)
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &Limits,
    ) -> Node {
        let limits = limits.width(self.width)
            .height(Length::Units(Renderer::DEFAULT_HEIGHT));

        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &iced_native::renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> iced_native::renderer::Output {
        renderer.draw(
            layout.bounds(),
            self.max,
            self.current,
            &self.style,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height().hash(state);
    }

    fn on_event(
        &mut self, _event: Event,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _messages: &mut Vec<Message>,
    ) -> Status {
        todo!()
    }
}

pub trait Renderer: iced_native::Renderer {
    type Style: Default;

    const DEFAULT_HEIGHT: u16;

    fn draw(
        &self,
        bounds: Rectangle,
        max: u32,
        current: u32,
        style: &Self::Style,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Slots<Renderer>> for Element<'a, Message, Renderer>
    where
        Renderer: 'a + self::Renderer,
        Message: 'a,
{
    fn from(slots: Slots<Renderer>) -> Self {
        Element::new(slots)
    }
}