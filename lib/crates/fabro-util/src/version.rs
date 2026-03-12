use std::sync::LazyLock;

pub const FABRO_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const FABRO_GIT_SHA: &str = env!("FABRO_GIT_SHA");
pub const FABRO_BUILD_DATE: &str = env!("FABRO_BUILD_DATE");

pub static LONG_VERSION: LazyLock<String> =
    LazyLock::new(|| format!("{FABRO_VERSION} ({FABRO_GIT_SHA} {FABRO_BUILD_DATE})"));
