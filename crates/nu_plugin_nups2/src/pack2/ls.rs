use std::fs::File;
use std::io::Read;

use nu_plugin::EngineInterface;
use nu_plugin::EvaluatedCall;
use nu_plugin::SimplePluginCommand;
use nu_protocol::Category;
use nu_protocol::LabeledError;
use nu_protocol::Record;
use nu_protocol::Signature;
use nu_protocol::SyntaxShape;
use nu_protocol::Type;
use nu_protocol::Value;
use nups2::pack2::Pack2;

use crate::Nups2Plugin;

pub struct Pack2LsCommand;

impl SimplePluginCommand for Pack2LsCommand {
    type Plugin = Nups2Plugin;

    fn name(&self) -> &str {
        "pack2 ls"
    }
    fn description(&self) -> &str {
        "List the contents of a pack2 file\nNOTE: name_hash is a string since it is often a bigger number than nushell can handle (u64::MAX > i64::MAX)"
    }
    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .input_output_type(
                Type::Nothing,
                Type::list(Type::Record(Box::new([
                    (String::from("name_hash"), Type::Any),
                    (String::from("decoded_name"), Type::Any),
                    (String::from("is_zipped"), Type::Bool),
                    (String::from("data_hash"), Type::Int),
                    (String::from("data_length"), Type::Int),
                    (String::from("unzipped_length"), Type::Int),
                ]))),
            )
            .required("file", SyntaxShape::Filepath, "The pack2 file")
            .named(
                "filename_list_file",
                SyntaxShape::Filepath,
                "A newline seperated list of filenames, which might be contained in the pack2",
                None,
            )
            .category(Category::Formats)
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let filename_list_file: Option<String> =
            crate::util::get_named_argument_str(call, "filename_list_file");
        let filename: String = match call.positional[0].clone() {
            Value::String { val, .. } => val,
            _ => {
                return Err(LabeledError::new(
                    "First argument (pack2-filepath) is not a string",
                ));
            }
        };
        let mut file: File = match File::open(filename) {
            Ok(v) => v,
            Err(e) => {
                return Err(LabeledError::new(format!(
                    "IO-Failed to open pack2 file: {:?}",
                    e
                )))
            }
        };
        let mut pack2: Pack2 = match Pack2::load_from_file(&mut file) {
            Ok(v) => v,
            Err(e) => {
                return Err(LabeledError::new(format!(
                    "Failed to parse pack2 file: {:?}",
                    e
                )));
            }
        };
        if let Some(fnlf) = filename_list_file {
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
            pack2.apply_filename_list(&buf.lines().map(String::from).collect::<Vec<String>>());
        }
        let nu_assets: Vec<Value> = pack2
            .assets
            .into_iter()
            .map(|asset| {
                let mut nu_asset: Record = Record::new();
                nu_asset.insert(
                    String::from("decoded_name"),
                    match asset.name {
                        Some(n) => Value::string(n, call.head),
                        None => Value::nothing(call.head),
                    },
                );
                // nushell can't handle i64 -> convert the hash to string
                nu_asset.insert(
                    String::from("name_hash"),
                    Value::string(asset.name_hash.to_string(), call.head),
                );
                nu_asset.insert(
                    String::from("is_zipped"),
                    Value::bool(asset.is_zipped, call.head),
                );
                nu_asset.insert(
                    String::from("data_hash"),
                    Value::int(asset.data_hash as i64, call.head),
                );
                nu_asset.insert(
                    String::from("data_length"),
                    Value::int(asset.data_length as i64, call.head),
                );
                nu_asset.insert(
                    String::from("unzipped_length"),
                    Value::int(asset.unzipped_length as i64, call.head),
                );
                Value::record(nu_asset, call.head)
            })
            .collect();

        Ok(Value::list(nu_assets, call.head))
    }
}
