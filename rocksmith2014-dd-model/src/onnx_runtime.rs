use crate::{ModelInput, ModelOutput};
use std::path::Path;
use tract_onnx::prelude::*;

type RunnableModel = SimplePlan<TypedFact, Box<dyn TypedOp>, TypedModel>;

pub struct OnnxRuntime {
    model: RunnableModel,
}

#[derive(Debug, thiserror::Error)]
pub enum OnnxRuntimeError {
    #[error("tract error: {0}")]
    Tract(#[from] tract_onnx::prelude::TractError),
    #[error("model output did not contain any tensors")]
    EmptyOutput,
    #[error("model output shape is not supported")]
    InvalidOutputShape,
}

impl OnnxRuntime {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, OnnxRuntimeError> {
        let model = tract_onnx::onnx()
            .model_for_path(path.as_ref())?
            .into_optimized()?
            .into_runnable()?;

        Ok(Self { model })
    }

    pub fn predict(&self, input: &ModelInput) -> Result<ModelOutput, OnnxRuntimeError> {
        let features = input.to_onnx_features();
        let tensor: Tensor = tract_ndarray::arr2(&[features]).into_tensor();

        let outputs = self.model.run(tvec!(tensor.into()))?;
        let first = outputs.first().ok_or(OnnxRuntimeError::EmptyOutput)?;
        let values = first.to_array_view::<f32>()?;
        let score = values
            .iter()
            .next()
            .copied()
            .ok_or(OnnxRuntimeError::InvalidOutputShape)?;

        Ok(ModelOutput { score })
    }
}

#[cfg(test)]
mod tests {
    use super::OnnxRuntime;

    #[test]
    fn loading_missing_onnx_model_returns_error() {
        let missing = std::env::temp_dir().join("rocksmith2014-dd-model-missing.onnx");
        let result = OnnxRuntime::load(missing);
        assert!(result.is_err());
    }
}
