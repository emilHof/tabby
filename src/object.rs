use std::{
    cell::UnsafeCell,
    collections::HashMap,
    fmt::{Debug, Display},
    sync::Arc,
};

use crate::ast::{Expression, Ident};

pub enum ObjectType {
    Function,
    Integer,
    Bool,
    Null,
}

pub struct VTable {
    inner: HashMap<&'static str, Arc<dyn Fn(Option<Reference>) -> Option<Reference>>>,
}

impl VTable {
    pub fn get(&self, s: &str) -> Option<&Arc<dyn Fn(Option<Reference>) -> Option<Reference>>> {
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

#[derive(Debug, Clone)]
pub struct Reference {
    inner: Arc<UnsafeCell<dyn Object>>,
}

impl Reference {
    fn as_ref(&self) -> &dyn Object {
        unsafe { &(*self.inner.get()) }
    }

    pub unsafe fn get_mut<T>(&self) -> &mut T {
        &mut (*(self.inner.get() as *mut T))
    }
}

impl std::ops::Deref for Reference {
    type Target = dyn Object;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.inner.get()) }
    }
}

impl Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", unsafe { &(*(*self.inner).get()) }))
    }
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

fn erase(obj: Arc<UnsafeCell<dyn Object>>) -> Arc<UnsafeCell<dyn Object>> {
    obj
}

impl Integer {
    pub fn erased(val: i32) -> Reference {
        let mut v_table = VTable {
            inner: HashMap::new(),
        };

        let is_int = |obj: Option<Reference>| -> Option<i32> {
            let Some(obj) = obj else {
                return None;
            };

            if !matches!(obj.r#type(), ObjectType::Integer) {
                return None;
            }

            let rhs = unsafe { obj.get_mut::<Integer>().val };

            Some(rhs)
        };

        v_table.inner.insert(
            "sub_lhs",
            Arc::new(move |obj| {
                let rhs = is_int(obj)?;
                Some(Integer::erased(val - rhs))
            }),
        );

        v_table.inner.insert(
            "add_lhs",
            Arc::new(move |obj| {
                let rhs = is_int(obj)?;
                Some(Integer::erased(val + rhs))
            }),
        );

        v_table.inner.insert(
            "mul_lhs",
            Arc::new(move |obj| {
                let rhs = is_int(obj)?;
                Some(Integer::erased(val * rhs))
            }),
        );

        v_table.inner.insert(
            "div_lhs",
            Arc::new(move |obj| {
                let rhs = is_int(obj)?;
                Some(Integer::erased(val / rhs))
            }),
        );

        v_table.inner.insert(
            "eq_lhs",
            Arc::new(move |obj| {
                let rhs = is_int(obj)?;
                Some(Bool::erased(val == rhs))
            }),
        );

        v_table.inner.insert(
            "neq_lhs",
            Arc::new(move |obj| {
                let rhs = is_int(obj)?;
                Some(Bool::erased(val != rhs))
            }),
        );

        v_table.inner.insert(
            "le_lhs",
            Arc::new(move |obj| {
                let rhs = is_int(obj)?;
                Some(Bool::erased(val < rhs))
            }),
        );

        v_table.inner.insert(
            "leq_lhs",
            Arc::new(move |obj| {
                let rhs = is_int(obj)?;
                Some(Bool::erased(val <= rhs))
            }),
        );

        v_table.inner.insert(
            "ge_lhs",
            Arc::new(move |obj| {
                let rhs = is_int(obj)?;
                Some(Bool::erased(val > rhs))
            }),
        );

        v_table.inner.insert(
            "geq_lhs",
            Arc::new(move |obj| {
                let rhs = is_int(obj)?;
                Some(Bool::erased(val >= rhs))
            }),
        );

        v_table.inner.insert(
            "truthy",
            Arc::new(move |_| if val > 0 { Some(Null::erased()) } else { None }),
        );

        Reference {
            inner: erase(Arc::new(UnsafeCell::new(Integer { val, v_table }))),
        }
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
    pub fn erased(val: bool) -> Reference {
        let mut v_table = VTable {
            inner: HashMap::new(),
        };

        let is_bool = |obj: Option<Reference>| -> Option<bool> {
            let Some(obj) = obj else {
                return None;
            };

            if !matches!(obj.r#type(), ObjectType::Bool) {
                return None;
            }

            let rhs = unsafe { obj.get_mut::<Bool>().val };

            Some(rhs)
        };

        v_table.inner.insert(
            "eq_lhs",
            Arc::new(move |obj| {
                let rhs = is_bool(obj)?;
                Some(Bool::erased(val == rhs))
            }),
        );

        v_table.inner.insert(
            "neq_lhs",
            Arc::new(move |obj| {
                let rhs = is_bool(obj)?;
                Some(Bool::erased(val == rhs))
            }),
        );

        v_table
            .inner
            .insert("neg", Arc::new(move |_| Some(Bool::erased(!val))));

        v_table
            .inner
            .insert("inv", Arc::new(move |_| Some(Bool::erased(!val))));

        v_table.inner.insert(
            "truthy",
            Arc::new(move |_| if val { Some(Null::erased()) } else { None }),
        );

        Reference {
            inner: erase(Arc::new(UnsafeCell::new(Bool { val, v_table }))),
        }
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
    pub fn erased() -> Reference {
        let mut v_table = VTable {
            inner: HashMap::new(),
        };

        v_table.inner.insert("truthy", Arc::new(move |_| None));

        Reference {
            inner: erase(Arc::new(UnsafeCell::new(Null { v_table }))),
        }
    }
}

impl Display for Null {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("null")
    }
}

#[derive(Debug)]
pub struct Function {
    v_table: VTable,
    pub parameters: Vec<Ident>,
    pub body: Box<Expression>,
}

impl Object for Function {
    fn r#type(&self) -> ObjectType {
        ObjectType::Function
    }

    fn v_table(&self) -> &VTable {
        &self.v_table
    }
}

impl Function {
    pub fn erased(parameters: Vec<Ident>, body: Box<Expression>) -> Reference {
        let mut v_table = VTable {
            inner: HashMap::new(),
        };

        v_table.inner.insert("truthy", Arc::new(move |_| None));

        Reference {
            inner: erase(Arc::new(UnsafeCell::new(Function {
                v_table,
                parameters,
                body,
            }))),
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Function")
    }
}
