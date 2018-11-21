use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::iter::FromIterator;

use graphviz as dot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateID(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Transition {
    pub target: StateID,
    pub match_char: Option<u8>,
}

#[derive(Clone)]
pub struct StateMachine {
    pub transitions: HashMap<StateID, HashMap<Option<u8>, Transition>>,
    pub accepting: HashSet<StateID>,
    pub state: StateID,
    pub description: String,
}

impl StateMachine {
    pub fn empty() -> Self {
        let mut transitions = HashMap::new();
        transitions.insert(StateID(0), HashMap::new());
        StateMachine {
            transitions,
            accepting: HashSet::new(),
            state: StateID(0),
            description: String::from("\\emptyset"),
        }
    }

    pub fn trans(
        &mut self,
        source: StateID,
        target: StateID,
        input: Option<u8>,
        match_char: Option<u8>,
    ) {
        self.transitions
            .entry(source)
            .or_insert_with(HashMap::new)
            .insert(input, Transition { target, match_char });
    }

    pub fn new_state(&mut self) -> StateID {
        self.state.0 += 1;
        self.state
    }

    pub fn from_seq(seq: &[u8]) -> Self {
        let mut sm = StateMachine::empty();
        for &c in seq {
            let prev = sm.state;
            let new = sm.new_state();
            sm.trans(prev, new, Some(c), Some(c));
        }
        sm.accepting.insert(sm.state);
        sm.description = String::from_utf8_lossy(seq).into();
        sm
    }

    pub fn convert<F: Fn(StateID, &mut HashMap<Option<u8>, Transition>)>(&mut self, f: F) {
        for (s, trans) in self.transitions.iter_mut() {
            f(*s, trans)
        }
    }

    pub fn union(&mut self, other: StateMachine) {
        let _offset = self.state.0 + 1;
        let offset = |sid: StateID| StateID(sid.0 + _offset);
        for (s, trans) in other.transitions {
            self.transitions.insert(
                offset(s),
                trans
                    .iter()
                    .map(|(&c, &transition)| {
                        (
                            c,
                            Transition {
                                target: offset(transition.target),
                                ..transition
                            },
                        )
                    })
                    .collect(),
            );
        }
        self.state.0 += other.state.0 + 1;
        for s in other.accepting {
            self.accepting.insert(offset(s));
        }
        self.trans(StateID(0), offset(StateID(0)), None, None);
        self.description = format!("{}|{}", self.description, other.description);
    }

    pub fn concat(&mut self, other: StateMachine) {
        let _offset = self.state.0 + 1;
        let offset = |sid: StateID| StateID(sid.0 + _offset);
        for (s, trans) in other.transitions {
            self.transitions.insert(
                offset(s),
                trans
                    .iter()
                    .map(|(&c, &transition)| {
                        (
                            c,
                            Transition {
                                target: offset(transition.target),
                                ..transition
                            },
                        )
                    })
                    .collect(),
            );
        }
        self.state.0 += other.state.0 + 1;
        for s in self.accepting.clone() {
            self.trans(s, offset(StateID(0)), None, None);
        }
        self.accepting.clear();
        for s in other.accepting {
            self.accepting.insert(offset(s));
        }
        self.description = format!("{}{}", self.description, other.description);
    }

    pub fn repeat(&self) -> StateMachine {
        // TODO: might introduce epsilon-cycles
        let mut sm = self.clone();
        for x in &self.accepting {
            sm.trans(*x, StateID(0), None, None);
        }
        sm.description.push('*');
        sm
    }

    pub fn matches<I: Iterator<Item = u8>>(&self, input: I) -> Matches<I> {
        Matches {
            state_machine: self,
            exec_stack: HashSet::from_iter([(StateID(0), Vec::new())].iter().cloned()),
            _swp_stack: HashSet::new(),
            input,
            result_buffer: VecDeque::new(),
        }
    }

    pub fn trans_closure(&self, state: StateID, input: Option<u8>) -> ClosureIter {
        ClosureIter {
            stack: self
                .transitions
                .get(&state)
                .map(|trans| {
                    trans
                        .iter()
                        .filter(|(&a, _)| a.is_none() || input == a)
                        .map(|(&a, &b)| (b, a.is_some()))
                        .collect()
                })
                .unwrap_or_else(VecDeque::new),
            input,
            state_machine: &self,
            visited: HashSet::new(),
        }
    }

    pub fn dump_dot(&self) {
        use std::fs::File;
        let mut f = File::create(format!("/tmp/statemachines/{}.dot", self.description)).unwrap();
        dot::render(&self, &mut f).unwrap()
    }
}

impl<'a> dot::Labeller<'a> for &StateMachine {
    type Node = StateID;
    type Edge = (StateID, StateID, Option<u8>);
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("example2").unwrap()
    }
    fn node_id(&'a self, n: &StateID) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n.0)).unwrap()
    }
    fn node_label<'b>(&'b self, n: &StateID) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr(
            if self.accepting.contains(n) {
                format!("{{ {} }}", n.0)
            } else {
                format!("{}", n.0)
            }
            .into(),
        )
    }
    fn edge_label<'b>(&'b self, (_, _, e): &Self::Edge) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr(
            match e {
                Some(c) => (*c as char).to_string(),
                None => "&epsilon;".to_owned(),
            }
            .into(),
        )
    }
}

impl<'a> dot::GraphWalk<'a> for &StateMachine {
    type Node = StateID;
    type Edge = (StateID, StateID, Option<u8>);
    fn nodes(&self) -> dot::Nodes<'a, StateID> {
        (0..=self.state.0).map(StateID).collect()
    }
    fn edges(&'a self) -> dot::Edges<'a, Self::Edge> {
        self.transitions
            .iter()
            .flat_map(|(&source, trans)| {
                trans
                    .iter()
                    .map(move |(&c, trans)| (source, trans.target, c))
            })
            .collect()
    }
    fn source(&self, e: &Self::Edge) -> StateID {
        let &(s, _, _) = e;
        s
    }
    fn target(&self, e: &Self::Edge) -> StateID {
        let &(_, t, _) = e;
        t
    }
}

impl fmt::Debug for StateMachine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "StateMachine {:?} {{
    accepting: {:?},
    transitions: {:?},
    state: {:?}
}}",
            self.description, self.accepting, self.transitions, self.state
        )
    }
}

pub struct ClosureIter<'a> {
    state_machine: &'a StateMachine,
    stack: VecDeque<(Transition, bool)>,
    input: Option<u8>,
    visited: HashSet<StateID>,
}

impl<'a> Iterator for ClosureIter<'a> {
    type Item = Transition;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((state, consumed_input)) = self.stack.pop_front() {
            if consumed_input {
                return Some(state);
            }
            for (&inp, &trans) in &self.state_machine.transitions[&state.target] {
                if self.visited.contains(&trans.target) {
                    continue;
                }
                if inp.is_none() || inp == self.input {
                    self.stack.push_back((trans, inp.is_some()));
                    self.visited.insert(trans.target);
                }
            }
        }
        None
    }
}

pub struct Matches<'a, I> {
    state_machine: &'a StateMachine,
    exec_stack: HashSet<(StateID, Vec<u8>)>,
    _swp_stack: HashSet<(StateID, Vec<u8>)>,
    result_buffer: VecDeque<Vec<u8>>,
    input: I,
}

impl<'a, I: Iterator<Item = u8>> Iterator for Matches<'a, I> {
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(x) = self.result_buffer.pop_front() {
            return Some(x);
        }
        println!("called next on Matches");
        while let Some(c) = self.input.next() {
            println!(
                "Matches iter tracked automatons: {}, input: {}",
                self.exec_stack.len(),
                c as char,
            );
            for (state, matched) in self.exec_stack.drain() {
                for Transition {
                    target: next,
                    match_char,
                } in self.state_machine.trans_closure(state, Some(c))
                {
                    let mut match_copy = matched.clone();
                    if let Some(reconstructed) = match_char {
                        match_copy.push(reconstructed);
                    }
                    if self.state_machine.accepting.contains(&next) {
                        self.result_buffer.push_back(match_copy.clone());
                    }
                    self._swp_stack.insert((next, match_copy));
                }
            }

            for val in self._swp_stack.drain() {
                self.exec_stack.insert(val);
            }
            self.exec_stack.insert((StateID(0), Vec::new()));

            if let Some(x) = self.result_buffer.pop_front() {
                return Some(x);
            }
        }

        self.result_buffer.pop_front()
    }
}
