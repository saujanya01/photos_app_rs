// Model

use iced::{
    widget::{button, column, text},
    Element,
};

struct Counter {
    value: i32,
}

impl Default for Counter {
    fn default() -> Self {
        Counter { value: 0 }
    }
}

// Events that can happen
#[derive(Debug, Clone)]
enum Messages {
    Increment,
    Decrement,
}

impl Counter {
    fn update(&mut self, message: Messages) {
        match message {
            Messages::Increment => self.value += 1,
            Messages::Decrement => self.value -= 1,
        }
    }

    fn view(&self) -> Element<Messages> {
        column![
            button("Increment").on_press(Messages::Increment),
            text(self.value),
            button("Decrement").on_press(Messages::Decrement)
        ]
        .into()
    }
}

fn main() -> iced::Result {
    iced::run(Counter::update, Counter::view)
}
