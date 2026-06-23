// use crate::parser::struct_parser::Struct;
// use std::collections::HashMap;
// use std::sync::atomic::AtomicU32;
// use std::sync::atomic::Ordering::SeqCst;

// static STRUCT_ID: AtomicU32 = AtomicU32::new(0);

// fn update_struct_id() {
//     STRUCT_ID.fetch_add(1, SeqCst);
// }

// // for sematic analysis, got a little ahead of myself
// struct StructCollection {
//     structs: HashMap<String, Option<Struct>>,
// }

// impl StructCollection {
//     pub fn add_struct(&mut self, mut name: Option<String>, new_struct: Struct) -> String {
//         if name.is_none() {
//             let generate_new_name =
//                 || String::from(format!("_AnonymousStruct_{}", STRUCT_ID.load(SeqCst)));

//             let mut new_name = generate_new_name();

//             while self.has_struct_been_declared(&new_name) {
//                 update_struct_id();
//                 new_name = generate_new_name();
//             }

//             name = Some(new_name);
//         }

//         self.structs.insert(name.clone().unwrap(), Some(new_struct));

//         name.unwrap()
//     }

//     pub fn has_struct_been_declared(&self, name: &str) -> bool {
//         self.structs.contains_key(name)
//     }

//     pub fn has_struct_been_defined(&self, name: &str) -> bool {
//         if !self.has_struct_been_declared(name) {
//             return false;
//         }

//         self.structs.get(name).is_some()
//     }

//     pub fn get_struct(&self, name: &str) -> Result<&Option<Struct>, String> {
//         if let Some(struct_data) = self.structs.get(name) {
//             return Ok(struct_data);
//         }

//         Err(String::from(&format!("Struct {name} does not exist")))
//     }

//     // pub fn display_struct(&self, name: &str) -> Result<String, String> {
//     //     let displayed_struct = self.get_struct(name)?;

//     //     let mut output = String::from(&format!("(Struct {name}"));

//     //     for member in &displayed_struct.unwrap().members {
//     //         output.push_str(&format!("{member}"));
//     //     }

//     //     output.push_str(")");

//     //     Ok(output)
//     // }
// }
