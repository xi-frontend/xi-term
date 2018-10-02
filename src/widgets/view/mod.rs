mod client;
mod style;
#[cfg_attr(feature = "clippy", allow(clippy::module_inception))]
mod view;
mod window;

pub use self::client::Client as ViewClient;
pub use self::view::View;
