pub mod app_state;
pub mod catalog;
pub mod commands;
pub mod compatibility;
pub mod domain;
pub mod instance;
pub mod journal;
pub mod live_save;
pub mod persistence;
pub mod profiles;
pub mod safety;
pub mod save;
pub mod spoilers;
pub mod steam_discovery;
pub mod tracking;

use std::error::Error;
use tauri::Manager;

#[cfg(test)]
mod test_support;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let Some(_instance) = instance::InstanceGuard::acquire() else {
        instance::focus_existing_tracker();
        return;
    };
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let tracker_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| -> Box<dyn Error> { Box::new(error) })?;
            // This opens only app-owned storage. Game-save reads remain behind the probe gate.
            let state = app_state::TrackerState::open(&tracker_data_dir)
                .map_err(|error| -> Box<dyn Error> { Box::new(error) })?;
            app.manage(state);
            app.manage(commands::LiveTrackingRuntime(std::sync::Mutex::new(None)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_journal_snapshot,
            commands::companion_log_available,
            commands::read_journal_note,
            commands::save_journal_note,
            commands::get_journal_art,
            commands::prepare_journal,
            commands::get_active_snapshot,
            commands::get_visible_item_icon,
            commands::import_latest_desktop_backup,
            commands::poll_live_tracking,
            commands::reconcile_active_live_save,
            commands::probe_readiness,
            commands::get_profile_summary,
            commands::get_preferences,
            commands::resolve_game_directory,
            commands::read_item_note,
            commands::save_item_note,
            commands::save_game_directory,
            commands::save_preferences,
            commands::select_profile,
            commands::start_live_tracking,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Mistria Tracker");
}
