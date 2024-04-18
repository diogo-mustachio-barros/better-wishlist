#![allow(dead_code)]

use std::fmt::Display;

use crate::util::text_util::{bold, colored_foreground, Color};

pub trait Logger {
    fn log<T: AsRef<str> + Display>(&self, priority:Priority, message: T);
    fn log_info<T: AsRef<str> + Display>(&self, message: T) { self.log(Priority::Info, message) }
    fn log_warning<T: AsRef<str> + Display>(&self, message: T) { self.log(Priority::Warning, message) }
    fn log_error<T: AsRef<str> + Display>(&self, message: T) { self.log(Priority::Error, message) }
}

pub enum Priority {
    Info,
    Warning,
    Error
}

pub struct VoidLogger;
unsafe impl Send for VoidLogger {}
unsafe impl Sync for VoidLogger {}
impl Logger for VoidLogger {
    fn log<T: AsRef<str> + Display>(&self, _:Priority, _: T) { () }
}

pub struct StdoutLogger;
unsafe impl Send for StdoutLogger {}
unsafe impl Sync for StdoutLogger {}
impl Logger for StdoutLogger {
    fn log<T: AsRef<str> + Display>(&self, priority:Priority, message: T) {
        let date_time = chrono::offset::Local::now();

        let date_text = date_time.format("%d-%m-%Y %H:%M:%S");
        let message_text = format_with_priority(priority, message);

        println!( "{date_text} {message_text}");
    }
}

fn format_with_priority<T: AsRef<str> + Display>(priority: Priority, message: T) -> String {
    match priority {
        Priority::Info    => 
            format!("{} {}"
                   , bold("INFO:")
                   , message),
        Priority::Warning => 
            format!("{} {}"
                   , bold(colored_foreground("WARNING:", Color::Yellow))
                   , colored_foreground(message, Color::Yellow)),
        Priority::Error   => 
            format!("{} {}"
                   , bold(colored_foreground("ERROR:", Color::Red))
                   , colored_foreground(message, Color::Red))
    }
}