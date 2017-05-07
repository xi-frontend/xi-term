error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error);
        SerdeJson(::serde_json::error::Error);
    }

    errors {
        RpcError {
            description("xi-rpc error")
            display("a xi-rpc error occured")
        }
        DisplayError {
            description("failed to draw screen")
            display("failed to draw screen")
        }
        UpdateError {
            description("failed to update a view")
            display("failed to update a view")
        }
        InputError {
            description("cannot handle input")
            display("cannot handle input")
        }
        TerminalSize {
            description("cannot determine terminal size")
            display("cannot determine terminal size")
        }
    }
}
