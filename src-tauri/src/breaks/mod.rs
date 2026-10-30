pub use duzenbaz_core::breaks::{machine, night, platform, usage};

use crate::{
    db::Db,
    error::{AppError, AppResult},
};
use chrono::{Local, Timelike};
use machine::{Event, Machine, Phase, IDLE_LIMIT};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

const BREAK_WINDOW: &str = "break";
const SETTINGS_KEY: &str = "breaks.settings";
const FLUSH_EVERY: Duration = Duration::from_secs(30);
const REAPPLY_EVERY: Duration = Duration::from_secs(15);
const PREVIEW_FOR: Duration = Duration::from_secs(8);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NightSettings {
    pub enabled: bool,
    pub start: String,
    pub end: String,
    pub warmth: u8,
}

impl Default for NightSettings {
    fn default() -> Self {
        Self { enabled: false, start: "20:00".into(), end: "07:00".into(), warmth: 40 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BreakSettings {
    pub enabled: bool,
    pub interval_min: u32,
    pub break_secs: u32,
    pub night: NightSettings,
}

impl Default for BreakSettings {
    fn default() -> Self {
        Self { enabled: true, interval_min: 20, break_secs: 20, night: NightSettings::default() }
    }
}

impl BreakSettings {
    fn sanitized(mut self) -> Self {
        self.interval_min = self.interval_min.clamp(1, 180);
        self.break_secs = self.break_secs.clamp(5, 900);
        self.night.warmth = self.night.warmth.min(100);
        let defaults = NightSettings::default();
        if night::parse_hhmm(&self.night.start).is_none() {
            self.night.start = defaults.start;
        }
        if night::parse_hhmm(&self.night.end).is_none() {
            self.night.end = defaults.end;
        }
        self
    }
}

fn load_settings(conn: &Connection) -> AppResult<BreakSettings> {
    let v: Option<String> = conn.query_row("SELECT value FROM settings WHERE key = ?1", [SETTINGS_KEY], |r| r.get(0)).optional()?;
    Ok(v.and_then(|s| serde_json::from_str::<BreakSettings>(&s).ok()).unwrap_or_default().sanitized())
}

fn save_settings(conn: &Connection, s: &BreakSettings) -> AppResult<()> {
    let json = serde_json::to_string(s).map_err(|_| AppError::msg("Ayarlar kaydedilemedi."))?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        (SETTINGS_KEY, json),
    )?;
    Ok(())
}

struct Runtime {
    settings: BreakSettings,
    machine: Machine,
    pending: usage::Pending,
    last_flush: Instant,
    preview: Option<(Instant, f32)>,
    applied: Option<(Instant, f32)>,
}

pub struct Breaks(Mutex<Runtime>);

impl Breaks {
    pub fn new(settings: BreakSettings) -> Self {
        let machine = Machine::new(settings.enabled, settings.interval_min * 60, settings.break_secs);
        Self(Mutex::new(Runtime { settings, machine, pending: Default::default(), last_flush: Instant::now(), preview: None, applied: None }))
    }

    fn lock(&self) -> AppResult<std::sync::MutexGuard<'_, Runtime>> {
        self.0.lock().map_err(|_| AppError::msg("Mola zamanlayıcısı beklenmedik şekilde durdu. Uygulamayı yeniden başlatın."))
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatePayload {
    phase: &'static str,
    work_secs: u32,
    interval_secs: u32,
    break_remaining: u32,
    break_secs: u32,
    enabled: bool,
}

fn payload(rt: &Runtime) -> StatePayload {
    let m = &rt.machine;
    StatePayload {
        phase: if matches!(m.phase, Phase::Break { .. }) { "break" } else { "working" },
        work_secs: m.work_secs,
        interval_secs: m.interval_secs,
        break_remaining: if let Phase::Break { remaining } = m.phase { remaining } else { 0 },
        break_secs: m.break_secs,
        enabled: m.enabled,
    }
}

/// Mola penceresi kısa süre görünür ve odağı çalmaz; başlangıçta gizli oluşturulur.
pub fn create_window(app: &AppHandle) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, BREAK_WINDOW, WebviewUrl::App("index.html?view=break".into()))
        .title("Mola Zamanı")
        .inner_size(380.0, 250.0)
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .visible(false)
        .center()
        .build()?;
    Ok(())
}

fn set_break_window(app: &AppHandle, visible: bool) {
    if let Some(w) = app.get_webview_window(BREAK_WINDOW) {
        if visible {
            let _ = w.center();
            let _ = w.show();
        } else {
            let _ = w.hide();
        }
    }
}

fn on_event(app: &AppHandle, ev: Event) {
    match ev {
        Event::BreakStarted => set_break_window(app, true),
        Event::BreakEnded { completed } => {
            set_break_window(app, false);
            if let Some(db) = app.try_state::<Db>() {
                let day = Local::now().format("%Y-%m-%d").to_string();
                let _ = db.with(|c| usage::log_break(c, &day, completed));
            }
        }
    }
}

fn now_minutes() -> f32 {
    let t = Local::now();
    t.hour() as f32 * 60.0 + t.minute() as f32 + t.second() as f32 / 60.0
}

/// Gece modunun şu an uygulanması gereken sıcaklığı (0 = hiç).
fn target_warmth(rt: &mut Runtime) -> f32 {
    if let Some((until, w)) = rt.preview {
        if Instant::now() < until {
            return w;
        }
        rt.preview = None;
    }
    let n = &rt.settings.night;
    if !n.enabled {
        return 0.0;
    }
    let (Some(s), Some(e)) = (night::parse_hhmm(&n.start), night::parse_hhmm(&n.end)) else { return 0.0 };
    n.warmth as f32 * night::night_factor(now_minutes(), s, e)
}

/// Windows ve bazı uygulamalar renk tablosunu sıfırlayabildiği için tablo düzenli aralıklarla yeniden uygulanır.
fn sync_colors(rt: &mut Runtime) {
    let want = target_warmth(rt);
    match rt.applied {
        Some((at, w)) if want > 0.0 && (w - want).abs() < 0.5 && at.elapsed() < REAPPLY_EVERY => {}
        _ if want > 0.0 => {
            if platform::apply_warmth(want) {
                rt.applied = Some((Instant::now(), want));
            }
        }
        Some(_) => {
            platform::reset_colors();
            rt.applied = None;
        }
        None => {}
    }
}

fn tick(app: &AppHandle) {
    let (Some(state), Some(db)) = (app.try_state::<Breaks>(), app.try_state::<Db>()) else { return };
    let Ok(mut guard) = state.lock() else { return };
    let rt = &mut *guard;

    let idle = platform::idle_secs();
    if idle < IDLE_LIMIT {
        let now = Local::now();
        *rt.pending.entry((now.format("%Y-%m-%d").to_string(), now.hour())).or_default() += 1;
    }
    let event = rt.machine.tick(idle);
    sync_colors(rt);

    if rt.last_flush.elapsed() >= FLUSH_EVERY {
        let _ = db.with(|c| usage::flush(c, &mut rt.pending));
        rt.last_flush = Instant::now();
    }

    let visible = |label: &str| app.get_webview_window(label).and_then(|w| w.is_visible().ok()).unwrap_or(false);
    if visible("main") || visible(BREAK_WINDOW) {
        let _ = app.emit("breaks-state", payload(rt));
    }
    drop(guard);
    if let Some(ev) = event {
        on_event(app, ev);
    }
}

pub fn start(app: AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        tick(&app);
    });
}

/// Uygulama kapanırken birikmiş süreyi yazar ve ekran rengini eski haline getirir.
pub fn on_exit(app: &AppHandle) {
    if let (Some(state), Some(db)) = (app.try_state::<Breaks>(), app.try_state::<Db>()) {
        if let Ok(mut rt) = state.lock() {
            let _ = db.with(|c| usage::flush(c, &mut rt.pending));
        }
    }
    platform::reset_colors();
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsResponse {
    settings: BreakSettings,
    night_supported: bool,
}

#[tauri::command]
pub fn breaks_get_settings(state: State<Breaks>) -> AppResult<SettingsResponse> {
    Ok(SettingsResponse { settings: state.lock()?.settings.clone(), night_supported: platform::NIGHT_SUPPORTED })
}

#[tauri::command]
pub fn breaks_set_settings(app: AppHandle, db: State<Db>, state: State<Breaks>, settings: BreakSettings) -> AppResult<SettingsResponse> {
    let s = settings.sanitized();
    db.with(|c| save_settings(c, &s))?;
    let ended = {
        let mut rt = state.lock()?;
        rt.machine.enabled = s.enabled;
        rt.machine.interval_secs = s.interval_min * 60;
        rt.machine.break_secs = s.break_secs;
        rt.settings = s.clone();
        let ended = if s.enabled { None } else { rt.machine.skip() };
        sync_colors(&mut rt);
        ended
    };
    if let Some(ev) = ended {
        on_event(&app, ev);
    }
    Ok(SettingsResponse { settings: s, night_supported: platform::NIGHT_SUPPORTED })
}

#[tauri::command]
pub fn breaks_get_state(state: State<Breaks>) -> AppResult<StatePayload> {
    let rt = state.lock()?;
    Ok(payload(&rt))
}

fn control(app: &AppHandle, state: &Breaks, f: impl FnOnce(&mut Machine) -> Option<Event>) -> AppResult<()> {
    let (ev, p) = {
        let mut rt = state.lock()?;
        let ev = f(&mut rt.machine);
        (ev, payload(&rt))
    };
    if let Some(ev) = ev {
        on_event(app, ev);
    }
    let _ = app.emit("breaks-state", p);
    Ok(())
}

#[tauri::command]
pub fn breaks_start_now(app: AppHandle, state: State<Breaks>) -> AppResult<()> {
    control(&app, &state, |m| m.start_now())
}

#[tauri::command]
pub fn breaks_skip(app: AppHandle, state: State<Breaks>) -> AppResult<()> {
    control(&app, &state, |m| m.skip())
}

#[tauri::command]
pub fn breaks_snooze(app: AppHandle, state: State<Breaks>, minutes: u32) -> AppResult<()> {
    control(&app, &state, |m| m.snooze(minutes.clamp(1, 60)))
}

/// Gece modunu birkaç saniyeliğine uygulayıp ardından zamanlamaya döner.
#[tauri::command]
pub fn breaks_preview_night(state: State<Breaks>, warmth: u8) -> AppResult<()> {
    if !platform::NIGHT_SUPPORTED {
        return Err(AppError::msg("Gece modu şimdilik yalnızca Windows'ta destekleniyor."));
    }
    let mut rt = state.lock()?;
    rt.preview = Some((Instant::now() + PREVIEW_FOR, warmth.min(100) as f32));
    sync_colors(&mut rt);
    Ok(())
}

#[tauri::command]
pub fn breaks_report(db: State<Db>, state: State<Breaks>, week_start: u32) -> AppResult<usage::Report> {
    {
        let mut rt = state.lock()?;
        db.with(|c| usage::flush(c, &mut rt.pending))?;
    }
    db.with(|c| usage::build_report(c, Local::now().date_naive(), week_start))
}

pub fn initial(db: &Db) -> Breaks {
    Breaks::new(db.with(|c| load_settings(c)).unwrap_or_default())
}
