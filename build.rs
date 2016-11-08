// You should probably turn back now. This is pretty bad.
//
// Note to self: redo this before showing it to _anyone_.

extern crate yaml_rust;

use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use yaml_rust::{YamlLoader, Yaml};

fn main() {
    let enum_data = build_enums();
    let struct_data = build_structs();

    let data = format!("{}\n\n{}", enum_data, struct_data);

    let out = &env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(out);
    let model_path = Path::new(out_dir)
        .join("models/built.rs");

    fs::create_dir_all(Path::new(&out_dir).join("models/")).ok();

    let mut f = match File::create(model_path) {
        Ok(f) => f,
        Err(why) => panic!("Path to built models: {:?}", why),
    };

    f.write_all(data.as_bytes()).ok();
}

fn build_yamls(kind: &str) -> Vec<Yaml> {
    let paths = fs::read_dir(&format!("./definitions/{}", kind))
        .expect(&format!("Error reading directory '{}'", kind));

    let mut docs = vec![];

    for path in paths {
        let path = path.expect("Error unwrapping path").path();
        let mut file = File::open(path).expect("Error opening file");

        let mut s = String::default();
        let _ = file.read_to_string(&mut s);

        docs.push(match YamlLoader::load_from_str(&s) {
            Ok(mut yaml) => yaml.remove(0),
            Err(why) => panic!("Error parsing yaml: {:?}", why),
        });
    }

    docs
}

fn build_enums() -> String {
    let docs = build_yamls("enums");

    let mut text = String::default();

    for doc in docs {
        let name = doc["name"].as_str().unwrap();
        let description = doc["description"]
            .as_str()
            .unwrap_or("")
            .trim()
            .replace('\n', "\n/// ");

        let mut enum_ = format!("\n\n/// {}
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum {} {{", description, name);

        let mut names = vec![];
        let mut numbers = vec![];

        let variants = doc["variants"].as_vec().expect("variants empty");

        for item in variants {
            let variant = item["variant"].as_str().expect("expected variant");
            let description = item["description"]
                .as_str()
                .unwrap_or("")
                .trim()
                .replace('\n', "\n    ///");

            if let Some(name) = item["name"].as_str() {
                names.push(name);
            }

            if let Some(number) = item["number"].as_i64() {
                numbers.push(number);
            }

            enum_.push_str("\n    /// ");
            enum_.push_str(&description);
            enum_.push_str("\n    ");
            enum_.push_str(variant);
            enum_.push(',');
        }

        enum_.push_str("\n}");

        if !names.is_empty() || !numbers.is_empty() {
            enum_.push_str("\n\nimpl ");
            enum_.push_str(name);
            enum_.push_str(" {");

            if !names.is_empty() {
                enum_.push_str("
    pub fn name(&self) -> &str {
        match *self {");

                for (i, name_item) in names.iter().enumerate() {
                    enum_.push_str("
            ");
                    enum_.push_str(name);
                    enum_.push_str("::");
                    enum_.push_str(unsafe {
                        variants.get_unchecked(i)["variant"].as_str().unwrap()
                    });
                    enum_.push_str(" => \"");
                    enum_.push_str(name_item);
                    enum_.push_str("\",\n");
                }

                enum_.push_str("
        }
    }

    pub fn from_str(name: &str) -> Option<Self> {
        match name {");

                for (i, name_item) in names.iter().enumerate() {
                    enum_.push_str("
            \"");
                    enum_.push_str(name_item);
                    enum_.push_str("\" => Some(");
                    enum_.push_str(name);
                    enum_.push_str("::");
                    enum_.push_str(unsafe {
                        variants.get_unchecked(i)["variant"].as_str().unwrap()
                    });
                    enum_.push_str("),");
                }

                enum_.push_str(&format!("
            _ => None,
        }}
    }}

    pub fn decode_str(value: Value) -> Result<Self> {{
        let name = try!(into_string(value));

        Self::from_str(&name)
            .ok_or_else(|| Error::Decode(\"Expected valid {}\",
                                         Value::String(name)))
    }}", name));
            }

            if !numbers.is_empty() {
                enum_.push_str("
    pub fn num(&self) -> u64 {
        match *self {");

                for (i, _name_item) in numbers.iter().enumerate() {
                    enum_.push_str("
            ");
                    enum_.push_str(name);
                    enum_.push_str("::");
                    enum_.push_str(unsafe {
                        variants.get_unchecked(i)["variant"].as_str().unwrap()
                    });
                    enum_.push_str(" => ");
                    enum_.push_str(&i.to_string());
                    enum_.push(',');
                }

                enum_.push_str("
        }
    }

    pub fn from_num(num: u64) -> Option<Self> {
        match num {");

                for (i, _name_item) in numbers.iter().enumerate() {
                    enum_.push_str("
            ");
                    enum_.push_str(&i.to_string());
                    enum_.push_str(" => Some(");
                    enum_.push_str(name);
                    enum_.push_str("::");
                    enum_.push_str(unsafe {
                        variants.get_unchecked(i)["variant"].as_str().unwrap()
                    });
                    enum_.push_str("),");
                }

                enum_.push_str(&format!("
            _ => None,
        }}
    }}

    fn decode(value: Value) -> Result<Self> {{
        into_u64(value)
            .ok()
            .and_then(Self::from_num)
            .ok_or_else(|| Error::Other(\"Expected valid {}\"))
    }}", name));
            }

            enum_.push_str("\n}");
        }

        text.push_str(&enum_);
    }

    text
}

fn build_structs() -> String {
    let docs = build_yamls("structs");

    let mut text = String::default();

    for doc in docs {
        let name = doc["name"].as_str().unwrap();
        let description = doc["description"]
            .as_str()
            .unwrap_or("")
            .trim()
            .replace('\n', "\n/// ");

        let mut struct_ = format!("\n\n/// {}
#[derive(Clone, Debug)]
pub struct {} {{", description, name);

        for field in doc["fields"].as_vec().expect("fields empty") {
            let array = field["array"].as_bool().unwrap_or(false);
            let description = field["description"]
                .as_str()
                .unwrap_or("")
                .trim()
                .replace('\n', "\n    ///");
            let name = field["name"].as_str().expect("expected name");
            let kind = match field["type"].as_str() {
                Some(kind) => kind,
                None => panic!("Expected type: {:?}", field),
            };
            let optional = field["optional"].as_bool().unwrap_or(false);

            let gen_kind = match kind {
                "btreemap" => {
                    let t = match field["t"].as_str() {
                        Some(t) => t,
                        None => panic!("No t: {:?}", field),
                    };

                    format!("BTreeMap<{}>", t)
                },
                "hashmap" => {
                    let t = match field["t"].as_str() {
                        Some(t) => t,
                        None => panic!("No t: {:?}", field),
                    };

                    format!("HashMap<{}>", t)
                },
                "string" => "String".to_owned(),
                other => other.to_owned(),
            };

            let content = if optional && array {
                format!("Option<Vec<{}>>", gen_kind)
            } else if optional {
                format!("Option<{}>", gen_kind)
            } else if array {
                format!("Vec<{}>", gen_kind)
            } else {
                gen_kind.to_owned()
            };

            struct_.push_str(&format!("\n    /// {}\n    pub {}: {},",
                                      description,
                                      name,
                                      content));
        }

        struct_.push_str("\n}");

        let mut decode_ = format!("\n
impl {0} {{
    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<{0}> {{
        let mut map = try!(into_map(value));

        missing!(map, {0} {{", name);

        let decoding = doc["decode"].as_bool().unwrap_or(true);

        for field in doc["fields"].as_vec().expect("fields empty") {
            let array = field["array"].as_bool().unwrap_or(false);
            let custom = field["custom"].as_str();
            let default = field["default"].as_str();
            let field_name = field["name"].as_str().unwrap();
            let from = field["from"].as_str();
            let kind = field["type"].as_str().unwrap();
            let optional = field["optional"].as_bool().unwrap_or(false);
            let t = field["t"].as_str();

            // let name = field["from"].as_str().unwrap_or(name);

            let operation = match (kind, optional, array, default, custom, from, t) {
                ("[u8; 2]", true, false, None, Some(custom), None, None) => {
                    format!(r#"try!(opt(&mut map, "{}", {}))"#,
                            field_name,
                            custom)
                },
                ("bool", true, false, None, None, None, None) => {
                    format!(r#"try!(opt(&mut map, "{}", |v| Ok(req!(v.as_bool()))))"#,
                            field_name)
                },
                ("bool", false, false, Some(def), None, None, None) => {
                    format!(r#"try!(opt(&mut map, "{}", |v| Ok(req!(v.as_bool())))).unwrap_or({})"#,
                            field_name,
                            def)
                },
                ("bool", false, false, None, None, None, None) => {
                    format!(r#"req!(try!(remove(&mut map, "{}")).as_bool())"#,
                            field_name)
                },
                ("u16", false, false, None, Some(custom_decoder), None, None) => {
                    format!(r#"try!(remove(&mut map, "{}").and_then({}))"#,
                            field_name,
                            custom_decoder)
                },
                ("i64", false, false, None, None, None, None) => {
                    format!(r#"req!(try!(remove(&mut map, "{}")).as_i64())"#,
                            field_name)
                },
                ("hashmap", false, false, None, Some(custom), None, Some(_t)) => {
                    format!(r#"try!(remove(&mut map, "{}").and_then({}))"#,
                            field_name,
                            custom)
                },
                ("hashmap", false, false, None, None, None, Some(_t)) => {
                    format!(r#"try!(remove(&mut map, "{}").and_then({}::decode))"#,
                            field_name,
                            "n/a")
                },
                ("hashmap", false, false, Some(default), Some(custom), None, Some(_t)) => {
                    format!(r#"try!(opt(&mut map, "{}", {})).unwrap_or({})"#,
                            field_name,
                            custom,
                            default)
                },
                ("u64", false, false, Some(default), None, Some(from), None) => {
                    format!(r#"try!(opt(&mut map, "{}", |v| Ok(req!(v.as_u64())))).unwrap_or({})"#,
                            from,
                            default.parse::<u64>().unwrap())
                },
                ("u64", false, false, None, None, Some(from), None) => {
                    format!(r#"req!(try!(remove(&mut map, "{}")).as_u64())"#,
                            from)
                },
                ("u64", true, false, None, None, None, None) => {
                    format!(r#"remove(&mut map, "{}").ok().and_then(|v| v.as_u64())"#,
                            field_name)
                },
                ("u64", false, false, None, None, None, None) => {
                    format!(r#"req!(try!(remove(&mut map, "{}")).as_u64())"#,
                            field_name)
                },
                ("string", false, false, None, Some(custom), None, None) => {
                    format!(r#"try!(remove(&mut map, "{}").and_then({}))"#,
                            field_name, custom)
                }
                ("string", false, false, None, None, Some(from), None) => {
                    format!(r#"try!(remove(&mut map, "{}").and_then(into_string))"#,
                            from)
                },
                ("string", false, false, None, None, None, None) => {
                    format!(r#"try!(remove(&mut map, "{}").and_then(into_string))"#,
                            field_name)
                },
                ("string", false, true, None, None, None, None) => {
                    format!(r#"try!(remove(&mut map, "{}").and_then(|v| decode_array(v, into_string)))"#,
                            field_name)
                },
                ("string", true, false, None, None, None, None) => {
                    format!(r#"try!(opt(&mut map, "{}", into_string))"#,
                            field_name)
                },
                ("string", true, true, None, None, Some(from), None) => {
                    format!(r#"try!(opt(&mut map, "{}", |v| decode_array(v, into_string)))"#,
                            from)
                },
                ("string", true, true, None, None, None, None) => {
                    format!(r#"try!(opt(&mut map, "{}", |v| decode_array(v, into_string)))"#,
                            field_name)
                },
                (struct_name, false, false, Some(def), None, Some(from), None) => {
                    format!(r#"try!(opt(&mut map, "{}", {}::decode)).unwrap_or({})"#,
                            from,
                            struct_name,
                            def)
                },
                (_struct_name, false, false, None, Some(custom), None, None) => {
                    format!(r#"try!(remove(&mut map, "{}").and_then({}))"#,
                            field_name,
                            custom)
                },
                (struct_name, false, false, None, None, Some(from), None) => {
                    format!(r#"try!(remove(&mut map, "{}").and_then({}::decode))"#,
                            from,
                            struct_name)
                },
                (struct_name, false, false, None, None, None, None) => {
                    format!(r#"try!(remove(&mut map, "{}").and_then({}::decode))"#,
                            field_name,
                            struct_name)
                },
                (_struct_name, false, true, None, Some(custom), None, None) => {
                    format!(r#"try!(remove(&mut map, "{}").and_then(|v| decode_array(v, {})))"#,
                            field_name,
                            custom)
                },
                (struct_name, false, true, Some(def), None, None, None) => {
                    format!(r#"try!(opt(&mut map, "{}", |v| decode_array(v, {}::decode))).unwrap_or({})"#,
                            field_name,
                            struct_name,
                            def)
                },
                (struct_name, false, true, None, None, None, None) => {
                    format!(r#"try!(remove(&mut map, "{}").and_then(|v| decode_array(v, {}::decode)))"#,
                            field_name,
                            struct_name)
                },
                (_struct_name, true, false, None, Some(custom), None, None) => {
                    format!(r#"try!(opt(&mut map, "{}", {}))"#,
                            field_name,
                            custom)
                },
                (struct_name, true, false, None, None, None, None) => {
                    format!(r#"try!(opt(&mut map, "{}", {}::decode))"#,
                            field_name,
                            struct_name)
                },
                (struct_name, true, true, None, None, None, None) => {
                    format!(r#"try!(opt(&mut map, "{}", |v| decode_array(v, {}::decode)))"#,
                            field_name,
                            struct_name)
                },
                _ => panic!("Unknown operation: {} {} {} {:?} {:?} {:?} {:?}",
                            kind,
                            optional,
                            array,
                            default,
                            custom,
                            from,
                            t),
            };

            decode_.push_str(&format!("
            {}: {},", field_name, operation));
        }

        text.push_str(&struct_);

        if decoding {
            decode_.push_str("\n        })\n    }\n}");
            text.push_str(&decode_);
        }
    }

    text
}
