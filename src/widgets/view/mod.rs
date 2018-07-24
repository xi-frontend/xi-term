mod client;
mod style;
#[cfg_attr(feature = "clippy", allow(module_inception))]
mod view;
mod window;

pub use self::view::View;
pub use self::client::Client as ViewClient;
