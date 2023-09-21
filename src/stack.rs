use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
};

use crate::{builtin::builtins, object::Reference};

#[derive(Debug)]
struct Frame {
    scope: Vec<HashSet<String>>,
    vars: HashMap<String, Vec<(Reference, u32)>>,
}

#[derive(Debug)]
pub struct Stack {
    frames: Vec<Frame>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            frames: vec![Frame {
                scope: vec![HashSet::new()],
                vars: builtins()
                    .into_iter()
                    .map(|(k, v)| (k, vec![(v, 0)]))
                    .collect(),
            }],
        }
    }

    pub fn push_frame(&mut self) {
        self.frames.push(Frame {
            scope: vec![HashSet::new()],
            vars: builtins()
                .into_iter()
                .map(|(k, v)| (k, vec![(v, 0)]))
                .collect(),
        })
    }

    pub fn pop_frame(&mut self) {
        self.frames.pop();
    }

    fn scope(&self) -> &Vec<HashSet<String>> {
        &self.frames[self.frames.len() - 1].scope
    }

    fn scope_mut(&mut self) -> &mut Vec<HashSet<String>> {
        &mut self.frames.last_mut().unwrap().scope
    }

    fn vars_mut(&mut self) -> &mut HashMap<String, Vec<(Reference, u32)>> {
        &mut self.frames.last_mut().unwrap().vars
    }

    pub fn add(&mut self, ident: String, val: Reference) {
        if let Some(frame) = self.scope_mut().last_mut() {
            frame.insert(ident.clone());
        } else {
            self.scope_mut().push(HashSet::from([ident.clone()]));
        }

        let cur_id = self.scope().len() as u32;

        self.vars_mut()
            .entry(ident.clone())
            .or_insert(vec![])
            .push((val, cur_id));
    }

    pub fn push(&mut self) {
        self.scope_mut().push(HashSet::new());
    }

    pub fn pop(&mut self) {
        let prev_id = self.scope().len() as u32 - 1;
        let Some(out) = self.scope_mut().pop() else {
            return;
        };

        for ident in out {
            if let Some(mut scope) = self.vars_mut().remove(&ident) {
                while let Some((val, id)) = scope.pop() {
                    if id < prev_id {
                        scope.push((val, id));
                        break;
                    }
                    drop(val)
                }

                if !scope.is_empty() {
                    self.vars_mut().insert(ident, scope);
                }
            }
        }
    }

    pub fn get(&mut self, ident: impl Borrow<String>) -> Option<Reference> {
        self.vars_mut()
            .get(ident.borrow())
            .and_then(|var| var.last())
            .map(|(obj, _)| obj.clone())
    }

    pub fn take(&mut self, ident: impl Borrow<String>) -> Option<Reference> {
        self.vars_mut()
            .get_mut(ident.borrow())
            .and_then(|var| var.pop())
            .map(|(obj, _)| obj)
    }

    pub fn assign(&mut self, ident: String, val: Reference) {
        let cur_id = self.scope().len() as u32 - 1;

        self.scope_mut().last_mut().unwrap().insert(ident.clone());

        if self.scope().is_empty() {
            self.scope_mut().push(HashSet::new());
        }

        let scope = self.vars_mut().entry(ident).or_insert(vec![]);

        while let Some((val, id)) = scope.pop() {
            if id < cur_id {
                scope.push((val, id));
                break;
            }
            drop(val)
        }

        scope.push((val, cur_id));
    }
}
