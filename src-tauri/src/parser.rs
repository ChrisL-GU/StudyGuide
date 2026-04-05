use std::io::Read;
use std::path::Path;

pub fn parse_file(path: &Path) -> Result<String, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "txt" | "md" => {
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
        }
        "pdf" => parse_pdf(path),
        "docx" => parse_docx(path),
        _ => Err(format!("Unsupported file type: .{}", ext)),
    }
}

fn parse_pdf(path: &Path) -> Result<String, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("Failed to read PDF: {}", e))?;
    pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| format!("Failed to extract PDF text: {}", e))
}

fn parse_docx(path: &Path) -> Result<String, String> {
    let file =
        std::fs::File::open(path).map_err(|e| format!("Failed to open DOCX: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read DOCX as ZIP: {}", e))?;

    let mut xml_content = String::new();
    {
        let mut doc_file = archive
            .by_name("word/document.xml")
            .map_err(|e| format!("Failed to find document.xml in DOCX: {}", e))?;
        doc_file
            .read_to_string(&mut xml_content)
            .map_err(|e| format!("Failed to read document.xml: {}", e))?;
    }

    extract_text_from_xml(&xml_content)
}

fn extract_text_from_xml(xml: &str) -> Result<String, String> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(xml);
    let mut text = String::new();
    let mut in_text_element = false;
    let mut in_paragraph = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local = e.local_name();
                if local.as_ref() == b"t" {
                    in_text_element = true;
                } else if local.as_ref() == b"p" {
                    in_paragraph = true;
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_text_element {
                    if let Ok(t) = e.unescape() {
                        text.push_str(&t);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let local = e.local_name();
                if local.as_ref() == b"t" {
                    in_text_element = false;
                } else if local.as_ref() == b"p" {
                    if in_paragraph {
                        text.push('\n');
                        in_paragraph = false;
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
    }

    Ok(text.trim().to_string())
}
