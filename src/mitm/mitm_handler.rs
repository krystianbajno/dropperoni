use std::error::Error;
use crate::transport::compression::{compress_body, decompress_body, detect_encoding, split_headers_and_body};
use crate::mitm::mitm::{is_text_content, MitmBuilder, RequestModifier, ResponseModifier};
use crate::mitm::mitm_payload::{CustomRequestModifier, CustomResponseModifier};

pub struct MitmHandler {
    request_modifier: Box<dyn RequestModifier + Send + Sync>,
    response_modifier: Box<dyn ResponseModifier + Send + Sync>,
}

impl MitmHandler {
    pub fn new() -> Self {
        let mitm = MitmBuilder::new()
            .with_request_modifier(Box::new(CustomRequestModifier))
            .with_response_modifier(Box::new(CustomResponseModifier))
            .build();

        MitmHandler {
            request_modifier: mitm.request_modifier,
            response_modifier: mitm.response_modifier,
        }
    }

    pub fn process_request(
        &self,
        request_data: &[u8],
        domain: &str,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        if let Ok(decoded) = std::str::from_utf8(request_data) {
            let modified_request = self.modify_request(decoded, "", domain);
            Ok(modified_request.into_bytes())
        } else {
            Ok(request_data.to_vec())
        }
    }

    pub fn process_response(
        &self,
        response_data: &[u8],
        domain: &str,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let (headers, body) = split_headers_and_body(response_data);

        if is_text_content(headers) {
            let decoded_body = if let Some(encoding) = detect_encoding(headers) {
                decompress_body(body, encoding)?
            } else {
                body.to_vec()
            };

            let modified_body = self.modify_response(&String::from_utf8_lossy(&decoded_body), "", domain);

            let recompressed_body = if let Some(encoding) = detect_encoding(headers) {
                compress_body(modified_body.as_bytes(), encoding)?
            } else {
                modified_body.into_bytes()
            };

            let mut final_response = Vec::new();
            final_response.extend_from_slice(headers);
            final_response.extend_from_slice(&recompressed_body);

            Ok(final_response)
        } else {
            // If not text content, return the original response
            Ok(response_data.to_vec())
        }
    }

    fn modify_request(&self, request: &str, needle: &str, payload: &str) -> String {
        self.request_modifier.modify(request, needle, payload)
    }

    fn modify_response(&self, response: &str, needle: &str, payload: &str) -> String {
        self.response_modifier.modify(response, needle, payload)
    }
}
