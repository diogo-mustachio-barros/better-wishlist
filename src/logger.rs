use crate::util::text_util::bold;

pub trait Logger {
    fn log(&self, priority:Priority, message: &str);
    fn log_info(&self, message: &str) { self.log(Priority::Info, message) }
    fn log_warning(&self, message: &str) { self.log(Priority::Warning, message) }
    fn log_error(&self, message: &str) { self.log(Priority::Error, message) }
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
    fn log(&self, _:Priority, _: &str) { () }
}

pub struct StdoutLogger;
unsafe impl Send for StdoutLogger {}
unsafe impl Sync for StdoutLogger {}
impl Logger for StdoutLogger {
    fn log(&self, priority:Priority, message: &str) {
        
        let date_time = chrono::offset::Local::now();

        println!( "{}\t{}"
                , date_time.format("%d/%m/%Y %H:%M:%S")
                , bold(message)
                );
    }
}