use std::collections::HashMap;

use std::fmt::{Display, Formatter};
use serde_json::Value;
use serde::{Serialize,Deserialize};
use crate::error::RustractError;
use crate::field::FieldDesign;
use crate::types::capitalize;

/// Describes a database table's design.
/// 
/// TODO: Change Vector to HashMap with the titles as keys.
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct TableDesign {
    pub table_design_title: String,
    pub fields: HashMap<String, FieldDesign>,
    order: Vec<String>
}

impl Display for TableDesign {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ({:?})", self.table_design_title, self.fields)
    }
}

impl TableDesign {
    pub fn new(title: &str) -> Self {
        TableDesign {
            table_design_title: String::from(title),
            fields: HashMap::new(),
            order: vec![]
        }
    }

    /// Tests the provided JSON values against this table's design.
    /// 
    /// Ignores the required check for any fields marked as generated if input is true.
    pub fn test(&self, fields: &[Value], input: bool) -> Result<(), RustractError> {
        // Iterates over the fields in this design and attempts to match each to the JSON
        for key in self.fields.keys() {
            let mut matched = false;
            let field_design = self.fields.get(key).unwrap();

            // Finds a match for this field design
            for field in fields {
                if let Some(val) = field.get(&field_design.field_design_title) {
                    matched = true;
                    field_design.extract(val)?;
                    break;
                }
            }

            // If a required field is missing in the request JSON, decline it
            if !matched && field_design.required && (!field_design.generated || !input) {
                return Err(RustractError {
                    message: format!(
                        "The {} field is required in {}, but was not included in the request.",
                        field_design.field_design_title,
                        self.table_design_title
                    ),
                });
            }
        }
        Ok(())
    }

    /// Saves the configuration info to a JSON file for quick loading.
    pub fn save(&self, filepath: &str) -> Result<(), RustractError> {
        std::fs::write(
            filepath,
            serde_json::to_string_pretty(self)?
        )?;
        Ok(())
    }

    /// Creates an instance of this struct from the JSON file at the specified path.
    pub fn from(filepath: &str) -> Result<Self, RustractError> {
        Ok(serde_json::from_str(&std::fs::read_to_string(filepath)?)?)
    }

    /// Adds the provided field to this table.
    pub fn add(&mut self, field: FieldDesign) {
        let title = field.field_design_title.clone();
        self.fields.insert(title.clone(), field);
        self.order.push(title);

    }

    /// Gets a reference to the specified field by its title.
    /// If there's a duplicate, the first is returned.
    pub fn get(&self, title: &str) -> Option<&FieldDesign> {
        self.fields.get(title)
    }

    /// Gets the specified field by its title.
    /// If there's a duplicate, the first is returned.
    pub fn get_mut(&mut self, title: &str) -> Option<&mut FieldDesign> {
        self.fields.get_mut(title)
    }

    /// Exports this table design to a TypeScript library of types.
    /// 
    /// These types can be used in the front-end to standardize routes.
    /// Note that depending on usage, scripts using these may reveal internal Database structure.
    pub fn export(&self, folder: &str) -> Result<(), RustractError> {
        // Creates a filepath for this table's type file
        let new_path = if folder.ends_with('/') {
            format!("{}{}.ts", folder, &self.table_design_title)
        } else {
            format!("{}/{}.ts", folder, &self.table_design_title)
        };
        let mut output = String::new();
        let mut second_output = String::new();
        let title: &str = &capitalize(&self.table_design_title)?;

        // Creates the interface
        output += &format!("/** Generated database type for the {} table. */\n", title);
        output += &format!("export interface {} {{\n", title);

        // Creates an input version of the interface
        second_output += &format!("/** Generated database type for the {} table. (Input version) */\n", title);
        second_output += &format!("export interface {}Input {{\n", title);

        // Exports each field to this file
        for key in &self.order {
            let field = self.get(key).unwrap();
            output += &field.export(false);
            second_output += &field.export(true);
        }

        output += "}\n\n";
        second_output += "}\n";
        output += &second_output;

        std::fs::write(new_path, output)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{filesystem::{_delete_file, read_file}, types::DataType};

    #[test]
    fn table_test() {
        let filepath = String::from("./tests/test_type.json");
        let table_design = default_table();
        table_design.save(&filepath).unwrap();
        let string_form = read_file(&filepath).unwrap();
        _delete_file(&filepath).unwrap();
        let new_table: TableDesign = serde_json::from_str(&string_form).unwrap();
        assert_eq!(table_design, new_table);
    }

    #[test]
    fn table_data_test() {
        let table_design = default_table();
        let json = serde_json::json!({
            "code": 200,
            "success": true,
            "payload": {
                "fields": [
                    {
                        "title": "User",
                        "email": "test@test.com"
                    },
                ]
            }
        });
        let fields = match json["payload"]["fields"].as_array() {
            Some(val) => val,
            None => panic!("Test failed, could not read JSON data as an array."),
        };
        table_design.test(fields, true).unwrap();
    }

    /// Creates a default TableDesign struct for use in testing.
    fn default_table() -> TableDesign {
        let mut table = TableDesign::new("User");
        table.add(FieldDesign {
                field_design_title: String::from("id"),
                datatype: DataType::Unsigned64,
                bytes: Some(64),
                characters: None,
                decimals: None,
                regex: None,
                primary: true,
                unique: true,
                required: true,
                foreign: None,
                increment: false,
                generated: true
        });
        table.add(FieldDesign {
                field_design_title: String::from("email"),
                datatype: DataType::String,
                bytes: Some(800),
                characters: Some(110),
                decimals: None,
                regex: Some(String::from("(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|\"(?:[\\x01-\\x08\\x0b\\x0c\\x0e-\\x1f\\x21\\x23-\\x5b\\x5d-\\x7f]|\\\\[\\x01-\\x09\\x0b\\x0c\\x0e-\\x7f])*\")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\\x01-\\x08\\x0b\\x0c\\x0e-\\x1f\\x21-\\x5a\\x53-\\x7f]|\\\\[\\x01-\\x09\\x0b\\x0c\\x0e-\\x7f])+)\\])")),
                primary: false,
                unique: false,
                required: true,
                foreign: None,
                increment: false,
                generated: false
        });
        table.add(FieldDesign {
                field_design_title: String::from("name"),
                datatype: DataType::String,
                bytes: Some(800),
                characters: Some(100),
                decimals: None,
                regex: None,
                primary: false,
                unique: false,
                required: false,
                foreign: None,
                increment: false,
                generated: false
        });

        table
    }

}
