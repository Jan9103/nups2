use nu_plugin::EngineInterface;
use nu_plugin::EvaluatedCall;
use nu_plugin::SimplePluginCommand;
use nu_protocol::Category;
use nu_protocol::LabeledError;
use nu_protocol::Signature;
use nu_protocol::Type;
use nu_protocol::Value;

use crate::Nups2Plugin;

pub struct Ps2Crc64Command;

impl SimplePluginCommand for Ps2Crc64Command {
    type Plugin = Nups2Plugin;

    fn name(&self) -> &str {
        "hash ps2crc64"
    }
    fn description(&self) -> &str {
        "Apply the ps2 variant of the crc64 hashing algorythm\nNOTE: since its a u64 and nu only supports i64 its returned as a string"
    }
    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .input_output_types(vec![
                (Type::String, Type::String),
                (Type::Binary, Type::String),
            ])
            .category(Category::Hash)
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let input: Value = input.clone();
        let result: u64 = match input {
            Value::Binary { val, .. } => nups2::crc64::hash(val.as_slice()),
            Value::String { val, .. } => nups2::crc64::hash(val.as_bytes()),
            _ => {
                return Err(LabeledError::new(
                    "invalid input data (only bytes and string supported)",
                ));
            }
        };
        Ok(Value::string(result.to_string(), call.head))
    }
}
