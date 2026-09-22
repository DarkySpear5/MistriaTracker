fn main() {
    if std::env::args().any(|argument| argument == "--print-game-directory") {
        match mistria_tracker_lib::steam_discovery::discover_windows_game_directory(None) {
            mistria_tracker_lib::steam_discovery::DiscoveryResult::Found(path) => {
                print!("{}", path.display());
                return;
            }
            mistria_tracker_lib::steam_discovery::DiscoveryResult::NotFound => {
                std::process::exit(2);
            }
        }
    }
    mistria_tracker_lib::run();
}
