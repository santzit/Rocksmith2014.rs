use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveFormat {
    MlNetZip,
    Onnx,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelArchiveInfo {
    pub format: ArchiveFormat,
    pub entries: Vec<String>,
    pub training_version: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

pub fn inspect_mlnet_model_archive(path: impl AsRef<Path>) -> Result<ModelArchiveInfo, ModelError> {
    let path = path.as_ref();

    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("onnx"))
    {
        return Ok(ModelArchiveInfo {
            format: ArchiveFormat::Onnx,
            entries: Vec::new(),
            training_version: None,
        });
    }

    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut entries = Vec::with_capacity(archive.len());
    let mut training_version = None;

    for i in 0..archive.len() {
        let mut zf = archive.by_index(i)?;
        let normalized_name = zf.name().replace('\\', "/");

        if normalized_name.eq_ignore_ascii_case("TrainingInfo/Version.txt") {
            let mut text = String::new();
            zf.read_to_string(&mut text)?;
            training_version = Some(text.trim().to_string());
        }

        entries.push(normalized_name);
    }

    let format = if entries
        .iter()
        .any(|e| e.eq_ignore_ascii_case("TransformerChain/Model.key"))
    {
        ArchiveFormat::MlNetZip
    } else {
        ArchiveFormat::Unknown
    };

    Ok(ModelArchiveInfo {
        format,
        entries,
        training_version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use std::path::Path;

    #[test]
    fn detects_mlnet_model_layout() {
        let mut bytes = Vec::new();
        {
            let cursor = Cursor::new(&mut bytes);
            let mut writer = zip::ZipWriter::new(cursor);
            let options = zip::write::SimpleFileOptions::default();

            writer
                .start_file("TrainingInfo\\Version.txt", options)
                .unwrap();
            writer.write_all(b"TestVersion").unwrap();
            writer
                .start_file("TransformerChain\\Model.key", options)
                .unwrap();
            writer.write_all(b"dummy").unwrap();
            writer
                .start_file("TransformerChain\\Transform_003\\Model\\Model.key", options)
                .unwrap();
            writer.write_all(b"dummy").unwrap();
            writer.finish().unwrap();
        }

        let temp_name = format!(
            "rocksmith2014-dd-model-test-{}-{}.zip",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let temp_path = std::env::temp_dir().join(temp_name);
        std::fs::write(&temp_path, bytes).unwrap();

        let info = inspect_mlnet_model_archive(&temp_path).unwrap();
        std::fs::remove_file(&temp_path).unwrap();

        assert_eq!(info.format, ArchiveFormat::MlNetZip);
        assert!(info
            .entries
            .iter()
            .any(|name| name == "TransformerChain/Transform_003/Model/Model.key"));
        assert_eq!(info.training_version.as_deref(), Some("TestVersion"));
    }

    #[test]
    fn detects_onnx_by_extension() {
        let info = inspect_mlnet_model_archive("model.onnx").unwrap();
        assert_eq!(info.format, ArchiveFormat::Onnx);
    }

    #[test]
    fn inspects_dotnet_dd_mlmodel_when_available() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../Rocksmith2014.NET/src/Rocksmith2014.DD.Model/MLModel.zip");
        if !path.exists() {
            return;
        }

        let info = inspect_mlnet_model_archive(path).unwrap();
        assert_eq!(info.format, ArchiveFormat::MlNetZip);
        assert!(info
            .entries
            .iter()
            .any(|e| e.eq_ignore_ascii_case("TrainingInfo/Version.txt")));
    }
}
