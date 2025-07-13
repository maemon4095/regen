use crate::pattern::{
    Pattern, PatternAtom, PatternCollect, PatternJoin, PatternOr, PatternRepeat, PatternSeq,
};
use proc_macro2::{Ident, Span};

pub struct MatchTree {
    states: Vec<MatchState>,
}

impl MatchTree {
    pub fn new() -> Self {
        Self { states: Vec::new() }
    }

    #[allow(unused)]
    pub fn add(&mut self, span: Span, pattern: Pattern, variant: syn::Variant) -> syn::Result<()> {
        if self.states.is_empty() {
            self.states.push(Default::default());
        }
        let states = self.insert(0, &pattern);
        for s in states {
            self.associate(s, span, variant.clone())?;
        }

        Ok(())
    }

    fn insert(&mut self, start_state: usize, pattern: &Pattern) -> Vec<usize> {
        match pattern {
            Pattern::Atom(p) => vec![self.insert_atom(start_state, p)],
            Pattern::Seq(p) => vec![self.insert_seq(start_state, p)],
            Pattern::Join(p) => self.insert_join(start_state, p),
            Pattern::Or(p) => self.insert_or(start_state, p),
            Pattern::Repeat(p) => self.insert_repeat(start_state, p),
            Pattern::Collect(p) => self.insert_collect(start_state, p),
        }
    }

    fn insert_atom(&mut self, state: usize, pattern: &PatternAtom) -> usize {
        let pattern = MatchPattern::from(pattern);
        if let Some(b) = self.states[state]
            .branches
            .iter_mut()
            .find(|b| b.pattern == pattern)
        {
            return b.next;
        }

        let next = self.states.len();
        self.states.push(MatchState::default());
        self.states[state]
            .branches
            .push(MatchBranch { pattern, next });

        next
    }

    fn insert_seq(&mut self, mut state: usize, pattern: &PatternSeq) -> usize {
        let (last_atom, atoms) = pattern.atoms.split_last().unwrap();

        for atom in atoms {
            state = self.insert_atom(state, atom);
        }

        self.insert_atom(state, last_atom)
    }

    fn insert_join(&mut self, state: usize, pattern: &PatternJoin) -> Vec<usize> {
        let mut states = Vec::new();
        for s in self.insert(state, &pattern.lhs) {
            states.append(&mut self.insert(s, &pattern.rhs));
        }
        states
    }

    fn insert_or(&mut self, state: usize, pattern: &PatternOr) -> Vec<usize> {
        let state_lhs = self.insert(state, &pattern.lhs);
        let state_rhs = self.insert(state, &pattern.rhs);
        concat_vec(state_lhs, state_rhs)
    }

    fn insert_repeat_n(&mut self, state: usize, pattern: &Pattern, count: usize) -> Vec<usize> {
        let mut states = vec![state];

        for _ in 0..count {
            states = states
                .iter()
                .flat_map(|s| self.insert(*s, pattern))
                .collect();
        }

        states
    }

    fn insert_repeat(&mut self, state: usize, pattern: &PatternRepeat) -> Vec<usize> {
        let range = &pattern.range;
        let pattern = &pattern.pattern;

        let start = range.start.as_ref().copied();
        let end = range.end.as_ref().copied();

        let states;
        if let Some(start) = start {
            states = self.insert_repeat_n(state, pattern, start);
        } else {
            states = vec![state];
        }

        match end {
            Some(end) => todo!(),
            None => {
                for &s in states.iter() {
                    // すでにつながっているステートがある場合、意図しないマッチが起きるため単にループはできない。何もつながっていないステートまですすめるか、新たに伸ばす必要がある。
                    // 逆向きのノードをつなげるか。
                    // a(bc)+
                    // 0 -[a]-> 1 -[b]-> 2 -[c]-> 3
                    //          + <-[c]- 4 <-[b]- +
                }
            }
        }

        todo!()
    }

    fn insert_collect(&mut self, state: usize, pattern: &PatternCollect) -> Vec<usize> {
        todo!()
    }

    fn associate(&mut self, state: usize, span: Span, variant: syn::Variant) -> syn::Result<()> {
        let assoc = &mut self.states[state].assoc;
        if assoc.is_some() {
            Err(syn::Error::new(span, "Confilicted pattern."))
        } else {
            *assoc = Some(variant);
            Ok(())
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum MatchPattern {
    Byte(syn::LitByte),
    Char(syn::LitChar),
    Int(syn::LitInt),
    Float(syn::LitFloat),
    Bool(syn::LitBool),
    Range(syn::ExprRange),
    Class(syn::Path),
}

impl From<&PatternAtom> for MatchPattern {
    fn from(value: &PatternAtom) -> Self {
        match value {
            PatternAtom::Byte(l) => Self::Byte(l.clone()),
            PatternAtom::Char(l) => Self::Char(l.clone()),
            PatternAtom::Int(l) => Self::Int(l.clone()),
            PatternAtom::Float(l) => Self::Float(l.clone()),
            PatternAtom::Bool(l) => Self::Bool(l.clone()),
            PatternAtom::Range(e) => Self::Range(e.clone()),
            PatternAtom::Class(e) => Self::Class(e.clone()),
        }
    }
}

#[derive(Debug)]
struct MatchBranch {
    pattern: MatchPattern,
    next: usize,
}

#[derive(Debug, Default)]
struct MatchState {
    branches: Vec<MatchBranch>,
    required_states: Vec<(Ident, syn::Type)>,
    assoc: Option<syn::Variant>,
}

pub fn concat_vec<T>(mut lhs: Vec<T>, mut rhs: Vec<T>) -> Vec<T> {
    lhs.append(&mut rhs);
    lhs
}
