use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    process::{self, Stdio},
    sync::atomic::AtomicUsize,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::{ChildStdin, Command},
    sync::mpsc::{self, error::TryRecvError},
};

static ID: AtomicUsize = AtomicUsize::new(1);

fn next_id() -> usize {
    ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

pub async fn lsp_send_request(
    stdin: &mut BufWriter<ChildStdin>,
    req: &RequestMessage,
) -> anyhow::Result<i64> {
    let id = req.id;
    let req = json!({
        "id": id,
        "jsonrpc": "2.0",
        "params": req.params,
        "method": req.method,
    });
    let body = serde_json::to_string(&req)?;
    let req = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
    stdin.write_all(req.as_bytes()).await?;
    stdin.flush().await?;
    Ok(id)
}

pub async fn lsp_send_notification(
    stdin: &mut BufWriter<ChildStdin>,
    req: &NotificationMessage,
) -> anyhow::Result<()> {
    let req = json!({
        "jsonrpc": "2.0",
        "params": req.params,
        "method": req.method,
    });
    let body = serde_json::to_string(&req)?;
    let req = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
    stdin.write_all(req.as_bytes()).await?;
    stdin.flush().await?;
    Ok(())
}

#[derive(Debug)]
pub enum OutgoingMessage {
    Request(RequestMessage),
    Notification(NotificationMessage),
}

#[derive(Debug)]
pub enum IncomingMessage {
    Message(ResponseMessage),
    _Notification(ParsedNotification),
    UnknownNotification(NotificationMessage),
    Error(ResponseError),
    ProcessingError(String),
}

async fn lsp_start() -> anyhow::Result<LspClient> {
    let mut process = Command::new("rust-analyzer")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = process.stdout.take().unwrap();
    let stdin = process.stdin.take().unwrap();
    let _stderr = process.stderr.take().unwrap();

    let (request_tx, mut request_rx) = mpsc::channel::<OutgoingMessage>(32);
    let (response_tx, response_rx) = mpsc::channel::<IncomingMessage>(32);

    let writer_rtx = response_tx.clone();
    tokio::spawn(async move {
        let mut stdin = BufWriter::new(stdin);
        while let Some(message) = request_rx.recv().await {
            let err_msg = match &message {
                OutgoingMessage::Request(req) => lsp_send_request(&mut stdin, req).await.err(),
                OutgoingMessage::Notification(notif) => {
                    lsp_send_notification(&mut stdin, notif).await.err()
                }
            };
            if let Some(err) = err_msg {
                let error_message = format!("Failed to process message: {:?}", err);
                writer_rtx
                    .send(IncomingMessage::ProcessingError(error_message))
                    .await
                    .ok();
            }
        }
    });

    let reader_rtx = response_tx.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout);

        loop {
            let mut line = String::new();
            let read = match reader.read_line(&mut line).await {
                Ok(size) => size,
                Err(err) => {
                    tracing::error!("[LSP] failed to read from stdout");
                    reader_rtx
                        .send(IncomingMessage::ProcessingError(err.to_string()))
                        .await
                        .unwrap();
                    continue;
                }
            };

            if read == 0 {
                continue;
            }

            tracing::trace!("[LSP] incoming message starts with: {:?}", line);
            if line.starts_with("Content-Length: ") {
                let Ok(len) = line
                    .trim_start_matches("Content-Length: ")
                    .trim()
                    .parse::<usize>()
                else {
                    tracing::error!("[LSP] error parsing Content-Length: {}", line);
                    reader_rtx
                        .send(IncomingMessage::ProcessingError(
                            "Error parsing Content-Length".to_string(),
                        ))
                        .await
                        .unwrap();
                    continue;
                };

                // we read an empty line to account for the \r\n on the response
                reader.read_line(&mut line).await.unwrap();

                let mut body = vec![0; len];
                if let Err(err) = reader.read_exact(&mut body).await {
                    tracing::error!("[LSP] error reading body {}", err);
                    reader_rtx
                        .send(IncomingMessage::ProcessingError(err.to_string()))
                        .await
                        .unwrap();
                    continue;
                }
                let body = String::from_utf8_lossy(&body);
                let res = serde_json::from_str::<serde_json::Value>(&body).unwrap();
                tracing::debug!(
                    "[lsp] incoming message: {}",
                    res.to_string().chars().take(100).collect::<String>()
                );

                if let Some(err) = res.get("error") {
                    let code = err["code"].as_i64().unwrap();
                    let message = err["message"].as_str().unwrap().to_string();
                    let data = err.get("data").cloned();

                    reader_rtx
                        .send(IncomingMessage::Error(ResponseError {
                            _code: code,
                            _message: message,
                            _data: data,
                        }))
                        .await
                        .unwrap();

                    continue;
                }

                if let Some(id) = res.get("id") {
                    let id = id.as_i64().unwrap();
                    let result = res["result"].clone();

                    reader_rtx
                        .send(IncomingMessage::Message(ResponseMessage { id, result }))
                        .await
                        .unwrap();
                    continue;
                }

                let method = res["method"].as_str().unwrap().to_string();
                let params = res["params"].clone();

                reader_rtx
                    .send(IncomingMessage::UnknownNotification(NotificationMessage {
                        method,
                        params,
                    }))
                    .await
                    .unwrap();
            }
        }
    });

    Ok(LspClient {
        request_tx,
        response_rx,
        pending_responses: HashMap::new(),
    })
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ParsedNotification {
    // PublishDiagnostics(TextDocumentPublishDiagnostics),
}

#[derive(Debug)]
pub struct RequestMessage {
    id: i64,
    method: String,
    params: Value,
}

#[derive(Debug)]
pub struct NotificationMessage {
    method: String,
    params: Value,
}

impl NotificationMessage {
    pub fn _new(method: &str, params: Value) -> Self {
        Self {
            method: method.to_string(),
            params,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResponseMessage {
    pub id: i64,
    pub result: Value,
}

#[derive(Debug)]
pub struct ResponseError {
    _code: i64,
    _message: String,
    _data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InitializeParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialization_options: Option<serde_json::Value>,
    pub capabilities: ClientCapabilities,
    // client_info: Option<ClientInfo>,
    // trace: Option<TraceOption>,
    // workspace_folders: Option<Vec<WorkspaceFolder>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ClientCapabilities {
    // workspace: Option<WorkspaceClientCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_document: Option<TextDocumentClientCapabilities>,
    // window: Option<WindowClientCapabilities>,
    // experimental: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextDocumentClientCapabilities {
    // pub completion: Option<CompletionClientCapabilities>,
    // syncrhonization: Option<TextDocumentSyncClientCapabilities>,
    hover: Option<HoverClientCapabilities>,
    // signature_help: Option<SignatureHelpClientCapabilities>,
    // declaration: Option<DeclarationClientCapabilities>,
    // definition: Option<DefinitionClientCapabilities>,
    // type_definition: Option<TypeDefinitionClientCapabilities>,
    // implementation: Option<ImplementationClientCapabilities>,
    // references: Option<ReferencesClientCapabilities>,
    // document_highlight: Option<DocumentHighlightClientCapabilities>,
    // document_symbol: Option<DocumentSymbolClientCapabilities>,
    // code_action: Option<CodeActionClientCapabilities>,
    // code_lens: Option<CodeLensClientCapabilities>,
    // document_link: Option<DocumentLinkClientCapabilities>,
    // color_provider: Option<ColorProviderClientCapabilities>,
    // formatting: Option<FormattingClientCapabilities>,
    // range_formatting: Option<RangeFormattingClientCapabilities>,
    // on_type_formatting: Option<OnTypeFormattingClientCapabilities>,
    // rename: Option<RenameClientCapabilities>,
    // publish_diagnostics: Option<PublishDiagnosticsClientCapabilities>,
    // folding_range: Option<FoldingRangeClientCapabilities>,
    // selection_range: Option<SelectionRangeClientCapabilities>,
    // linked_editing_range: Option<LinkedEditingRangeClientCapabilities>,
    // call_hierarchy: Option<CallHierarchyClientCapabilities>,
    // semantic_tokens: Option<SemanticTokensClientCapabilities>,
    // moniker: Option<MonikerClientCapabilities>,
    // type_hierarchy: Option<TypeHierarchyClientCapabilities>,
    // inline_value: Option<InlineValueClientCapabilities>,
    // inlay_hint: Option<InlayHintClientCapabilities>,
    // diagnostic: Option<DiagnosticClientCapabilities>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HoverClientCapabilities {
    dynamic_registration: Option<bool>,
    content_format: Option<MarkupKind>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionClientCapabilities {
    // dynamic_registration: Option<bool>,
    // pub completion_item: Option<CompletionItem>,
    // completion_item_kind: Option<CompletionItemKindCapabilities>,
    // context_support: Option<bool>,
    // insert_text_mode: Option<InsertTextMode>,
    // completion_list: Option<CompletionListCapabilities>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CompletionItem {
    // pub snippet_support: Option<bool>,
    // pub commit_characters_support: Option<bool>,
    // pub documentation_format: Option<Vec<MarkupKind>>,
    // pub deprecated_support: Option<bool>,
    // pub preselect_support: Option<bool>,
    // pub tag_support: Option<CompletionTag>,
    // pub insert_replace_support: Option<bool>,
    // pub resolve_support: Option<CompletionResolveSupport>,
    // pub insert_text_mode_support: Option<InsertTextMode>,
    // pub label_details_support: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionTag {
    value_set: Vec<CompletionItemTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResolveSupport {
    properties: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertTextMode {
    value_set: Vec<InsertTextMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionItemTag {
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarkupKind {
    PlainText,
    Markdown,
}

impl RequestMessage {
    pub fn new(method: &str, params: Value) -> Self {
        Self {
            id: next_id() as i64,
            method: method.to_string(),
            params,
        }
    }
}

#[derive(Debug)]
pub struct LspClient {
    pub request_tx: mpsc::Sender<OutgoingMessage>,
    pub response_rx: mpsc::Receiver<IncomingMessage>,
    pub pending_responses: HashMap<i64, String>,
}

impl LspClient {
    pub async fn start() -> anyhow::Result<LspClient> {
        lsp_start().await
    }

    pub async fn send_request(&mut self, method: &str, params: Value) -> anyhow::Result<i64> {
        let req = RequestMessage::new(method, params);
        let id = req.id;

        self.pending_responses.insert(id, method.to_string());
        self.request_tx.send(OutgoingMessage::Request(req)).await?;

        tracing::debug!("[LSP] request {id} sent: {:?}", method);
        Ok(id)
    }

    pub async fn send_notification(&mut self, method: &str, params: Value) -> anyhow::Result<()> {
        self.request_tx
            .send(OutgoingMessage::Notification(NotificationMessage {
                method: method.to_string(),
                params,
            }))
            .await?;
        tracing::debug!("[LSP] notification {:?} sent", method);
        Ok(())
    }

    pub async fn try_read_message(
        &mut self,
    ) -> anyhow::Result<Option<(IncomingMessage, Option<String>)>> {
        match self.response_rx.try_recv() {
            Ok(msg) => {
                if let IncomingMessage::Message(msg) = &msg {
                    if let Some(method) = self.pending_responses.remove(&msg.id) {
                        return Ok(Some((IncomingMessage::Message(msg.clone()), Some(method))));
                    }
                }
                Ok(Some((msg, None)))
            }
            Err(TryRecvError::Empty) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn request_hover(
        &mut self,
        file_path: &str,
        row: usize,
        col: usize,
    ) -> anyhow::Result<i64> {
        let file_path = std::fs::canonicalize(file_path)?;
        let params = json!({
            "textDocument": {
                "uri": format!("file://{}", file_path.to_string_lossy()),
            },
            "position": {
                "line": row,
                "character": col
            }
        });

        self.send_request("textDocument/hover", params).await
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        self.send_request(
            "initialize",
            json!({
                "processId": process::id(),
                "clientInfo": {
                    "name": "glyph",
                    "version": "0.1.0",
                },
                "capabilities": {
                    "textDocument": {
                        "hover": {
                            "dynamicRegistration": true,
                            "contentFormat": ["plaintext"]
                        },
                        "completion": {
                            "completionItem": {
                                "snippetSupport": true,
                            }
                        },
                        "definition": {
                            "dynamicRegistration": true,
                            "linkSupport": false,
                        }
                    }
                }
            }),
        )
        .await?;

        _ = self.try_read_message().await;

        self.send_notification("initialized", json!({})).await?;

        Ok(())
    }
}
