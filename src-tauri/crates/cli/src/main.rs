//! `duzenbaz-cli`: Düzen-Baz'ın komut satırı aracı ve MCP (Model Context Protocol) sunucusu.
//! Masaüstü uygulamasıyla aynı çekirdeği ve aynı veritabanını kullanır.
mod mcp;

use clap::{Args, Parser, Subcommand, ValueEnum};
use duzenbaz_core::{
    breaks::usage::{self, Report},
    clipboard::{crypto::Vault, store},
    db::Db,
    error::{AppError, AppResult},
    history::Batch,
    organizer::{
        dedupe::{DupGroup, TrashRequest},
        plan::{Options, PlannedMove},
        service as organizer,
    },
    renamer::{
        rules::{CaseMode, DateSource, Position, PreviewItem, Rules, Sort, Status},
        service as renamer,
    },
};
use serde::Serialize;
use std::{
    io::{BufRead, IsTerminal, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Parser)]
#[command(name = "duzenbaz-cli", version, about = "Düzen-Baz command line tool and MCP server")]
struct Cli {
    /// Use this data folder instead of the default one shared with the desktop app
    #[arg(long, global = true, value_name = "DIR")]
    data_dir: Option<PathBuf>,
    /// Print machine-readable JSON instead of text
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Sort the files of a folder into type (and month) subfolders
    Organize {
        #[command(subcommand)]
        action: OrganizeCmd,
    },
    /// Find duplicate files and send the extra copies to the Recycle Bin
    Duplicates {
        #[command(subcommand)]
        action: DuplicatesCmd,
    },
    /// Rename many files at once
    Rename {
        #[command(subcommand)]
        action: RenameCmd,
    },
    /// Search the clipboard history kept by the desktop app
    Clipboard {
        #[command(subcommand)]
        action: ClipboardCmd,
    },
    /// Show today's screen time, the week chart data and break statistics
    ScreenTime {
        /// First day of the week: 1 = Monday, 0 = Sunday
        #[arg(long, default_value_t = 1)]
        week_start: u32,
    },
    /// Run as an MCP (Model Context Protocol) server over stdio
    Mcp(mcp::McpArgs),
}

#[derive(Subcommand)]
enum OrganizeCmd {
    /// Show what would be moved (changes nothing)
    Plan {
        root: PathBuf,
        /// Also split by month (YYYY-MM) under each type
        #[arg(long)]
        by_date: bool,
    },
    /// Move the files (asks for confirmation unless --yes is given)
    Apply {
        root: PathBuf,
        #[arg(long)]
        by_date: bool,
        /// Do not ask for confirmation
        #[arg(short, long)]
        yes: bool,
    },
    /// Undo a previous operation
    Undo { batch_id: String },
    /// List recent operations
    History,
}

#[derive(Clone, Copy, ValueEnum)]
enum Keep {
    Oldest,
    Newest,
}

#[derive(Subcommand)]
enum DuplicatesCmd {
    /// List groups of identical files (compared by content)
    Find {
        root: PathBuf,
        /// Look into subfolders as well
        #[arg(long)]
        recursive: bool,
    },
    /// Keep one file per group and send the rest to the Recycle Bin
    Trash {
        root: PathBuf,
        #[arg(long)]
        recursive: bool,
        /// Which copy to keep
        #[arg(long, value_enum, default_value_t = Keep::Oldest)]
        keep: Keep,
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum RenameCmd {
    /// Show the new names (changes nothing)
    Preview(RenameInput),
    /// Rename the files (asks for confirmation unless --yes is given)
    Apply {
        #[command(flatten)]
        input: RenameInput,
        #[arg(short, long)]
        yes: bool,
    },
    /// Undo a previous rename
    Undo { batch_id: String },
    /// List recent renames
    History,
}

#[derive(Subcommand)]
enum ClipboardCmd {
    /// List or search clipboard history, newest first
    List {
        /// Text to search for (content, category and tags)
        #[arg(short, long)]
        query: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(short = 'n', long, default_value_t = 20)]
        limit: usize,
    },
    /// Print one item's content (text) or write it to a file
    Get {
        id: i64,
        /// Write the content to this file (required for images, which are PNG)
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum SortArg {
    None,
    Name,
    Modified,
}

#[derive(Clone, Copy, ValueEnum)]
enum CaseArg {
    Lower,
    Upper,
    Title,
    Sentence,
}

#[derive(Clone, Copy, ValueEnum)]
enum PositionArg {
    Prefix,
    Suffix,
    Replace,
}

#[derive(Clone, Copy, ValueEnum)]
enum DateSourceArg {
    Exif,
    Modified,
    Created,
}

#[derive(Args)]
struct RenameInput {
    /// Files to rename (a folder adds the files directly inside it)
    #[arg(required = true, num_args = 1..)]
    files: Vec<PathBuf>,
    #[command(flatten)]
    rules: RuleArgs,
}

#[derive(Args)]
struct RuleArgs {
    /// Text to find in the file name (without the extension)
    #[arg(long)]
    find: Option<String>,
    /// Replacement for --find (may be empty)
    #[arg(long, requires = "find")]
    replace: Option<String>,
    /// Make --find case sensitive
    #[arg(long)]
    case_sensitive: bool,
    /// Change the letter case
    #[arg(long = "case", value_enum)]
    letter_case: Option<CaseArg>,
    #[arg(long)]
    prefix: Option<String>,
    #[arg(long)]
    suffix: Option<String>,
    /// Order used for sequence numbers
    #[arg(long, value_enum, default_value_t = SortArg::None)]
    sort: SortArg,
    /// Add a date taken from this source
    #[arg(long, value_enum)]
    date: Option<DateSourceArg>,
    /// strftime format of the date
    #[arg(long, default_value = "%Y-%m-%d")]
    date_format: String,
    #[arg(long, value_enum, default_value_t = PositionArg::Prefix)]
    date_position: PositionArg,
    #[arg(long, default_value = "_")]
    date_separator: String,
    /// Add a sequence number
    #[arg(long)]
    number: bool,
    #[arg(long, default_value_t = 1)]
    number_start: i64,
    #[arg(long, default_value_t = 1)]
    number_step: i64,
    #[arg(long, default_value_t = 3)]
    number_digits: usize,
    #[arg(long, value_enum, default_value_t = PositionArg::Suffix)]
    number_position: PositionArg,
    #[arg(long, default_value = "_")]
    number_separator: String,
}

fn position(p: PositionArg) -> Position {
    match p {
        PositionArg::Prefix => Position::Prefix,
        PositionArg::Suffix => Position::Suffix,
        PositionArg::Replace => Position::Replace,
    }
}

impl RuleArgs {
    fn to_rules(&self) -> Rules {
        let mut r = Rules::default();
        r.sort = match self.sort {
            SortArg::None => Sort::None,
            SortArg::Name => Sort::Name,
            SortArg::Modified => Sort::Modified,
        };
        r.find = self.find.clone().unwrap_or_default();
        r.replace = self.replace.clone().unwrap_or_default();
        r.case_sensitive = self.case_sensitive;
        r.case = match self.letter_case {
            None => CaseMode::None,
            Some(CaseArg::Lower) => CaseMode::Lower,
            Some(CaseArg::Upper) => CaseMode::Upper,
            Some(CaseArg::Title) => CaseMode::Title,
            Some(CaseArg::Sentence) => CaseMode::Sentence,
        };
        r.prefix = self.prefix.clone().unwrap_or_default();
        r.suffix = self.suffix.clone().unwrap_or_default();
        if let Some(src) = self.date {
            r.date.enabled = true;
            r.date.source = match src {
                DateSourceArg::Exif => DateSource::Exif,
                DateSourceArg::Modified => DateSource::Modified,
                DateSourceArg::Created => DateSource::Created,
            };
        }
        r.date.format = self.date_format.clone();
        r.date.position = position(self.date_position);
        r.date.separator = self.date_separator.clone();
        r.number.enabled = self.number;
        r.number.start = self.number_start;
        r.number.step = self.number_step;
        r.number.digits = self.number_digits;
        r.number.position = position(self.number_position);
        r.number.separator = self.number_separator.clone();
        r
    }
}

impl RenameInput {
    /// Klasörleri dosyalara açar ve kuralları çekirdeğin anladığı biçime çevirir.
    fn resolve(&self) -> AppResult<(Vec<String>, Rules)> {
        let paths: Vec<String> = self.files.iter().map(|p| p.to_string_lossy().to_string()).collect();
        Ok((renamer::expand(&paths)?, self.rules.to_rules()))
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    if let Some(dir) = &cli.data_dir {
        std::env::set_var("DUZENBAZ_DATA_DIR", dir);
    }
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e.user_message());
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> AppResult<()> {
    let json = cli.json;
    match cli.command {
        Command::Organize { action } => organize_cmd(json, action),
        Command::Duplicates { action } => duplicates_cmd(json, action),
        Command::Rename { action } => rename_cmd(json, action),
        Command::Clipboard { action } => clipboard_cmd(json, action),
        Command::ScreenTime { week_start } => {
            let db = Db::open_default()?;
            let report = db.with(|c| usage::build_report(c, chrono::Local::now().date_naive(), week_start))?;
            emit(json, &report, screen_time_text)
        }
        Command::Mcp(args) => mcp::run(args, Db::open_default()?),
    }
}

fn organize_cmd(json: bool, action: OrganizeCmd) -> AppResult<()> {
    match action {
        OrganizeCmd::Plan { root, by_date } => {
            let moves = organizer::preview(&path_string(&root), &Options { by_date })?;
            emit(json, &moves, |m| plan_text(m))
        }
        OrganizeCmd::Apply { root, by_date, yes } => {
            let root = path_string(&root);
            let options = Options { by_date };
            let moves = organizer::preview(&root, &options)?;
            if moves.is_empty() {
                return emit(json, &moves, |_| "Nothing to organize.".into());
            }
            if !json {
                println!("{}", plan_text(&moves));
            }
            confirm(yes, &format!("Move {} file(s)?", moves.len()))?;
            let selected = moves.iter().map(|m| m.src.clone()).collect();
            let report = organizer::apply(&Db::open_default()?, &root, &options, &selected)?;
            emit(json, &report, |r| {
                let mut s = format!("Moved {} file(s). Undo with: duzenbaz-cli organize undo {}", r.moved, r.batch_id);
                for f in &r.failed {
                    s.push_str(&format!("\n  could not move {}: {}", f.path, f.message));
                }
                s
            })
        }
        OrganizeCmd::Undo { batch_id } => {
            let report = organizer::undo(&Db::open_default()?, &batch_id)?;
            emit(json, &report, |r| {
                let mut s = format!("Restored {} file(s).", r.restored);
                for f in &r.failed {
                    s.push_str(&format!("\n  could not restore {}: {}", f.path, f.message));
                }
                s
            })
        }
        OrganizeCmd::History => emit(json, &organizer::list_history(&Db::open_default()?)?, |b| history_text(b)),
    }
}

fn duplicates_cmd(json: bool, action: DuplicatesCmd) -> AppResult<()> {
    match action {
        DuplicatesCmd::Find { root, recursive } => {
            let groups = organizer::find_duplicates(&path_string(&root), recursive)?;
            emit(json, &groups, |g| duplicates_text(g))
        }
        DuplicatesCmd::Trash { root, recursive, keep, yes } => {
            let groups = organizer::find_duplicates(&path_string(&root), recursive)?;
            if groups.is_empty() {
                return emit(json, &groups, |_| "No duplicate files found.".into());
            }
            if !json {
                println!("{}", duplicates_text(&groups));
            }
            // Gruplar eskiden yeniye sıralı gelir: en eski dosya ilk sıradadır.
            let requests: Vec<TrashRequest> = groups
                .iter()
                .map(|g| {
                    let keep_at = match keep {
                        Keep::Oldest => 0,
                        Keep::Newest => g.files.len() - 1,
                    };
                    TrashRequest {
                        keep: g.files[keep_at].path.clone(),
                        remove: g.files.iter().enumerate().filter(|(i, _)| *i != keep_at).map(|(_, f)| f.path.clone()).collect(),
                    }
                })
                .collect();
            let count: usize = requests.iter().map(|r| r.remove.len()).sum();
            confirm(yes, &format!("Send {count} duplicate file(s) to the Recycle Bin?"))?;
            let report = organizer::trash_duplicates(&requests)?;
            emit(json, &report, |r| format!("Sent {} file(s) to the Recycle Bin ({} skipped).", r.trashed, r.skipped))
        }
    }
}

fn rename_cmd(json: bool, action: RenameCmd) -> AppResult<()> {
    match action {
        RenameCmd::Preview(input) => {
            let (files, rules) = input.resolve()?;
            emit(json, &renamer::preview(&files, &rules)?, |i| rename_text(i))
        }
        RenameCmd::Apply { input, yes } => {
            let (files, rules) = input.resolve()?;
            let items = renamer::preview(&files, &rules)?;
            if items.iter().any(|i| matches!(i.status, Status::Conflict | Status::Invalid)) {
                if !json {
                    println!("{}", rename_text(&items));
                }
                return Err(AppError::msg("Some names conflict or are invalid; nothing was changed. Fix the rules and try again."));
            }
            let changes = items.iter().filter(|i| i.status == Status::Ok).count();
            if changes == 0 {
                return emit(json, &items, |_| "Nothing to rename.".into());
            }
            if !json {
                println!("{}", rename_text(&items));
            }
            confirm(yes, &format!("Rename {changes} file(s)?"))?;
            let report = renamer::apply(&Db::open_default()?, &files, &rules)?;
            emit(json, &report, |r| format!("Renamed {} file(s). Undo with: duzenbaz-cli rename undo {}", r.renamed, r.batch_id))
        }
        RenameCmd::Undo { batch_id } => {
            let report = renamer::undo(&Db::open_default()?, &batch_id)?;
            emit(json, &report, |r| format!("Restored {} file name(s).", r.restored))
        }
        RenameCmd::History => emit(json, &renamer::list_history(&Db::open_default()?)?, |b| history_text(b)),
    }
}

fn clipboard_cmd(json: bool, action: ClipboardCmd) -> AppResult<()> {
    let db = Db::open_default()?;
    let vault = Vault::load()?;
    match action {
        ClipboardCmd::List { query, category, limit } => {
            let listing = db.with(|c| store::list(c, &vault, &query.unwrap_or_default(), &category.unwrap_or_default(), limit))?;
            emit(json, &listing.items, |items| {
                if items.is_empty() {
                    return "Clipboard history is empty or nothing matched.".into();
                }
                items
                    .iter()
                    .map(|i| {
                        let first_line = i.preview.lines().next().unwrap_or("").chars().take(100).collect::<String>();
                        let text = if i.kind == "image" { format!("image {}x{}", i.width.unwrap_or(0), i.height.unwrap_or(0)) } else { first_line };
                        let pin = if i.pinned { " (pinned)" } else { "" };
                        format!("#{:<5} [{}]{} {}", i.id, i.kind, pin, text)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }
        ClipboardCmd::Get { id, out } => {
            let (kind, bytes) = db.with(|c| store::get_content(c, &vault, id))?;
            match out {
                Some(path) => {
                    std::fs::write(&path, &bytes)?;
                    eprintln!("Wrote {} byte(s) to {}", bytes.len(), path.display());
                }
                None if kind == "image" => return Err(AppError::msg("This item is an image. Use --out <file.png> to save it.")),
                None => std::io::stdout().write_all(&bytes)?,
            }
            Ok(())
        }
    }
}

fn path_string(p: &Path) -> String {
    p.to_string_lossy().to_string()
}

fn emit<T: Serialize>(json: bool, value: &T, human: impl FnOnce(&T) -> String) -> AppResult<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(value).map_err(|e| AppError::msg(e.to_string()))?);
    } else {
        println!("{}", human(value));
    }
    Ok(())
}

/// Değişiklik yapmadan önce onay ister. Etkileşimli değilse `--yes` şarttır.
fn confirm(yes: bool, question: &str) -> AppResult<()> {
    if yes {
        return Ok(());
    }
    if !std::io::stdin().is_terminal() {
        return Err(AppError::msg("Confirmation required. Run again with --yes to proceed without a prompt."));
    }
    eprint!("{question} [y/N] ");
    std::io::stderr().flush()?;
    let mut answer = String::new();
    std::io::stdin().lock().read_line(&mut answer)?;
    if matches!(answer.trim().to_lowercase().as_str(), "y" | "yes" | "e" | "evet") {
        Ok(())
    } else {
        Err(AppError::msg("Cancelled. Nothing was changed."))
    }
}

fn size_text(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let (mut v, mut i) = (bytes as f64, 0);
    while v >= 1024.0 && i < units.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{bytes} B")
    } else {
        format!("{v:.1} {}", units[i])
    }
}

fn duration_text(secs: u32) -> String {
    let m = (secs as f64 / 60.0).round() as u32;
    if m < 60 {
        format!("{m} min")
    } else {
        format!("{} h {} min", m / 60, m % 60)
    }
}

fn plan_text(moves: &[PlannedMove]) -> String {
    if moves.is_empty() {
        return "Nothing to organize.".into();
    }
    let total: u64 = moves.iter().map(|m| m.size).sum();
    let mut lines: Vec<String> = moves.iter().map(|m| format!("  {}  ->  {}", m.src, m.dst)).collect();
    lines.push(format!("{} file(s), {}", moves.len(), size_text(total)));
    lines.join("\n")
}

fn history_text(batches: &[Batch]) -> String {
    if batches.is_empty() {
        return "No operations yet.".into();
    }
    batches
        .iter()
        .map(|b| {
            let state = if b.remaining == 0 { "undone".to_string() } else { format!("{} to undo", b.remaining) };
            format!("{}  {}  {} file(s)  [{}]", b.batch_id, b.created_at, b.total, state)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn duplicates_text(groups: &[DupGroup]) -> String {
    if groups.is_empty() {
        return "No duplicate files found.".into();
    }
    let reclaim: u64 = groups.iter().map(|g| g.size * (g.files.len() as u64 - 1)).sum();
    let mut out = String::new();
    for g in groups {
        out.push_str(&format!("{} x {}\n", g.files.len(), size_text(g.size)));
        for f in &g.files {
            out.push_str(&format!("  {}  ({})\n", f.path, f.modified));
        }
    }
    out.push_str(&format!("{} group(s), {} can be reclaimed", groups.len(), size_text(reclaim)));
    out
}

fn rename_text(items: &[PreviewItem]) -> String {
    if items.is_empty() {
        return "No files.".into();
    }
    items
        .iter()
        .map(|i| match i.status {
            Status::Unchanged => format!("  {}  (unchanged)", i.old_name),
            Status::Ok if i.message.is_empty() => format!("  {}  ->  {}", i.old_name, i.new_name),
            Status::Ok => format!("  {}  ->  {}   ({})", i.old_name, i.new_name, i.message),
            Status::Conflict => format!("  {}  ->  {}   CONFLICT: {}", i.old_name, i.new_name, i.message),
            Status::Invalid => format!("  {}   INVALID: {}", i.old_name, i.message),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn screen_time_text(r: &Report) -> String {
    let today: u32 = r.hours.iter().sum();
    let mut out = format!(
        "Today: {}   breaks taken: {}   skipped: {}\n\nThis week:\n",
        duration_text(today),
        r.breaks_taken,
        r.breaks_skipped
    );
    let max = r.days.iter().map(|d| d.seconds).max().unwrap_or(0).max(1);
    for (i, d) in r.days.iter().enumerate() {
        let bar = "#".repeat((d.seconds as f64 / max as f64 * 30.0).round() as usize);
        let marker = if i == r.today_index { " <- today" } else { "" };
        out.push_str(&format!("  {}  {:>9}  {}{}\n", d.day, duration_text(d.seconds), bar, marker));
    }
    out.trim_end().to_string()
}
