//! Like [`iced::widget_button`] but supports right and middle clicks as well.

use iced::overlay;
use iced_core::{Background, Color, Vector};
use iced_native::{Clipboard, Element, Event, event, Layout, layout, Length, mouse, Padding, Point, Rectangle, renderer, Shell, touch, Widget};
use iced_native::widget::{Operation, Tree, tree};
use iced_native::widget::button::{layout, mouse_interaction};
use iced_style::button::{Appearance, StyleSheet};

impl<'a, Message, Renderer> From<ClickButton<'a, Message, Renderer>> for Element<'a, Message, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_native::Renderer + 'a,
        Renderer::Theme: StyleSheet,
{
    fn from(value: ClickButton<'a, Message, Renderer>) -> Self {
        Element::new(value)
    }
}

pub struct ClickButton<'a, Message, Renderer>
    where
        Renderer: iced_native::Renderer,
        Renderer::Theme: StyleSheet,
{
    content: Element<'a, Message, Renderer>,
    left_press: Option<Message>,
    right_press: Option<Message>,
    middle_press: Option<Message>,
    width: Length,
    height: Length,
    padding: Padding,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> ClickButton<'a, Message, Renderer>
    where
        Renderer: iced_native::Renderer,
        Renderer::Theme: StyleSheet,
{
    /// Creates a new [`ClickButton`] with the given content.
    pub fn new<C: Into<Element<'a, Message, Renderer>>>(content: C) -> Self {
        Self {
            content: content.into(),
            left_press: None,
            right_press: None,
            middle_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::new(5.0),
            style: Default::default(),
        }
    }

    /// Sets the width of the [`ClickButton`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`ClickButton`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`Padding`] of the [`ClickButton`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the message that will be produced when the [`ClickButton`] is pressed with the left
    /// (main) mouse button.
    ///
    /// Unless one of `on_left_press`, `on_left_press`, or `on_left_press` is called, the
    /// [`ClickButton`] will be disabled.
    pub fn on_left_press(mut self, msg: Message) -> Self {
        self.left_press = Some(msg);
        self
    }

    /// Sets the message that will be produced when the [`ClickButton`] is pressed with the right
    /// mouse button.
    ///
    /// Unless one of `on_left_press`, `on_left_press`, or `on_left_press` is called, the
    /// [`ClickButton`] will be disabled.
    pub fn on_right_press(mut self, msg: Message) -> Self {
        self.right_press = Some(msg);
        self
    }

    /// Sets the message that will be produced when the [`ClickButton`] is pressed with the middle
    /// mouse button.
    ///
    /// Unless one of `on_left_press`, `on_left_press`, or `on_left_press` is called, the
    /// [`ClickButton`] will be disabled.
    pub fn on_middle_press(mut self, msg: Message) -> Self {
        self.middle_press = Some(msg);
        self
    }

    /// Sets the style variant of this [`ClickButton`].
    pub fn style(
        mut self,
        style: <Renderer::Theme as StyleSheet>::Style,
    ) -> Self {
        self.style = style;
        self
    }

    fn is_enabled(&self) -> bool {
        self.left_press.is_some() || self.middle_press.is_some() || self.right_press.is_some()
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for ClickButton<'a, Message, Renderer>
    where
        Message: 'a + Clone,
        Renderer: 'a + iced_native::Renderer,
        Renderer::Theme: StyleSheet,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
            renderer,
            limits,
            self.width,
            self.height,
            self.padding,
            |renderer, limits| {
                self.content.as_widget().layout(renderer, limits)
            },
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();

        let styling = draw(
            renderer,
            bounds,
            cursor_position,
            self.is_enabled(),
            theme,
            &self.style,
            || tree.state.downcast_ref::<State>(),
        );

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: styling.text_color,
            },
            content_layout,
            cursor_position,
            &bounds,
        );
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        if let event::Status::Captured = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
        ) {
            return event::Status::Captured;
        }

        update(
            &event,
            layout,
            cursor_position,
            shell,
            &self.left_press,
            &self.right_press,
            &self.middle_press,
            || tree.state.downcast_mut::<State>(),
        )
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse_interaction(layout, cursor_position, self.is_enabled())
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
        )
    }
}

/// The local state of a [`ClickButton`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_middle_pressed: bool,
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> State {
        Self::default()
    }

    fn is_pressed(&mut self, button: mouse::Button) -> &mut bool {
        match button {
            mouse::Button::Left => &mut self.is_left_pressed,
            mouse::Button::Right => &mut self.is_right_pressed,
            mouse::Button::Middle => &mut self.is_middle_pressed,
            mouse::Button::Other(_) => {
                todo!("maybe just ignore?")
            }
        }
    }

    fn any_pressed(self) -> bool {
        self.is_left_pressed || self.is_right_pressed || self.is_middle_pressed
    }
}

/// Processes the given [`Event`] and updates the [`State`] of a [`ClickButton`]
/// accordingly.
pub fn update<'a, Message: Clone>(
    event: &Event,
    layout: Layout<'_>,
    cursor_position: Point,
    shell: &mut Shell<'_, Message>,
    on_left_press: &Option<Message>,
    on_right_press: &Option<Message>,
    on_middle_press: &Option<Message>,
    state: impl FnOnce() -> &'a mut State,
) -> event::Status {
    fn button_pressed<'a, Message: Clone>(
        on_press: &Option<Message>,
        button: mouse::Button,
        layout: Layout<'_>,
        cursor_position: Point,
        state: impl FnOnce() -> &'a mut State,
    ) -> event::Status {
        if on_press.is_some() {
            let bounds = layout.bounds();
            if bounds.contains(cursor_position) {
                let state = state();

                *state.is_pressed(button) = true;

                event::Status::Captured
            } else {
                event::Status::Ignored
            }
        } else {
            event::Status::Ignored
        }
    }
    fn button_released<'a, Message: Clone>(
        on_press: &Option<Message>,
        button: mouse::Button,
        layout: Layout<'_>,
        cursor_position: Point,
        state: impl FnOnce() -> &'a mut State,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        if let Some(on_press) = on_press.clone() {
            let state = state();
            let is_pressed = state.is_pressed(button);
            if *is_pressed {
                *is_pressed = false;
                let bounds = layout.bounds();
                if bounds.contains(cursor_position) {
                    shell.publish(on_press);
                }
                event::Status::Captured
            } else {
                event::Status::Ignored
            }
        } else {
            event::Status::Ignored
        }
    }

    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) =>
            button_pressed(on_left_press, mouse::Button::Left, layout, cursor_position, state),
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) =>
            button_pressed(on_right_press, mouse::Button::Right, layout, cursor_position, state),
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) =>
            button_pressed(on_middle_press, mouse::Button::Middle, layout, cursor_position, state),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. }) =>
            button_released(on_left_press, mouse::Button::Left, layout, cursor_position, state, shell),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)) =>
            button_released(on_right_press, mouse::Button::Right, layout, cursor_position, state, shell),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) =>
            button_released(on_middle_press, mouse::Button::Middle, layout, cursor_position, state, shell),
        Event::Touch(touch::Event::FingerLost { .. }) => {
            let state = state();

            state.is_left_pressed = false;

            event::Status::Ignored
        }
        _ => event::Status::Ignored
    }
}

/// Draws a [`ClickButton`].
pub fn draw<'a, Renderer: iced_native::Renderer>(
    renderer: &mut Renderer,
    bounds: Rectangle,
    cursor_position: Point,
    is_enabled: bool,
    style_sheet: &dyn StyleSheet<Style=<Renderer::Theme as StyleSheet>::Style, >,
    style: &<Renderer::Theme as StyleSheet>::Style,
    state: impl FnOnce() -> &'a State,
) -> Appearance
    where
        Renderer::Theme: StyleSheet,
{
    let is_mouse_over = bounds.contains(cursor_position);

    let styling = if !is_enabled {
        style_sheet.disabled(style)
    } else if is_mouse_over {
        let state = state();

        if state.any_pressed() {
            style_sheet.pressed(style)
        } else {
            style_sheet.hovered(style)
        }
    } else {
        style_sheet.active(style)
    };

    if styling.background.is_some() || styling.border_width > 0.0 {
        if styling.shadow_offset != Vector::default() {
            // TODO: Implement proper shadow support
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x + styling.shadow_offset.x,
                        y: bounds.y + styling.shadow_offset.y,
                        ..bounds
                    },
                    border_radius: styling.border_radius.into(),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
                Background::Color([0.0, 0.0, 0.0, 0.5].into()),
            );
        }

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: styling.border_radius.into(),
                border_width: styling.border_width,
                border_color: styling.border_color,
            },
            styling
                .background
                .unwrap_or(Background::Color(Color::TRANSPARENT)),
        );
    }

    styling
}
