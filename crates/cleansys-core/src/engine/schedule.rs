//! Scheduled automatic cleaning.
//!
//! A [`Schedule`] (frequency, time, scope) is persisted to
//! `~/.config/cleansys/schedule.json` and turned into a native OS job that
//! runs `cleansys auto --scheduled`:
//!
//! | OS | Backend |
//! |----|---------|
//! | Linux | systemd user timer (preferred) or the user's crontab |
//! | macOS | launchd user agent (`~/Library/LaunchAgents`) |
//! | Windows | Task Scheduler (`schtasks`) |
//!
//! The job reads `schedule.json` at run time, so changing the *scope* needs
//! no reinstall — only a changed frequency/time does (the UI does this for
//! you). Unattended runs only ever touch user-land cleaners and never
//! `caution`-risk ones unless you explicitly picked them.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(all(unix, not(target_os = "macos")))]
use std::process::Stdio;

pub const TASK_NAME: &str = "CleanSys";
pub const LAUNCHD_LABEL: &str = "io.github.sorinirimies.cleansys";
const CRON_BEGIN: &str = "# >>> cleansys (managed) >>>";
const CRON_END: &str = "# <<< cleansys (managed) <<<";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
}

/// What an unattended run cleans.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Only `safe` user-land cleaners (the "recommended" preset).
    Recommended,
    /// `safe` + `moderate` user-land cleaners (slower to regenerate).
    Extended,
    /// Exactly the cleaner ids listed in [`Schedule::ids`].
    Selected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Backend {
    /// Best available for this OS (Linux: systemd if usable, else cron).
    Auto,
    Systemd,
    Cron,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Schedule {
    pub frequency: Frequency,
    pub hour: u8,
    pub minute: u8,
    /// 0 = Sunday … 6 = Saturday (weekly).
    pub weekday: u8,
    /// 1–28 (monthly).
    pub day_of_month: u8,
    pub scope: Scope,
    /// Cleaner ids for [`Scope::Selected`].
    pub ids: Vec<String>,
    pub backend: Backend,
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            frequency: Frequency::Weekly,
            hour: 3,
            minute: 30,
            weekday: 0,
            day_of_month: 1,
            scope: Scope::Recommended,
            ids: Vec::new(),
            backend: Backend::Auto,
        }
    }
}

pub const WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

/// Parse `3:30`, `03:30`, `0330`-less forms: `H:MM`/`HH:MM`.
pub fn parse_time(s: &str) -> Result<(u8, u8)> {
    let (h, m) = s
        .trim()
        .split_once(':')
        .with_context(|| format!("time '{s}' must look like HH:MM"))?;
    let (h, m): (u8, u8) = (
        h.parse().with_context(|| format!("bad hour in '{s}'"))?,
        m.parse().with_context(|| format!("bad minute in '{s}'"))?,
    );
    if h > 23 || m > 59 {
        bail!("time '{s}' out of range (00:00–23:59)");
    }
    Ok((h, m))
}

/// Parse a weekday name/abbreviation (`sun`, `Monday`) or number 0–6.
pub fn parse_weekday(s: &str) -> Result<u8> {
    let l = s.trim().to_lowercase();
    if let Ok(n) = l.parse::<u8>() {
        if n <= 6 {
            return Ok(n);
        }
    }
    WEEKDAYS
        .iter()
        .position(|d| d.to_lowercase().starts_with(&l) && l.len() >= 3)
        .map(|i| i as u8)
        .with_context(|| format!("unknown weekday '{s}'"))
}

impl Schedule {
    pub fn validate(&self) -> Result<()> {
        if self.hour > 23 || self.minute > 59 {
            bail!("invalid time {:02}:{:02}", self.hour, self.minute);
        }
        if self.weekday > 6 {
            bail!("weekday must be 0–6");
        }
        if !(1..=28).contains(&self.day_of_month) {
            bail!("day of month must be 1–28");
        }
        if self.scope == Scope::Selected && self.ids.is_empty() {
            bail!("scope 'selected' needs at least one cleaner id");
        }
        Ok(())
    }

    /// Human-readable summary, e.g. "Weekly on Sunday at 03:30".
    pub fn describe(&self) -> String {
        let at = format!("{:02}:{:02}", self.hour, self.minute);
        match self.frequency {
            Frequency::Daily => format!("Daily at {at}"),
            Frequency::Weekly => format!(
                "Weekly on {} at {at}",
                WEEKDAYS[usize::from(self.weekday.min(6))]
            ),
            Frequency::Monthly => format!("Monthly on day {} at {at}", self.day_of_month),
        }
    }

    pub fn scope_label(&self) -> String {
        match self.scope {
            Scope::Recommended => "Recommended (safe caches only)".into(),
            Scope::Extended => "Extended (safe + moderate)".into(),
            Scope::Selected => format!("{} selected cleaner(s)", self.ids.len()),
        }
    }

    // ── persistence ────────────────────────────────────────────────────

    pub fn path() -> Result<PathBuf> {
        Ok(crate::settings::settings_dir()?.join("schedule.json"))
    }

    pub fn load() -> Option<Self> {
        let s = std::fs::read_to_string(Self::path().ok()?).ok()?;
        serde_json::from_str(&s).ok()
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        if let Some(d) = path.parent() {
            std::fs::create_dir_all(d)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

/// Outcome of the most recent scheduled/auto run, shown in the UIs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LastRun {
    /// Seconds since the Unix epoch.
    pub at: u64,
    pub bytes_freed: u64,
    pub items: usize,
    pub cleaners_run: usize,
    pub skipped: usize,
    pub failed: usize,
    pub scheduled: bool,
}

impl LastRun {
    pub fn path() -> Result<PathBuf> {
        Ok(crate::settings::settings_dir()?.join("last_run.json"))
    }

    pub fn load() -> Option<Self> {
        serde_json::from_str(&std::fs::read_to_string(Self::path().ok()?).ok()?).ok()
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        if let Some(d) = path.parent() {
            std::fs::create_dir_all(d)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// "3 hours ago", "2 days ago", …
    pub fn ago(&self) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let s = now.saturating_sub(self.at);
        match s {
            0..=59 => "just now".into(),
            60..=3599 => format!("{} min ago", s / 60),
            3600..=86_399 => format!("{} h ago", s / 3600),
            _ => format!("{} d ago", s / 86_400),
        }
    }
}

// ── installation state ────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledJob {
    /// Which mechanism holds the job ("systemd timer", "cron", "launchd", "Task Scheduler").
    pub backend: &'static str,
}

pub fn exe_path() -> Result<PathBuf> {
    std::env::current_exe().context("cannot determine the cleansys executable path")
}

/// Directory for scheduled-run logs.
pub fn log_path() -> PathBuf {
    if cfg!(target_os = "macos") {
        crate::cleaners::platform::home_dir().map(|h| h.join("Library/Logs/cleansys-scheduled.log"))
    } else {
        crate::settings::settings_dir()
            .ok()
            .map(|d| d.join("scheduled.log"))
    }
    .unwrap_or_else(|| std::env::temp_dir().join("cleansys-scheduled.log"))
}

// ── generators (pure, unit-tested) ────────────────────────────────────

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// launchd plist for the schedule.
pub fn launchd_plist(s: &Schedule, exe: &Path, log: &Path) -> String {
    let mut cal = format!(
        "    <key>Hour</key><integer>{}</integer>\n    <key>Minute</key><integer>{}</integer>\n",
        s.hour, s.minute
    );
    match s.frequency {
        Frequency::Daily => {}
        Frequency::Weekly => cal.push_str(&format!(
            "    <key>Weekday</key><integer>{}</integer>\n",
            s.weekday
        )),
        Frequency::Monthly => cal.push_str(&format!(
            "    <key>Day</key><integer>{}</integer>\n",
            s.day_of_month
        )),
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>{label}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{exe}</string>
    <string>auto</string>
    <string>--scheduled</string>
  </array>
  <key>StartCalendarInterval</key>
  <dict>
{cal}  </dict>
  <key>ProcessType</key><string>Background</string>
  <key>Nice</key><integer>19</integer>
  <key>LowPriorityIO</key><true/>
  <key>StandardOutPath</key><string>{log}</string>
  <key>StandardErrorPath</key><string>{log}</string>
</dict>
</plist>
"#,
        label = LAUNCHD_LABEL,
        exe = xml_escape(&exe.to_string_lossy()),
        log = xml_escape(&log.to_string_lossy()),
        cal = cal
    )
}

/// systemd `OnCalendar=` expression.
pub fn systemd_on_calendar(s: &Schedule) -> String {
    let t = format!("{:02}:{:02}:00", s.hour, s.minute);
    match s.frequency {
        Frequency::Daily => format!("*-*-* {t}"),
        Frequency::Weekly => {
            const D: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
            format!("{} *-*-* {t}", D[usize::from(s.weekday.min(6))])
        }
        Frequency::Monthly => format!("*-*-{:02} {t}", s.day_of_month),
    }
}

/// `(service, timer)` unit file contents.
pub fn systemd_units(s: &Schedule, exe: &Path) -> (String, String) {
    let service = format!(
        "[Unit]\nDescription=CleanSys scheduled cleanup\n\n[Service]\nType=oneshot\nExecStart=\"{}\" auto --scheduled\nNice=19\nIOSchedulingClass=idle\n",
        exe.display()
    );
    let timer = format!(
        "[Unit]\nDescription=CleanSys scheduled cleanup ({})\n\n[Timer]\nOnCalendar={}\nPersistent=true\nRandomizedDelaySec=300\n\n[Install]\nWantedBy=timers.target\n",
        s.describe(),
        systemd_on_calendar(s)
    );
    (service, timer)
}

/// crontab line (without the managed-block markers).
pub fn cron_line(s: &Schedule, exe: &Path, log: &Path) -> String {
    let (dom, dow) = match s.frequency {
        Frequency::Daily => ("*".to_string(), "*".to_string()),
        Frequency::Weekly => ("*".to_string(), s.weekday.to_string()),
        Frequency::Monthly => (s.day_of_month.to_string(), "*".to_string()),
    };
    format!(
        "{} {} {} * {}  '{}' auto --scheduled >> '{}' 2>&1",
        s.minute,
        s.hour,
        dom,
        dow,
        exe.display(),
        log.display()
    )
}

/// `schtasks /Create` argument list.
pub fn schtasks_args(s: &Schedule, exe: &Path) -> Vec<String> {
    const D: [&str; 7] = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
    let mut a: Vec<String> = vec![
        "/Create".into(),
        "/TN".into(),
        TASK_NAME.into(),
        "/F".into(),
    ];
    match s.frequency {
        Frequency::Daily => a.extend(["/SC".into(), "DAILY".into()]),
        Frequency::Weekly => a.extend([
            "/SC".into(),
            "WEEKLY".into(),
            "/D".into(),
            D[usize::from(s.weekday.min(6))].into(),
        ]),
        Frequency::Monthly => a.extend([
            "/SC".into(),
            "MONTHLY".into(),
            "/D".into(),
            s.day_of_month.to_string(),
        ]),
    }
    a.extend([
        "/ST".into(),
        format!("{:02}:{:02}", s.hour, s.minute),
        "/TR".into(),
        format!("\"{}\" auto --scheduled", exe.display()),
    ]);
    a
}

/// Insert/replace the managed block in an existing crontab text.
pub fn cron_with_block(existing: &str, line: &str) -> String {
    let mut out = strip_cron_block(existing);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&format!("{CRON_BEGIN}\n{line}\n{CRON_END}\n"));
    out
}

/// Remove the managed block (if any) from a crontab text.
pub fn strip_cron_block(existing: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for l in existing.lines() {
        if l.trim() == CRON_BEGIN {
            inside = true;
            continue;
        }
        if l.trim() == CRON_END {
            inside = false;
            continue;
        }
        if !inside {
            out.push_str(l);
            out.push('\n');
        }
    }
    out
}

// ── platform operations ───────────────────────────────────────────────

fn run(cmd: &str, args: &[&str]) -> Result<std::process::Output> {
    Command::new(cmd)
        .args(args)
        .output()
        .with_context(|| format!("failed to run `{cmd}`"))
}

fn run_ok(cmd: &str, args: &[&str]) -> Result<()> {
    let out = run(cmd, args)?;
    if !out.status.success() {
        bail!(
            "`{cmd} {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(())
}

fn home() -> Result<PathBuf> {
    crate::cleaners::platform::home_dir().context("no home directory")
}

#[cfg(all(unix, not(target_os = "macos")))]
fn systemd_usable() -> bool {
    super::paths::find_program("systemctl").is_some()
        && Command::new("systemctl")
            .args(["--user", "show-environment"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn systemd_dir() -> Result<PathBuf> {
    Ok(home()?.join(".config/systemd/user"))
}

#[cfg(target_os = "macos")]
fn launchd_plist_path() -> Result<PathBuf> {
    Ok(home()?
        .join("Library/LaunchAgents")
        .join(format!("{LAUNCHD_LABEL}.plist")))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn read_crontab() -> String {
    run("crontab", &["-l"])
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

#[cfg(all(unix, not(target_os = "macos")))]
fn write_crontab(text: &str) -> Result<()> {
    use std::io::Write;
    let mut child = Command::new("crontab")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to run `crontab`")?;
    child
        .stdin
        .take()
        .context("no stdin")?
        .write_all(text.as_bytes())?;
    let out = child.wait_with_output()?;
    if !out.status.success() {
        bail!(
            "crontab rejected the entry: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(())
}

/// Install (or replace) the OS job for `schedule` and persist it.
pub fn install(schedule: &Schedule) -> Result<InstalledJob> {
    schedule.validate()?;
    let exe = exe_path()?;
    let log = log_path();
    if let Some(d) = log.parent() {
        std::fs::create_dir_all(d).ok();
    }
    // Remove any previous job (possibly of another backend) first.
    remove_os_job().ok();
    schedule.save()?;

    #[cfg(target_os = "macos")]
    {
        let plist = launchd_plist_path()?;
        std::fs::create_dir_all(plist.parent().context("no parent")?)?;
        std::fs::write(&plist, launchd_plist(schedule, &exe, &log))?;
        let uid = String::from_utf8_lossy(&run("id", &["-u"])?.stdout)
            .trim()
            .to_string();
        let target = format!("gui/{uid}");
        let p = plist.to_string_lossy();
        if run_ok("launchctl", &["bootstrap", &target, &p]).is_err() {
            run_ok("launchctl", &["load", "-w", &p])?;
        }
        return Ok(InstalledJob { backend: "launchd" });
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let use_systemd = match schedule.backend {
            Backend::Systemd => true,
            Backend::Cron => false,
            Backend::Auto => systemd_usable(),
        };
        if use_systemd {
            let dir = systemd_dir()?;
            std::fs::create_dir_all(&dir)?;
            let (svc, timer) = systemd_units(schedule, &exe);
            std::fs::write(dir.join("cleansys.service"), svc)?;
            std::fs::write(dir.join("cleansys.timer"), timer)?;
            run_ok("systemctl", &["--user", "daemon-reload"])?;
            run_ok(
                "systemctl",
                &["--user", "enable", "--now", "cleansys.timer"],
            )?;
            return Ok(InstalledJob {
                backend: "systemd timer",
            });
        }
        if super::paths::find_program("crontab").is_none() {
            bail!("neither a usable systemd user session nor `crontab` was found");
        }
        let text = cron_with_block(&read_crontab(), &cron_line(schedule, &exe, &log));
        write_crontab(&text)?;
        return Ok(InstalledJob { backend: "cron" });
    }

    #[cfg(windows)]
    {
        let args = schtasks_args(schedule, &exe);
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        run_ok("schtasks", &refs)?;
        return Ok(InstalledJob {
            backend: "Task Scheduler",
        });
    }

    #[allow(unreachable_code)]
    {
        let _ = (exe, log);
        bail!("scheduling is not supported on this platform")
    }
}

fn remove_os_job() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let plist = launchd_plist_path()?;
        if plist.exists() {
            let uid = String::from_utf8_lossy(&run("id", &["-u"])?.stdout)
                .trim()
                .to_string();
            let label = format!("gui/{uid}/{LAUNCHD_LABEL}");
            if run_ok("launchctl", &["bootout", &label]).is_err() {
                run("launchctl", &["unload", "-w", &plist.to_string_lossy()]).ok();
            }
            std::fs::remove_file(plist)?;
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let dir = systemd_dir()?;
        let timer = dir.join("cleansys.timer");
        if timer.exists() {
            run(
                "systemctl",
                &["--user", "disable", "--now", "cleansys.timer"],
            )
            .ok();
            std::fs::remove_file(&timer).ok();
            std::fs::remove_file(dir.join("cleansys.service")).ok();
            run("systemctl", &["--user", "daemon-reload"]).ok();
        }
        if super::paths::find_program("crontab").is_some() {
            let cur = read_crontab();
            if cur.contains(CRON_BEGIN) {
                write_crontab(&strip_cron_block(&cur))?;
            }
        }
    }
    #[cfg(windows)]
    {
        run("schtasks", &["/Delete", "/TN", TASK_NAME, "/F"]).ok();
    }
    Ok(())
}

/// Remove the OS job and the saved schedule.
pub fn remove() -> Result<()> {
    remove_os_job()?;
    if let Ok(p) = Schedule::path() {
        std::fs::remove_file(p).ok();
    }
    Ok(())
}

/// Whether an OS job is currently installed.
pub fn installed() -> Option<InstalledJob> {
    #[cfg(target_os = "macos")]
    {
        return launchd_plist_path()
            .ok()
            .filter(|p| p.exists())
            .map(|_| InstalledJob { backend: "launchd" });
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if systemd_dir().ok()?.join("cleansys.timer").exists() {
            return Some(InstalledJob {
                backend: "systemd timer",
            });
        }
        if read_crontab().contains(CRON_BEGIN) {
            return Some(InstalledJob { backend: "cron" });
        }
        return None;
    }
    #[cfg(windows)]
    {
        return run("schtasks", &["/Query", "/TN", TASK_NAME])
            .ok()
            .filter(|o| o.status.success())
            .map(|_| InstalledJob {
                backend: "Task Scheduler",
            });
    }
    #[allow(unreachable_code)]
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sched(f: Frequency) -> Schedule {
        Schedule {
            frequency: f,
            hour: 4,
            minute: 5,
            weekday: 1,
            day_of_month: 15,
            ..Schedule::default()
        }
    }

    #[test]
    fn parses_times_and_weekdays() {
        assert_eq!(parse_time("3:30").unwrap(), (3, 30));
        assert_eq!(parse_time("23:59").unwrap(), (23, 59));
        assert!(parse_time("24:00").is_err());
        assert!(parse_time("noon").is_err());
        assert_eq!(parse_weekday("sun").unwrap(), 0);
        assert_eq!(parse_weekday("Friday").unwrap(), 5);
        assert_eq!(parse_weekday("6").unwrap(), 6);
        assert!(parse_weekday("x").is_err());
    }

    #[test]
    fn validation() {
        assert!(Schedule::default().validate().is_ok());
        let mut s = Schedule {
            day_of_month: 31,
            ..Schedule::default()
        };
        assert!(s.validate().is_err());
        s = Schedule {
            scope: Scope::Selected,
            ..Schedule::default()
        };
        assert!(s.validate().is_err());
    }

    #[test]
    fn describes() {
        assert_eq!(sched(Frequency::Daily).describe(), "Daily at 04:05");
        assert_eq!(
            sched(Frequency::Weekly).describe(),
            "Weekly on Monday at 04:05"
        );
        assert_eq!(
            sched(Frequency::Monthly).describe(),
            "Monthly on day 15 at 04:05"
        );
    }

    #[test]
    fn cron_lines() {
        let exe = Path::new("/usr/bin/cleansys");
        let log = Path::new("/tmp/l.log");
        assert!(cron_line(&sched(Frequency::Daily), exe, log).starts_with("5 4 * * *  "));
        assert!(cron_line(&sched(Frequency::Weekly), exe, log).starts_with("5 4 * * 1  "));
        assert!(cron_line(&sched(Frequency::Monthly), exe, log).starts_with("5 4 15 * *  "));
        assert!(cron_line(&sched(Frequency::Daily), exe, log).contains("auto --scheduled"));
    }

    #[test]
    fn cron_block_is_idempotent_and_preserves_other_lines() {
        let base = "0 1 * * * backup.sh\n";
        let once = cron_with_block(base, "line1");
        let twice = cron_with_block(&once, "line2");
        assert!(twice.contains("backup.sh"));
        assert!(twice.contains("line2") && !twice.contains("line1"));
        assert_eq!(twice.matches(CRON_BEGIN).count(), 1);
        assert_eq!(strip_cron_block(&twice), base);
    }

    #[test]
    fn systemd_calendar() {
        assert_eq!(
            systemd_on_calendar(&sched(Frequency::Daily)),
            "*-*-* 04:05:00"
        );
        assert_eq!(
            systemd_on_calendar(&sched(Frequency::Weekly)),
            "Mon *-*-* 04:05:00"
        );
        assert_eq!(
            systemd_on_calendar(&sched(Frequency::Monthly)),
            "*-*-15 04:05:00"
        );
        let (svc, timer) = systemd_units(&sched(Frequency::Daily), Path::new("/x/cleansys"));
        assert!(svc.contains("ExecStart=\"/x/cleansys\" auto --scheduled"));
        assert!(timer.contains("Persistent=true"));
    }

    #[test]
    fn launchd_plist_has_calendar_keys() {
        let p = launchd_plist(
            &sched(Frequency::Weekly),
            Path::new("/a&b/cleansys"),
            Path::new("/l"),
        );
        assert!(p.contains("<key>Weekday</key><integer>1</integer>"));
        assert!(p.contains("<key>Hour</key><integer>4</integer>"));
        assert!(p.contains("/a&amp;b/cleansys"));
        assert!(
            !launchd_plist(&sched(Frequency::Daily), Path::new("/c"), Path::new("/l"))
                .contains("Weekday")
        );
        assert!(
            launchd_plist(&sched(Frequency::Monthly), Path::new("/c"), Path::new("/l"))
                .contains("<key>Day</key><integer>15</integer>")
        );
    }

    #[test]
    fn schtasks_arguments() {
        let a = schtasks_args(&sched(Frequency::Weekly), Path::new("C:\\cs\\cleansys.exe"));
        assert!(a.windows(2).any(|w| w == ["/SC", "WEEKLY"]));
        assert!(a.windows(2).any(|w| w == ["/D", "MON"]));
        assert!(a.windows(2).any(|w| w == ["/ST", "04:05"]));
        assert!(a.last().unwrap().contains("auto --scheduled"));
    }

    #[test]
    fn last_run_ago() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_eq!(
            LastRun {
                at: now,
                ..Default::default()
            }
            .ago(),
            "just now"
        );
        assert!(LastRun {
            at: now - 7200,
            ..Default::default()
        }
        .ago()
        .contains('h'));
    }
}
