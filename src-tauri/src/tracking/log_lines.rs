pub const EVENT_MARKER: &str = "MISTRIA_TRACKER_EVENT|";
const MAX_EVENT_JSON_BYTES: usize = 16 * 1024;

pub fn event_json_from_log_line(line: &str) -> Option<&str> {
    let (_, event) = line.split_once(EVENT_MARKER)?;
    let event = event.trim();
    (!event.is_empty() && event.len() <= MAX_EVENT_JSON_BYTES).then_some(event)
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

    #[test]
    fn rejects_an_event_payload_larger_than_the_companion_contract_limit() {
        let line = format!("{EVENT_MARKER}{}", "x".repeat(16 * 1024 + 1));

        assert_eq!(event_json_from_log_line(&line), None);
    }
}
