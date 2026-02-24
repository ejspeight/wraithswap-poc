use anyhow::{Context, Result};
use chrono::Local;
use colored::Colorize;
use dirs::home_dir;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
struct SwapRow {
    swap_id: String,
    state: String,
    entered_at: String,
}

#[derive(Debug, Clone)]
struct SwapView {
    swap_id: String,
    state: String,
    entered_at: String,
    changed: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let db_path = resolve_asb_db_path();
    let mut previous_states: HashMap<String, String> = HashMap::new();
    let mut pool: Option<SqlitePool> = None;

    loop {
        clear_screen();
        render_header(&db_path);

        match db_path {
            Some(ref path) if path.exists() => {
                // Open pool once; reuse across iterations
                if pool.is_none() {
                    match open_read_only_pool(path).await {
                        Ok(p) => pool = Some(p),
                        Err(err) => {
                            render_error(&format!("Failed to connect (read-only): {err}"));
                            sleep(Duration::from_secs(2)).await;
                            continue;
                        }
                    }
                }

                match fetch_swaps(pool.as_ref().unwrap()).await {
                    Ok(rows) => {
                        if rows.is_empty() {
                            println!("{}", "No swaps yet.".yellow());
                        } else {
                            let views = build_views(rows, &mut previous_states);
                            render_table(&views);
                        }
                        println!();
                        println!("{}", "Watching for changes... (Ctrl+C to exit)".dimmed());
                    }
                    Err(err) => {
                        render_error(&format!("Failed to query swaps: {err}"));
                        // Drop the pool so we reconnect next iteration
                        pool = None;
                    }
                }
            }
            Some(ref path) => {
                render_error(&format!("Database not found yet: {}", path.display()));
                println!("{}", "Start ASB first: ./bin/asb --testnet start".dimmed());
            }
            None => {
                render_error("Could not resolve ASB data directory for this OS.");
            }
        }

        sleep(Duration::from_secs(2)).await;
    }
}

fn resolve_asb_db_path() -> Option<PathBuf> {
    let home = home_dir()?;
    #[cfg(target_os = "macos")]
    {
        Some(home.join("Library/Application Support/xmr-btc-swap/asb/testnet/sqlite"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        Some(home.join(".local/share/xmr-btc-swap/asb/testnet/sqlite"))
    }
}

async fn open_read_only_pool(db_path: &Path) -> Result<SqlitePool> {
    let opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))?
        .read_only(true)
        .create_if_missing(false);

    SqlitePool::connect_with(opts)
        .await
        .with_context(|| format!("open database at {}", db_path.display()))
}

async fn fetch_swaps(pool: &SqlitePool) -> Result<Vec<SwapRow>> {
    // Get the latest state per swap_id from the swap_states table
    let rows = sqlx::query(
        "SELECT swap_id, state, entered_at \
         FROM swap_states \
         WHERE id IN (SELECT MAX(id) FROM swap_states GROUP BY swap_id) \
         ORDER BY entered_at DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|r| SwapRow {
            swap_id: r.get("swap_id"),
            state: r.get("state"),
            entered_at: r.get("entered_at"),
        })
        .collect())
}

fn build_views(rows: Vec<SwapRow>, prev: &mut HashMap<String, String>) -> Vec<SwapView> {
    rows.into_iter()
        .map(|row| {
            let prev_state = prev.get(&row.swap_id).cloned();
            let changed = prev_state.is_some() && prev_state.as_deref() != Some(row.state.as_str());
            prev.insert(row.swap_id.clone(), row.state.clone());

            SwapView {
                swap_id: row.swap_id,
                state: row.state,
                entered_at: row.entered_at,
                changed,
            }
        })
        .collect()
}

fn render_header(db_path: &Option<PathBuf>) {
    let title = "WraithSwap ASB Monitor";
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║{:^62}║", title);
    println!("╠══════════════════════════════════════════════════════════════╣");

    let status = if db_path.as_ref().map(|p| p.exists()).unwrap_or(false) {
        "Connected".green()
    } else {
        "Disconnected".red()
    };

    let db_display = db_path
        .as_ref()
        .map(|p| {
            let s = p.display().to_string();
            if let Some(home) = home_dir() {
                let home_str = home.display().to_string();
                if s.starts_with(&home_str) {
                    return format!("~{}", &s[home_str.len()..]);
                }
            }
            s
        })
        .unwrap_or_else(|| "unknown".to_string());

    let last_updated = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    println!("║ Status: {:<52}║", status);
    println!("║ Database: {:<49}║", db_display);
    println!("║ Last updated: {:<47}║", last_updated);
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
}

fn render_table(views: &[SwapView]) {
    println!("┌──────────┬─────────────────────────┬─────────────────────────┐");
    println!("│ Swap ID  │ State                   │ Entered At              │");
    println!("├──────────┼─────────────────────────┼─────────────────────────┤");

    for view in views {
        let swap_id = truncate_id(&view.swap_id);
        let state = format_state(&view.state, view.changed);
        let entered = if view.entered_at.len() > 23 {
            &view.entered_at[..23]
        } else {
            &view.entered_at
        };

        println!("│ {:<8} │ {:<23} │ {:<23} │", swap_id, state, entered);
    }

    println!("└──────────┴─────────────────────────┴─────────────────────────┘");
}

fn format_state(state: &str, changed: bool) -> String {
    let base = match state {
        "Started" => state.cyan(),
        "BtcLockProofReceived" => state.blue(),
        "XmrLockProofSent" => state.blue(),
        "EncSigSent" => state.yellow(),
        "BtcRedeemed" => format!("{state} ✓").green(),
        "XmrRefunded" => state.magenta(),
        "BtcCancelled" => state.magenta(),
        "BtcPunished" => state.red(),
        "SafelyAborted" => state.dimmed(),
        _ => state.normal(),
    };

    if changed {
        base.bold().to_string()
    } else {
        base.to_string()
    }
}

fn truncate_id(id: &str) -> String {
    if id.len() <= 8 {
        id.to_string()
    } else {
        format!("{}..", &id[0..6])
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

fn render_error(message: &str) {
    println!("{}", format!("Error: {message}").red());
}
