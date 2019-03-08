mod client;
mod style;
#[allow(clippy::module_inception)]
mod view;
mod window;
mod cfg;

pub use self::client::Client as ViewClient;
pub use self::view::View;
