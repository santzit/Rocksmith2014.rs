pub mod consume_model;
pub mod model_input;
pub mod model_output;
pub mod onnx_runtime;

/// Bundled ML.NET archive path, relative to `rocksmith2014-dd-model/`.
pub const RUST_MLMODEL_RELATIVE_PATH: &str = "model/MLModel.zip";
/// Legacy reference to the .NET submodule model archive path.
pub const DOTNET_MLMODEL_RELATIVE_PATH: &str =
    "../Rocksmith2014.NET/src/Rocksmith2014.DD.Model/MLModel.zip";

pub use consume_model::{inspect_mlnet_model_archive, ArchiveFormat, ModelArchiveInfo, ModelError};
pub use model_input::ModelInput;
pub use model_output::ModelOutput;
pub use onnx_runtime::{OnnxRuntime, OnnxRuntimeError};
