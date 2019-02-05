use js_sys as js;
use wasm_bindgen::prelude::*;

struct Clock {
    date: js::Date,
    subscription: Option<draco::Unsubscribe>,
}

enum Message {
    Toggle,
    Tick,
}

impl Clock {
    fn new() -> Self {
        Clock {
            date: js::Date::new_0(),
            subscription: None,
        }
    }
}

impl draco::App for Clock {
    type Message = Message;

    fn update(&mut self, mailbox: &draco::Mailbox<Message>, message: Self::Message) {
        match message {
            Message::Toggle => {
                if let Some(_) = self.subscription.take() {
                } else {
                    self.subscription = Some(
                        mailbox
                            .subscribe(draco::subscription::Interval::new(16), |()| Message::Tick),
                    );
                }
            }
            Message::Tick => {
                self.date = js::Date::new_0();
            }
        }
    }

    fn render(&self) -> draco::Node<Self::Message> {
        use draco::{html as h, svg as s};
        let circle = s::circle()
            .attr("cx", "100")
            .attr("cy", "100")
            .attr("r", "98")
            .attr("fill", "none")
            .attr("stroke", "black");

        let line = |rotate: f64, stroke, stroke_width, height: u32| {
            s::line()
                .attr("x1", "100")
                .attr("y1", "100")
                .attr("x2", (100 - height).to_string())
                .attr("y2", "100")
                .attr("stroke", stroke)
                .attr("stroke-width", stroke_width)
                .attr("stroke-linecap", "round")
                .attr(
                    "transform",
                    format!("rotate({} 100 100)", (rotate * 10.0).round() / 10.0),
                )
        };

        let d = &self.date;
        let ms = ((((d.get_hours() * 60 + d.get_minutes()) * 60) + d.get_seconds()) * 1000
            + d.get_milliseconds()) as f64;

        let second_rotate = 90.0 + ((ms / 1000.0) % 60.0) * 360.0 / 60.0;
        let minute_rotate = 90.0 + ((ms / 1000.0 / 60.0) % 60.0) * 360.0 / 60.0;
        let hour_rotate = 90.0 + ((ms / 1000.0 / 60.0 / 60.0) % 12.0) * 360.0 / 12.0;

        h::div()
            .attr(
                "style",
                "display: flex; align-items: center; flex-direction: column;",
            )
            .push(
                h::div().push(
                    h::button()
                        .attr("style", "margin: 1rem;")
                        .push(if self.subscription.is_some() {
                            "Stop"
                        } else {
                            "Start"
                        })
                        .on("click", |_| Message::Toggle),
                ),
            )
            .push(
                s::svg()
                    .attr("width", "400")
                    .attr("height", "400")
                    .attr("viewBox", "0 0 200 200")
                    .push(circle)
                    .push(line(hour_rotate, "#333", "4", 50))
                    .push(line(minute_rotate, "#333", "3", 70))
                    .push(line(second_rotate, "crimson", "2", 90)),
            )
            .into()
    }
}

#[wasm_bindgen]
pub fn start() {
    let mailbox = draco::start(Clock::new(), draco::select("main").expect("main").into());
    mailbox.send(Message::Toggle);
}

pub fn main() {}
