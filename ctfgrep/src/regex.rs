use crate::statemachine::{StateID, StateMachine};
use regex_syntax::hir;
use regex_syntax::hir::{Hir, HirKind, Visitor};
use regex_syntax::ParserBuilder;

use std::collections::VecDeque;

pub struct SMBuilder {
  stack: VecDeque<VecDeque<StateMachine>>,
}

impl Visitor for SMBuilder {
  type Output = StateMachine;
  type Err = ();

  fn start(&mut self) {
    println!("start");
    self.stack.push_back(VecDeque::new());
  }

  fn finish(mut self) -> Result<Self::Output, Self::Err> {
    println!("finish");
    let sm = self.pop_single();
    Ok(sm)
  }

  fn visit_pre(&mut self, _hir: &Hir) -> Result<(), Self::Err> {
    self.stack.push_back(VecDeque::new());
    match _hir.kind() {
      HirKind::Literal(literal) => {
        self
          .stack
          .back_mut()
          .unwrap()
          .push_back(StateMachine::from_seq(&[match literal {
            hir::Literal::Byte(b) => *b,
            hir::Literal::Unicode(c) => *c as u8,
          }]))
      }
      HirKind::Class(class) => self
        .stack
        .back_mut()
        .unwrap()
        .push_back(Self::from_class(class)),
      _ => {}
    };
    println!("visit_pre  {:?}", _hir);
    Ok(())
  }

  fn visit_post(&mut self, _hir: &Hir) -> Result<(), Self::Err> {
    match _hir.kind() {
      HirKind::Concat(..) => self.flatten_with(|a, b| a.concat(b)),
      HirKind::Alternation(..) => self.flatten_with(|a, b| a.union(b)),
      HirKind::Repetition(rep) => {
        let mut sm = self.pop_single();
        match &rep.kind {
          hir::RepetitionKind::OneOrMore => sm.concat(sm.repeat()),
          hir::RepetitionKind::ZeroOrMore => sm = sm.repeat(),
          hir::RepetitionKind::ZeroOrOne => sm.union(StateMachine::empty()),
          hir::RepetitionKind::Range(range) => unimplemented!(),
        }
        self.stack.back_mut().unwrap().push_back(sm);
      }
      _ => {
        // lower top of stack
        let mut top_of_stack = self.stack.pop_back().unwrap_or_default();
        while let Some(contents) = top_of_stack.pop_front() {
          self.stack.back_mut().unwrap().push_back(contents);
        }
      }
    }
    println!("visit_post {:?}", _hir);
    println!("{:#?}", self.stack);
    Ok(())
  }

  fn visit_alternation_in(&mut self) -> Result<(), Self::Err> {
    println!("visit_alternation_in");
    Ok(())
  }
}

impl SMBuilder {
  pub fn construct_statemachine(regex: &str) -> Result<StateMachine, ()> {
    let mut parser = ParserBuilder::new()
      .allow_invalid_utf8(true)
      .unicode(false)
      .build();
    let res = parser.parse(regex).expect("invalid regex");
    println!("{:#?}", res);
    let builder = SMBuilder {
      stack: VecDeque::new(),
    };
    hir::visit(&res, builder)
  }

  fn flatten_with<F: Fn(&mut StateMachine, StateMachine)>(&mut self, f: F) {
    let mut components = self.stack.pop_back().unwrap();
    let mut sm = components.pop_front().unwrap();
    while let Some(alt) = components.pop_front() {
      f(&mut sm, alt);
    }
    self.stack.back_mut().expect("stack empty").push_back(sm);
  }

  fn pop_single(&mut self) -> StateMachine {
    let mut tos = self.stack.pop_back().expect("stack is empty");
    assert_eq!(tos.len(), 1);
    tos.pop_front().unwrap()
  }

  fn from_class(class: &hir::Class) -> StateMachine {
    match class {
      hir::Class::Bytes(class) => {
        let mut sm = StateMachine::empty();
        sm.accepting.insert(StateID(1));
        sm.description = String::new();
        sm.description.push('[');
        for byterange in class.iter() {
          sm.description.push(byterange.start() as char);
          if byterange.start() != byterange.end() {
            sm.description.push('-');
            sm.description.push(byterange.end() as char);
          }
          for c in byterange.start()..=byterange.end() {
            sm.trans(StateID(0), StateID(1), Some(c), Some(c));
          }
        }
        sm.description.push(']');
        sm.state.0 = 1;
        sm
      }
      _ => unimplemented!(),
    }
  }
}
