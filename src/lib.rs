use zed_extension_api::{
    http_client::{HttpMethod, HttpRequestBuilder},
    Extension, SlashCommand, SlashCommandOutput, SlashCommandOutputSection, Worktree,
};

pub trait HttpHandler: Send + Sync {
    fn fetch(&self, url: &str) -> Result<String, String>;
}

pub struct ZedHttpHandler;

impl HttpHandler for ZedHttpHandler {
    fn fetch(&self, url: &str) -> Result<String, String> {
        let request = HttpRequestBuilder::new()
            .method(HttpMethod::Get)
            .url(url.to_string())
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let response = request
            .fetch()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        String::from_utf8(response.body).map_err(|e| format!("bytes should be valid utf8: {}", e))
    }
}

pub struct JinaReaderExtension {
    http_handler: Box<dyn HttpHandler>,
}

impl JinaReaderExtension {
    pub fn with_http_handler(http_handler: Box<dyn HttpHandler>) -> Self {
        Self { http_handler }
    }
}

zed_extension_api::register_extension!(JinaReaderExtension);

impl Extension for JinaReaderExtension {
    fn new() -> Self {
        Self {
            http_handler: Box::new(ZedHttpHandler),
        }
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
        _worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        if command.name == "r" {
            if args.is_empty() {
                return Err("Please provide a URL.".to_string());
            }

            let url = args[0].clone();
            let jina_url = format!("https://r.jina.ai/{}", url);

            // Build and send the HTTP request synchronously
            let text = self.http_handler.fetch(&jina_url)?;

            // Prepare SlashCommandOutput
            Ok(SlashCommandOutput {
                sections: vec![SlashCommandOutputSection {
                    range: (0..text.len()).into(),
                    label: "Jina Reader".to_string(),
                }],
                text,
            })
        } else {
            Err(format!("Unknown slash command: {}", command.name))
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

    struct MockHttpHandler;

    impl HttpHandler for MockHttpHandler {
        fn fetch(&self, _url: &str) -> Result<String, String> {
            println!("{}", "hello");
            Ok("Mocked response text".to_string())
        }
    }

    #[test]
    fn test_run_slash_command_with_url() {
        let mock_handler = Box::new(MockHttpHandler);
        let extension = JinaReaderExtension::with_http_handler(mock_handler);

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
        assert_eq!(output.text, "Mocked response text");
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
