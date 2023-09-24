use serde_json::{from_str, Value};
use std::collections::HashMap;

pub fn load_config(object_list: &str) -> Result<HashMap<String, String>, String> {
  let config_json = include_str!("../../config.json");

  // Parse the JSON string into a serde_json::Value
  let config: Value = match from_str(config_json) {
    Ok(val) => val,
    Err(err) => {
      eprintln!("Failed to parse config.json: {}", err);
      return Err(err.to_string()); // Return an Err variant with the error message
    }
  };

  // Extract the specified object list
  let object = config.get(object_list);

  // Create a HashMap to store the result
  let mut result = HashMap::new();

  // Convert the object into a HashMap
  if let Some(obj) = object.and_then(|o| o.as_array()) {
    for entry in obj.iter() {
      if let (Some(key), Some(value)) = (
        entry.get("key").and_then(|k| k.as_str()),
        entry.get("value").and_then(|v| v.as_str()),
      ) {
        result.insert(key.to_string(), value.to_string());
      }
    }
  }

  Ok(result) // Return the result as Ok variant
}
