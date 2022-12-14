use std::{
    fs::{self, File},
    io,
};

use ::zip::ZipArchive;
use anyhow::{anyhow, Result};
use lapce_plugin::{
    psp_types::{
        lsp_types::{
            request::Initialize, DocumentFilter, DocumentSelector, InitializeParams, MessageType,
            Url,
        },
        Request,
    },
    register_plugin, Http, LapcePlugin, VoltEnvironment, PLUGIN_RPC,
};
use serde_json::Value;

#[derive(Default)]
struct State {}

register_plugin!(State);

macro_rules! string {
    ( $x:expr ) => {
        String::from($x)
    };
}

fn initialize(params: InitializeParams) -> Result<()> {
    let document_selector: DocumentSelector = vec![
        DocumentFilter {
            language: Some(String::from("javascript")),
            pattern: Some(String::from("**/*.js")),
            scheme: None,
        },
        DocumentFilter {
            language: Some(String::from("javascriptreact")),
            pattern: Some(String::from("**/*.jsx")),
            scheme: None,
        },
        DocumentFilter {
            language: Some(String::from("jsx")),
            pattern: Some(String::from("**/*.jsx")),
            scheme: None,
        },
        DocumentFilter {
            language: Some(String::from("typescript")),
            pattern: Some(String::from("**/*.ts")),
            scheme: None,
        },
        DocumentFilter {
            language: Some(String::from("typescriptreact")),
            pattern: Some(String::from("**/*.tsx")),
            scheme: None,
        },
        DocumentFilter {
            language: Some(String::from("tsx")),
            pattern: Some(String::from("**/*.tsx")),
            scheme: None,
        },
        DocumentFilter {
            language: Some(String::from("json")),
            pattern: Some(String::from("**/*.json")),
            scheme: None,
        },
        DocumentFilter {
            language: Some(String::from("jsonc")),
            pattern: Some(String::from("**/*.jsonc")),
            scheme: None,
        },
        DocumentFilter {
            language: Some(String::from("markdown")),
            pattern: Some(String::from("**/*.{md,markdown}")),
            scheme: None,
        },
    ];
    let mut server_args = vec![string!("lsp")];

    if let Some(options) = params.initialization_options.as_ref() {
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
                            params.initialization_options,
                        );
                        return Ok(());
                    }
                }
            }
        }
    }

    let filename = match (
        VoltEnvironment::operating_system().as_deref(),
        VoltEnvironment::architecture().as_deref(),
        VoltEnvironment::libc().as_deref(),
    ) {
        (Ok("macos"), Ok("x86_64"), _) => "x86_64-apple-darwin",
        (Ok("macos"), Ok("aarch64"), _) => "aarch64-apple-darwin",
        (Ok("linux"), Ok("x86_64"), Ok("glibc")) => "x86_64-unknown-linux-gnu",
        (Ok("windows"), Ok("x86_64"), _) => "x86_64-pc-windows-msvc",
        _ => return Err(anyhow!("Unsupported OS/Arch/Libc")),
    };

    let zip_file = format!("deno-{filename}.zip");

    // Download URL
    let url = format!("https://github.com/denoland/deno/releases/download/v1.26.2/{zip_file}");

    let mut resp = Http::get(&url)?;
    if resp.status_code.is_success() {
        let body = resp.body_read_all()?;
        std::fs::write(&zip_file, body)?;

        let mut zip = ZipArchive::new(File::open(&zip_file)?)?;

        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            if (*file.name()).ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p)?;
                    }
                }
                let mut outfile = File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }
    }

    let filename = match VoltEnvironment::operating_system().as_deref() {
        Ok("windows") => {
            string!("deno.exe")
        }
        _ => string!("deno"),
    };

    let volt_uri = VoltEnvironment::uri()?;
    let server_path = Url::parse(&volt_uri)?.join(&filename)?;

    PLUGIN_RPC.start_lsp(
        server_path,
        server_args,
        document_selector,
        params.initialization_options,
    );

    Ok(())
}

impl LapcePlugin for State {
    fn handle_request(&mut self, _id: u64, method: String, params: Value) {
        #[allow(clippy::single_match)]
        match method.as_str() {
            Initialize::METHOD => {
                let params: InitializeParams = serde_json::from_value(params).unwrap();
                if let Err(e) = initialize(params) {
                    PLUGIN_RPC.window_log_message(MessageType::ERROR, e.to_string());
                    PLUGIN_RPC.window_show_message(MessageType::ERROR, e.to_string());
                }
            }
            _ => {}
        }
    }
}
