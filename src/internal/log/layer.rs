use super::visitor::Visitor;
use chrono::Local;
use colored::Colorize;
use tracing::Event;
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

pub struct LogLayer;

impl<S> Layer<S> for LogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, _event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = Visitor::new();
        _event.record(&mut visitor);
        let time = Local::now().format("%Y-%m-%d %H:%M:%S.%6f");
        let level = format!("{}", _event.metadata().level());
        let colored_level = match level.as_str() {
            "ERROR" => "ERROR".red(),
            "WARN" => "WARN".yellow(),
            "INFO" => "INFO".blue(),
            other => other.normal()
        };
        if level == "ERROR" {
            eprintln!("[{}] [{}] {}", colored_level, time, visitor.message());
        } else {
            println!("[{}] [{}] {}", colored_level, time, visitor.message());
        }
    }
}