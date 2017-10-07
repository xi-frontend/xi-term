error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error);
    }

    errors {
    }
}

use error_chain::ChainedError;

pub fn log_error<E: ChainedError>(e: &E) {
    error!("error: {}", e);
    for e in e.iter().skip(1) {
        error!("caused by: {}", e);
    }
}
