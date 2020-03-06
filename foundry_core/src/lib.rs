use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicU32, Ordering};
use std::rc::{Rc};
use std::cell::{RefCell, Ref, RefMut};

pub struct StateMutRef<'a, T> {
    owner: &'a State<T>,
    guard: RefMut<'a, T>,
}

impl<'a, T> Deref for StateMutRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl<'a, T> DerefMut for StateMutRef<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut()
    }
}

impl<'a, T> Drop for StateMutRef<'a, T> {
    fn drop(&mut self) {
        self.owner.invalidate(&mut *self.guard)
    }
}

pub struct State<T> {
    value: RefCell<T>,
    listeners: RefCell<Vec<Box<dyn Fn(&T)>>>,
    invalidated: AtomicU32,
}

impl<T> State<T> {
    pub fn new(value: T) -> State<T> {
        State {
            value: RefCell::new(value),
            listeners: RefCell::new(Vec::new()),
            invalidated: AtomicU32::new(0),
        }
    }

    fn bind(&self, render_count: u32, callback: Box<dyn Fn(&T)>) {
        self.invalidated.store(render_count, Ordering::Relaxed);
        self.listeners.borrow_mut().push(callback);
    }

    pub fn get_mut(&self) -> StateMutRef<T> {
        let guard = self.value.borrow_mut();
        StateMutRef {
            owner: &self,
            guard: guard
        }
    }
    pub fn get(&self) -> Ref<'_, T> {
        self.value.borrow()
    }
    fn invalidate(&self, state: &T) {
        self.invalidated.fetch_add(1, Ordering::Relaxed);
        for listener in self.listeners.borrow().iter() {
            listener(state);
        }
    }
}

pub enum Value {
    Str(String),
    Event((Rc<dyn Fn()>, u64) ),
}

impl std::convert::From<String> for Value {
    fn from(item: String) -> Self {
        Value::Str(item)
    }
}

impl std::convert::From<&str> for Value {
    fn from(item: &str) -> Self {
        Value::Str(item.to_string())
    }
}

impl<S: 'static, F: Fn(CallbackInfo<S>) + 'static> std::convert::From<Rc<Event<S, F>>> for Value {
    fn from(item: Rc<Event<S, F>>) -> Self {
        let state = item.state.clone();
        let action = item.action.clone();

        let func = move || {
            //let state = state_rc.clone();
            let ci = CallbackInfo{state: &state};

            action(ci);
        };

        Value::Event((Rc::new(func), item.event_id))
    }
}

static mut EVENT_ID_COUNTER: u64 = 0;

pub struct Event<S, F: Fn(CallbackInfo<S>) + 'static> {
    event_id: u64,
    action: Rc<F>,
    state: Rc<State<S>>
}

impl<S, F: Fn(CallbackInfo<S>) + 'static> Event<S, F> {
    pub fn new(state: &Rc<State<S>>, action: F) -> Rc<Event<S, F>> {
        let event_id;
        
        //TODO: remove this unsafe.
        unsafe {
            event_id = EVENT_ID_COUNTER;
            EVENT_ID_COUNTER += 1;
        }

        Rc::new(Event { event_id, action: Rc::new(action), state: state.clone() })
    }
}

pub trait DomNode<T> {
    fn get_children<'a>(&'a self) -> Box<dyn ExactSizeIterator<Item= &'a Box<dyn DomNode<T>>> + 'a>;
    fn get_inner(&self) -> T;
    fn get_params(&self) -> Option<Rc<Vec<(String, Value)>>>;
}

impl DomNode<String> for String {
    fn get_children<'a>(&'a self) -> Box<dyn ExactSizeIterator<Item= &'a Box<dyn DomNode<String>>> + 'a> {
        Box::new(std::iter::empty())
    }
    fn get_inner(&self) -> String {
        self.clone()
    }
    fn get_params<'a>(&'a self) -> Option<Rc<Vec<(String, Value)>>> {
        None
    }
}

pub trait DomIntoIterator {
    type Item;
    type IntoIter: ExactSizeIterator<Item = Self::Item>;
    fn into_iter(self) -> Self::IntoIter;
}

impl<'a, K> DomIntoIterator for &'a Vec<K>
{
    type Item = &'a K;
    type IntoIter = std::slice::Iter<'a, K>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct CallbackInfo<'a, T> {
    pub state: &'a State<T>
}

pub struct RenderInfo<'a, T> {
    pub state: &'a T
}

pub trait Context { //TODO: this should have a better name.
    type Node: std::cmp::Eq + std::fmt::Debug;
    fn set_recent_tree(&mut self, tree: Option<Box<dyn DomNode<Self::Node>>>);
    fn get_recent_tree(&self) -> Option<&Box<dyn DomNode<Self::Node>>>;
    fn commit_changes(&mut self, changes: Vec<ReconciliationNote<Self::Node>>);
}
pub struct Component<CONTEXT: Context, STATE> {
    context: Rc<RefCell<Option<CONTEXT>>>,
    state: Rc<State<STATE>>,
    render_func: Box<dyn Fn(RenderInfo<STATE>) -> Box<dyn DomNode<CONTEXT::Node>>>,
    last_redraw: AtomicU32
}

impl<C: Context + 'static,
     S: 'static> Component<C, S> {
    pub fn new(state_obj: S, render_func: impl Fn(RenderInfo<S>) -> Box<dyn DomNode<C::Node>> + 'static) -> (Rc<State<S>>, Rc<Component::<C, S>>) {
        let last_redraw = 0;
        let state = Rc::new(State::new(state_obj));

        let component = Rc::new(Component {
            context: Rc::new(RefCell::new(Option::None)),
            state: state.clone(),
            render_func: Box::new(render_func),
            last_redraw: AtomicU32::new(last_redraw),
        });

        let component_clone = component.clone();
        state.bind(last_redraw + 1, Box::new(move |s| {
            component_clone.redraw(s);
        }));

        (state.clone(), component)
    }

    pub fn from_state(state: Rc<State<S>>, render_func: impl Fn(RenderInfo<S>) -> Box<dyn DomNode<C::Node>> + 'static) -> Rc<Component::<C, S>> {
        let last_redraw = 0;

        let component = Rc::new(Component {
            context: Rc::new(RefCell::new(Option::None)),
            state: state.clone(),
            render_func: Box::new(render_func),
            last_redraw: AtomicU32::new(last_redraw),
        });

        let component_clone = component.clone();
        state.bind(last_redraw + 1, Box::new(move |s| {
            component_clone.clone().redraw(s);
        }));

        component
    }

    pub fn bind_context(&self, context: C) {
        {
            let mut x = self.context.borrow_mut();
            std::mem::replace(&mut *x, Option::Some(context));
        }
        let s = &*self.state.as_ref().get();
        self.state.invalidate(s);
        //self.redraw();
    }

    fn redraw(&self, state: &S) {
        let invalidated_number = self.state.invalidated.load(Ordering::Relaxed);
        if self.last_redraw.swap(invalidated_number, Ordering::Relaxed) < invalidated_number {
            let ri = RenderInfo {state};
            let root = (self.render_func)(ri);
            
            let context = &mut *self.context.borrow_mut();
            match context {
                Some(c) => {
                    let change_list = reconcile_tree(c.get_recent_tree(), &root);
                    c.commit_changes(change_list);
                    c.set_recent_tree(Some(root))
                },
                None => {
                    //TODO: remove this panic.
                    panic!("No context found.");
                }
            }
        }
    }
}

impl<C: Context<Node=String>, S> Component<C, S> {
    pub fn render_to_string(&self) -> String {
        let s = self.state.as_ref();
        let g = s.get();
        let ri = RenderInfo {state: &*g};

        let root = (self.render_func)(ri);

        let x = root.get_inner().clone();
        return x.get_inner();
    }
}

pub enum ReconciliationOperation<T> {
    Remove,
    Add((T, Option<Rc<Vec<(String, Value)>>>)),
}

impl<T: std::fmt::Debug> std::fmt::Debug for ReconciliationOperation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReconciliationOperation::<T>::Remove => write!(f, "[Rem note]"),
            ReconciliationOperation::<T>::Add(a) => write!(f, "[Add note: {:?}]", a.0)
        }
    }
}

#[derive(std::fmt::Debug)]
pub struct ReconciliationNote<T> {
    pub operation: ReconciliationOperation<T>,
    pub key: Option<String>,
    pub path: Vec<u32>, //Used to seek through the DOM.
    pub index: u32,
}
impl<T: std::fmt::Debug> ReconciliationNote<T> {
    fn create_with_key(operation: ReconciliationOperation<T>, key: impl Into<String>, path: Vec<u32>, index: u32) -> ReconciliationNote<T> {
        ReconciliationNote {
            operation,
            key: Some(key.into()),
            path,
            index,
        }
    }
    fn create(operation: ReconciliationOperation<T>, path: Vec<u32>, index: u32) -> ReconciliationNote<T> {
        ReconciliationNote {
            operation,
            key: None,
            path,
            index,
        }
    }
}

//TODO: Test these methods.
fn recursively_add<T: std::cmp::PartialEq + std::cmp::Eq + std::fmt::Debug>(new: &Box<dyn DomNode<T>>, path: &mut Vec<u32>, index: u32, list: &mut Vec<ReconciliationNote<T>>) {
    let note = ReconciliationNote::create(ReconciliationOperation::Add((new.get_inner(), new.get_params())), path.clone(), index);
    list.push(note);

    path.push(index);
    for (i, ch) in new.get_children().enumerate() {
        recursively_add(ch, path, i as u32, list);
    }
    path.pop();
}

/// Recursive function.
fn reconcile_nodes<T: std::cmp::PartialEq + std::cmp::Eq + std::fmt::Debug>(base: &Box<dyn DomNode<T>>, new: &Box<dyn DomNode<T>>, path: &mut Vec<u32>, index: u32, list: &mut Vec<ReconciliationNote<T>>) {
    //TODO: Consider keys.
    //TODO: Consider calculating hashes for each node to more quickly compare subtrees.

    if base.get_inner() != new.get_inner() {
        list.push(ReconciliationNote::create(ReconciliationOperation::Remove, path.clone(), index));

        recursively_add(new, path, index, list);
    } else {
        path.push(index);
        for (i, ch) in new.get_children().enumerate() {
            if base.get_children().len() <= i {
                recursively_add(ch, path, i as u32, list);
            } else {
                let base_ch_lookup = &base.get_children().nth(i); //TODO: this is not optimal, we should not use an iterator here.
                if let Some(base_ch) = base_ch_lookup {
                    reconcile_nodes(base_ch, ch, path, i as u32, list);
                }
            }
        }
        path.pop();
    }
}

fn reconcile_tree<T: std::cmp::PartialEq + std::cmp::Eq + std::fmt::Debug>(base: Option<&Box<dyn DomNode<T>>>, new: &Box<dyn DomNode<T>>) -> Vec<ReconciliationNote<T>> {
    let mut list = Vec::new();
    let mut path = Vec::new();

    match base {
        None => {
            recursively_add(new, &mut path, 0, &mut list)
        },
        Some(old_base) => {
            reconcile_nodes(old_base, new, &mut path, 0, &mut list);
        }
    }
    list
}




#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
