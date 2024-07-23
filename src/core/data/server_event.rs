use crate::core::util::base64_encode;

#[derive(Debug, Clone)]
pub struct ServerEvent {
    pub event: Option<String>,
    pub data: String,
    pub id: Option<String>,
    pub retry: Option<u64>,
}

impl ServerEvent {
    /**
     * Create a new ServerEvent with just data.
     */
    pub fn data(data: &str) -> Self {
        ServerEvent {
            data: data.to_string(),
            event: None,
            id: None,
            retry: None,
        }
    }

    pub fn keep_alive() -> Self {
        ServerEvent {
            data: "event: keep-alive\ndata: ping\n".to_string(),
            event: None,
            id: None,
            retry: None,
        }
    }

    /**
     * Create a new ServerEvent with an event and data, the data will be base64 encoded.
     * when creating a new instance.
     */
    pub fn event(event: &str, data: String) -> Self {
        let base_64 = base64_encode(data.as_bytes());

        ServerEvent {
            data: base_64,
            event: Some(event.to_string()),
            id: None,
            retry: None,
        }
    }

    /**
     * Format the ServerEvent to be sent to the client.
     * https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events
     */
    pub fn format(&self) -> String {
        let mut event = String::new();
        if let Some(event_name) = &self.event {
            event.push_str(&format!("event: {}\n", event_name));
        }
        if let Some(event_id) = &self.id {
            event.push_str(&format!("id: {}\n", event_id));
        }
        if let Some(retry) = &self.retry {
            event.push_str(&format!("retry: {}\n", retry));
        }
        event.push_str(&format!("data: {}\n\n", self.data));
        event
    }

    /**
     * Format the ServerEvent and convert to bytes to be sent to the client.
     */
    pub fn to_bytes(&self) -> Vec<u8> {
        self.format().as_bytes().to_vec()
    }
}
