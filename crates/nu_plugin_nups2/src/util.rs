use nu_plugin::EvaluatedCall;
use nu_protocol::Value;

pub fn get_named_argument(call: &EvaluatedCall, arg_name: &str) -> Option<Value> {
    match call.named.iter().find(|i| i.0.item.as_str() == arg_name) {
        Some((_, Some(v))) => Some(v.clone()),
        _ => None,
    }
}

pub fn get_named_argument_str(call: &EvaluatedCall, arg_name: &str) -> Option<String> {
    match call.named.iter().find(|i| i.0.item.as_str() == arg_name) {
        Some((_, Some(Value::String { val, .. }))) => Some(val.clone()),
        _ => None,
    }
}
