use std::fmt::Debug;
use std::marker::PhantomData;

use Expr;

pub struct SimplifyContext<T> {
    changed: bool,
    _t: PhantomData<T>,
}

impl<T> SimplifyContext<T>
    where T: Clone + Debug + Eq
{
    pub fn new() -> SimplifyContext<T> {
        SimplifyContext {
            changed: false,
            _t: PhantomData,
        }
    }

    fn step(&mut self, e: Expr<T>) -> Expr<T> {
        let mut changed = false;
        let newval = match e {
            Expr::And(x, y) => {
                changed = true;
                let (x, y) = (*x, *y);
                match (x, y) {
                    (Expr::Const(false), _) => Expr::Const(false),
                    (Expr::Const(true), a) => self.step(a),
                    (_, Expr::Const(false)) => Expr::Const(false),
                    (a, Expr::Const(true)) => self.step(a),
                    (Expr::Or(a, b), c) => {
                        Expr::Or(Box::new(self.step(Expr::And(a, Box::new(c.clone())))),
                                 Box::new(self.step(Expr::And(b, Box::new(c)))))
                    }
                    (c, Expr::Or(a, b)) => {
                        Expr::Or(Box::new(self.step(Expr::And(Box::new(c.clone()), a))),
                                 Box::new(self.step(Expr::And(Box::new(c), b))))
                    }
                    (x, y) => {
                        if x == y {
                            self.step(x)
                        } else {
                            changed = false;
                            Expr::And(Box::new(self.step(x)), Box::new(self.step(y)))
                        }
                    }
                }
            }
            Expr::Or(x, y) => {
                changed = true;
                let (x, y) = (*x, *y);
                match (x, y) {
                    (Expr::Const(true), _) => Expr::Const(true),
                    (Expr::Const(false), a) => self.step(a),
                    (_, Expr::Const(true)) => Expr::Const(true),
                    (a, Expr::Const(false)) => self.step(a),
                    (x, y) => {
                        if x == y {
                            self.step(x)
                        } else {
                            changed = false;
                            Expr::Or(Box::new(self.step(x)), Box::new(self.step(y)))
                        }
                    }
                }
            }
            Expr::Not(x) => {
                changed = true;
                let x = *x;
                match x {
                    Expr::Const(false) => Expr::Const(true),
                    Expr::Const(true) => Expr::Const(false),
                    Expr::And(a, b) => {
                        Expr::Or(Box::new(self.step(Expr::Not(a))),
                                 Box::new(self.step(Expr::Not(b))))
                    }
                    Expr::Or(a, b) => {
                        Expr::And(Box::new(self.step(Expr::Not(a))),
                                  Box::new(self.step(Expr::Not(b))))
                    }
                    Expr::Not(a) => self.step(*a),
                    x => {
                        changed = false;
                        Expr::Not(Box::new(self.step(x)))
                    }
                }
            }
            Expr::Terminal(t) => Expr::Terminal(t),
            Expr::Const(c) => Expr::Const(c),
        };
        if changed {
            self.changed = true;
        }
        newval
    }

    pub fn simplify(&mut self, e: Expr<T>) -> Expr<T> {
        let mut e = e;
        self.changed = true;
        while self.changed {
            self.changed = false;
            let e_new = self.step(e);
            e = e_new;
        }
        e
    }

    pub fn sum_of_products(&mut self, e: Expr<T>) -> Expr<T> {
        // First simplify the expression. The nodes should be in order (from
        // tree root to leaves) OR > AND > NOT.
        let e = self.simplify(e);

        // Now collect all factors joined by top-level ORs.
        e
    }
}

mod test {
    use Expr;
    use super::*;
    use std::fmt::Debug;

    fn run_test<T>(orig: Expr<T>, expected: Expr<T>)
        where T: Clone + Debug + Eq
    {
        let mut ctx = SimplifyContext::new();
        let output = ctx.simplify(orig.clone());
        println!("Simplify: {:?} -> {:?} (expected {:?})",
                 orig,
                 output,
                 expected);
        assert!(output == expected);
    }

    #[test]
    fn distributive_law() {
        run_test(Expr::and(Expr::or(Expr::Terminal(1), Expr::Terminal(2)),
                           Expr::or(Expr::Terminal(3), Expr::Terminal(4))),
                 Expr::or(Expr::or(Expr::and(Expr::Terminal(1), Expr::Terminal(3)),
                                   Expr::and(Expr::Terminal(1), Expr::Terminal(4))),
                          Expr::or(Expr::and(Expr::Terminal(2), Expr::Terminal(3)),
                                   Expr::and(Expr::Terminal(2), Expr::Terminal(4)))));
    }

    #[test]
    fn demorgan_and() {
        run_test(Expr::not(Expr::or(Expr::Terminal(1), Expr::Terminal(2))),
                 Expr::and(Expr::not(Expr::Terminal(1)), Expr::not(Expr::Terminal(2))));
    }

    #[test]
    fn demorgan_or() {
        run_test(Expr::not(Expr::or(Expr::Terminal(1), Expr::Terminal(2))),
                 Expr::and(Expr::not(Expr::Terminal(1)), Expr::not(Expr::Terminal(2))));
    }

    #[test]
    fn shortcircuit_and() {
        run_test(Expr::and(Expr::Const(false), Expr::Terminal(1)),
                 Expr::Const(false));
    }

    #[test]
    fn shortcircuit_or() {
        run_test(Expr::or(Expr::Const(true), Expr::Terminal(1)),
                 Expr::Const(true));
    }
}