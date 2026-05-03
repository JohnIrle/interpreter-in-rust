// SPDX-FileCopyrightText: 2026 John Irle
//
// SPDX-License-Identifier: MIT

type ObjectType = String;

pub enum Object {
    Integer(i64),
    Boolean(bool),
    Null,
}

impl Object {
    pub fn inspect(&self) -> String {
        match self {
            Self::Integer(value) => format!("{value}"),
            Self::Boolean(value) => format!("{value}"),
            Self::Null => "null".to_string(),
        }
    }

    pub fn object_type(&self) -> ObjectType {
        match self {
            Self::Integer(_) => "INTEGER".to_string(),
            Self::Boolean(_) => "BOOLEAN".to_string(),
            Self::Null => "NULL".to_string(),
        }
    }
}
