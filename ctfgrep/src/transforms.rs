use crate::statemachine::*;

fn get_hexpair(c: u8) -> (u8, u8) {
    #[rustfmt::skip]
    const LUT: [u8; 16] = [
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
        b'A', b'B', b'C', b'D', b'E', b'F',
    ];
    let a = (c >> 4) & 0xF;
    let b = c & 0xF;
    (LUT[a as usize], LUT[b as usize])
}

pub fn hexlify(sm: &StateMachine) -> StateMachine {
    let mut n = sm.clone();
    n.transitions.clear();
    for (&s, trans) in sm.transitions.iter() {
        for (&c, trans) in trans.iter() {
            for o in trans {
                if let Some(c) = c {
                    let hexpair = get_hexpair(c);
                    let intermediate = n.new_state();
                    n.trans(s, intermediate, Some(hexpair.0), None);
                    n.trans(intermediate, o.target, Some(hexpair.1), o.match_char);
                } else {
                    n.trans(s, o.target, None, o.match_char);
                }
            }
        }
    }
    n.description = format!("hex({})", sm.description);
    n
}

pub fn base64ify(sm: &StateMachine) -> StateMachine {
    let mut n = sm.clone();
    n.transitions.clear();
    for (&s, trans) in sm.transitions.iter() {
        for (&c, trans) in trans.iter() {
            for o in trans {
                if let Some(c) = c {
                    let hexpair = get_hexpair(c);
                    let intermediate = n.new_state();
                    n.trans(s, intermediate, Some(hexpair.0), None);
                    n.trans(intermediate, o.target, Some(hexpair.1), o.match_char);
                } else {
                    n.trans(s, o.target, None, o.match_char);
                }
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
            .map(|(&x, y)| (x, y.clone()))
            .collect::<Vec<_>>();
        for (c, targets) in trans_ {
            if let Some(c) = c {
                if char::from(c).is_alphabetic() {
                    let flipped = Some(c ^ (1 << 5));
                    let res = transitions // TODO: mutating union?
                        .entry(flipped)
                        .or_default()
                        .union(&targets)
                        .cloned()
                        .collect();
                    transitions.insert(flipped, res);
                };
            }
        }
    });
    sm.description = format!("ignorecase({})", sm.description);
}
