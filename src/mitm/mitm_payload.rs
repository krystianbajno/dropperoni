use crate::mitm::mitm::{RequestModifier, ResponseModifier, DefaultRequestModifier, DefaultResponseModifier};

pub struct CustomRequestModifier;

impl RequestModifier for CustomRequestModifier {
    fn modify(&self, request: &str, needle: &str, payload: &str) -> String {
        // Modifying the HOST header is important for proxy to work correctly.
        let payload = format!("Host: {}", payload);
        DefaultRequestModifier.modify(request, "Host:", &payload)
    }
}

pub struct CustomResponseModifier;

impl ResponseModifier for CustomResponseModifier {
    fn modify(&self, response: &str, needle: &str, payload: &str) -> String {
        // Custom logic for modifying the response
        DefaultResponseModifier.modify(response, needle, payload)
    }
}
