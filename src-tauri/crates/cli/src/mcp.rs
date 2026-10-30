//! MCP (Model Context Protocol) sunucusu. stdio üzerinden, satır başına bir JSON-RPC 2.0 iletisi konuşur.
//! stdout'a yalnızca protokol iletileri yazılır; günlükler stderr'e gider.
//!
//! Güvenlik: araçlar yalnızca izin verilen klasörlerde çalışır (varsayılan: İndirilenler), değişiklik yapan
//! araçlar `--read-only` ile kapatılabilir, pano araçları yalnızca `--allow-clipboard` ile açılır ve dosyalar
//! hiçbir zaman kalıcı silinmez (yalnızca Geri Dönüşüm Kutusu'na gider).
use clap::Args;
use duzenbaz_core::{
    breaks::usage,
    clipboard::{crypto::Vault, store},
    db::Db,
    error::{AppError, AppResult},
    organizer::{dedupe::TrashRequest, plan::Options, service as organizer},
    renamer::{
        rules::{Rules, Status},
        service as renamer,
    },
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    io::{BufRead, Write},
    path::PathBuf,
};

const SUPPORTED_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];
const MAX_MOVES: usize = 200;
const MAX_GROUPS: usize = 100;
const MAX_RENAMES: usize = 500;

const INSTRUCTIONS: &str = "Düzen-Baz tools: organize a folder's files by type, find duplicate files, batch rename files and read \
screen-time statistics. Every organize/rename call returns a batchId that can be undone. Files are never deleted permanently, \
only sent to the Recycle Bin. Always call the *_preview tool first, show the result to the user and only then call the matching \
*_apply tool. Paths are limited to the folders this server was started with.";

#[derive(Args, Debug, Clone)]
pub struct McpArgs {
    /// Only expose tools that never change anything
    #[arg(long)]
    pub read_only: bool,
    /// Also expose clipboard history search (the history may contain sensitive text)
    #[arg(long)]
    pub allow_clipboard: bool,
    /// A folder the tools may work in (repeatable). Default: the Downloads folder
    #[arg(long = "allow-dir", value_name = "DIR")]
    pub allow_dirs: Vec<PathBuf>,
    /// Do not restrict paths at all. Use with care: the connected AI can then touch any file you can
    #[arg(long)]
    pub allow_any_path: bool,
}

pub struct Settings {
    pub read_only: bool,
    pub allow_clipboard: bool,
    /// `None`: kısıtlama yok. Yollar kanonik (gerçek) biçimde tutulur.
    pub allowed_dirs: Option<Vec<PathBuf>>,
}

pub struct Server {
    db: Db,
    cfg: Settings,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Read,
    Change,
    Trash,
}

struct Tool {
    name: &'static str,
    title: &'static str,
    description: &'static str,
    schema: fn() -> Value,
    kind: Kind,
    clipboard: bool,
}

fn object(props: Value, required: &[&str]) -> Value {
    json!({ "type": "object", "properties": props, "required": required, "additionalProperties": false })
}

fn root_schema(extra: Value) -> Value {
    let mut props = json!({
        "root": { "type": "string", "description": "Absolute path of the folder" },
        "byDate": { "type": "boolean", "description": "Also split by month (YYYY-MM) under each type. Default false" }
    });
    if let (Some(p), Some(e)) = (props.as_object_mut(), extra.as_object()) {
        p.extend(e.clone());
    }
    object(props, &["root"])
}

fn rules_schema() -> Value {
    let position = json!({ "type": "string", "enum": ["prefix", "suffix", "replace"] });
    json!({
        "type": "object",
        "description": "How the new names are built. Every field is optional; the extension is always kept.",
        "properties": {
            "sort": { "type": "string", "enum": ["none", "name", "modified"], "description": "Order of the files; sequence numbers follow it" },
            "find": { "type": "string", "description": "Text to find in the name (without extension)" },
            "replace": { "type": "string", "description": "Replacement for `find`" },
            "caseSensitive": { "type": "boolean" },
            "case": { "type": "string", "enum": ["none", "lower", "upper", "title", "sentence"] },
            "prefix": { "type": "string" },
            "suffix": { "type": "string" },
            "date": { "type": "object", "properties": {
                "enabled": { "type": "boolean" },
                "source": { "type": "string", "enum": ["exif", "modified", "created"], "description": "exif = photo date taken" },
                "format": { "type": "string", "description": "strftime format, e.g. %Y-%m-%d" },
                "position": position,
                "separator": { "type": "string" } } },
            "number": { "type": "object", "properties": {
                "enabled": { "type": "boolean" },
                "start": { "type": "integer" },
                "step": { "type": "integer" },
                "digits": { "type": "integer", "description": "Zero padding width" },
                "position": position,
                "separator": { "type": "string" } } }
        }
    })
}

fn batch_schema() -> Value {
    object(json!({ "batchId": { "type": "string", "description": "A batchId returned earlier or listed by the history tool" } }), &["batchId"])
}

fn files_schema() -> Value {
    object(
        json!({
            "files": { "type": "array", "items": { "type": "string" }, "minItems": 1, "description": "Absolute paths of the files (a folder adds the files directly inside it)" },
            "rules": rules_schema()
        }),
        &["files"],
    )
}

fn catalog() -> Vec<Tool> {
    vec![
        Tool {
            name: "organize_preview",
            title: "Preview organizing a folder",
            description: "Shows which files in a folder would be moved into type subfolders (Images, Documents, Videos, Music, Archives, Programs, Other), \
                optionally split by month. Only looks at files directly inside the folder. Changes nothing.",
            schema: || root_schema(json!({})),
            kind: Kind::Read,
            clipboard: false,
        },
        Tool {
            name: "organize_apply",
            title: "Organize a folder",
            description: "Moves the files of a folder into type subfolders. Never overwrites a file. Returns a batchId for organize_undo. \
                Call organize_preview first and confirm with the user.",
            schema: || {
                root_schema(json!({ "sources": { "type": "array", "items": { "type": "string" },
                    "description": "Only move these files (paths exactly as returned by organize_preview). Default: all" } }))
            },
            kind: Kind::Change,
            clipboard: false,
        },
        Tool {
            name: "organize_undo",
            title: "Undo an organize operation",
            description: "Moves the files of an earlier organize_apply back to where they were.",
            schema: batch_schema,
            kind: Kind::Change,
            clipboard: false,
        },
        Tool {
            name: "organize_history",
            title: "List organize operations",
            description: "Lists recent organize operations with their batchId and how many files can still be undone.",
            schema: || object(json!({}), &[]),
            kind: Kind::Read,
            clipboard: false,
        },
        Tool {
            name: "duplicates_find",
            title: "Find duplicate files",
            description: "Finds groups of files with identical content (compared by hash). Within each group the files are sorted oldest first. Changes nothing.",
            schema: || {
                object(
                    json!({ "root": { "type": "string", "description": "Absolute path of the folder" },
                        "recursive": { "type": "boolean", "description": "Look into subfolders too. Default false" } }),
                    &["root"],
                )
            },
            kind: Kind::Read,
            clipboard: false,
        },
        Tool {
            name: "duplicates_trash",
            title: "Send duplicate files to the Recycle Bin",
            description: "For each group keeps one file and sends the listed copies to the Recycle Bin (never deletes permanently). \
                Every copy is re-verified to be identical to the kept file first. Use duplicates_find to build the groups and confirm with the user.",
            schema: || {
                object(
                    json!({ "groups": { "type": "array", "minItems": 1, "items": object(json!({
                        "keep": { "type": "string", "description": "Path of the file to keep" },
                        "remove": { "type": "array", "items": { "type": "string" }, "description": "Identical copies to send to the Recycle Bin" } }),
                        &["keep", "remove"]) } }),
                    &["groups"],
                )
            },
            kind: Kind::Trash,
            clipboard: false,
        },
        Tool {
            name: "rename_preview",
            title: "Preview a batch rename",
            description: "Shows old and new names for the given rules, and flags conflicts or invalid names. Changes nothing.",
            schema: files_schema,
            kind: Kind::Read,
            clipboard: false,
        },
        Tool {
            name: "rename_apply",
            title: "Batch rename files",
            description: "Renames the files all at once or not at all (refuses when any name conflicts). Returns a batchId for rename_undo. \
                Call rename_preview first and confirm with the user.",
            schema: files_schema,
            kind: Kind::Change,
            clipboard: false,
        },
        Tool {
            name: "rename_undo",
            title: "Undo a batch rename",
            description: "Restores the old names of an earlier rename_apply.",
            schema: batch_schema,
            kind: Kind::Change,
            clipboard: false,
        },
        Tool {
            name: "rename_history",
            title: "List batch renames",
            description: "Lists recent renames with their batchId and how many can still be undone.",
            schema: || object(json!({}), &[]),
            kind: Kind::Read,
            clipboard: false,
        },
        Tool {
            name: "screen_time_report",
            title: "Screen time report",
            description: "Today's screen time by hour, the totals of the current week and how many breaks were taken or skipped. Only durations are recorded.",
            schema: || object(json!({ "weekStart": { "type": "integer", "enum": [0, 1], "description": "First day of the week: 1 = Monday (default), 0 = Sunday" } }), &[]),
            kind: Kind::Read,
            clipboard: false,
        },
        Tool {
            name: "clipboard_search",
            title: "Search clipboard history",
            description: "Searches the clipboard history kept by the Düzen-Baz desktop app (text and links; images are listed without content). \
                Newest first. The history can contain sensitive text, so only use it for what the user asked.",
            schema: || {
                object(
                    json!({ "query": { "type": "string", "description": "Text to look for in content, category and tags" },
                        "category": { "type": "string" },
                        "limit": { "type": "integer", "minimum": 1, "maximum": 100, "description": "Default 20" } }),
                    &[],
                )
            },
            kind: Kind::Read,
            clipboard: true,
        },
    ]
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RootArgs {
    root: String,
    #[serde(default)]
    by_date: bool,
    sources: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct DupArgs {
    root: String,
    #[serde(default)]
    recursive: bool,
}

#[derive(Deserialize)]
struct TrashArgs {
    groups: Vec<TrashRequest>,
}

#[derive(Deserialize)]
struct RenameArgs {
    files: Vec<String>,
    #[serde(default)]
    rules: Rules,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchArgs {
    batch_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScreenArgs {
    week_start: Option<u32>,
}

#[derive(Deserialize)]
struct ClipArgs {
    query: Option<String>,
    category: Option<String>,
    limit: Option<usize>,
}

fn parse<T: DeserializeOwned>(args: &Value) -> Result<T, String> {
    serde_json::from_value(args.clone()).map_err(|e| format!("Invalid arguments: {e}"))
}

fn app<T>(r: AppResult<T>) -> Result<T, String> {
    r.map_err(|e| e.user_message())
}

fn cap<T>(mut items: Vec<T>, max: usize) -> (Vec<T>, usize, bool) {
    let total = items.len();
    items.truncate(max);
    let truncated = total > max;
    (items, total, truncated)
}

fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn tool_result(outcome: Result<Value, String>) -> Value {
    match outcome {
        Ok(v) => json!({
            "content": [{ "type": "text", "text": serde_json::to_string_pretty(&v).unwrap_or_default() }],
            "structuredContent": v,
            "isError": false
        }),
        Err(message) => json!({ "content": [{ "type": "text", "text": message }], "isError": true }),
    }
}

impl Server {
    pub fn new(db: Db, cfg: Settings) -> Self {
        Self { db, cfg }
    }

    fn available(&self) -> Vec<Tool> {
        catalog()
            .into_iter()
            .filter(|t| !(self.cfg.read_only && t.kind != Kind::Read))
            .filter(|t| !t.clipboard || self.cfg.allow_clipboard)
            .collect()
    }

    /// Bir JSON-RPC iletisini işler. Bildirimlere (kimliksiz iletiler) yanıt verilmez.
    pub fn handle(&self, msg: &Value) -> Option<Value> {
        let Some(obj) = msg.as_object() else {
            return Some(error_response(Value::Null, -32600, "Invalid Request"));
        };
        let id = obj.get("id").cloned();
        let Some(method) = obj.get("method").and_then(Value::as_str) else {
            // İstemciden gelen yanıtlar yok sayılır; diğerleri geçersiz istektir.
            let is_response = obj.contains_key("result") || obj.contains_key("error");
            return if is_response { None } else { Some(error_response(id.unwrap_or(Value::Null), -32600, "Invalid Request")) };
        };
        let id = id?;
        let params = obj.get("params").cloned().unwrap_or(Value::Null);

        let result: Result<Value, (i64, String)> = match method {
            "initialize" => Ok(self.initialize(&params)),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({ "tools": self.tool_list() })),
            "tools/call" => self.call_tool(&params),
            other => Err((-32601, format!("Method not found: {other}"))),
        };
        Some(match result {
            Ok(r) => json!({ "jsonrpc": "2.0", "id": id, "result": r }),
            Err((code, message)) => error_response(id, code, &message),
        })
    }

    fn initialize(&self, params: &Value) -> Value {
        let requested = params.get("protocolVersion").and_then(Value::as_str);
        let version = requested.filter(|v| SUPPORTED_VERSIONS.contains(v)).unwrap_or(SUPPORTED_VERSIONS[0]);
        json!({
            "protocolVersion": version,
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": "duzenbaz", "title": "Düzen-Baz", "version": env!("CARGO_PKG_VERSION") },
            "instructions": INSTRUCTIONS
        })
    }

    fn tool_list(&self) -> Vec<Value> {
        self.available()
            .into_iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "title": t.title,
                    "description": t.description,
                    "inputSchema": (t.schema)(),
                    "annotations": {
                        "title": t.title,
                        "readOnlyHint": t.kind == Kind::Read,
                        "destructiveHint": t.kind == Kind::Trash,
                        "idempotentHint": t.kind == Kind::Read,
                        "openWorldHint": false
                    }
                })
            })
            .collect()
    }

    fn call_tool(&self, params: &Value) -> Result<Value, (i64, String)> {
        let name = params.get("name").and_then(Value::as_str).ok_or((-32602, "Missing tool name".to_string()))?;
        if !self.available().iter().any(|t| t.name == name) {
            return Err((-32602, format!("Unknown tool: {name}")));
        }
        let args = match params.get("arguments") {
            None | Some(Value::Null) => json!({}),
            Some(a) => a.clone(),
        };
        Ok(tool_result(self.run_tool(name, &args)))
    }

    /// Yolun izin verilen klasörlerden birinin içinde olduğunu doğrular (sembolik bağlar çözülerek).
    fn allow(&self, path: &str) -> Result<(), String> {
        let Some(allowed) = &self.cfg.allowed_dirs else { return Ok(()) };
        let canonical = std::fs::canonicalize(path).map_err(|_| format!("Path not found: {path}"))?;
        if allowed.iter().any(|a| canonical.starts_with(a)) {
            return Ok(());
        }
        let list: Vec<String> = allowed.iter().map(|a| pretty(a)).collect();
        Err(format!(
            "Path is outside the folders this server may use: {path}. Allowed: {}. Start the server with --allow-dir to permit more.",
            list.join(", ")
        ))
    }

    fn run_tool(&self, name: &str, args: &Value) -> Result<Value, String> {
        match name {
            "organize_preview" => {
                let a: RootArgs = parse(args)?;
                self.allow(&a.root)?;
                let (moves, total, truncated) = cap(app(organizer::preview(&a.root, &Options { by_date: a.by_date }))?, MAX_MOVES);
                Ok(json!({ "total": total, "truncated": truncated, "moves": moves }))
            }
            "organize_apply" => {
                let a: RootArgs = parse(args)?;
                self.allow(&a.root)?;
                let options = Options { by_date: a.by_date };
                let selected: HashSet<String> = match a.sources {
                    Some(s) => s.into_iter().collect(),
                    None => app(organizer::preview(&a.root, &options))?.into_iter().map(|m| m.src).collect(),
                };
                to_value(&app(organizer::apply(&self.db, &a.root, &options, &selected))?)
            }
            "organize_undo" => {
                let a: BatchArgs = parse(args)?;
                to_value(&app(organizer::undo(&self.db, &a.batch_id))?)
            }
            "organize_history" => Ok(json!({ "batches": app(organizer::list_history(&self.db))? })),
            "duplicates_find" => {
                let a: DupArgs = parse(args)?;
                self.allow(&a.root)?;
                let groups = app(organizer::find_duplicates(&a.root, a.recursive))?;
                let reclaimable: u64 = groups.iter().map(|g| g.size * (g.files.len() as u64 - 1)).sum();
                let (groups, total, truncated) = cap(groups, MAX_GROUPS);
                Ok(json!({ "totalGroups": total, "truncated": truncated, "reclaimableBytes": reclaimable, "groups": groups }))
            }
            "duplicates_trash" => {
                let a: TrashArgs = parse(args)?;
                for g in &a.groups {
                    self.allow(&g.keep)?;
                    for r in &g.remove {
                        self.allow(r)?;
                    }
                }
                to_value(&app(organizer::trash_duplicates(&a.groups))?)
            }
            "rename_preview" | "rename_apply" => {
                let a: RenameArgs = parse(args)?;
                for f in &a.files {
                    self.allow(f)?;
                }
                let files = app(renamer::expand(&a.files))?;
                if name == "rename_apply" {
                    return to_value(&app(renamer::apply(&self.db, &files, &a.rules))?);
                }
                let items = app(renamer::preview(&files, &a.rules))?;
                let will_change = items.iter().filter(|i| i.status == Status::Ok).count();
                let has_problems = items.iter().any(|i| matches!(i.status, Status::Conflict | Status::Invalid));
                let (items, total, truncated) = cap(items, MAX_RENAMES);
                Ok(json!({ "total": total, "truncated": truncated, "willChange": will_change, "hasProblems": has_problems, "items": items }))
            }
            "rename_undo" => {
                let a: BatchArgs = parse(args)?;
                to_value(&app(renamer::undo(&self.db, &a.batch_id))?)
            }
            "rename_history" => Ok(json!({ "batches": app(renamer::list_history(&self.db))? })),
            "screen_time_report" => {
                let a: ScreenArgs = parse(args)?;
                let week_start = a.week_start.unwrap_or(1);
                let report = app(self.db.with(|c| usage::build_report(c, chrono::Local::now().date_naive(), week_start)))?;
                to_value(&report)
            }
            "clipboard_search" => {
                let a: ClipArgs = parse(args)?;
                let vault = app(Vault::load())?;
                let limit = a.limit.unwrap_or(20).clamp(1, 100);
                let listing = app(self.db.with(|c| {
                    store::list(c, &vault, &a.query.clone().unwrap_or_default(), &a.category.clone().unwrap_or_default(), limit)
                }))?;
                Ok(json!({ "items": listing.items, "categories": listing.categories }))
            }
            other => Err(format!("Unknown tool: {other}")),
        }
    }
}

/// Windows'un kanonik `\\?\C:\...` yollarını kullanıcıya okunur biçimde gösterir (karşılaştırmalarda kullanılmaz).
fn pretty(path: &std::path::Path) -> String {
    let s = path.display().to_string();
    match s.strip_prefix(r"\\?\") {
        Some(rest) if !rest.starts_with("UNC\\") => rest.to_string(),
        _ => s,
    }
}

fn to_value<T: Serialize>(v: &T) -> Result<Value, String> {
    serde_json::to_value(v).map_err(|e| e.to_string())
}

/// Satır satır okur ve yanıtlar; girdi kapanınca (EOF) biter.
pub fn serve<R: BufRead, W: Write>(server: &Server, input: R, mut out: W) -> std::io::Result<()> {
    for line in input.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Value>(line) {
            Ok(msg) => server.handle(&msg),
            Err(e) => Some(error_response(Value::Null, -32700, &format!("Parse error: {e}"))),
        };
        if let Some(r) = response {
            writeln!(out, "{r}")?;
            out.flush()?;
        }
    }
    Ok(())
}

pub fn run(args: McpArgs, db: Db) -> AppResult<()> {
    let allowed_dirs = if args.allow_any_path {
        None
    } else {
        let mut dirs = args.allow_dirs.clone();
        if dirs.is_empty() {
            dirs.extend(dirs::download_dir());
        }
        if dirs.is_empty() {
            return Err(AppError::msg("No folder is allowed. Pass --allow-dir <DIR> (or --allow-any-path)."));
        }
        let mut canonical = Vec::new();
        for d in dirs {
            canonical.push(std::fs::canonicalize(&d).map_err(|_| AppError::msg(format!("Allowed folder not found: {}", d.display())))?);
        }
        Some(canonical)
    };
    let cfg = Settings { read_only: args.read_only, allow_clipboard: args.allow_clipboard, allowed_dirs };
    eprintln!(
        "duzenbaz MCP server {} ready on stdio (read-only: {}, clipboard: {}, folders: {})",
        env!("CARGO_PKG_VERSION"),
        cfg.read_only,
        cfg.allow_clipboard,
        match &cfg.allowed_dirs {
            None => "any".to_string(),
            Some(d) => d.iter().map(|p| pretty(p)).collect::<Vec<_>>().join(", "),
        }
    );
    let server = Server::new(db, cfg);
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    serve(&server, stdin.lock(), stdout.lock())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    fn settings(dir: Option<&Path>) -> Settings {
        Settings {
            read_only: false,
            allow_clipboard: false,
            allowed_dirs: dir.map(|d| vec![fs::canonicalize(d).unwrap()]),
        }
    }

    fn server(cfg: Settings) -> Server {
        Server::new(Db::in_memory().unwrap(), cfg)
    }

    fn rpc(s: &Server, msg: Value) -> Value {
        s.handle(&msg).expect("yanıt bekleniyordu")
    }

    fn call(s: &Server, name: &str, args: Value) -> Value {
        rpc(s, json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": { "name": name, "arguments": args } }))["result"].clone()
    }

    fn tool_names(s: &Server) -> HashSet<String> {
        let r = rpc(s, json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }));
        r["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap().to_string()).collect()
    }

    #[test]
    fn initialize_negotiates_version_and_notifications_get_no_reply() {
        let s = server(settings(None));
        let r = rpc(&s, json!({ "jsonrpc": "2.0", "id": 7, "method": "initialize", "params": { "protocolVersion": "2024-11-05" } }));
        assert_eq!(r["id"], 7);
        assert_eq!(r["result"]["protocolVersion"], "2024-11-05");
        assert!(r["result"]["capabilities"]["tools"].is_object());
        assert_eq!(r["result"]["serverInfo"]["name"], "duzenbaz");

        let r = rpc(&s, json!({ "jsonrpc": "2.0", "id": 8, "method": "initialize", "params": { "protocolVersion": "1999-01-01" } }));
        assert_eq!(r["result"]["protocolVersion"], "2025-06-18");

        assert!(s.handle(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" })).is_none());
        assert_eq!(rpc(&s, json!({ "jsonrpc": "2.0", "id": 9, "method": "ping" }))["result"], json!({}));
    }

    #[test]
    fn tool_list_follows_the_flags_and_every_tool_is_well_formed() {
        let all = tool_names(&server(settings(None)));
        assert!(all.contains("organize_apply") && all.contains("rename_preview"));
        assert!(!all.contains("clipboard_search"), "pano araçları varsayılan olarak kapalı olmalı");

        let mut cfg = settings(None);
        cfg.allow_clipboard = true;
        assert!(tool_names(&server(cfg)).contains("clipboard_search"));

        let mut cfg = settings(None);
        cfg.read_only = true;
        let ro = tool_names(&server(cfg));
        for mutating in ["organize_apply", "organize_undo", "duplicates_trash", "rename_apply", "rename_undo"] {
            assert!(!ro.contains(mutating), "{mutating} salt okunur kipte görünmemeli");
        }
        assert!(ro.contains("organize_preview") && ro.contains("screen_time_report"));

        let mut cfg = settings(None);
        cfg.allow_clipboard = true;
        let r = rpc(&server(cfg), json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }));
        for t in r["result"]["tools"].as_array().unwrap() {
            assert_eq!(t["inputSchema"]["type"], "object", "{}", t["name"]);
            assert!(t["description"].as_str().unwrap().len() > 20);
            assert!(t["annotations"]["readOnlyHint"].is_boolean());
        }
        let trash = r["result"]["tools"].as_array().unwrap().iter().find(|t| t["name"] == "duplicates_trash").unwrap().clone();
        assert_eq!(trash["annotations"]["destructiveHint"], true);
    }

    #[test]
    fn organize_preview_apply_undo_over_mcp() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.jpg"), "x").unwrap();
        let s = server(settings(Some(dir.path())));
        let root = dir.path().to_string_lossy().to_string();

        let r = call(&s, "organize_preview", json!({ "root": root }));
        assert_eq!(r["isError"], false);
        assert_eq!(r["structuredContent"]["total"], 1);
        assert!(dir.path().join("a.jpg").exists(), "önizleme dosyaları taşımamalı");

        let r = call(&s, "organize_apply", json!({ "root": root }));
        assert_eq!(r["structuredContent"]["moved"], 1);
        assert!(dir.path().join("Resimler").join("a.jpg").exists());
        let batch = r["structuredContent"]["batchId"].as_str().unwrap().to_string();

        let h = call(&s, "organize_history", json!({}));
        assert_eq!(h["structuredContent"]["batches"][0]["remaining"], 1);

        let r = call(&s, "organize_undo", json!({ "batchId": batch }));
        assert_eq!(r["structuredContent"]["restored"], 1);
        assert!(dir.path().join("a.jpg").exists());
    }

    #[test]
    fn rename_preview_apply_undo_over_mcp() {
        let dir = tempfile::tempdir().unwrap();
        let files: Vec<String> = ["a.txt", "b.txt"]
            .iter()
            .map(|n| {
                fs::write(dir.path().join(n), n).unwrap();
                dir.path().join(n).to_string_lossy().to_string()
            })
            .collect();
        let s = server(settings(Some(dir.path())));

        let r = call(&s, "rename_preview", json!({ "files": files, "rules": { "prefix": "x_" } }));
        assert_eq!(r["structuredContent"]["willChange"], 2);
        assert_eq!(r["structuredContent"]["hasProblems"], false);

        let r = call(&s, "rename_apply", json!({ "files": files, "rules": { "prefix": "x_" } }));
        assert_eq!(r["structuredContent"]["renamed"], 2);
        assert!(dir.path().join("x_a.txt").exists());

        let batch = r["structuredContent"]["batchId"].as_str().unwrap().to_string();
        assert_eq!(call(&s, "rename_undo", json!({ "batchId": batch }))["structuredContent"]["restored"], 2);
        assert!(dir.path().join("a.txt").exists());

        // Çakışan kural: hiçbir şey değişmemeli ve hata bildirilmeli.
        let r = call(&s, "rename_apply", json!({ "files": files, "rules": { "find": "a", "replace": "b" } }));
        assert_eq!(r["isError"], true);
        assert!(dir.path().join("a.txt").exists());
    }

    #[test]
    fn duplicates_find_over_mcp() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "aynı").unwrap();
        fs::write(dir.path().join("b.txt"), "aynı").unwrap();
        let s = server(settings(Some(dir.path())));
        let r = call(&s, "duplicates_find", json!({ "root": dir.path().to_string_lossy() }));
        assert_eq!(r["structuredContent"]["totalGroups"], 1);
        assert_eq!(r["structuredContent"]["groups"][0]["files"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn paths_outside_the_allowed_folders_are_refused() {
        let allowed = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        fs::write(other.path().join("secret.txt"), "x").unwrap();
        let s = server(settings(Some(allowed.path())));

        let r = call(&s, "organize_preview", json!({ "root": other.path().to_string_lossy() }));
        assert_eq!(r["isError"], true);
        assert!(r["content"][0]["text"].as_str().unwrap().contains("outside"));

        let file = other.path().join("secret.txt").to_string_lossy().to_string();
        assert_eq!(call(&s, "rename_apply", json!({ "files": [file], "rules": { "prefix": "x" } }))["isError"], true);
        assert!(other.path().join("secret.txt").exists());

        // '..' ile dışarı kaçmaya çalışmak da engellenir.
        let sneaky = allowed.path().join("..").join(other.path().file_name().unwrap());
        assert_eq!(call(&s, "organize_preview", json!({ "root": sneaky.to_string_lossy() }))["isError"], true);

        // Kısıtlama kapalıyken (allow-any-path) aynı çağrı çalışır.
        let open = server(settings(None));
        assert_eq!(call(&open, "organize_preview", json!({ "root": other.path().to_string_lossy() }))["isError"], false);
    }

    #[test]
    fn verbatim_windows_prefix_is_hidden_from_messages() {
        assert_eq!(pretty(Path::new(r"\\?\C:\Users\ben")), r"C:\Users\ben");
        assert_eq!(pretty(Path::new(r"\\?\UNC\sunucu\paylasim")), r"\\?\UNC\sunucu\paylasim");
        assert_eq!(pretty(Path::new("/home/ben")), "/home/ben");
    }

    #[test]
    fn errors_are_reported_the_mcp_way() {
        let s = server(settings(None));
        let r = rpc(&s, json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": { "name": "yok_boyle_arac" } }));
        assert_eq!(r["error"]["code"], -32602);

        // Gizli (kapalı) araç da bilinmeyen sayılır.
        let r = rpc(&s, json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": { "name": "clipboard_search" } }));
        assert_eq!(r["error"]["code"], -32602);

        assert_eq!(rpc(&s, json!({ "jsonrpc": "2.0", "id": 1, "method": "resources/list" }))["error"]["code"], -32601);
        assert_eq!(s.handle(&json!([1, 2])).unwrap()["error"]["code"], -32600);

        let r = call(&s, "organize_preview", json!({}));
        assert_eq!(r["isError"], true, "eksik argüman araç hatası olarak dönmeli");
        assert!(r["content"][0]["text"].as_str().unwrap().contains("Invalid arguments"));
    }

    #[test]
    fn serve_reads_lines_and_writes_one_reply_per_request() {
        let s = server(settings(None));
        let input = [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            "bu json degil",
            "",
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        ]
        .join("\n");
        let mut out = Vec::new();
        serve(&s, input.as_bytes(), &mut out).unwrap();

        let lines: Vec<Value> = String::from_utf8(out).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect();
        assert_eq!(lines.len(), 3, "bildirim ve boş satır yanıtsız kalmalı");
        assert_eq!(lines[0]["id"], 1);
        assert_eq!(lines[1]["error"]["code"], -32700);
        assert_eq!(lines[1]["id"], Value::Null);
        assert!(lines[2]["result"]["tools"].as_array().unwrap().len() >= 10);
    }
}
