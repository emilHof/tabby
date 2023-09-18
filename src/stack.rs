use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
};

use crate::object::Reference;

#[derive(Debug)]
pub struct Stack {
    frames: Vec<HashSet<String>>,
    scopes: HashMap<String, Vec<(Reference, u32)>>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            frames: vec![HashSet::new()],
            scopes: HashMap::new(),
        }
    }

    pub fn add(&mut self, ident: String, val: Reference) {
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(ident.clone());
        } else {
            self.frames.push(HashSet::from([ident.clone()]));
        }

        self.scopes
            .entry(ident.clone())
            .or_insert(vec![])
            .push((val, self.frames.len() as u32));
    }

    pub fn push(&mut self) {
        self.frames.push(HashSet::new());
    }

    pub fn pop(&mut self) {
        let prev_id = self.frames.len() as u32 - 1;
        let Some(out) = self.frames.pop() else {
            return;
        };

        for ident in out {
            if let Some(mut scope) = self.scopes.remove(&ident) {
                while let Some((val, id)) = scope.pop() {
                    if id < prev_id {
                        scope.push((val, id));
                        break;
                    }
                    drop(val)
                }

                if !scope.is_empty() {
                    self.scopes.insert(ident, scope);
                }
            }
        }
    }

    pub fn get(&mut self, ident: impl Borrow<String>) -> Option<Reference> {
        self.scopes
            .get(ident.borrow())
            .and_then(|scope| scope.last())
            .map(|(obj, _)| obj.clone())
    }

    pub fn take(&mut self, ident: impl Borrow<String>) -> Option<Reference> {
        self.scopes
            .get_mut(ident.borrow())
            .and_then(|scope| scope.pop())
            .map(|(obj, _)| obj)
    }

    pub fn assign(&mut self, ident: String, val: Reference) {
        let cur_id = self.frames.len() as u32 - 1;

        self.frames.last_mut().unwrap().insert(ident.clone());
        let scope = self.scopes.entry(ident).or_insert(vec![]);

        if self.frames.is_empty() {
            self.frames.push(HashSet::new());
        }

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
