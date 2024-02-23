use anyhow::{bail, Result};
use lapce_plugin::{
    psp_types::{
        lsp_types::{
            request::Initialize, DocumentFilter, DocumentSelector, InitializeParams, MessageType,
            Url,
        },
        Request,
    },
    register_plugin, LapcePlugin, VoltEnvironment, PLUGIN_RPC,
};
use serde_json::Value;

#[derive(Default)]
struct State {}

register_plugin!(State);

mod internal;

fn initialize(params: InitializeParams) -> Result<()> {
    let document_selector: DocumentSelector = vec![
        DocumentFilter {
            language: Some(string!("javascript")),
            pattern:  Some(string!("**/*.js")),
            scheme:   None,
        },
        DocumentFilter {
            language: Some(string!("javascriptreact")),
            pattern:  Some(string!("**/*.jsx")),
            scheme:   None,
        },
        DocumentFilter {
            language: Some(string!("jsx")),
            pattern:  Some(string!("**/*.jsx")),
            scheme:   None,
        },
        DocumentFilter {
            language: Some(string!("typescript")),
            pattern:  Some(string!("**/*.ts")),
            scheme:   None,
        },
        DocumentFilter {
            language: Some(string!("typescriptreact")),
            pattern:  Some(string!("**/*.tsx")),
            scheme:   None,
        },
        DocumentFilter {
            language: Some(string!("tsx")),
            pattern:  Some(string!("**/*.tsx")),
            scheme:   None,
        },
        DocumentFilter {
            language: Some(string!("json")),
            pattern:  Some(string!("**/*.json")),
            scheme:   None,
        },
        DocumentFilter {
            language: Some(string!("jsonc")),
            pattern:  Some(string!("**/*.jsonc")),
            scheme:   None,
        },
        DocumentFilter {
            language: Some(string!("markdown")),
            pattern:  Some(string!("**/*.{md,markdown}")),
            scheme:   None,
        },
    ];
    let mut server_args = vec![string!("lsp")];
    let mut initialization_options = None;

    if let Some(options) = params.initialization_options.as_ref() {
        if let Some(deno) = options.get("deno") {
            initialization_options = Some(deno.to_owned());
        }

        if let Some(volt) = options.get("volt") {
            if let Some(args) = volt.get("serverArgs") {
                if let Some(args) = args.as_array() {
                    if !args.is_empty() {
                        server_args = vec![];
                    }
                    for arg in args {
                        if let Some(arg) = arg.as_str() {
                            server_args.push(arg.to_string());
                        }
                    }
                }
            }

            if let Some(server_path) = volt.get("serverPath") {
                if let Some(server_path) = server_path.as_str() {
                    if !server_path.is_empty() {
                        let server_uri = Url::parse(&format!("urn:{}", server_path))?;
                        PLUGIN_RPC.start_lsp(
                            server_uri,
                            server_args,
                            document_selector,
                            initialization_options,
                        )?;
                        return Ok(());
                    }
                }
            }
        }
    }

    let filename = internal::download()?;

    let volt_uri = VoltEnvironment::uri()?;
    info!(format!("Volt URI: {volt_uri}"));
    let Ok(server_uri) = Url::parse(&volt_uri) else {
        bail!("Failed to parse URL!");
    };
    let server_uri = server_uri.join(filename)?;

    info!(format!("Starting LSP server with URI: {server_uri}"));
    PLUGIN_RPC.start_lsp(
        server_uri,
        server_args,
        document_selector,
        initialization_options,
    )?;

    Ok(())
}

impl LapcePlugin for State {
    fn handle_request(&mut self, _id: u64, method: String, params: Value) {
        #[allow(clippy::single_match)]
        match method.as_str() {
            Initialize::METHOD => {
                let params: InitializeParams = serde_json::from_value(params).unwrap();
                if let Err(e) = initialize(params) {
                    let _ = PLUGIN_RPC.window_log_message(MessageType::ERROR, e.to_string());
                    let _ = PLUGIN_RPC.window_show_message(MessageType::ERROR, e.to_string());
                }
            }
            _ => {}
        }
    }
}
