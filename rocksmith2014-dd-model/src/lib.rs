pub mod consume_model;
pub mod model_input;
pub mod model_output;
pub mod onnx_runtime;

/// This crate does not currently bundle a model binary.
/// For ML.NET archive inspection, use the model shipped in the .NET submodule:
/// `../Rocksmith2014.NET/src/Rocksmith2014.DD.Model/MLModel.zip`
pub const DOTNET_MLMODEL_RELATIVE_PATH: &str =
    "../Rocksmith2014.NET/src/Rocksmith2014.DD.Model/MLModel.zip";

pub use consume_model::{inspect_mlnet_model_archive, ArchiveFormat, ModelArchiveInfo, ModelError};
pub use model_input::ModelInput;
pub use model_output::ModelOutput;
pub use onnx_runtime::{OnnxRuntime, OnnxRuntimeError};
