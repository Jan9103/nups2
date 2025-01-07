use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use nu_plugin::EngineInterface;
use nu_plugin::EvaluatedCall;
use nu_plugin::SimplePluginCommand;
use nu_protocol::Category;
use nu_protocol::LabeledError;
use nu_protocol::Signature;
use nu_protocol::SyntaxShape;
use nu_protocol::Type;
use nu_protocol::Value;
use nups2::pack2::Pack2;

use crate::Nups2Plugin;

pub struct Pack2ScrapeFilenamesCommand;

impl SimplePluginCommand for Pack2ScrapeFilenamesCommand {
    type Plugin = Nups2Plugin;

    fn name(&self) -> &str {
        "pack2 scrape_filenames"
    }
    fn description(&self) -> &str {
        "scrape potential filenames from a pack2 file for a filename_list_file"
    }
    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .input_output_type(Type::Nothing, Type::Nothing)
            .required("pack2_file", SyntaxShape::Filepath, "The pack2 file")
            .required("output_file", SyntaxShape::Filepath, "The filename-list file (output)")
            .named(
                "scrape_mode",
                SyntaxShape::Int,
                "The scrape-mode (valid values: 0, 1, 2, 3, 4) (recommended: 3) (most results: two runs (0 + 4) for the biggest database)",
                None,
            )
            .named(
                "filesize_limit",
                SyntaxShape::Filesize,
                "Don't scrape files bigger than this",
                None
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
        let filesize_limit: u32 = match crate::util::get_named_argument(call, "filesize_limit") {
            Some(Value::Filesize { val, .. }) => {
                let val: i64 = val.get();
                if val < 1 || val > u32::MAX as i64 {
                    return Err(LabeledError::new("Filesize not supported."));
                };
                val as u32
            }
            _ => 256u32 * 1024 * 1024,
        };
        let search_mode: usize = match crate::util::get_named_argument(call, "scrape_mode") {
            Some(Value::Int { val, .. }) => {
                if val < 0 || val > 4 {
                    return Err(LabeledError::new("Only modes 0, 1, 2, 3, and 4 exist."));
                };
                val as usize
            }
            _ => 3,
        };

        let pack2_filename: String = match call.positional[0].clone() {
            Value::String { val, .. } => val,
            _ => {
                return Err(LabeledError::new(
                    "First argument (pack2-filepath) is not a string",
                ));
            }
        };
        let output_filepath: PathBuf = match call.positional[1].clone() {
            Value::String { val, .. } => PathBuf::from(val),
            _ => {
                return Err(LabeledError::new(
                    "First argument (pack2-filepath) is not a string",
                ));
            }
        };
        // early check since this takes litteral hours to execute
        if output_filepath.exists() {
            return Err(LabeledError::new(
                "Output file already exists. remove it first.",
            ));
        }
        if !output_filepath.parent().unwrap().is_dir() {
            return Err(LabeledError::new(
                "Outputs parent directory does not exist (or is not a directory)",
            ));
        }
        let mut pack2_file: File = match File::open(pack2_filename) {
            Ok(v) => v,
            Err(e) => {
                return Err(LabeledError::new(format!(
                    "IO-Failed to open pack2 file: {:?}",
                    e
                )))
            }
        };
        let pack2: Pack2 = match Pack2::load_from_file(&mut pack2_file) {
            Ok(v) => v,
            Err(e) => {
                return Err(LabeledError::new(format!(
                    "Failed to parse pack2 file: {:?}",
                    e
                )));
            }
        };
        let res: Vec<String> = match nups2::filename_extractor::extract_names(
            &pack2,
            &mut pack2_file,
            filesize_limit,
            search_mode,
            None,
        ) {
            Ok(v) => v,
            Err(err) => {
                return Err(LabeledError::new(format!(
                    "Failed to extract filenames: {:?}",
                    err
                )));
            }
        };

        let mut output_file: File = match File::create_new(output_filepath) {
            Ok(v) => v,
            Err(err) => {
                return Err(LabeledError::new(format!(
                    "Failed to create output file: {:?}",
                    err
                )));
            }
        };

        match output_file.write(res.join("\n").as_bytes()) {
            Ok(_) => (),
            Err(err) => {
                return Err(LabeledError::new(format!(
                    "Failed to write to output file: {:?}",
                    err
                )));
            }
        };

        Ok(Value::nothing(call.head.clone()))
    }
}
