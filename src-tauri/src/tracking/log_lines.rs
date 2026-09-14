pub const EVENT_MARKER: &str = "MISTRIA_TRACKER_EVENT|";

pub fn event_json_from_log_line(line: &str) -> Option<&str> {
    let (_, event) = line.split_once(EVENT_MARKER)?;
    let event = event.trim();
    (!event.is_empty()).then_some(event)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_only_non_empty_companion_event_payloads() {
        assert_eq!(
            event_json_from_log_line("[info] MISTRIA_TRACKER_EVENT| {\"type\":\"item_obtained\"}"),
            Some("{\"type\":\"item_obtained\"}")
        );
        assert_eq!(event_json_from_log_line("ordinary game log"), None);
        assert_eq!(event_json_from_log_line("MISTRIA_TRACKER_EVENT|   "), None);
    }
}
