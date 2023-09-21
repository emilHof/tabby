use std::{
    cell::UnsafeCell,
    collections::HashMap,
    fmt::{Debug, Display},
    sync::Arc,
};

use crate::ast::{Expression, Ident};

use crate::eval::error::Result;

pub enum ObjectType {
    Bool,
    Builtin,
    Collection,
    Vector,
    Function,
    Integer,
    Str,
    Unit,
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
            "str",
            Arc::new(move |_| Some(Str::erased(format!("{val}")))),
        );

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
            Arc::new(move |_| if val > 0 { Some(Unit::erased()) } else { None }),
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
            "str",
            Arc::new(move |_| Some(Str::erased(format!("{val}")))),
        );

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
            Arc::new(move |_| if val { Some(Unit::erased()) } else { None }),
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
pub struct Unit {
    v_table: VTable,
}

impl Object for Unit {
    fn r#type(&self) -> ObjectType {
        ObjectType::Unit
    }

    fn v_table(&self) -> &VTable {
        &self.v_table
    }
}

impl Unit {
    pub fn erased() -> Reference {
        let mut v_table = VTable {
            inner: HashMap::new(),
        };

        v_table.inner.insert("truthy", Arc::new(move |_| None));

        Reference {
            inner: erase(Arc::new(UnsafeCell::new(Unit { v_table }))),
        }
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("()")
    }
}

#[derive(Debug)]
pub struct Function {
    v_table: VTable,
    pub parameters: Vec<Ident>,
    pub body: Expression,
    pub capture: HashMap<Ident, Reference>,
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
    pub fn erased(
        parameters: Vec<Ident>,
        body: Expression,
        capture: HashMap<Ident, Reference>,
    ) -> Reference {
        let mut v_table = VTable {
            inner: HashMap::new(),
        };

        v_table.inner.insert("truthy", Arc::new(move |_| None));

        Reference {
            inner: erase(Arc::new(UnsafeCell::new(Function {
                v_table,
                parameters,
                body,
                capture,
            }))),
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Function")
    }
}

#[derive(Debug)]
pub struct Collection {
    v_table: VTable,
    pub members: Arc<HashMap<Ident, Reference>>,
}

impl Object for Collection {
    fn r#type(&self) -> ObjectType {
        ObjectType::Collection
    }

    fn v_table(&self) -> &VTable {
        &self.v_table
    }
}

impl Collection {
    pub fn erased(members: HashMap<Ident, Reference>) -> Reference {
        let members = Arc::new(members);
        let mut v_table = VTable {
            inner: HashMap::new(),
        };

        let is_collection = |obj: Option<Reference>| {
            let Some(obj) = obj else {
                return None;
            };

            if !matches!(obj.r#type(), ObjectType::Collection) {
                return None;
            }

            let rhs = unsafe { obj.get_mut::<Collection>().members.clone() };

            Some(rhs)
        };

        v_table.inner.insert("truthy", Arc::new(move |_| None));
        {
            let members = members.clone();
            v_table.inner.insert(
                "uni_lhs",
                Arc::new(move |obj| {
                    let rhs = is_collection(obj)?;
                    let mut union = HashMap::new();

                    for (ident, member) in rhs.iter() {
                        union.insert(ident.clone(), member.clone());
                    }

                    for (ident, member) in members.iter() {
                        union.insert(ident.clone(), member.clone());
                    }

                    Some(Collection::erased(union))
                }),
            );
        }
        {
            let members = members.clone();
            v_table.inner.insert(
                "ins_lhs",
                Arc::new(move |obj| {
                    let rhs = is_collection(obj)?;
                    let mut intersection = HashMap::new();

                    for (ident, member) in members.iter() {
                        if rhs.contains_key(&ident) {
                            intersection.insert(ident.clone(), member.clone());
                        }
                    }

                    Some(Collection::erased(intersection))
                }),
            );
        }

        Reference {
            inner: erase(Arc::new(UnsafeCell::new(Collection { v_table, members }))),
        }
    }
}

impl Display for Collection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct("Collection");
        for (ident, member) in self.members.iter() {
            dbg.field(&ident.name, member);
        }
        dbg.finish()
    }
}

#[derive(Debug)]
pub struct Vector {
    v_table: VTable,
    pub elements: Arc<Vec<Reference>>,
}

impl Object for Vector {
    fn r#type(&self) -> ObjectType {
        ObjectType::Vector
    }

    fn v_table(&self) -> &VTable {
        &self.v_table
    }
}

impl Vector {
    pub fn erased(elements: Vec<Reference>) -> Reference {
        let elements = Arc::new(elements);
        let mut v_table = VTable {
            inner: HashMap::new(),
        };

        let is_vec = |obj: Option<Reference>| {
            let Some(obj) = obj else {
                return None;
            };

            if !matches!(obj.r#type(), ObjectType::Vector) {
                return None;
            }

            let rhs = unsafe { obj.get_mut::<Vector>().elements.clone() };

            Some(rhs)
        };

        v_table.inner.insert("truthy", Arc::new(move |_| None));
        {
            let elements = elements.clone();
            v_table.inner.insert(
                "add_lhs",
                Arc::new(move |obj| {
                    let rhs = is_vec(obj)?;

                    let new = elements
                        .iter()
                        .cloned()
                        .chain(rhs.iter().cloned())
                        .collect();

                    Some(Vector::erased(new))
                }),
            );
        }
        {
            let elements = elements.clone();
            v_table.inner.insert(
                "len",
                Arc::new(move |_| Some(Integer::erased(elements.len() as i32))),
            );
        }
        {
            let elements = elements.clone();
            v_table.inner.insert(
                "str",
                Arc::new(move |_| {
                    let elements = elements
                        .iter()
                        .try_fold(String::new(), |acc, element| {
                            element
                                .v_table()
                                .get("str")
                                .ok_or(())
                                .and_then(|f| f(None).ok_or(()))
                                .and_then(|obj| {
                                    if matches!(obj.r#type(), ObjectType::Str) {
                                        Ok(format!("{acc}{}, ", unsafe {
                                            obj.get_mut::<Str>().str.as_ref()
                                        }))
                                    } else {
                                        Err(())
                                    }
                                })
                        })
                        .ok()?;

                    Some(Str::erased(format!(
                        "[{}]",
                        &elements[..elements.len().saturating_sub(2)]
                    )))
                }),
            );
        }
        {
            let elements = elements.clone();
            v_table.inner.insert(
                "idx",
                Arc::new(move |obj| {
                    let Some(obj) = obj else {
                        return None;
                    };

                    if !matches!(obj.r#type(), ObjectType::Integer) {
                        return None;
                    }

                    let rhs = unsafe { obj.get_mut::<Integer>().val };

                    Some(
                        elements
                            .get(rhs as usize)
                            .cloned()
                            .unwrap_or(Unit::erased()),
                    )
                }),
            );
        }

        Reference {
            inner: erase(Arc::new(UnsafeCell::new(Vector { v_table, elements }))),
        }
    }
}

impl Display for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_list();
        for element in self.elements.iter() {
            dbg.entry(element);
        }
        dbg.finish()
    }
}

pub struct Builtin {
    v_table: VTable,
    r#fn: Arc<dyn Fn(Vec<Reference>) -> Result<Reference>>,
}

impl Object for Builtin {
    fn r#type(&self) -> ObjectType {
        ObjectType::Builtin
    }

    fn v_table(&self) -> &VTable {
        &self.v_table
    }
}

impl Builtin {
    pub fn erased(r#fn: impl Fn(Vec<Reference>) -> Result<Reference> + 'static) -> Reference {
        let mut v_table = VTable {
            inner: HashMap::new(),
        };

        v_table.inner.insert("truthy", Arc::new(move |_| None));

        Reference {
            inner: erase(Arc::new(UnsafeCell::new(Builtin {
                v_table,
                r#fn: Arc::new(r#fn),
            }))),
        }
    }

    pub fn call(&self, args: Vec<Reference>) -> Result<Reference> {
        (self.r#fn)(args)
    }
}

impl Display for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Builtin")
    }
}

impl Debug for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Builtin")
    }
}

#[derive(Debug)]
pub struct Str {
    v_table: VTable,
    pub str: Arc<str>,
}

impl Object for Str {
    fn r#type(&self) -> ObjectType {
        ObjectType::Str
    }

    fn v_table(&self) -> &VTable {
        &self.v_table
    }
}

impl Str {
    pub fn erased(str: String) -> Reference {
        let mut v_table = VTable {
            inner: HashMap::new(),
        };

        let str: Arc<str> = Arc::from(str.as_str());

        let is_str = |obj: Option<Reference>| -> Option<Arc<str>> {
            let Some(obj) = obj else {
                return None;
            };

            if !matches!(obj.r#type(), ObjectType::Str) {
                return None;
            }

            let rhs = unsafe { obj.get_mut::<Str>().str.clone() };

            Some(rhs)
        };
        {
            let str = str.clone();
            v_table.inner.insert(
                "truthy",
                Arc::new(move |_| Some(Bool::erased(str.len() > 0))),
            );
        }
        {
            let str = str.clone();
            v_table.inner.insert(
                "add_lhs",
                Arc::new(move |rhs| {
                    let rhs = is_str(rhs)?;
                    Some(Str::erased(format!("{}{}", str, rhs)))
                }),
            );
        }
        {
            let str = str.clone();
            v_table
                .inner
                .insert("str", Arc::new(move |_| Some(Str::erased(str.to_string()))));
        }
        {
            let str = str.clone();
            v_table.inner.insert(
                "len",
                Arc::new(move |_| Some(Integer::erased(str.len() as i32))),
            );
        }

        Reference {
            inner: erase(Arc::new(UnsafeCell::new(Str { v_table, str }))),
        }
    }
}

impl Display for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.str))
    }
}
