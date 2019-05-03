use std::collections::BTreeMap;
use std::rc::Rc;
use unique::Id;

use crate::formula::Formula;
use crate::term::Term;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Flavour {
    True,
    False,
    Variable,
    Constant,
    Proposition,
    FunctionSymbol,
    PredicateSymbol,
    Application,
    Arguments,
    Equality,
    Negation,
    Disjunction,
    Conjunction,
    Equivalence,
    Universal,
    Existential,
    Top,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node {
    flavour: Flavour,
    data: Option<usize>,
    children: Vec<Rc<Node>>,
}

fn from_term(t: &Id<Term>, bound: &[usize]) -> Rc<Node> {
    use Flavour::*;
    use Term::*;

    Rc::new(match **t {
        Var(n) => Node {
            flavour: Variable,
            data: Some(bound[bound.len() - (n + 1)]),
            children: vec![],
        },
        Fn(ref c, ref ts) if ts.is_empty() => Node {
            flavour: Constant,
            data: Some(Id::id(c)),
            children: vec![]
        },
        Fn(ref f, ref ts) => Node {
            flavour: Application,
            data: None,
            children: vec![
                Rc::new(Node {
                    flavour: FunctionSymbol,
                    data: Some(Id::id(f)),
                    children: vec![],
                }),
                Rc::new(Node {
                    flavour: Arguments,
                    data: None,
                    children: ts.into_iter().map(|t| from_term(t, bound)).collect()
                })
            ]
        },
    })
}

fn from_formula(f: &Id<Formula>, bound: &mut Vec<usize>) -> Rc<Node> {
    use Flavour::*;
    use Formula::*;

    Rc::new(match **f {
        T => Node {
            flavour: True,
            data: None,
            children: vec![]
        },
        F => Node {
            flavour: False,
            data: None,
            children: vec![]
        },
        Eq(ref ps) => Node {
            flavour: Equality,
            data: None,
            children: ps.into_iter().map(|t| from_term(t, bound)).collect(),
        },
        Prd(ref p, ref ts) if ts.is_empty() => Node {
            flavour: Proposition,
            data: Some(Id::id(p)),
            children: vec![]
        },
        Prd(ref p, ref ts) => Node {
            flavour: Application,
            data: None,
            children: vec![
                Rc::new(Node {
                    flavour: PredicateSymbol,
                    data: Some(Id::id(p)),
                    children: vec![],
                }),
                Rc::new(Node {
                    flavour: Arguments,
                    data: None,
                    children: ts.into_iter().map(|t| from_term(t, bound)).collect()
                })
            ]
        },
        Not(ref p) => Node {
            flavour: Negation,
            data: None,
            children: vec![from_formula(p, bound)],
        },
        Imp(ref p, ref q) => Node {
            flavour: Disjunction,
            data: None,
            children: vec![
                Rc::new(Node {
                    flavour: Negation,
                    data: None,
                    children: vec![from_formula(p, bound)],
                }),
                from_formula(q, bound),
            ],
        },
        Or(ref ps) => Node {
            flavour: Disjunction,
            data: None,
            children: ps.into_iter().map(|p| from_formula(p, bound)).collect(),
        },
        And(ref ps) => Node {
            flavour: Conjunction,
            data: None,
            children: ps.into_iter().map(|p| from_formula(p, bound)).collect(),
        },
        Eqv(ref ps) => Node {
            flavour: Equivalence,
            data: None,
            children: ps.into_iter().map(|p| from_formula(p, bound)).collect(),
        },
        All(ref p) => {
            bound.push(Id::id(p));
            let node = Node {
                flavour: Universal,
                data: None,
                children: vec![
                    Rc::new(Node {
                        flavour: Variable,
                        data: Some(Id::id(p)),
                        children: vec![],
                    }),
                    from_formula(p, bound),
                ],
            };
            bound.pop();
            node
        }
        Ex(ref p) => {
            bound.push(Id::id(p));
            let node = Node {
                flavour: Existential,
                data: None,
                children: vec![
                    Rc::new(Node {
                        flavour: Variable,
                        data: Some(Id::id(p)),
                        children: vec![],
                    }),
                    from_formula(p, bound),
                ],
            };
            bound.pop();
            node
        }
    })
}

impl From<&Id<Formula>> for Node {
    fn from(f: &Id<Formula>) -> Self {
        Node {
            flavour: Flavour::Top,
            data: None,
            children: vec![from_formula(f, &mut vec![])],
        }
    }
}

fn flatten_step(
    n: &Rc<Node>,
    nodes: &mut Vec<u8>,
    edges: &mut Vec<(usize, usize)>,
    cache: &mut BTreeMap<Rc<Node>, usize>,
) -> usize {
    if cache.contains_key(n) {
        cache[n]
    } else {
        let children: Vec<_> = n
            .children
            .iter()
            .map(|n| flatten_step(n, nodes, edges, cache))
            .collect();
        let index = nodes.len();
        nodes.push(n.flavour as u8);
        for child in children {
            edges.push((index, child));
        }
        cache.insert(n.clone(), index);
        index
    }
}

pub fn flatten(n: Node) -> (Vec<u8>, Vec<(usize, usize)>) {
    let mut nodes = vec![];
    let mut edges = vec![];
    let mut cache = BTreeMap::new();

    flatten_step(&Rc::new(n), &mut nodes, &mut edges, &mut cache);
    (nodes, edges)
}
