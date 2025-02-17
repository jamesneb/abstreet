use instant::Instant;
use winit::event::{
    ElementState, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};

use geom::Duration;

use crate::{EventCtx, Line, ScreenDims, ScreenPt, TextSpan};

// Mouse-up events longer than this will be considered single clicks
// Ideally the delay would be a little more tolerant - e.g. 500ms, but because we don't actually
// have a way to indicate that a single click was handled (and thus *shouldn't* be counted as part of a double click)
// it's too easy to have false positives.
const MAX_DOUBLE_CLICK_DURATION: instant::Duration = instant::Duration::from_millis(300);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    // Used to initialize the application and also to recalculate menu state when some other event
    // is used.
    NoOp,
    LeftMouseButtonDown,
    /// Note: When double clicking, there will be two `LeftMouseButtonUp` events in short
    /// succession - first a `LeftMouseButtonUp { is_double_click: false }`, followed by
    /// a `LeftMouseButtonUp { is_double_click: true }`.
    ///
    /// This was done for ease of implementation - it allows a target to ignore single clicks and
    /// handle double clicks (or vice versa), but it precludes an obvious way to have a target
    /// handle single clicks one way while handling double clicks a different way.
    ///
    /// e.g. a typical file browser highlights a file with a single click and opens the file with a
    /// double click, the way we've implemented double clicks here wouldn't work well for that
    /// case.
    LeftMouseButtonUp {
        is_double_click: bool,
    },
    RightMouseButtonDown,
    RightMouseButtonUp,
    // TODO KeyDown and KeyUp might be nicer, but piston (and probably X.org) hands over repeated
    // events while a key is held down.
    KeyPress(Key),
    KeyRelease(Key),
    // Some real amount of time has passed since the last update
    Update(Duration),
    MouseMovedTo(ScreenPt),
    WindowLostCursor,
    WindowGainedCursor,
    MouseWheelScroll(f64, f64),
    WindowResized(ScreenDims),
}

impl Event {
    pub fn from_winit_event(
        ev: WindowEvent,
        scale_factor: f64,
        previous_click: Instant,
    ) -> Option<Event> {
        match ev {
            WindowEvent::MouseInput { state, button, .. } => match (button, state) {
                (MouseButton::Left, ElementState::Pressed) => Some(Event::LeftMouseButtonDown),
                (MouseButton::Left, ElementState::Released) => {
                    let is_double_click = previous_click.elapsed().le(&MAX_DOUBLE_CLICK_DURATION);
                    Some(Event::LeftMouseButtonUp { is_double_click })
                }
                (MouseButton::Right, ElementState::Pressed) => Some(Event::RightMouseButtonDown),
                (MouseButton::Right, ElementState::Released) => Some(Event::RightMouseButtonUp),
                _ => None,
            },
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key) = Key::from_winit_key(input) {
                    if input.state == ElementState::Pressed {
                        Some(Event::KeyPress(key))
                    } else {
                        Some(Event::KeyRelease(key))
                    }
                } else {
                    None
                }
            }
            WindowEvent::CursorMoved { position, .. } => Some(Event::MouseMovedTo(
                position.to_logical(scale_factor).into(),
            )),
            WindowEvent::MouseWheel { delta, .. } => match delta {
                // "In the beginning" a spinnable mouse wheel was the only input hardware for
                // scrolling. Each "increment" of the mouse wheel indicated that the application
                // should scroll one line of text.
                //
                // Since the advent of touchpads and tablets, much finer grained scrolling is used,
                // and consequently some input systems started expressing scroll distances from such
                // input devices in "pixels" rather than "lines".
                //
                // However some backends (e.g. x11) will express all scrolling as `LineDelta` — on
                // those systems, touchpad drags will simply be scaled to some presumed equivalent
                // number of lines.
                //
                // Widgetry expresses all scrolling in terms of MouseWheelScroll, which uses
                // "Lines".
                //
                // Anymore "a line" usually doesn't correspond to "a literal line of text" in the
                // application, and must usually just be considered an abstract unit of
                // measurement, because of things like configurable scroll sensitivity, variable
                // text size, and the afforementioned advent of touchpads.
                //
                // With "Reverse" scrolling, positive values indicate upward or rightward scrolling.
                // With "Natural" scrolling, it's the opposite.
                MouseScrollDelta::LineDelta(dx, dy) => {
                    if dx == 0.0 && dy == 0.0 {
                        None
                    } else {
                        Some(Event::MouseWheelScroll(f64::from(dx), f64::from(dy)))
                    }
                }
                MouseScrollDelta::PixelDelta(pos) => {
                    // Widgetry expresses all scroll activity in units of "lines", so convert from
                    // a PixelDelta to a LineDelta.

                    // This scale factor is just a guess - but feels about right.
                    let scale_factor = 0.1;

                    Some(Event::MouseWheelScroll(
                        scale_factor * pos.x,
                        scale_factor * pos.y,
                    ))
                }
            },
            WindowEvent::Resized(size) => {
                Some(Event::WindowResized(size.to_logical(scale_factor).into()))
            }
            WindowEvent::Focused(gained) => Some(if gained {
                Event::WindowGainedCursor
            } else {
                Event::WindowLostCursor
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Key {
    // Case is unspecified.
    // TODO Would be cool to represent A and UpperA, but then release semantics get weird... hold
    // shift and A, release shift -- does that trigger a Release(UpperA) and a Press(A)?
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    // Numbers (not the numpad)
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,
    // symbols
    // TODO shift+number keys
    LeftBracket,
    RightBracket,
    Space,
    Slash,
    Dot,
    Comma,
    Semicolon,
    Colon,
    Equals,
    SingleQuote,
    // Stuff without a straightforward single-character display
    Escape,
    Enter,
    Tab,
    Backspace,
    LeftShift,
    LeftControl,
    LeftAlt,
    RightAlt,
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

impl Key {
    pub const NUM_KEYS: [Key; 9] = [
        Key::Num1,
        Key::Num2,
        Key::Num3,
        Key::Num4,
        Key::Num5,
        Key::Num6,
        Key::Num7,
        Key::Num8,
        Key::Num9,
    ];

    pub fn to_char(self, shift_pressed: bool) -> Option<char> {
        match self {
            Key::A => Some(if shift_pressed { 'A' } else { 'a' }),
            Key::B => Some(if shift_pressed { 'B' } else { 'b' }),
            Key::C => Some(if shift_pressed { 'C' } else { 'c' }),
            Key::D => Some(if shift_pressed { 'D' } else { 'd' }),
            Key::E => Some(if shift_pressed { 'E' } else { 'e' }),
            Key::F => Some(if shift_pressed { 'F' } else { 'f' }),
            Key::G => Some(if shift_pressed { 'G' } else { 'g' }),
            Key::H => Some(if shift_pressed { 'H' } else { 'h' }),
            Key::I => Some(if shift_pressed { 'I' } else { 'i' }),
            Key::J => Some(if shift_pressed { 'J' } else { 'j' }),
            Key::K => Some(if shift_pressed { 'K' } else { 'k' }),
            Key::L => Some(if shift_pressed { 'L' } else { 'l' }),
            Key::M => Some(if shift_pressed { 'M' } else { 'm' }),
            Key::N => Some(if shift_pressed { 'N' } else { 'n' }),
            Key::O => Some(if shift_pressed { 'O' } else { 'o' }),
            Key::P => Some(if shift_pressed { 'P' } else { 'p' }),
            Key::Q => Some(if shift_pressed { 'Q' } else { 'q' }),
            Key::R => Some(if shift_pressed { 'R' } else { 'r' }),
            Key::S => Some(if shift_pressed { 'S' } else { 's' }),
            Key::T => Some(if shift_pressed { 'T' } else { 't' }),
            Key::U => Some(if shift_pressed { 'U' } else { 'u' }),
            Key::V => Some(if shift_pressed { 'V' } else { 'v' }),
            Key::W => Some(if shift_pressed { 'W' } else { 'w' }),
            Key::X => Some(if shift_pressed { 'X' } else { 'x' }),
            Key::Y => Some(if shift_pressed { 'Y' } else { 'y' }),
            Key::Z => Some(if shift_pressed { 'Z' } else { 'z' }),
            Key::Num1 => Some(if shift_pressed { '!' } else { '1' }),
            Key::Num2 => Some(if shift_pressed { '@' } else { '2' }),
            Key::Num3 => Some(if shift_pressed { '#' } else { '3' }),
            Key::Num4 => Some(if shift_pressed { '$' } else { '4' }),
            Key::Num5 => Some(if shift_pressed { '%' } else { '5' }),
            Key::Num6 => Some(if shift_pressed { '^' } else { '6' }),
            Key::Num7 => Some(if shift_pressed { '&' } else { '7' }),
            Key::Num8 => Some(if shift_pressed { '*' } else { '8' }),
            Key::Num9 => Some(if shift_pressed { '(' } else { '9' }),
            Key::Num0 => Some(if shift_pressed { ')' } else { '0' }),
            Key::LeftBracket => Some(if shift_pressed { '{' } else { '[' }),
            Key::RightBracket => Some(if shift_pressed { '}' } else { ']' }),
            Key::Space => Some(' '),
            Key::Slash => Some(if shift_pressed { '?' } else { '/' }),
            Key::Dot => Some(if shift_pressed { '>' } else { '.' }),
            Key::Comma => Some(if shift_pressed { '<' } else { ',' }),
            Key::Semicolon => Some(';'),
            Key::Colon => Some(':'),
            Key::Equals => Some(if shift_pressed { '+' } else { '=' }),
            Key::SingleQuote => Some(if shift_pressed { '"' } else { '\'' }),
            Key::Escape
            | Key::Enter
            | Key::Tab
            | Key::Backspace
            | Key::LeftShift
            | Key::LeftControl
            | Key::LeftAlt
            | Key::RightAlt
            | Key::LeftArrow
            | Key::RightArrow
            | Key::UpArrow
            | Key::DownArrow
            | Key::F1
            | Key::F2
            | Key::F3
            | Key::F4
            | Key::F5
            | Key::F6
            | Key::F7
            | Key::F8
            | Key::F9
            | Key::F10
            | Key::F11
            | Key::F12 => None,
        }
    }

    pub fn describe(self) -> String {
        match self {
            Key::Escape => "Escape".to_string(),
            Key::Enter => "Enter".to_string(),
            Key::Tab => "Tab".to_string(),
            Key::Backspace => "Backspace".to_string(),
            Key::LeftShift => "Shift".to_string(),
            Key::LeftControl => "left Control".to_string(),
            Key::LeftAlt => "left Alt".to_string(),
            Key::RightAlt => "right Alt".to_string(),
            Key::LeftArrow => "← arrow".to_string(),
            Key::RightArrow => "→ arrow".to_string(),
            Key::UpArrow => "↑".to_string(),
            Key::DownArrow => "↓".to_string(),
            Key::F1 => "F1".to_string(),
            Key::F2 => "F2".to_string(),
            Key::F3 => "F3".to_string(),
            Key::F4 => "F4".to_string(),
            Key::F5 => "F5".to_string(),
            Key::F6 => "F6".to_string(),
            Key::F7 => "F7".to_string(),
            Key::F8 => "F8".to_string(),
            Key::F9 => "F9".to_string(),
            Key::F10 => "F10".to_string(),
            Key::F11 => "F11".to_string(),
            Key::F12 => "F12".to_string(),
            // These have to_char, but override here
            Key::Space => "Space".to_string(),
            _ => self.to_char(false).unwrap().to_string(),
        }
    }

    fn from_winit_key(input: KeyboardInput) -> Option<Key> {
        let key = input.virtual_keycode?;
        Some(match key {
            VirtualKeyCode::A => Key::A,
            VirtualKeyCode::B => Key::B,
            VirtualKeyCode::C => Key::C,
            VirtualKeyCode::D => Key::D,
            VirtualKeyCode::E => Key::E,
            VirtualKeyCode::F => Key::F,
            VirtualKeyCode::G => Key::G,
            VirtualKeyCode::H => Key::H,
            VirtualKeyCode::I => Key::I,
            VirtualKeyCode::J => Key::J,
            VirtualKeyCode::K => Key::K,
            VirtualKeyCode::L => Key::L,
            VirtualKeyCode::M => Key::M,
            VirtualKeyCode::N => Key::N,
            VirtualKeyCode::O => Key::O,
            VirtualKeyCode::P => Key::P,
            VirtualKeyCode::Q => Key::Q,
            VirtualKeyCode::R => Key::R,
            VirtualKeyCode::S => Key::S,
            VirtualKeyCode::T => Key::T,
            VirtualKeyCode::U => Key::U,
            VirtualKeyCode::V => Key::V,
            VirtualKeyCode::W => Key::W,
            VirtualKeyCode::X => Key::X,
            VirtualKeyCode::Y => Key::Y,
            VirtualKeyCode::Z => Key::Z,
            VirtualKeyCode::Key1 | VirtualKeyCode::Numpad1 => Key::Num1,
            VirtualKeyCode::Key2 | VirtualKeyCode::Numpad2 => Key::Num2,
            VirtualKeyCode::Key3 | VirtualKeyCode::Numpad3 => Key::Num3,
            VirtualKeyCode::Key4 | VirtualKeyCode::Numpad4 => Key::Num4,
            VirtualKeyCode::Key5 | VirtualKeyCode::Numpad5 => Key::Num5,
            VirtualKeyCode::Key6 | VirtualKeyCode::Numpad6 => Key::Num6,
            VirtualKeyCode::Key7 | VirtualKeyCode::Numpad7 => Key::Num7,
            VirtualKeyCode::Key8 | VirtualKeyCode::Numpad8 => Key::Num8,
            VirtualKeyCode::Key9 | VirtualKeyCode::Numpad9 => Key::Num9,
            VirtualKeyCode::Key0 | VirtualKeyCode::Numpad0 => Key::Num0,
            VirtualKeyCode::LBracket => Key::LeftBracket,
            VirtualKeyCode::RBracket => Key::RightBracket,
            VirtualKeyCode::Space => Key::Space,
            VirtualKeyCode::Slash => Key::Slash,
            VirtualKeyCode::Period => Key::Dot,
            VirtualKeyCode::Comma => Key::Comma,
            VirtualKeyCode::Semicolon => Key::Semicolon,
            VirtualKeyCode::Colon => Key::Colon,
            VirtualKeyCode::Equals => Key::Equals,
            VirtualKeyCode::Apostrophe => Key::SingleQuote,
            VirtualKeyCode::Escape => Key::Escape,
            VirtualKeyCode::Return => Key::Enter,
            VirtualKeyCode::Tab => Key::Tab,
            VirtualKeyCode::Back => Key::Backspace,
            VirtualKeyCode::LShift => Key::LeftShift,
            VirtualKeyCode::LControl => Key::LeftControl,
            VirtualKeyCode::LAlt => Key::LeftAlt,
            VirtualKeyCode::RAlt => Key::RightAlt,
            VirtualKeyCode::Left => Key::LeftArrow,
            VirtualKeyCode::Right => Key::RightArrow,
            VirtualKeyCode::Up => Key::UpArrow,
            VirtualKeyCode::Down => Key::DownArrow,
            VirtualKeyCode::F1 => Key::F1,
            VirtualKeyCode::F2 => Key::F2,
            VirtualKeyCode::F3 => Key::F3,
            VirtualKeyCode::F4 => Key::F4,
            VirtualKeyCode::F5 => Key::F5,
            VirtualKeyCode::F6 => Key::F6,
            VirtualKeyCode::F7 => Key::F7,
            VirtualKeyCode::F8 => Key::F8,
            VirtualKeyCode::F9 => Key::F9,
            VirtualKeyCode::F10 => Key::F10,
            VirtualKeyCode::F11 => Key::F11,
            VirtualKeyCode::F12 => Key::F12,
            _ => {
                println!("Unknown winit key {:?}", key);
                return None;
            }
        })
    }

    pub fn txt(self, ctx: &EventCtx) -> TextSpan {
        Line(self.describe()).fg(ctx.style().text_hotkey_color)
    }
}

// TODO This is not an ideal representation at all.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum MultiKey {
    Normal(Key),
    LCtrl(Key),
    Any(Vec<Key>),
}

impl MultiKey {
    pub fn describe(&self) -> String {
        match self {
            MultiKey::Normal(key) => key.describe(),
            MultiKey::LCtrl(key) => format!("Ctrl+{}", key.describe()),
            MultiKey::Any(ref keys) => keys
                .iter()
                .map(|k| k.describe())
                .collect::<Vec<_>>()
                .join(", "),
        }
    }

    pub fn txt(&self, ctx: &EventCtx) -> TextSpan {
        Line(self.describe()).fg(ctx.style().text_hotkey_color)
    }
}

pub fn lctrl(key: Key) -> MultiKey {
    MultiKey::LCtrl(key)
}

pub fn hotkeys(keys: Vec<Key>) -> MultiKey {
    MultiKey::Any(keys)
}

impl std::convert::From<Key> for Option<MultiKey> {
    fn from(key: Key) -> Option<MultiKey> {
        Some(MultiKey::Normal(key))
    }
}

impl std::convert::From<Key> for MultiKey {
    fn from(key: Key) -> MultiKey {
        MultiKey::Normal(key)
    }
}
