pub trait RequestModifier: Send + Sync {
    fn modify(&self, request: &str, needle: &str, payload: &str) -> String;
}

pub trait ResponseModifier: Send + Sync {
    fn modify(&self, response: &str, needle: &str, payload: &str) -> String;
}

pub struct DefaultRequestModifier;

impl RequestModifier for DefaultRequestModifier {
    fn modify(&self, request: &str, needle: &str, payload: &str) -> String {
        let mut modified_request = String::new();
        for line in request.lines() {
            if line.starts_with(needle) {
                modified_request.push_str(&format!("{}\r\n", payload));
            } else {
                modified_request.push_str(line);
                modified_request.push_str("\r\n");
            }
        }
        modified_request
    }
}

pub struct DefaultResponseModifier;

impl ResponseModifier for DefaultResponseModifier {
    fn modify(&self, response: &str, needle: &str, payload: &str) -> String {
        let mut modified_response = String::new();
        for line in response.lines() {
            if line.contains(needle) {
                modified_response.push_str(&format!("{}\r\n", payload));
                modified_response.push_str("\r\n");
            } else {
                modified_response.push_str(line);
                modified_response.push_str("\r\n");
            }
        }
        modified_response
    }
}

pub struct MitmBuilder {
    request_modifier: Option<Box<dyn RequestModifier + Send + Sync>>,
    response_modifier: Option<Box<dyn ResponseModifier + Send + Sync>>,
}

impl MitmBuilder {
    pub fn new() -> Self {
        Self {
            request_modifier: None,
            response_modifier: None,
        }
    }

    pub fn with_request_modifier(mut self, modifier: Box<dyn RequestModifier + Send + Sync>) -> Self {
        self.request_modifier = Some(modifier);
        self
    }

    pub fn with_response_modifier(mut self, modifier: Box<dyn ResponseModifier + Send + Sync>) -> Self {
        self.response_modifier = Some(modifier);
        self
    }

    pub fn build(self) -> MitmProxy {
        MitmProxy {
            request_modifier: self.request_modifier.unwrap_or_else(|| Box::new(DefaultRequestModifier)),
            response_modifier: self.response_modifier.unwrap_or_else(|| Box::new(DefaultResponseModifier)),
        }
    }
}

pub struct MitmProxy {
    request_modifier: Box<dyn RequestModifier + Send + Sync>,
    response_modifier: Box<dyn ResponseModifier + Send + Sync>,
}

impl MitmProxy {
    pub fn modify_request(&self, request: &str, needle: &str, payload: &str) -> String {
        self.request_modifier.modify(request, needle, payload)
    }

    pub fn modify_response(&self, response: &str, needle: &str, payload: &str) -> String {
        self.response_modifier.modify(response, needle, payload)
    }
}
