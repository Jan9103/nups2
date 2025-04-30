use std::fs::File;
use std::io::Read;

use nu_plugin::EngineInterface;
use nu_plugin::EvaluatedCall;
use nu_plugin::SimplePluginCommand;
use nu_protocol::Category;
use nu_protocol::LabeledError;
use nu_protocol::Signature;
use nu_protocol::SyntaxShape;
use nu_protocol::Type;
use nu_protocol::Value;
use nups2::pack2::Asset;
use nups2::pack2::Pack2;

use crate::util;
use crate::Nups2Plugin;

pub struct Pack2ExtractCommand;

impl SimplePluginCommand for Pack2ExtractCommand {
    type Plugin = Nups2Plugin;

    fn name(&self) -> &str {
        "pack2 extract"
    }
    fn description(&self) -> &str {
        "Extracts a file from a pack2 file"
    }
    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .input_output_type(Type::Nothing, Type::Nothing)
            .required("pack2_file", SyntaxShape::Filepath, "The pack2 file")
            .named(
                "filename_list_file",
                SyntaxShape::Filepath,
                "A newline seperated list of filenames, which might be contained in the pack2",
                None,
            )
            .named(
                "extract_filename",
                SyntaxShape::String,
                "The name of the file to extract",
                Some('n'),
            )
            .named(
                "extract_filehash",
                SyntaxShape::String,
                "The filename_hash of the file to extract",
                None,
            )
            .named(
                "output_file",
                SyntaxShape::Filepath,
                "The path to where the extracted file shall be stored (defaults to ./{filename} or ./crc64{filename_hash})",
                Some('o')
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
            util::get_named_argument_str(call, "filename_list_file");
        let output_file: Option<String> = util::get_named_argument_str(call, "output_file");
        let extract_filehash: Option<String> =
            util::get_named_argument_str(call, "extract_filehash");
        let extract_filename: Option<String> =
            util::get_named_argument_str(call, "extract_filename");
        if !(extract_filename.is_some() || extract_filehash.is_some()) {
            return Err(LabeledError::new(
                "Either --extract-filehash or --extract-filename has to be defined.",
            ));
        }
        let pack2_filename: String = match call.positional[0].clone() {
            Value::String { val, .. } => val,
            _ => {
                return Err(LabeledError::new(
                    "First argument (pack2-filepath) is not a string",
                ));
            }
        };
        let mut pack2_file: File = match File::open(pack2_filename) {
            Ok(v) => v,
            Err(e) => {
                return Err(LabeledError::new(format!(
                    "IO-Failed to open pack2 file: {:?}",
                    e
                )))
            }
        };
        let mut pack2: Pack2 = match Pack2::load_from_file(&mut pack2_file) {
            Ok(v) => v,
            Err(e) => {
                return Err(LabeledError::new(format!(
                    "Failed to parse pack2 file: {:?}",
                    e
                )));
            }
        };
        if extract_filename.is_some() || output_file.is_none() {
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
        }

        let asset: &Asset = if let Some(f) = extract_filename {
            match pack2
                .assets
                .get(match pack2.find_asset_index_by_name(f.as_str()) {
                    Some(v) => v,
                    None => {
                        return Err(LabeledError::new(format!(
                            "Unable to find asset with the name {}",
                            &f
                        )));
                    }
                }) {
                Some(v) => v,
                None => {
                    return Err(LabeledError::new(format!(
                        "Unable to find asset with the name {}",
                        &f
                    )));
                }
            }
        } else if let Some(f) = extract_filehash {
            match pack2.assets.get(
                match pack2.find_asset_index_by_name_hash(match f.parse::<u64>() {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(LabeledError::new(
                            "the provided filename_hash is not a valid u64",
                        ));
                    }
                }) {
                    Some(v) => v,
                    None => {
                        return Err(LabeledError::new(format!(
                            "Unable to find asset with the hash {}",
                            &f
                        )));
                    }
                },
            ) {
                Some(v) => v,
                None => {
                    return Err(LabeledError::new(format!(
                        "Unable to find asset with the hash {}",
                        &f
                    )));
                }
            }
        } else {
            return Err(LabeledError::new(
                "Neither filename nor filehash is defined",
            ));
        };

        let output_file_path: String = match output_file {
            Some(v) => v,
            None => match asset.name.clone() {
                Some(v) => v,
                None => format!("crc64_{}", asset.name_hash),
            },
        };
        let mut output_file: File = match File::open(output_file_path) {
            Ok(v) => v,
            Err(err) => {
                return Err(LabeledError::new(format!(
                    "Failed to open output file: {:?}",
                    err
                )));
            }
        };
        match asset.extract_to_file(&mut pack2_file, &mut output_file) {
            Ok(_) => (),
            Err(err) => {
                return Err(LabeledError::new(format!(
                    "Failed to extract asset: {:?}",
                    err
                )))
            }
        };

        Ok(Value::nothing(call.head))
    }
}
