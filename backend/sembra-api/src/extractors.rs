//! File extraction utilities for PDF, DOCX, TXT, MD

use anyhow::{Result, Context};
use bytes::Bytes;

/// Extract text from PDF bytes
pub fn extract_pdf(data: &[u8]) -> Result<String> {
    pdf_extract::extract_text_from_mem(data)
        .context("Failed to extract text from PDF")
}

/// Extract text from DOCX bytes
pub fn extract_docx(data: &[u8]) -> Result<String> {
    let doc = docx_rs::read_docx(data)
        .map_err(|e| anyhow::anyhow!("Failed to parse DOCX: {:?}", e))?;
    
    let mut text = String::new();
    
    // Extract text from document body
    for child in doc.document.children {
        if let docx_rs::DocumentChild::Paragraph(p) = child {
            for pc in p.children {
                if let docx_rs::ParagraphChild::Run(r) = pc {
                    for rc in r.children {
                        if let docx_rs::RunChild::Text(t) = rc {
                            text.push_str(&t.text);
                        }
                    }
                }
            }
            text.push('\n');
        }
    }
    
    Ok(text)
}

/// Extract text from plain text or markdown
pub fn extract_text(data: &[u8]) -> Result<String> {
    String::from_utf8(data.to_vec())
        .context("Failed to decode text file as UTF-8")
}

/// Detect file type and extract text
pub fn extract_from_bytes(data: &[u8], filename: &str) -> Result<String> {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    
    match ext.as_str() {
        "pdf" => extract_pdf(data),
        "docx" => extract_docx(data),
        "txt" | "md" | "markdown" | "text" => extract_text(data),
        _ => Err(anyhow::anyhow!("Unsupported file format: {}", ext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_text() {
        let data = b"Hello, world!";
        let result = extract_text(data).unwrap();
        assert_eq!(result, "Hello, world!");
    }
}
