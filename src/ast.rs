use std::collections::HashMap;

#[derive(Hash, PartialEq, Debug)]
struct Contract {
    state: Vec<StateVariable>,
    functions: Vec<Function>,
}

#[derive(Hash, PartialEq, Debug)]
struct StateVariable {
    name: String,
    _type: Type,
}

#[derive(Hash, PartialEq, Debug)]
struct Function {
    name: String,
    cases: Vec<Case>,
    recursive: bool,
    signature: Signature,
}

#[derive(Hash, PartialEq, Debug)]
struct Case {
    parameters: Vec<Variable>, // TODO implement patterns
    expressions: Vec<BooleanExpression>,
    return_values: Vec<Variable>,
}

#[derive(Hash, PartialEq, Debug, Clone)]
enum BooleanExpression {
    Identifier(String),

    EqBool(Box<BooleanExpression>, Box<BooleanExpression>),
    NeBool(Box<BooleanExpression>, Box<BooleanExpression>),
    EqUint(Box<UintExpression>, Box<UintExpression>),
    NeUint(Box<UintExpression>, Box<UintExpression>),
    Lt(Box<UintExpression>, Box<UintExpression>),
    Gt(Box<UintExpression>, Box<UintExpression>),
    Le(Box<UintExpression>, Box<UintExpression>),
    Ge(Box<UintExpression>, Box<UintExpression>),

    And(Box<BooleanExpression>, Box<BooleanExpression>),
    Or(Box<BooleanExpression>, Box<BooleanExpression>),
    Not(Box<BooleanExpression>),
    Ite(
        Box<BooleanExpression>,
        Box<BooleanExpression>,
        Box<BooleanExpression>,
    ),

    FunctionCall(String, Vec<Expression>),
}

#[derive(Hash, PartialEq, Debug, Clone)]
enum Expression {
    Boolean(BooleanExpression),
    Uint(UintExpression),
}

#[derive(Hash, PartialEq, Debug, Clone)]
enum UintExpression {
    Identifier(String),

    Plus(Box<UintExpression>, Box<UintExpression>),
    Minus(Box<UintExpression>, Box<UintExpression>),
    Times(Box<UintExpression>, Box<UintExpression>),
    Div(Box<UintExpression>, Box<UintExpression>),

    Ite(
        Box<BooleanExpression>,
        Box<UintExpression>,
        Box<UintExpression>,
    ),

    Select(Box<ArrayExpression>, Box<UintExpression>),
}

#[derive(Hash, PartialEq, Debug, Clone)]
enum ArrayExpression {
    Identifier(String),
    Store(
        Box<ArrayExpression>,
        Box<UintExpression>,
        Box<UintExpression>,
    ),
}

#[derive(Hash, PartialEq, Debug)]
struct Variable {
    name: String,
    _type: Type,
}

#[derive(Hash, PartialEq, Debug, Clone)]
enum Type {
    Unknown,
    Uint,
    Array,
    Bool,
    List,
}

#[derive(Hash, PartialEq, Debug)]
struct Signature {
    inputs: Vec<Type>,
    outputs: Vec<Type>,
}

// impl<'a> From<Pair<'a, Rule>> for Parameter {
// 	fn from(pair: Pair<Rule>) -> Function {
// 		let mut name = String::new();
// 		let mut _type = Type::Bool;

// 		for p in pair.into_inner() {
// 			match p.as_rule() {
// 				Rule::identifier => {
// 					name = p.as_str().to_string();
// 				},

// 				c => panic!("{:?}", c)
// 			}
// 		}
// 	}
// }

impl<'a> Type {
    fn from(pair: Pair<Rule>) -> Type {
        match pair.as_rule() {
            Rule::logikon_type => match pair.as_str() {
                "Uint" => Type::Uint,
                "Bool" => Type::Bool,
                "List" => Type::List,
                "Array" => Type::Array,
                _ => panic!("die"),
            },
            _ => panic!("not a type"),
        }
    }
}

impl<'a> Function {
    fn from(pair: Pair<Rule>) -> Function {
        let mut name = String::new();
        let mut cases: Vec<Case> = vec![];
        let mut recursive = false;
        let mut inputs = vec![];
        let mut outputs = vec![];

        let mut parsed_input_types = false;

        for p in pair.into_inner() {
            match p.as_rule() {
                Rule::identifier => {
                    name = p.as_str().to_string();
                }
                Rule::case => {
                    cases.push(Case::from(p, HashMap::new()));
                }
                Rule::recursive => {
                    recursive = true;
                }
                Rule::type_list => {
                    if parsed_input_types {
                        for t in p.into_inner() {
                            outputs.push(Type::from(t))
                        }
                    } else {
                        for t in p.into_inner() {
                            inputs.push(Type::from(t))
                        }
                        parsed_input_types = true;
                    }
                }
                c => panic!("{:?}", c),
            }
        }

        let cases = cases
            .iter()
            .map(|c| Case {
                parameters: c
                    .parameters
                    .iter()
                    .enumerate()
                    .map(|(index, v)| Variable {
                        _type: inputs[index].clone(),
                        name: v.name.clone(),
                    }).collect(),
                return_values: c
                    .return_values
                    .iter()
                    .enumerate()
                    .map(|(index, v)| Variable {
                        _type: outputs[index].clone(),
                        name: v.name.clone(),
                    }).collect(),
                expressions: c.expressions.clone(),
            }).collect();

        Function {
            name,
            recursive,
            cases,
            signature: Signature { inputs, outputs },
        }
    }
}

impl<'a> Variable {
    fn from(pair: Pair<Rule>) -> Variable {
        Variable {
            name: String::from(pair.as_str()),
            _type: Type::Unknown,
        }
    }
}

impl<'a> UintExpression {
    fn from(pair: Pair<Rule>) -> UintExpression {
        let mut arg_count = 0;

        let mut token_iter = pair.into_inner();

        let op = token_iter.next().unwrap();

        match op.as_rule() {
            Rule::identifier => UintExpression::Identifier(String::from(op.as_str())),
            Rule::statement => UintExpression::from(op),
            Rule::function_identifier => panic!(""),
            Rule::binary_op => {
                let argument1 = token_iter.next().unwrap();
                let argument2 = token_iter.next().unwrap();

                match op.as_str() {
                    "+" => UintExpression::Plus(
                        Box::new(UintExpression::from(argument1)),
                        Box::new(UintExpression::from(argument2)),
                    ),
                    "-" => UintExpression::Minus(
                        Box::new(UintExpression::from(argument1)),
                        Box::new(UintExpression::from(argument2)),
                    ),
                    "*" => UintExpression::Times(
                        Box::new(UintExpression::from(argument1)),
                        Box::new(UintExpression::from(argument2)),
                    ),
                    "/" => UintExpression::Div(
                        Box::new(UintExpression::from(argument1)),
                        Box::new(UintExpression::from(argument2)),
                    ),
                    "select" => UintExpression::Select(
                        Box::new(ArrayExpression::from(argument1)),
                        Box::new(UintExpression::from(argument2)),
                    ),
                    _ => panic!(""),
                }
            }
            Rule::ternary_op => {
                let argument1 = token_iter.next().unwrap();
                let argument2 = token_iter.next().unwrap();
                let argument3 = token_iter.next().unwrap();

                match op.as_str() {
                    "ite" => UintExpression::Ite(
                        Box::new(BooleanExpression::from(argument1)),
                        Box::new(UintExpression::from(argument2)),
                        Box::new(UintExpression::from(argument3)),
                    ),
                    _ => panic!(""),
                }
            }
            _ => panic!(""),
        }
    }
}

impl<'a> Expression {
    fn from(pair: Pair<Rule>) -> Expression {
        Expression::Boolean(BooleanExpression::Identifier(String::from("lol")))
    }
}

impl<'a> ArrayExpression {
    fn from(pair: Pair<Rule>) -> ArrayExpression {
        let mut token_iter = pair.into_inner();

        let op = token_iter.next().unwrap();

        match op.as_rule() {
            Rule::function_identifier => panic!(""),
            Rule::ternary_op => {
                let argument1 = token_iter.next().unwrap();
                let argument2 = token_iter.next().unwrap();
                let argument3 = token_iter.next().unwrap();

                match op.as_str() {
                    "store" => ArrayExpression::Store(
                        Box::new(ArrayExpression::from(argument1)),
                        Box::new(UintExpression::from(argument2)),
                        Box::new(UintExpression::from(argument3)),
                    ),
                    _ => panic!(""),
                }
            }
            _ => panic!(""),
        }
    }
}

impl<'a> BooleanExpression {
    fn from(pair: Pair<Rule>) -> BooleanExpression {
        let mut token_iter = pair.into_inner();

        let op = token_iter.next().unwrap();

        match op.as_rule() {
            Rule::statement => BooleanExpression::from(op),
            Rule::identifier => BooleanExpression::Identifier(String::from(op.as_str())),
            Rule::function_identifier => panic!("NO FUNCTIONS"),
            Rule::unary_op => {
                let argument = token_iter.next().unwrap();
                match op.as_str() {
                    "prove" => panic!("NO PROVE"),
                    "not" => BooleanExpression::Not(Box::new(BooleanExpression::from(argument))),
                    _ => panic!(""),
                }
            }
            Rule::binary_op => {
                let argument1 = token_iter.next().unwrap();
                let argument2 = token_iter.next().unwrap();

                match op.as_str() {
                    "<=" => BooleanExpression::Le(
                        Box::new(UintExpression::from(argument1)),
                        Box::new(UintExpression::from(argument2)),
                    ),
                    ">=" => BooleanExpression::Ge(
                        Box::new(UintExpression::from(argument1)),
                        Box::new(UintExpression::from(argument2)),
                    ),
                    "=" => BooleanExpression::EqBool(
                        Box::new(BooleanExpression::from(argument1)),
                        Box::new(BooleanExpression::from(argument2)),
                    ),
                    "!=" => BooleanExpression::NeBool(
                        Box::new(BooleanExpression::from(argument1)),
                        Box::new(BooleanExpression::from(argument2)),
                    ),
                    ">" => BooleanExpression::Gt(
                        Box::new(UintExpression::from(argument1)),
                        Box::new(UintExpression::from(argument2)),
                    ),
                    "<" => BooleanExpression::Lt(
                        Box::new(UintExpression::from(argument1)),
                        Box::new(UintExpression::from(argument2)),
                    ),
                    _ => panic!(""),
                }
            }
            Rule::ternary_op => {
                let argument1 = token_iter.next().unwrap();
                let argument2 = token_iter.next().unwrap();
                let argument3 = token_iter.next().unwrap();

                match op.as_str() {
                    "ite" => BooleanExpression::Ite(
                        Box::new(BooleanExpression::from(argument1)),
                        Box::new(BooleanExpression::from(argument2)),
                        Box::new(BooleanExpression::from(argument3)),
                    ),
                    _ => panic!(""),
                }
            }
            e => panic!("{:?}", e),
        }
    }
}

impl<'a> Case {
    fn from(pair: Pair<Rule>, symbols: HashMap<String, Type>) -> Case {
        let mut parameters = vec![];
        let mut return_values = vec![];
        let mut expressions = vec![];

        let mut parsed_parameters = false;

        for p in pair.into_inner() {
            match p.as_rule() {
                Rule::parameter_list => {
                    if parsed_parameters {
                        for t in p.into_inner() {
                            return_values.push(Variable::from(t))
                        }
                    } else {
                        for t in p.into_inner() {
                            parameters.push(Variable::from(t))
                        }
                        parsed_parameters = true;
                    }
                }
                Rule::case_body => {
                    for t in p.into_inner() {
                        expressions.push(BooleanExpression::from(t));
                    }
                }
                c => panic!("{:?}", c),
            }
        }

        Case {
            parameters,
            expressions,
            return_values,
        }
    }
}

use pest::iterators::Pair;
use pest::Parser;

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("logikon.pest"); // relative to this file

#[derive(Parser)]
#[grammar = "logikon.pest"]
struct ContractParser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_function() {
        let source = r#"define f (Uint) -> (Uint Uint) 
		case () ().
	"#;

        let mut pairs = ContractParser::parse(Rule::function_def, &source).unwrap();

        let pair = pairs.next().unwrap();

        assert_eq!(
            Function::from(pair),
            Function {
                name: String::from("f"),
                recursive: false,
                cases: vec![Case {
                    parameters: vec![],
                    expressions: vec![],
                    return_values: vec![]
                }],
                signature: Signature {
                    inputs: vec![Type::Uint],
                    outputs: vec![Type::Uint, Type::Uint]
                }
            }
        );
    }

    #[test]
    fn identity_function() {
        let source = r#"define f (Bool) -> (Bool) 
        case (a) (x) :-
            (= x y).
    "#;

        let mut pairs = ContractParser::parse(Rule::function_def, &source).unwrap();

        let pair = pairs.next().unwrap();

        assert_eq!(
            Function::from(pair),
            Function {
                name: String::from("f"),
                recursive: false,
                cases: vec![Case {
                    parameters: vec![Variable {
                        name: String::from("a"),
                        _type: Type::Bool
                    }],
                    expressions: vec![BooleanExpression::EqBool(
                        Box::new(BooleanExpression::Identifier(String::from("x"))),
                        Box::new(BooleanExpression::Identifier(String::from("y")))
                    )],
                    return_values: vec![Variable {
                        name: String::from("x"),
                        _type: Type::Bool
                    }]
                }],
                signature: Signature {
                    inputs: vec![Type::Bool],
                    outputs: vec![Type::Bool]
                }
            }
        );
    }

    #[test]
    fn ite_function() {
        let source = r#"define f (Bool) -> (Bool) 
        case (a) (x) :-
            (= x (ite a a a)).
    "#;

        let mut pairs = ContractParser::parse(Rule::function_def, &source).unwrap();

        let pair = pairs.next().unwrap();

        assert_eq!(
            Function::from(pair),
            Function {
                name: String::from("f"),
                recursive: false,
                cases: vec![Case {
                    parameters: vec![Variable {
                        name: String::from("a"),
                        _type: Type::Bool
                    }],
                    expressions: vec![BooleanExpression::EqBool(
                        Box::new(BooleanExpression::Identifier(String::from("x"))),
                        Box::new(BooleanExpression::Ite(
                            Box::new(BooleanExpression::Identifier(String::from("a"))),
                            Box::new(BooleanExpression::Identifier(String::from("a"))),
                            Box::new(BooleanExpression::Identifier(String::from("a"))),
                        ))
                    )],
                    return_values: vec![Variable {
                        name: String::from("x"),
                        _type: Type::Bool
                    }]
                }],
                signature: Signature {
                    inputs: vec![Type::Bool],
                    outputs: vec![Type::Bool]
                }
            }
        );
    }

    #[test]
    fn ite_uint_function() {
        let source = r#"define f (Uint Bool) -> (Uint)
        case (a b) (x) :-
            (= x (ite b a a)).
    "#;

        let mut pairs = ContractParser::parse(Rule::function_def, &source).unwrap();

        let pair = pairs.next().unwrap();

        assert_eq!(
            Function::from(pair),
            Function {
                name: String::from("f"),
                recursive: false,
                cases: vec![Case {
                    parameters: vec![
                        Variable {
                            name: String::from("a"),
                            _type: Type::Uint
                        },
                        Variable {
                            name: String::from("b"),
                            _type: Type::Bool
                        }
                    ],
                    expressions: vec![BooleanExpression::EqUint(
                        Box::new(UintExpression::Identifier(String::from("x"))),
                        Box::new(UintExpression::Ite(
                            Box::new(BooleanExpression::Identifier(String::from("b"))),
                            Box::new(UintExpression::Identifier(String::from("a"))),
                            Box::new(UintExpression::Identifier(String::from("a"))),
                        ))
                    )],
                    return_values: vec![Variable {
                        name: String::from("x"),
                        _type: Type::Uint
                    }]
                }],
                signature: Signature {
                    inputs: vec![Type::Uint, Type::Bool],
                    outputs: vec![Type::Uint]
                }
            }
        );
    }
}
