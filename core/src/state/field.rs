pub enum StateField {
    Endpoint,
    PreviousEndpoint,
    Env,
}

pub struct State {
    pub handle: Option<String>,
    pub previous_handle: Option<String>,
}

impl Into<&'static str> for StateField {
    fn into(self) -> &'static str {
        match self {
            StateField::Endpoint => "endpoint",
            StateField::PreviousEndpoint => "prev-endpoint",
            StateField::Env => "env",
        }
    }
}
