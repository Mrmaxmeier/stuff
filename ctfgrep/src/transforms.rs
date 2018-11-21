use crate::statemachine::*;

fn get_hexpair(c: u8) -> (u8, u8) {
  const LUT: [u8; 16] = [
    '0' as u8, '1' as u8, '2' as u8, '3' as u8, '4' as u8, '5' as u8, '6' as u8, '7' as u8,
    '8' as u8, '9' as u8, 'A' as u8, 'B' as u8, 'C' as u8, 'D' as u8, 'E' as u8, 'F' as u8,
  ];
  let a = (c >> 4) & 0xF;
  let b = c & 0xF;
  (LUT[a as usize], LUT[b as usize])
}

pub fn hexlify(sm: &StateMachine) -> StateMachine {
  let mut n = sm.clone();
  n.transitions.drain();
  for (&s, trans) in sm.transitions.iter() {
    for (&c, &o) in trans.iter() {
      if let Some(c) = c {
        let hexpair = get_hexpair(c);
        let intermediate = n.new_state();
        n.trans(s, intermediate, Some(hexpair.0), None);
        n.trans(intermediate, o.target, Some(hexpair.1), o.match_char);
      } else {
        n.trans(s, o.target, None, None);
      }
    }
  }
  n.description = format!("hex({})", sm.description);
  n
}

pub fn ignore_case(sm: &mut StateMachine) {
  sm.convert(|_state, transitions| {
    let trans_ = transitions
      .iter()
      .map(|(&x, &y)| (x, y))
      .collect::<Vec<_>>();
    for (c, target) in trans_ {
      if let Some(c) = c {
        if char::from(c).is_alphabetic() {
          let flipped = c ^ (1 << 5);
          transitions.insert(Some(flipped), target);
        };
      }
    }
  });
  sm.description = format!("ignorecase({})", sm.description);
}
