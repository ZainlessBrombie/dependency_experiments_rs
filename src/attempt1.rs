#![allow(unused)]

use std::any::{Any, TypeId};
use std::cell::OnceCell;
use std::collections::HashMap;
use std::mem::take;
use std::ops::Deref;
use std::rc::Rc;

pub trait Autowirable: 'static {
    // fn new(context: &Context) -> Self;
    fn post_init(&self) {}
}

struct ContextEntry {
    item: Rc<dyn Any>,
}

pub struct Context {
    content: HashMap<TypeId, ContextEntry>,
    constructors: Vec<Box<dyn Fn(&Context)>>,
    initializers: Vec<Box<dyn Fn(&Context)>>,
}

struct ContextInner {}

impl Context {
    pub fn new() -> Context {
        Context {
            content: Default::default(),
            constructors: vec![],
            initializers: vec![],
        }
    }

    pub fn register_type<T: Autowirable, F: Fn(&Context) -> T + 'static>(&mut self, getter: F) {
        let item = Rc::new(OnceCell::<T>::new());
        let entry = ContextEntry {
            item: item.clone() as _,
        };
        self.constructors.push(Box::new(move |context| {
            item.set(getter(context));
        }));
        self.initializers.push(Box::new(move |context| {
            context.get::<T>().inner.get().unwrap().post_init()
        }));

        self.content.insert(TypeId::of::<T>(), entry);
    }

    pub fn init(&mut self) {
        for constructor in take(&mut self.constructors) {
            constructor(&self);
        }

        for initializer in take(&mut self.initializers) {
            initializer(&self);
        }
    }

    pub fn get<T: Autowirable>(&self) -> Dep<T> {
        let inner = self
            .content
            .get(&TypeId::of::<T>())
            .unwrap()
            .item
            .clone()
            .downcast()
            .unwrap();
        return Dep { inner };
    }
}

pub struct Dep<T: Autowirable> {
    inner: Rc<OnceCell<T>>,
}

impl<'a, T: Autowirable> Deref for Dep<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.get().unwrap()
    }
}

struct A {
    b: Dep<B>,
    v: String,
}

struct B {
    a: Dep<A>,
    v: String,
}

impl Autowirable for A {
    fn post_init(&self) {
        println!("Post init A {}", self.b.v);
    }
}

impl Autowirable for B {
    fn post_init(&self) {
        println!("Post init B {}", self.a.v);
    }
}

#[test]
fn test_basic() {
    let mut context = Context::new();
    context.register_type(|context| A {
        b: context.get(),
        v: "VA".to_string(),
    });
    context.register_type(|context| B {
        a: context.get(),
        v: "VB".to_string(),
    });

    context.init();

    println!("{}", context.get::<A>().v);
    println!("{}", context.get::<B>().v);
}
