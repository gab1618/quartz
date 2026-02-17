pub enum StateField {
    Endpoint,
    PreviousEndpoint,
    Env,
}

pub struct State {
    pub handle: Option<String>,
    pub previous_handle: Option<String>,
}

impl From<StateField> for &'static str {
    fn from(value: StateField) -> Self {
        match value {
            StateField::Endpoint => "endpoint",
            StateField::PreviousEndpoint => "prev-endpoint",
            StateField::Env => "env",
        }
    }
}
