#[macro_export]
macro_rules! error {
    ($x:expr) => {
        let _ = lapce_plugin::PLUGIN_RPC
            .window_log_message(lapce_plugin::psp_types::lsp_types::MessageType::ERROR, $x);
    };
}

#[macro_export]
macro_rules! info {
    ($x:expr) => {
        let _ = PLUGIN_RPC
            .window_log_message(lapce_plugin::psp_types::lsp_types::MessageType::INFO, $x);
    };
}

#[macro_export]
macro_rules! string {
    ( $x:expr ) => {
        String::from($x)
    };
}
