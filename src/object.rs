use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

pub enum ObjectType {
    Integer,
    Bool,
    Null,
}

pub struct VTable {
    inner: HashMap<&'static str, Box<dyn Fn(Option<Box<dyn Object>>) -> Option<Box<dyn Object>>>>,
    pub inverte: Box<dyn Fn() -> Box<dyn Object>>,
    pub negate: Box<dyn Fn() -> Box<dyn Object>>,
    pub integer: Box<dyn Fn() -> i32>,
    pub bool: Box<dyn Fn() -> bool>,
    pub as_bytes: Box<dyn Fn() -> Box<[u8]>>,
}

impl VTable {
    pub fn get(
        &self,
        s: &str,
    ) -> Option<&Box<dyn Fn(Option<Box<dyn Object>>) -> Option<Box<dyn Object>>>> {
        self.inner.get(s)
    }
}

impl Debug for VTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VTable").finish()
    }
}

pub trait Object: Debug + Display {
    fn r#type(&self) -> ObjectType;
    fn v_table(&self) -> &VTable;
}

#[derive(Debug)]
pub struct Integer {
    pub val: i32,
    v_table: VTable,
}

impl Object for Integer {
    fn r#type(&self) -> ObjectType {
        ObjectType::Integer
    }

    fn v_table(&self) -> &VTable {
        &self.v_table
    }
}

fn erase(obj: Box<dyn Object>) -> Box<dyn Object> {
    obj
}

impl Integer {
    pub fn erased(val: i32) -> Box<dyn Object> {
        let mut v_table = VTable {
            inner: HashMap::new(),
            inverte: Box::new(move || Integer::erased(val * -1)),
            negate: Box::new(move || Integer::erased(val * -1)),
            integer: Box::new(move || val),
            bool: Box::new(move || val > 0),
            as_bytes: Box::new(move || Box::new(val.to_le_bytes())),
        };

        v_table.inner.insert(
            "sub_lhs",
            Box::new(move |obj| {
                let Some(obj) = obj else {
                    return None;
                };

                if !matches!(obj.r#type(), ObjectType::Integer) {
                    return None;
                }

                let bytes = (obj.v_table().as_bytes)();

                let rhs = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

                Some(Integer::erased(val - rhs))
            }),
        );

        v_table.inner.insert(
            "add_lhs",
            Box::new(move |obj| {
                let Some(obj) = obj else {
                    return None;
                };

                if !matches!(obj.r#type(), ObjectType::Integer) {
                    return None;
                }

                let bytes = (obj.v_table().as_bytes)();

                let rhs = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

                Some(Integer::erased(val + rhs))
            }),
        );

        v_table.inner.insert(
            "mul_lhs",
            Box::new(move |obj| {
                let Some(obj) = obj else {
                    return None;
                };

                if !matches!(obj.r#type(), ObjectType::Integer) {
                    return None;
                }

                let bytes = (obj.v_table().as_bytes)();

                let rhs = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

                Some(Integer::erased(val * rhs))
            }),
        );

        v_table.inner.insert(
            "div_lhs",
            Box::new(move |obj| {
                let Some(obj) = obj else {
                    return None;
                };

                if !matches!(obj.r#type(), ObjectType::Integer) {
                    return None;
                }

                let bytes = (obj.v_table().as_bytes)();

                let rhs = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

                Some(Integer::erased(val / rhs))
            }),
        );

        erase(Box::new(Integer { val, v_table }))
    }
}

impl Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.val))
    }
}

#[derive(Debug)]
pub struct Bool {
    val: bool,
    v_table: VTable,
}

impl Object for Bool {
    fn r#type(&self) -> ObjectType {
        ObjectType::Bool
    }

    fn v_table(&self) -> &VTable {
        &self.v_table
    }
}

impl Bool {
    pub fn erased(val: bool) -> Box<dyn Object> {
        let v_table = VTable {
            inner: HashMap::new(),
            inverte: Box::new(move || Bool::erased(!val)),
            negate: Box::new(move || Bool::erased(!val)),
            integer: Box::new(move || if val { 1 } else { 0 }),
            bool: Box::new(move || val),
            as_bytes: Box::new(move || Box::new([if val { 1 } else { 0 }])),
        };
        erase(Box::new(Bool { val, v_table }))
    }
}

impl Display for Bool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.val))
    }
}

#[derive(Debug)]
pub struct Null {
    v_table: VTable,
}

impl Object for Null {
    fn r#type(&self) -> ObjectType {
        ObjectType::Null
    }

    fn v_table(&self) -> &VTable {
        &self.v_table
    }
}

impl Null {
    pub fn erased() -> Box<dyn Object> {
        let v_table = VTable {
            inner: HashMap::new(),
            inverte: Box::new(|| Null::erased()),
            negate: Box::new(|| Null::erased()),
            integer: Box::new(|| 0),
            bool: Box::new(|| false),
            as_bytes: Box::new(move || Box::new([])),
        };

        erase(Box::new(Null { v_table }))
    }
}

impl Display for Null {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("null")
    }
}
