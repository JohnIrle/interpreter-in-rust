// SPDX-FileCopyrightText: 2026 John Irle
//
// SPDX-License-Identifier: MIT

type ObjectType = String;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    Return(Box<Self>),
    Null,
}

impl Object {
    pub fn inspect(&self) -> String {
        match self {
            Self::Integer(value) => format!("{value}"),
            Self::Boolean(value) => format!("{value}"),
            Self::Return(object) => object.inspect(),
            Self::Null => "null".to_string(),
        }
    }
}
