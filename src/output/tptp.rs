use std::borrow::Cow;
use std::io;
use std::io::Write;
use tptp::syntax::*;
use unique::Id;

use crate::formula::Formula;
use crate::symbol::Symbol;
use crate::term::Term;

fn symbol_name(symbol: &Symbol) -> Name {
    use Symbol::*;
    match symbol {
        Original(ref s) => Name::LowerWord(Cow::Borrowed(s)),
        Introduced(ref id) => {
            Name::LowerWord(Cow::Owned(format!("k_{:x}", id)))
        }
    }
}

fn term(t: &Id<Term>, bound: usize) -> FofTerm {
    use Term::*;
    match **t {
        Var(n) => {
            FofTerm::Variable(Cow::Owned(format!("X{}", (bound - 1) - n)))
        }
        Fn(ref f, ref ts) => FofTerm::Functor(
            symbol_name(f),
            ts.into_iter().map(|t| term(t, bound)).collect(),
        ),
    }
}

fn formula(f: &Id<Formula>, bound: usize) -> FofFormula {
    use Formula::*;
    match **f {
        T => FofFormula::Boolean(true),
        F => FofFormula::Boolean(false),
        Eq(ref ts) => FofFormula::Assoc(
            AssocConnective::And,
            ts.pairs()
                .map(|(t, s)| {
                    FofFormula::Infix(
                        InfixEquality::Equal,
                        term(t, bound),
                        term(s, bound),
                    )
                })
                .collect(),
        ),
        Prd(ref p, ref ts) => FofFormula::Predicate(
            symbol_name(p),
            ts.into_iter().map(|t| term(t, bound)).collect(),
        ),
        Not(ref p) => {
            FofFormula::Unary(UnaryConnective::Not, Box::new(formula(p, bound)))
        }
        Imp(ref p, ref q) => FofFormula::NonAssoc(
            NonAssocConnective::LRImplies,
            Box::new(formula(p, bound)),
            Box::new(formula(q, bound)),
        ),
        Or(ref ps) => FofFormula::Assoc(
            AssocConnective::Or,
            ps.into_iter().map(|p| formula(p, bound)).collect(),
        ),
        And(ref ps) => FofFormula::Assoc(
            AssocConnective::And,
            ps.into_iter().map(|p| formula(p, bound)).collect(),
        ),
        Eqv(ref ps) => FofFormula::Assoc(
            AssocConnective::And,
            ps.pairs()
                .map(|(p, q)| {
                    FofFormula::NonAssoc(
                        NonAssocConnective::Equivalent,
                        Box::new(formula(p, bound)),
                        Box::new(formula(q, bound)),
                    )
                })
                .collect(),
        ),
        All(ref p) => FofFormula::Quantified(
            FofQuantifier::Forall,
            vec![Cow::Owned(format!("X{}", bound))],
            Box::new(formula(p, bound + 1)),
        ),
        Ex(ref p) => FofFormula::Quantified(
            FofQuantifier::Exists,
            vec![Cow::Owned(format!("X{}", bound))],
            Box::new(formula(p, bound + 1)),
        ),
    }
}

fn statement_name(f: &Id<Formula>) -> Name {
    Name::LowerWord(Cow::Owned(format!("f{:x}", Id::id(f))))
}

fn statement(f: &Id<Formula>) -> Statement {
    Statement::Fof(statement_name(f), FormulaRole::Plain, formula(f, 0), None)
}

pub fn write_statement<W: Write>(w: &mut W, f: &Id<Formula>) -> io::Result<()> {
    writeln!(w, "{}", statement(f))
}
