use env_logger::{
    fmt::{Color, Style, StyledValue},
    Builder, Env, Target,
};
use log::Level;

/// Initializes the global logger with the built env logger.
///
/// This should be called early in the execution of a Rust program. Any log events that occur before initialization will be ignored.
///
/// # Panics
///
/// This function will panic if it is called more than once, or if another
/// library has already initialized a global logger.
pub fn init() {
    let mut builder = Builder::from_env(
        Env::default()
            .filter_or("H2O2_LOG_LEVEL", "info")
            .write_style_or("H2O2_LOG_STYLE", "always"),
    );

    builder.format(|f, record| {
        use std::io::Write;

        let mut style = f.style();
        let level = colored_level(&mut style, record.level());

        writeln!(f, " {} > {}", level, record.args(),)
    });

    builder.target(Target::Stderr).init()
}

fn colored_level<'a>(style: &'a mut Style, level: Level) -> StyledValue<'a, &'static str> {
    match level {
        Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
        Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
        Level::Info => style.set_color(Color::Green).value("INFO "),
        Level::Warn => style.set_color(Color::Yellow).value("WARN "),
        Level::Error => style.set_color(Color::Red).value("ERROR"),
    }
}
