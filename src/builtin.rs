use std::collections::HashMap;

use crate::{
    eval::{error::Error, ops::Flow},
    object::{Builtin, Integer, ObjectType, Reference, Str, Unit},
};

/*
 {
    .me = fn(a, b) {
        a + b
    }
    .you
}
 */

pub fn builtins() -> HashMap<String, Reference> {
    [
        (
            "len".to_string(),
            Builtin::erased(|args| {
                if args.len() != 1 {
                    return Err(crate::eval::error::Error::Eval(
                        "Incorrect number of arguments used for len()".into(),
                    ));
                }

                let int = args[0]
                    .v_table()
                    .get("len")
                    .ok_or(crate::eval::error::Error::Eval(
                        "Object does not implement len operation.".into(),
                    ))?(None)
                .ok_or(crate::eval::error::Error::Eval(
                    "Object does not implement len operation.".into(),
                ))?;

                if !matches!(int.r#type(), ObjectType::Integer) {
                    return Err(crate::eval::error::Error::Eval(
                        "Object does not implement len operation.".into(),
                    ));
                };

                let len = unsafe { int.get_mut::<Integer>().val };

                return Ok(Flow::Continue(Integer::erased(len)));
            }),
        ),
        (
            "print".to_string(),
            Builtin::erased(|args| {
                if args.len() != 1 {
                    return Err(crate::eval::error::Error::Eval(
                        "Incorrect number of arguments used for print()".into(),
                    ));
                }

                let f = args[0].v_table().get("str").ok_or(Error::Eval(
                    "Object passed to print does not have a string represenetation.".into(),
                ))?;

                let str = f(None).unwrap_or(Str::erased("".into()));

                if !matches!(str.r#type(), ObjectType::Str) {
                    return Err(crate::eval::error::Error::Eval(
                        "Object did not return valid string representation.".into(),
                    ));
                }

                let str = unsafe { str.get_mut::<Str>() };

                println!("{}", str);

                return Ok(Flow::Continue(Unit::erased()));
            }),
        ),
        (
            "yeet".to_string(),
            Builtin::erased(|args| {
                if args.len() != 0 {
                    return Err(crate::eval::error::Error::Eval(
                        "Incorrect number of arguments used for len()".into(),
                    ));
                }

                std::process::exit(0);
            }),
        ),
        (
            "exit".to_string(),
            Builtin::erased(|args| {
                if args.len() != 0 {
                    return Err(crate::eval::error::Error::Eval(
                        "Incorrect number of arguments used for len()".into(),
                    ));
                }

                std::process::exit(0);
            }),
        ),
    ]
    .into_iter()
    .collect()
}
