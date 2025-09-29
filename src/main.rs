use std::ops::Sub;

use iced::widget::{button, column, container, text, Column};
use iced::{window, Center, Color, Element, Subscription, Task};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

pub fn main() -> iced::Result {
    let app = iced::application("beatcode", Counter::update, Counter::view)
        .subscription(Counter::subscription)
        .window({
            window::Settings {
                size: (100 as f32,100.0 as f32).into(),
                transparent: true,
                decorations: false,
                ..window::Settings::default()
            }
        });

    app.run()
}

#[derive(Default)]
struct Counter {
    value: i64,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
    WindowEvent(window::Id, window::Event),
    GotRawId(u64),
}

impl Counter {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increment => {
                self.value += 1;
                Task::none()
            }
            Message::Decrement => {
                self.value -= 1;
                Task::none()
            }
            Message::WindowEvent(id, window::Event::Opened { .. }) => {
                return window::get_raw_id::<Task<u64>>(id).map(Message::GotRawId)
            }
            Message::GotRawId(raw_id) => {
                println!("Raw window ID: {}", raw_id);
                #[cfg(target_os = "windows")]
                unsafe {
                    use std::ffi::c_void;
                    let hwnd = raw_id as usize as *mut c_void;
                    let _ = SetWindowDisplayAffinity(hwnd, 0x11);
                    let mut style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
                    // Remove "application window" flag if present
                    style &= !WS_EX_APPWINDOW;

                    // Add "tool window" (small caption, hidden from Alt+Tab)
                    style |= WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TRANSPARENT;
                    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style as isize);

                    SetWindowPos(
                        hwnd,
                        HWND_TOPMOST,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                    );
                    Task::none()
                }
            }
            _ => {
                println!("Handle other messages");
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        // Your content column
        let content = column![
            button("Increment").on_press(Message::Increment),
            text(self.value).size(50),
            button("Decrement").on_press(Message::Decrement),
        ]
        .padding(20)
        .align_x(Center);

        // Wrap it in a container with a semi-transparent background
        container(content)
            .style(|_| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.6, // 60% opaque black
                })),
                ..Default::default()
            })
            .into()
    }
    fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch([
            iced::window::events().map(|(id, event)| Message::WindowEvent(id, event)),
        ])
    }
}
