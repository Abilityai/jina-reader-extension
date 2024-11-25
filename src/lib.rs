use std::sync::Arc;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use zed_extension_api as zed;
use zed::{
    Extension, SlashCommand, SlashCommandOutput, SlashCommandOutputSection, Worktree,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct JinaResponse {
    text: String,
}

use std::sync::Mutex;

type SharedState = Arc<Mutex<Option<Result<String, String>>>>;

pub struct JinaReaderExtension {
    client: reqwest::Client,
    state: SharedState,
}

zed::register_extension!(JinaReaderExtension);

impl Extension for JinaReaderExtension {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            state: Arc::new(Mutex::new(None)),
        }
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
        _worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        match command.name.as_str() {
            "r" => {
                if args.is_empty() {
                    return Err("Please provide a URL.".to_string());
                }

                let url = args[0].clone();
                let client = self.client.clone();
                let state = self.state.clone();

                spawn_local(async move {
                    let jina_url = format!("https://r.jina.ai/{}", url);

                    let result = match client.get(&jina_url).send().await {
                        Ok(response) => {
                            match response.json::<JinaResponse>().await {
                                Ok(jina_response) => {
                                    console::log_1(&"Successfully fetched content".into());
                                    Ok(jina_response.text)
                                }
                                Err(e) => {
                                    let error = format!("Failed to parse response: {}", e);
                                    console::log_1(&error.clone().into());
                                    Err(error)
                                }
                            }
                        }
                        Err(e) => {
                            let error = format!("Request failed: {}", e);
                            console::log_1(&error.clone().into());
                            Err(error)
                        }
                    };

                    if let Ok(mut state) = state.lock() {
                        *state = Some(result);
                    }
                });

                Ok(SlashCommandOutput {
                    sections: vec![SlashCommandOutputSection {
                        range: (0..22u32).into(),
                        label: "Jina Reader".to_string(),
                    }],
                    text: "Fetching content...".to_string(),
                })
            }
            _ => Err(format!("Unknown slash command: {}", command.name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_slash_command_without_url() {
        let extension = JinaReaderExtension::new();
        let result = extension.run_slash_command(
            SlashCommand {
                name: "r".to_string(),
                description: "".to_string(),
                requires_argument: true,
                tooltip_text: "".to_string(),
            },
            vec![],
            None,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Please provide a URL.");
    }

    #[test]
    fn test_run_slash_command_with_url() {
        let extension = JinaReaderExtension::new();
        let result = extension.run_slash_command(
            SlashCommand {
                name: "r".to_string(),
                description: "".to_string(),
                requires_argument: true,
                tooltip_text: "".to_string(),
            },
            vec!["https://example.com".to_string()],
            None,
        );

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.text, "Fetching content...");
    }

    #[test]
    fn test_unknown_command() {
        let extension = JinaReaderExtension::new();
        let result = extension.run_slash_command(
            SlashCommand {
                name: "unknown".to_string(),
                description: "".to_string(),
                requires_argument: true,
                tooltip_text: "".to_string(),
            },
            vec![],
            None,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unknown slash command: unknown");
    }
}
