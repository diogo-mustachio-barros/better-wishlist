// const RESET_ANSI: &str = "\x1B[0m";

const BOLD_CODE_ANSI: &str  = "\x1B[1m";
const BOLD_RESET_ANSI: &str = "\x1B[22m";


pub fn bold(text: &str) -> String {
    return format!("{BOLD_CODE_ANSI}{text}{BOLD_RESET_ANSI}")
}