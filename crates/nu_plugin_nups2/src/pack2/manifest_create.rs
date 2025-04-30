use std::{fs::File, path::PathBuf};

use nu_plugin::SimplePluginCommand;
use nu_protocol::{Category, LabeledError, Signature, SyntaxShape, Type, Value};
use nups2::pack2::Pack2;

use crate::Nups2Plugin;

pub struct Pack2ManifestCreate;

impl SimplePluginCommand for Pack2ManifestCreate {
    type Plugin = Nups2Plugin;

    fn name(&self) -> &str {
        "pack2 manifest create"
    }
    fn description(&self) -> &str {
        "Create a new manifest file from a pack2 file"
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .input_output_type(Type::Nothing, Type::Nothing)
            .required("pack2_file", SyntaxShape::Filepath, "The pack2 file")
            .required(
                "manifest_file",
                SyntaxShape::Filepath,
                "The target manifest file",
            )
            .category(Category::Misc)
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &nu_plugin::EngineInterface,
        call: &nu_plugin::EvaluatedCall,
        _input: &nu_protocol::Value,
    ) -> Result<nu_protocol::Value, nu_protocol::LabeledError> {
        let pack2_filename: String = match call.positional[0].clone() {
            Value::String { val, .. } => val,
            _ => {
                return Err(LabeledError::new(
                    "First argument (pack2-filepath) is not a string",
                ));
            }
        };
        let manifest_filename: String = match call.positional[1].clone() {
            Value::String { val, .. } => val,
            _ => {
                return Err(LabeledError::new(
                    "Second argument (manifest-filepath) is not a string",
                ));
            }
        };
        let mut file: File = match File::open(pack2_filename) {
            Ok(v) => v,
            Err(e) => {
                return Err(LabeledError::new(format!(
                    "IO-Failed to open pack2 file: {:?}",
                    e
                )))
            }
        };
        let pack2: Pack2 = match Pack2::load_from_file(&mut file) {
            Ok(v) => v,
            Err(e) => {
                return Err(LabeledError::new(format!(
                    "Failed to parse pack2 file: {:?}",
                    e
                )));
            }
        };

        let manifest_file: PathBuf = PathBuf::from(manifest_filename);
        pack2
            .write_manifest_file(&manifest_file)
            .map_err(|_| LabeledError::new("Failed to write manifest file"))?;

        Ok(Value::Nothing {
            internal_span: call.head,
        })
    }
}
