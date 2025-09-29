use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use iced::{
    exit, keyboard::key, stream, time, widget::{button, column, container, text, Column}
};
use iced::{window, Center, Color, Element, Subscription, Task};
use windows_sys::Win32::UI::WindowsAndMessaging::*;
use iced::futures::SinkExt;




pub fn main() -> iced::Result {
    use global_hotkey::{
        hotkey::{Code, HotKey, Modifiers},
        GlobalHotKeyManager,
    };

    let mut key_lookup = KeyLookup { table: [0; 10] };
    // initialize the hotkeys manager
    let manager = GlobalHotKeyManager::new().unwrap();

    // construct the hotkey, add it to the list and register it
    let close_app= HotKey::new(None, Code::End);
    key_lookup.table[0] = close_app.id;
    let _ = manager.register(close_app);

    // construct the hotkey, add it to the list and register it
    let move_right= HotKey::new(Some(Modifiers::FN), Code::ArrowRight);
    key_lookup.table[1] = move_right.id;
    println!("Registered move right hotkey with id {}", move_right.id);
    let _ = manager.register(move_right);

    let move_left = HotKey::new(Some(Modifiers::FN), Code::ArrowLeft);
    key_lookup.table[2] = move_left.id;
    let _ = manager.register(move_left);

    let move_up = HotKey::new(Some(Modifiers::FN), Code::ArrowUp);
    key_lookup.table[3] = move_up.id;
    let _ = manager.register(move_up);

    let move_down = HotKey::new(Some(Modifiers::FN), Code::ArrowDown);
    key_lookup.table[4] = move_down.id;
    let _ = manager.register(move_down);

    let app = iced::application("beatcode", Counter::update, Counter::view)
        .subscription(Counter::subscription)
        .window({
            window::Settings {
                size: (300 as f32, 200.0 as f32).into(),
                transparent: true,
                decorations: false,
                ..window::Settings::default()
            }
        });

    app.run_with(|| {
        (Counter {
            key_lookup: key_lookup,
            value: 0,
            inuse_window_pos: None,
            inuse_window_id: None,
        }, Task::none())
    })
}

struct KeyLookup {
    table: [u32; 10]
}


#[derive(Debug, Clone, Copy)]
enum MoveDirection {
    Left,
    Right,
    Up,
    Down
}
struct Counter {
    value: i64,
    key_lookup: KeyLookup,
    // Store the position and id of the window when it's opened
    inuse_window_pos: Option<iced::Point>,
    inuse_window_id: Option<window::Id>,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
    WindowEvent(window::Id, window::Event),
    GotRawId(u64),
    HotKeyPressed(GlobalHotKeyEvent),
    MoveWindow(MoveDirection)
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
            Message::WindowEvent(id, window::Event::Opened { position, ..}) => {
                self.inuse_window_pos = position;
                self.inuse_window_id = Some(id); 
                return window::get_raw_id::<Task<u64>>(id).map(Message::GotRawId);
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
            Message::HotKeyPressed(global_hotkey) => {
                println!("Hotkey event received: {:?}", global_hotkey);
                match global_hotkey {
                    GlobalHotKeyEvent { id, state } if state == HotKeyState::Pressed  => {
                        println!("Hotkey ID: {}", id);
                        println!("Lookup Table: {:?}", self.key_lookup.table);
                        if id == self.key_lookup.table[0] {
                            println!("Close app hotkey pressed, exiting.");
                            return exit()
                        }
                        else if id == self.key_lookup.table[1] {
                            println!("Move right hotkey pressed.");
                            return Task::perform(async { MoveDirection::Right }, Message::MoveWindow);
                        }
                        else if id == self.key_lookup.table[2] {
                            println!("Move left hotkey pressed.");
                            return Task::perform(async { MoveDirection::Left }, Message::MoveWindow);
                        }
                        else if id == self.key_lookup.table[3] {
                            println!("Move up hotkey pressed.");
                            return Task::perform(async { MoveDirection::Up }, Message::MoveWindow);
                        }
                        else if id == self.key_lookup.table[4] {
                            println!("Move down hotkey pressed.");
                            return Task::perform(async { MoveDirection::Down }, Message::MoveWindow);
                        }
                        else {
                            println!("Unrecognized hotkey ID: {}", id); 
                        }
                    }
                    
                    _ => {}
                }
                Task::none()
            }
            Message::MoveWindow(direction) => {
                println!("Move window: {:?}", direction);
                let id = match self.inuse_window_id {
                    Some(id) => id,
                    None => {
                        println!("No window to move");
                        return Task::none();
                    }
                };
                let position = match self.inuse_window_pos {
                    Some(pos) => pos,
                    None => {
                        println!("No position info available");
                        return Task::none();
                    }
                };  
                match direction {
                    MoveDirection::Left => {
                        let new_position = position + iced::Vector::new(-40.0, 0.0);
                        self.inuse_window_pos = Some(new_position);
                        return window::move_to(id, new_position);
                    }
                    MoveDirection::Right => {
                        let new_position = position + iced::Vector::new(40.0, 0.0);
                        self.inuse_window_pos = Some(new_position);
                        return window::move_to(id, new_position);
                    }
                    MoveDirection::Up => {
                        let new_position = position + iced::Vector::new(0.0, -40.0);
                        self.inuse_window_pos = Some(new_position);
                        return window::move_to(id, new_position);
                    }
                    MoveDirection::Down => {
                        let new_position = position + iced::Vector::new(0.0, 40.0);
                        self.inuse_window_pos = Some(new_position);
                        return window::move_to(id, new_position);
                    }
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
                    a: 0.1, // 60% opaque black
                })),
                ..Default::default()
            })
            .into()
    }
    fn subscription(&self) -> iced::Subscription<Message> {
        let rx = GlobalHotKeyEvent::receiver();
        iced::Subscription::batch([
            iced::window::events().map(|(id, event)| Message::WindowEvent(id, event)),
            Subscription::run(move || {
                stream::channel(100, |mut output| async move {
                    let rx = GlobalHotKeyEvent::receiver();

                    loop {
                        // Offload blocking `recv`:
                        let result = tokio::task::spawn_blocking({
                            let rx = rx.clone();
                            move || rx.recv()
                        })
                        .await
                        .unwrap();

                        match result {
                            Ok(event) => {
                                if let Err(e) = output.send(Message::HotKeyPressed(event)).await {
                                    eprintln!("Failed to send hotkey event: {e}");
                                    break;
                                }
                            }
                            Err(_) => break, // channel closed
                        }
                    }
                })
            }),
        ])
    }
}
