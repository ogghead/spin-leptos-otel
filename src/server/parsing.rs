use opentelemetry_proto::tonic::common::v1::{any_value::Value, AnyValue, KeyValue};

pub(crate) fn parse_value_to_str(value: Value) -> String {
    match value {
        opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue(s) => s,
        opentelemetry_proto::tonic::common::v1::any_value::Value::BoolValue(_) => todo!(),
        opentelemetry_proto::tonic::common::v1::any_value::Value::IntValue(_) => todo!(),
        opentelemetry_proto::tonic::common::v1::any_value::Value::DoubleValue(_) => todo!(),
        opentelemetry_proto::tonic::common::v1::any_value::Value::ArrayValue(_) => {
            todo!()
        }
        opentelemetry_proto::tonic::common::v1::any_value::Value::KvlistValue(_) => {
            todo!()
        }
        opentelemetry_proto::tonic::common::v1::any_value::Value::BytesValue(_) => todo!(),
    }
}

pub(crate) fn parse_key_values_to_sorted_string(mut key_values: Vec<KeyValue>) -> String {
    key_values.sort_unstable_by_key(|kv| kv.key.clone());

    key_values
        .into_iter()
        .filter_map(|kv| match kv.value {
            Some(AnyValue { value: Some(val) }) => Some((kv.key, val)),
            _ => None,
        })
        .map(|(key, val)| format!("{}={}", key, parse_value_to_str(val)))
        .collect::<Vec<_>>()
        .join(",")
}
