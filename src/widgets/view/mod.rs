mod cfg;
mod client;
mod style;
#[allow(clippy::module_inception)]
mod view;
mod window;

pub use self::client::Client as ViewClient;
pub use self::view::View;
