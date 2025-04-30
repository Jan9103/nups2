use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use nu_plugin::SimplePluginCommand;
use nu_protocol::{Category, LabeledError, Record, Signature, SyntaxShape, Type, Value};
use nups2::pack2_manifest::{Manifest, ManifestDiff};

use crate::Nups2Plugin;

pub struct Pack2ManifestDiff;

impl SimplePluginCommand for Pack2ManifestDiff {
    type Plugin = Nups2Plugin;

    fn name(&self) -> &str {
        "pack2 manifest diff"
    }

    fn description(&self) -> &str {
        "Diff 2 manifest files"
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .input_output_type(
                Type::Nothing,
                Type::list(Type::Record(Box::new([
                    (String::from("name_hash"), Type::String),
                    (String::from("decoded_name"), Type::Any),
                    (String::from("old_hash"), Type::Any),
                    (String::from("new_hash"), Type::Any),
                ]))),
            )
            .required(
                "new_manifest",
                SyntaxShape::Filepath,
                "Path to the newer manifest file",
            )
            .required(
                "old_manifest",
                SyntaxShape::Filepath,
                "Path to the older manifest file",
            )
            .named(
                "filename_list_file",
                SyntaxShape::Filepath,
                "A newline seperated list of filenames, which might be contained in the pack2",
                None,
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
        // parse args
        let filename_list_file: Option<String> =
            crate::util::get_named_argument_str(call, "filename_list_file");
        let new_filename: String = match call.positional[0].clone() {
            Value::String { val, .. } => val,
            _ => {
                return Err(LabeledError::new(
                    "First argument (new manifest) is not a string",
                ));
            }
        };
        let old_filename: String = match call.positional[1].clone() {
            Value::String { val, .. } => val,
            _ => {
                return Err(LabeledError::new(
                    "Second argument (old manifest) is not a string",
                ));
            }
        };

        // load filename list
        let name_lookup_table: HashMap<u64, String> = if let Some(fnlf) = filename_list_file {
            let mut file: File = match File::open(fnlf) {
                Ok(v) => v,
                Err(e) => {
                    return Err(LabeledError::new(format!(
                        "IO-Failed to open filename_list_file: {:?}",
                        e
                    )));
                }
            };
            let mut buf: String = String::new();
            match file.read_to_string(&mut buf) {
                Ok(_) => (),
                Err(e) => {
                    return Err(LabeledError::new(format!(
                        "IO-Failed to read filename_list_file: {:?}",
                        e
                    )));
                }
            };
            let filenames: Vec<String> = buf
                .lines()
                .map(|i| -> String { i.into() })
                .collect::<Vec<String>>();
            nups2::crc64::filename_list_to_lookup_table(&filenames)
        } else {
            HashMap::new()
        };

        // load files
        let new_manifest: Manifest =
            nups2::pack2_manifest::read_manifest_file(&PathBuf::from(new_filename))
                .map_err(|_| LabeledError::new("Failed to read newer manifest file"))?;
        let old_manifest: Manifest =
            nups2::pack2_manifest::read_manifest_file(&PathBuf::from(old_filename))
                .map_err(|_| LabeledError::new("Failed to read older manifest file"))?;

        // diff
        let full_diff: ManifestDiff =
            nups2::pack2_manifest::diff_two_manifests(&old_manifest, &new_manifest);

        // render
        let diff_list: Vec<Value> = full_diff
            .iter()
            .filter(|diff_entry| diff_entry.old_data_hash != diff_entry.new_data_hash)
            .map(|diff_entry| -> Value {
                let mut nu_record = Record::new();

                nu_record.insert(
                    String::from("file_hash"),
                    Value::string(format!("0x{:X}", diff_entry.name_hash), call.head),
                );
                nu_record.insert(
                    String::from("decoded_name"),
                    if let Some(decoded_name) = name_lookup_table.get(&diff_entry.name_hash) {
                        Value::string(decoded_name, call.head)
                    } else {
                        Value::nothing(call.head)
                    },
                );
                nu_record.insert(
                    String::from("old_hash"),
                    if let Some(old_hash) = diff_entry.old_data_hash {
                        Value::string(format!("0x{:X}", old_hash), call.head)
                    } else {
                        Value::nothing(call.head)
                    },
                );
                nu_record.insert(
                    String::from("new_hash"),
                    if let Some(new_hash) = diff_entry.new_data_hash {
                        Value::string(format!("0x{:X}", new_hash), call.head)
                    } else {
                        Value::nothing(call.head)
                    },
                );

                Value::record(nu_record, call.head)
            })
            .collect();

        Ok(Value::list(diff_list, call.head))
    }
}
