use std::collections::{HashMap, HashSet, VecDeque};
use std::iter::FromIterator;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateID(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Transition {
  pub target: StateID,
  pub match_char: Option<u8>,
}

#[derive(Debug, Clone)]
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
    self
      .transitions
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
    self.state.0 += other.state.0;
    for s in other.accepting {
      self.accepting.insert(offset(s));
    }
    self.trans(StateID(0), offset(StateID(0)), None, None);
    self.description = format!("{}|{}", self.description, other.description);
  }

  pub fn concat(&mut self, other: StateMachine) {
    unimplemented!()
  }

  pub fn matches<'a, I: Iterator<Item = u8>>(&'a self, input: I) -> Matches<'a, I> {
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
    }
  }
}

pub struct ClosureIter<'a> {
  state_machine: &'a StateMachine,
  stack: VecDeque<(Transition, bool)>,
  input: Option<u8>,
}

impl<'a> Iterator for ClosureIter<'a> {
  type Item = Transition;

  fn next(&mut self) -> Option<Self::Item> {
    while let Some((state, consumed_input)) = self.stack.pop_front() {
      if consumed_input {
        return Some(state);
      }
      for (&inp, &trans) in self.state_machine.transitions.get(&state.target).unwrap() {
        if inp.is_none() || inp == self.input {
          self.stack.push_back((trans, inp.is_some()));
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
