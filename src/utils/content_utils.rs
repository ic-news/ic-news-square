// Utility functions for content processing
use regex::Regex;
use lazy_static::lazy_static;

// Function to strip HTML tags from content
pub fn strip_html_tags(content: &str) -> String {
    lazy_static! {
        // Regex pattern for matching HTML tags
        static ref HTML_TAG_PATTERN: Regex = Regex::new(
            r"<[^>]*>"
        ).unwrap();
    }
    
    // Replace all HTML tags with empty strings
    HTML_TAG_PATTERN.replace_all(content, "").to_string()
}

// Function to calculate content length excluding base64 encoded images and videos and HTML tags
pub fn calculate_content_length_excluding_base64_and_html(content: &str) -> usize {
    // First strip HTML tags
    let content_without_html = strip_html_tags(content);
    
    // Then exclude base64 content
    calculate_content_length_excluding_base64(&content_without_html)
}

// Function to calculate content length excluding base64 encoded images and videos
pub fn calculate_content_length_excluding_base64(content: &str) -> usize {
    lazy_static! {
        // Regex pattern for matching base64 encoded images and videos
        // Matches patterns like data:image/jpeg;base64,... and data:video/mp4;base64,...
        static ref BASE64_PATTERN: Regex = Regex::new(
            r"data:(image|video)/[a-zA-Z0-9]+;base64,[a-zA-Z0-9+/=]+"
        ).unwrap();
    }
    
    // Clone the content to avoid modifying the original
    let mut modified_content = content.to_string();
    
    // Replace all base64 encoded content with empty strings
    for capture in BASE64_PATTERN.find_iter(content) {
        modified_content = modified_content.replace(capture.as_str(), "");
    }
    
    // Return the length of the modified content
    modified_content.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_strip_html_tags() {
        // Test with simple HTML tags
        let content = "<p>This is a paragraph</p>";
        assert_eq!(strip_html_tags(content), "This is a paragraph");
        
        // Test with nested HTML tags
        let content = "<div><p>This is <strong>nested</strong> content</p></div>";
        assert_eq!(strip_html_tags(content), "This is nested content");
        
        // Test with HTML attributes
        let content = "<div class=\"container\"><p style=\"color: red;\">Styled text</p></div>";
        assert_eq!(strip_html_tags(content), "Styled text");
        
        // Test with self-closing tags
        let content = "This has an image <img src=\"image.jpg\" alt=\"image\"/> and a break <br/> in it";
        assert_eq!(strip_html_tags(content), "This has an image  and a break  in it");
        
        // Test with no HTML tags
        let content = "This has no HTML tags";
        assert_eq!(strip_html_tags(content), content);
    }
    
    #[test]
    fn test_calculate_content_length_excluding_base64_and_html() {
        // Test with HTML tags and base64 content
        let content = "<p>This is text with an image: <img src=\"data:image/jpeg;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=\"/></p>";
        let expected_length = "This is text with an image: ".len();
        assert_eq!(calculate_content_length_excluding_base64_and_html(content), expected_length);
        
        // Test with complex HTML and base64
        let content = "<div><h1>Title</h1><p>Text with <strong>bold</strong> and an image: <img src=\"data:image/png;base64,iVBORw0KGgo=\"/></p></div>";
        let expected_length = "TitleText with bold and an image: ".len();
        assert_eq!(calculate_content_length_excluding_base64_and_html(content), expected_length);
    }
    
    #[test]
    fn test_calculate_content_length_excluding_base64() {
        // Test with no base64 content
        let content = "This is a regular text with no base64 content";
        assert_eq!(calculate_content_length_excluding_base64(content), content.len());
        
        // Test with base64 image
        let content_with_image = "This is text with an image: data:image/jpeg;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=";
        let expected_length = "This is text with an image: ".len();
        assert_eq!(calculate_content_length_excluding_base64(content_with_image), expected_length);
        
        // Test with base64 video
        let content_with_video = "This is text with a video: data:video/mp4;base64,AAAAIGZ0eXBpc29tAAACAGlzb21pc28yYXZjMW1wNDEAAAAIZnJlZQAAA";
        let expected_length = "This is text with a video: ".len();
        assert_eq!(calculate_content_length_excluding_base64(content_with_video), expected_length);
        
        // Test with multiple base64 content
        let content_with_multiple = "Image1: data:image/png;base64,iVBORw0KGgo= and Video: data:video/mp4;base64,AAAAIGZ0eXA= and Image2: data:image/jpeg;base64,/9j/4AAQSkZJRg==";
        let expected_length = "Image1:  and Video:  and Image2: ".len();
        assert_eq!(calculate_content_length_excluding_base64(content_with_multiple), expected_length);
    }
}
