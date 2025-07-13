use crate::expr::eval_as_usize;
use syn::{parse::Parse, punctuated::Punctuated, spanned::Spanned};

#[allow(unused)]
pub enum Pattern {
    Atom(PatternAtom),
    Seq(PatternSeq),
    Join(Box<PatternJoin>),
    Or(Box<PatternOr>),
    Repeat(Box<PatternRepeat>),
    Collect(Box<PatternCollect>),
}

impl Pattern {
    pub fn new(expr: &syn::Expr) -> Result<Self, syn::Error> {
        let p = match expr {
            syn::Expr::Lit(e) => {
                use syn::Lit::*;

                match &e.lit {
                    Str(l) => Pattern::Seq(PatternSeq::from_str(l)?),
                    ByteStr(l) => Pattern::Seq(PatternSeq::from_bstr(l)?),
                    Byte(l) => Pattern::Atom(PatternAtom::Byte(l.clone())),
                    Char(l) => Pattern::Atom(PatternAtom::Char(l.clone())),
                    Int(l) => Pattern::Atom(PatternAtom::Int(l.clone())),
                    Float(l) => Pattern::Atom(PatternAtom::Float(l.clone())),
                    Bool(l) => Pattern::Atom(PatternAtom::Bool(l.clone())),
                    _ => {
                        return Err(syn::Error::new(e.span(), "Unexpected literal."));
                    }
                }
            }
            syn::Expr::Path(e) => Pattern::Atom(PatternAtom::Class(e.path.clone())),
            syn::Expr::Range(e) => Pattern::Atom(PatternAtom::Range(e.clone())),
            syn::Expr::Binary(e) => {
                let lhs = Pattern::new(&e.left)?;
                let rhs = Pattern::new(&e.right)?;
                match &e.op {
                    syn::BinOp::Add(_) => Pattern::Join(Box::new(PatternJoin { lhs, rhs })),
                    syn::BinOp::BitOr(_) => Pattern::Or(Box::new(PatternOr { lhs, rhs })),
                    _ => {
                        return Err(syn::Error::new(
                            e.span(),
                            "Unexpected operator. `+` or `|` was expected.",
                        ));
                    }
                }
            }
            syn::Expr::Array(e) => Pattern::Seq(PatternSeq::from_array(e)?),
            syn::Expr::Macro(e) => {
                let ident = e.mac.path.require_ident()?;

                if ident == "repeat" {
                    PatternRepeat::from_mac(&e.mac)
                        .map(Box::new)
                        .map(Pattern::Repeat)?
                } else if ident == "collect" {
                    PatternCollect::from_mac(&e.mac)
                        .map(Box::new)
                        .map(Pattern::Collect)?
                } else {
                    return Err(syn::Error::new(e.span(), "Unexpected pattern."));
                }
            }
            syn::Expr::Paren(e) => Pattern::new(&e.expr)?,
            syn::Expr::Repeat(e) => PatternRepeat::from_repeat(e)
                .map(Box::new)
                .map(Pattern::Repeat)?,
            _ => return Err(syn::Error::new(expr.span(), "Unexpected expression.")),
        };

        Ok(p)
    }
}

// PatternAtom ::= char | num
pub enum PatternAtom {
    Byte(syn::LitByte),
    Char(syn::LitChar),
    Int(syn::LitInt),
    Float(syn::LitFloat),
    Bool(syn::LitBool),
    Range(syn::ExprRange),
    Class(syn::Path),
}

// PatternSeq ::= array | bstr | str
pub struct PatternSeq {
    pub atoms: Vec<PatternAtom>,
}

impl PatternSeq {
    fn from_str(str: &syn::LitStr) -> syn::Result<Self> {
        let atoms: Vec<PatternAtom> = str
            .value()
            .chars()
            .map(|e| -> syn::LitChar { syn::parse_quote!(#e) })
            .map(|e| PatternAtom::Char(e))
            .collect();

        if atoms.is_empty() {
            Err(syn::Error::new(
                str.span(),
                "Sequence pattern must not be empty.",
            ))
        } else {
            Ok(PatternSeq { atoms })
        }
    }

    fn from_bstr(str: &syn::LitByteStr) -> syn::Result<Self> {
        let atoms: Vec<PatternAtom> = str
            .value()
            .iter()
            .map(|e| -> syn::LitInt { syn::parse_quote!(#e) })
            .map(|e| PatternAtom::Int(e))
            .collect();

        if atoms.is_empty() {
            Err(syn::Error::new(
                str.span(),
                "Sequence pattern must not be empty.",
            ))
        } else {
            Ok(PatternSeq { atoms })
        }
    }

    fn from_array(arr: &syn::ExprArray) -> syn::Result<Self> {
        use syn::Lit::*;
        let atoms: Result<Vec<_>, _> = arr
            .elems
            .iter()
            .map(|e| {
                let syn::Expr::Lit(e) = e else {
                    return Err(syn::Error::new(
                        e.span(),
                        "Array pattern element must be atom literal.",
                    ));
                };

                let p = match &e.lit {
                    Byte(l) => PatternAtom::Byte(l.clone()),
                    Char(l) => PatternAtom::Char(l.clone()),
                    Int(l) => PatternAtom::Int(l.clone()),
                    Float(l) => PatternAtom::Float(l.clone()),
                    Bool(l) => PatternAtom::Bool(l.clone()),
                    _ => {
                        return Err(syn::Error::new(
                            e.span(),
                            "Array pattern element must be atom literal.",
                        ));
                    }
                };
                Ok(p)
            })
            .collect();

        let atoms = atoms?;

        if atoms.is_empty() {
            Err(syn::Error::new(
                arr.span(),
                "Sequence pattern must not be empty.",
            ))
        } else {
            Ok(PatternSeq { atoms })
        }
    }
}

// PatternJoin ::= pattern + "+" +  pattern
pub struct PatternJoin {
    pub lhs: Pattern,
    pub rhs: Pattern,
}

// PatternOr ::= pattern + "|" + pattern
pub struct PatternOr {
    pub lhs: Pattern,
    pub rhs: Pattern,
}

// PatternRepeat ::=  "[" + pattern + ";" + range "]" "repeat!(" + pattern ")"  | "repeat!(" + pattern + "," + range + ")"
pub struct PatternRepeat {
    pub pattern: Pattern,
    pub range: AnyRange,
}

#[derive(Debug, Clone)]
pub struct AnyRange {
    pub start: Option<usize>,
    pub end: Option<usize>,
}

impl TryFrom<&syn::Expr> for AnyRange {
    type Error = syn::Error;

    fn try_from(value: &syn::Expr) -> Result<Self, Self::Error> {
        match value {
            syn::Expr::Range(e) => {
                let start = e.start.as_ref().map(|e| eval_as_usize(&e)).transpose()?;
                let end = e.end.as_ref().map(|e| eval_as_usize(&e)).transpose()?;

                Ok(Self { start, end })
            }
            e => {
                let count = eval_as_usize(e)?;
                Ok(Self {
                    start: Some(count),
                    end: Some(count),
                })
            }
        }
    }
}

impl PatternRepeat {
    fn from_repeat(e: &syn::ExprRepeat) -> syn::Result<Self> {
        Ok(Self {
            pattern: Pattern::new(&e.expr)?,
            range: AnyRange::try_from(e.len.as_ref())?,
        })
    }

    fn from_mac(mac: &syn::Macro) -> syn::Result<Self> {
        let e = mac.parse_body_with(Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated)?;

        let r = match e.len() {
            1 => {
                let pattern = Pattern::new(&e[0])?;
                PatternRepeat {
                    pattern,
                    range: AnyRange {
                        start: None,
                        end: None,
                    },
                }
            }
            2 => {
                let pattern = Pattern::new(&e[0])?;
                let syn::Expr::Range(range) = &e[1] else {
                    return Err(syn::Error::new(e[1].span(), "Range literal was expected."));
                };
                let start = range.start.as_ref().map(|e| eval_as_usize(e)).transpose()?;
                let end = range.end.as_ref().map(|e| eval_as_usize(e)).transpose()?;

                PatternRepeat {
                    pattern,
                    range: AnyRange { start, end },
                }
            }
            _ => {
                return Err(syn::Error::new(
                    e.span(),
                    "One or two arguments were expected.",
                ));
            }
        };

        Ok(r)
    }
}

// PatternCollect ::= "collect!(" + path  + "," + pattern ")"
pub struct PatternCollect {
    pub path: syn::Path,
    pub pattern: Pattern,
}

impl PatternCollect {
    fn from_mac(mac: &syn::Macro) -> syn::Result<Self> {
        let e = mac.parse_body_with(parser_fn(move |input| {
            let path = syn::Path::parse(input)?;
            let _ = <syn::Token![,]>::parse(input)?;
            let e = syn::Expr::parse(input)?;
            let pattern = Pattern::new(&e)?;

            if !input.is_empty() {
                return Err(input.error("Unexpected arguments of `collect!`."));
            }

            Ok(PatternCollect { path, pattern })
        }))?;

        Ok(e)
    }
}

fn parser_fn<F, T>(f: F) -> F
where
    F: for<'a> FnOnce(&'a syn::parse::ParseBuffer<'a>) -> T,
{
    f
}
