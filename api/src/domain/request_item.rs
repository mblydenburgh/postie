#[derive(Debug, PartialEq)]
pub struct RequestHistoryItem {
    pub id: String,
    pub request_id: String,
    pub response_id: String,
    pub sent_at: String,
    pub response_time: usize,
}
