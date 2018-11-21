use crate::statemachine::StateMachine;
use regex_syntax::hir;
use regex_syntax::hir::{Hir, HirKind, Visitor};
use regex_syntax::{Parser, ParserBuilder};

use std::collections::VecDeque;

struct SMBuilder {
  stack: VecDeque<VecDeque<StateMachine>>,
}

impl Visitor for SMBuilder {
  type Output = StateMachine;
  type Err = ();

  fn start(&mut self) {
    println!("start");
  }

  fn finish(self) -> Result<Self::Output, Self::Err> {
    println!("finish");
    unimplemented!()
  }

  fn visit_pre(&mut self, _hir: &Hir) -> Result<(), Self::Err> {
    self.stack.push_back(VecDeque::new());
    match _hir.kind() {
      HirKind::Literal(..) => self
        .stack
        .front_mut()
        .unwrap()
        .push_front(StateMachine::from_seq(b"X")),
      _ => {}
    };
    println!("visit_pre  {:?}", _hir);
    Ok(())
  }

  fn visit_post(&mut self, _hir: &Hir) -> Result<(), Self::Err> {
    match _hir.kind() {
      HirKind::Concat(..) => self.flatten_with(|a, b| a.concat(b)),
      HirKind::Alternation(..) => self.flatten_with(|a, b| a.union(b)),
      HirKind::Repetition(..) => {
        self.pop_single();
        unimplemented!()
      }
      _ => {
        // lower top of stack
        let mut top_of_stack = self.stack.pop_back().unwrap_or_default();
        while let Some(contents) = top_of_stack.pop_front() {
          self.stack.front_mut().unwrap().push_back(contents);
        }
      }
    }
    println!("visit_post {:?}", _hir);
    Ok(())
  }

  fn visit_alternation_in(&mut self) -> Result<(), Self::Err> {
    println!("visit_alternation_in");
    Ok(())
  }
}

impl SMBuilder {
  fn construct_statemachine(regex: &str) {
    let mut parser = ParserBuilder::new().unicode(false).build();
    let res = parser.parse(regex).expect("invalid regex");
    println!("{:#?}", res);
    hir::visit(
      &res,
      SMBuilder {
        stack: VecDeque::new(),
      },
    )
    .unwrap();
  }

  fn flatten_with<F: Fn(&mut StateMachine, StateMachine)>(&mut self, f: F) {
    let mut components = self.stack.pop_back().unwrap();
    let mut sm = components.pop_front().unwrap();
    while let Some(alt) = components.pop_front() {
      f(&mut sm, alt);
    }
    self.stack.back_mut().unwrap().push_back(sm);
  }

  fn pop_single(&mut self) -> StateMachine {
    let mut tos = self.stack.pop_back().expect("stack is empty");
    assert_eq!(tos.len(), 1);
    tos.pop_front().unwrap()
  }
}

pub fn meme() {
  SMBuilder::construct_statemachine(r"(FLAG|RITSEC)\{\w{31}=\}");
  SMBuilder::construct_statemachine(r"(FLAG|RITSEC)\{\w+\}");
}
