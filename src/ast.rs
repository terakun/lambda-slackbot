use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum Stat {
    Let(String, Box<AST>),
    Exp(Box<AST>),
}

impl Stat {
    pub fn get_expr(&self) -> AST {
        match *self {
            Stat::Let(_, ref e) => *e.clone(),
            Stat::Exp(ref e) => *e.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AST {
    Var(String),
    App(Box<AST>, Box<AST>),
    Abs(String, Box<AST>),
}

impl ToString for AST {
    fn to_string(&self) -> String {
        match *self {
            AST::Var(ref s) => s.to_string(),
            AST::App(ref e1, ref e2) => {
                let e1str = if let AST::Var(_) = **e1 {
                    e1.to_string()
                } else {
                    format!("({})", e1.to_string())
                };
                let e2str = if let AST::Var(_) = **e2 {
                    e2.to_string()
                } else {
                    format!("({})", e2.to_string())
                };

                format!("{} {}", e1str, e2str)
            }
            AST::Abs(ref s, ref e) => format!("\\{}.{}", s, e.to_string()),
        }
    }
}

impl AST {
    // 自由な出現かをチェック
    fn free(&self, v: &String) -> bool {
        match *self {
            AST::Var(ref v2) => v == v2,
            AST::App(ref e1, ref e2) => e1.free(v) || e2.free(v),
            AST::Abs(ref v2, ref e) => if v == v2 {
                false
            } else {
                e.free(v)
            },
        }
    }

    pub fn variables(&self, vars: &mut Vec<String>) {
        match *self {
            AST::Var(ref v) => {
                vars.push(v.clone());
            }
            AST::App(ref e1, ref e2) => {
                e1.variables(vars);
                e2.variables(vars);
            }
            AST::Abs(_, ref e) => {
                e.variables(vars);
            }
        }
    }

    pub fn free_variables(&self, free_vars: &mut Vec<String>) {
        let mut vars: Vec<String> = Vec::new();
        self.variables(&mut vars);
        for v in vars {
            if self.free(&v) {
                free_vars.push(v);
            }
        }
    }

    fn new_variable(&self) -> Option<String> {
        for i in 0.. {
            let var = "v".to_string() + &i.to_string();
            if !self.free(&var) {
                return Some(var);
            }
        }
        None
    }

    pub fn assign(&self, e: &Self, v: &String) -> Self {
        match *self {
            AST::Var(ref v2) => if v == v2 {
                e.clone()
            } else {
                self.clone()
            },
            AST::App(ref e1, ref e2) => {
                AST::App(Box::new(e1.assign(e, v)), Box::new(e2.assign(e, v)))
            }
            AST::Abs(ref v2, ref e2) => {
                if v == v2 {
                    self.clone()
                } else {
                    if e.free(v2) {
                        let new_v = e.new_variable().unwrap();
                        AST::Abs(
                            new_v.clone(),
                            Box::new(e2.assign(&AST::Var(new_v), v2).assign(e, v)),
                        )
                    } else {
                        AST::Abs(v2.clone(), Box::new(e2.assign(e, v)))
                    }
                }
            }
        }
    }

    // 1ステップの最左簡約
    pub fn step(&self) -> Self {
        match *self {
            AST::Var(_) => self.clone(),
            AST::App(ref e1, ref e2) => {
                if let AST::Abs(ref v, ref e) = **e1 {
                    e.assign(e2, v)
                } else {
                    if e1.reductive() {
                        AST::App(Box::new(e1.step()), Box::new(*e2.clone()))
                    } else {
                        AST::App(Box::new(*e1.clone()), Box::new(e2.step()))
                    }
                }
            }
            AST::Abs(ref v, ref e) => AST::Abs(v.to_string(), Box::new(e.step())),
        }
    }

    pub fn reductive(&self) -> bool {
        match *self {
            AST::Var(_) => false,
            AST::App(ref e1, ref e2) => {
                if let AST::Abs(_, _) = **e1 {
                    true
                } else {
                    e1.reductive() | e2.reductive()
                }
            }
            AST::Abs(_, ref e) => e.reductive(),
        }
    }
    pub fn beta_reduction(&self, limitsec: u64) -> Option<Self> {
        let mut e = self.step();

        let instant = Instant::now();
        let limitsec = Duration::from_secs(limitsec);
        loop {
            if !e.reductive() {
                return Some(e);
            } else if instant.elapsed() >= limitsec {
                return None;
            }
            e = e.step();
        }
    }
}
