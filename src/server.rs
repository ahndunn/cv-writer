use std::path::PathBuf;
use base64::Engine;
use schemars::schema_for;
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info};

use crate::compiler::LatexCompiler;
use crate::mcp::{JsonRpcRequest, JsonRpcResponse, McpCallToolResult, McpContentItem, McpTool};
use crate::schema::{sample_cv_profile, CvProfile};
use crate::template::{render_latex, STAR_ROVER_TEMPLATE};

pub struct Server {
    compiler: LatexCompiler,
}

impl Server {
    pub fn new() -> Result<Self, crate::compiler::CompilerError> {
        let compiler = LatexCompiler::new()?;
        Ok(Self { compiler })
    }

    /// Run the MCP server over standard I/O (stdin/stdout)
    pub async fn run_stdio(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        info!("CV Writer MCP Server started on stdio");

        while reader.read_line(&mut line).await? > 0 {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                line.clear();
                continue;
            }

            debug!("Received MCP message: {}", trimmed);
            let req: Result<JsonRpcRequest, _> = serde_json::from_str(trimmed);
            match req {
                Ok(request) => {
                    let maybe_response = self.handle_request(request).await;
                    if let Some(resp) = maybe_response {
                        let serialized = serde_json::to_string(&resp)? + "\n";
                        stdout.write_all(serialized.as_bytes()).await?;
                        stdout.flush().await?;
                    }
                }
                Err(e) => {
                    error!("Invalid JSON-RPC request: {}", e);
                    let resp = JsonRpcResponse::error(
                        None,
                        -32700,
                        format!("Parse error: {}", e),
                        None,
                    );
                    let serialized = serde_json::to_string(&resp)? + "\n";
                    stdout.write_all(serialized.as_bytes()).await?;
                    stdout.flush().await?;
                }
            }
            line.clear();
        }

        info!("CV Writer MCP Server stdin closed. Exiting.");
        Ok(())
    }

    pub async fn handle_request(&self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = req.id.clone();
        match req.method.as_str() {
            "initialize" => {
                let init_result = json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "cv-writer-mcp",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                });
                Some(JsonRpcResponse::success(id, init_result))
            }
            "notifications/initialized" => {
                // Initialized notification, no response required
                None
            }
            "ping" => Some(JsonRpcResponse::success(id, json!({}))),
            "tools/list" => {
                let tools = self.list_tools();
                Some(JsonRpcResponse::success(id, json!({ "tools": tools })))
            }
            "tools/call" => {
                let response = self.call_tool(req.params).await;
                Some(JsonRpcResponse::success(id, response))
            }
            unknown => Some(JsonRpcResponse::error(
                id,
                -32601,
                format!("Method '{}' not found", unknown),
                None,
            )),
        }
    }

    fn list_tools(&self) -> Vec<McpTool> {
        let cv_schema = schema_for!(CvProfile);
        let cv_schema_json = serde_json::to_value(cv_schema).unwrap_or(json!({}));

        vec![
            McpTool {
                name: "get_cv_schema".to_string(),
                description: "Returns the JSON Schema of the CV Profile required by the Star Rover template. AI Agents MUST call this tool or consult its schema to structure arbitrary profile data accurately before calling render_cv.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }),
            },
            McpTool {
                name: "get_sample_profile".to_string(),
                description: "Returns a complete, realistic example of a structured CV profile conforming to the Star Rover template schema.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }),
            },
            McpTool {
                name: "get_template_info".to_string(),
                description: "Returns details about the underlying Star Rover LaTeX template, typographical choices, and supported formatting tags.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }),
            },
            McpTool {
                name: "render_cv".to_string(),
                description: "Synthesizes a structured profile into the Star Rover LuaLaTeX template and builds the actual PDF using the native LuaLaTeX engine. Returns base64 encoded PDF bytes and compilation status. Stateless with zero disk leaks.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "profile": cv_schema_json,
                        "output_path": {
                            "type": "string",
                            "description": "Optional file path where the generated PDF should be written on disk. If omitted, the PDF is returned directly as base64 in the response."
                        }
                    },
                    "required": ["profile"]
                }),
            },
        ]
    }

    async fn call_tool(&self, params: Option<serde_json::Value>) -> serde_json::Value {
        let params = match params {
            Some(p) => p,
            None => {
                return serde_json::to_value(McpCallToolResult {
                    is_error: Some(true),
                    content: vec![McpContentItem::Text {
                        text: "Missing params object in tools/call".to_string(),
                    }],
                })
                .unwrap();
            }
        };

        let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        match tool_name {
            "get_cv_schema" => {
                let schema = schema_for!(CvProfile);
                let text = serde_json::to_string_pretty(&schema).unwrap_or_default();
                serde_json::to_value(McpCallToolResult {
                    is_error: Some(false),
                    content: vec![McpContentItem::Text { text }],
                })
                .unwrap()
            }
            "get_sample_profile" => {
                let sample = sample_cv_profile();
                let text = serde_json::to_string_pretty(&sample).unwrap_or_default();
                serde_json::to_value(McpCallToolResult {
                    is_error: Some(false),
                    content: vec![McpContentItem::Text { text }],
                })
                .unwrap()
            }
            "get_template_info" => {
                let info = json!({
                    "template": "Star Rover (Migrated to native LuaLaTeX with fontspec & OpenType)",
                    "engine": "LuaLaTeX",
                    "font": "Fira Sans (SemiBold/Regular) with FontAwesome5 icons",
                    "palette": {
                        "accent": "#141E61 (Navy/Indigo)",
                        "secondary": "gray"
                    },
                    "sections_supported": [
                        "contact (name, phone, email, github, linkedin, website, location)",
                        "summary",
                        "education (institution, degree, dates, highlights)",
                        "experience (company, location, roles: [title, dates, highlights])",
                        "projects (name, url, dates, highlights)",
                        "skills (category, items)",
                        "certifications (name, issuer, date)",
                        "publications (citation, url)",
                        "awards (title, date, summary)"
                    ],
                    "raw_template_length": STAR_ROVER_TEMPLATE.len()
                });
                let text = serde_json::to_string_pretty(&info).unwrap_or_default();
                serde_json::to_value(McpCallToolResult {
                    is_error: Some(false),
                    content: vec![McpContentItem::Text { text }],
                })
                .unwrap()
            }
            "render_cv" => {
                let profile_val = match arguments.get("profile") {
                    Some(v) => v,
                    None => {
                        return serde_json::to_value(McpCallToolResult {
                            is_error: Some(true),
                            content: vec![McpContentItem::Text {
                                text: "Missing required argument 'profile'".to_string(),
                            }],
                        })
                        .unwrap();
                    }
                };

                let profile: CvProfile = match serde_json::from_value(profile_val.clone()) {
                    Ok(p) => p,
                    Err(e) => {
                        return serde_json::to_value(McpCallToolResult {
                            is_error: Some(true),
                            content: vec![McpContentItem::Text {
                                text: format!("Invalid CV profile format against schema: {}", e),
                            }],
                        })
                        .unwrap();
                    }
                };

                let rendered_tex = match render_latex(&profile) {
                    Ok(tex) => tex,
                    Err(e) => {
                        return serde_json::to_value(McpCallToolResult {
                            is_error: Some(true),
                            content: vec![McpContentItem::Text {
                                text: format!("Template rendering error: {}", e),
                            }],
                        })
                        .unwrap();
                    }
                };

                let compile_res = match self.compiler.compile(&rendered_tex).await {
                    Ok(res) => res,
                    Err(e) => {
                        return serde_json::to_value(McpCallToolResult {
                            is_error: Some(true),
                            content: vec![McpContentItem::Text {
                                text: format!("LuaLaTeX compilation failed: {}", e),
                            }],
                        })
                        .unwrap();
                    }
                };

                let b64_pdf = base64::engine::general_purpose::STANDARD.encode(&compile_res.pdf_bytes);
                let mut out_text = format!(
                    "Successfully compiled CV for '{}' using LuaLaTeX! PDF size: {} bytes.\n",
                    profile.contact.name,
                    compile_res.pdf_bytes.len()
                );

                if let Some(dest_path_val) = arguments.get("output_path").and_then(|v| v.as_str()) {
                    let dest_path = PathBuf::from(dest_path_val);
                    if let Some(parent) = dest_path.parent() {
                        let _ = tokio::fs::create_dir_all(parent).await;
                    }
                    match tokio::fs::write(&dest_path, &compile_res.pdf_bytes).await {
                        Ok(_) => {
                            out_text.push_str(&format!("Saved PDF to disk at: {}\n", dest_path.display()));
                        }
                        Err(err) => {
                            out_text.push_str(&format!("Warning: Failed to save PDF to {}: {}\n", dest_path.display(), err));
                        }
                    }
                }

                let response_payload = json!({
                    "message": out_text,
                    "pdf_base64": b64_pdf,
                    "pdf_size_bytes": compile_res.pdf_bytes.len(),
                    "status": "success"
                });

                serde_json::to_value(McpCallToolResult {
                    is_error: Some(false),
                    content: vec![McpContentItem::Text {
                        text: serde_json::to_string_pretty(&response_payload).unwrap(),
                    }],
                })
                .unwrap()
            }
            unknown => serde_json::to_value(McpCallToolResult {
                is_error: Some(true),
                content: vec![McpContentItem::Text {
                    text: format!("Unknown tool: {}", unknown),
                }],
            })
            .unwrap(),
        }
    }
}
