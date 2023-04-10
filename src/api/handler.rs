pub enum RequestStatus {
    Success,
    Processing,
    Error(String),
}

impl<S> From<S> for RequestStatus
where
    S: ToString,
{
    fn from(m: S) -> Self {
        Self::Error(m.to_string())
    }
}

pub trait ApiHandler {
    fn start_request(&self, connection: &super::ApiConnection);
    fn status_request(&self) -> RequestStatus;
}
