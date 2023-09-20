use std::collections::HashMap;

use crate::{
    eval::ops::Flow,
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

                if !matches!(args[0].r#type(), ObjectType::Str) {
                    return Err(crate::eval::error::Error::Eval(
                        "Incorrect number of arguments used for len()".into(),
                    ));
                }

                let str = unsafe { args[0].get_mut::<Str>() };

                return Ok(Flow::Continue(Integer::erased(str.str.len() as i32)));
            }),
        ),
        (
            "print".to_string(),
            Builtin::erased(|args| {
                if args.len() != 1 {
                    return Err(crate::eval::error::Error::Eval(
                        "Incorrect number of arguments used for len()".into(),
                    ));
                }

                if !matches!(args[0].r#type(), ObjectType::Str) {
                    return Err(crate::eval::error::Error::Eval(
                        "Incorrect number of arguments used for len()".into(),
                    ));
                }

                let str = unsafe { args[0].get_mut::<Str>() };

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
