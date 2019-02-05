use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct HtmlParser;

type Pair<'a> = pest::iterators::Pair<'a, Rule>;

type IsHash = bool;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Expression<'a> {
    Str(&'a str),
    Block(&'a str, IsHash),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Arg<'a> {
    pub name: &'a str,
    pub value: Option<Expression<'a>>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Tag<'a> {
    pub name: &'a str,
    pub args: Vec<Arg<'a>>,
    pub children: Vec<Node<'a>>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Node<'a> {
    Expression(Expression<'a>),
    Tag(Tag<'a>),
    List(Vec<Node<'a>>),
}

impl<'a> Node<'a> {
    fn empty() -> Self {
        Node::List(Vec::new())
    }
}

fn parse_expression<'a>(pair: Pair<'a>) -> Expression<'a> {
    match pair.as_rule() {
        Rule::block_regular | Rule::block_hashed => {
            let raw = pair.as_str();
            let hashed = pair.as_rule() == Rule::block_hashed && raw.starts_with('#');
            let content = if hashed {
                &raw[1..]
            } else {
                raw
            };
            Expression::Block(content, hashed)
        },
        Rule::literal_string => {
            let s = pair.as_str();
            Expression::Str(&s[1..s.len() - 1])
        }
        rule => {
            panic!("Invalid rule in <arg>: {:?}", rule);
        }
    }
}

fn parse_arg<'a>(pair: Pair<'a>) -> Arg<'a> {
    assert_eq!(pair.as_rule(), Rule::arg);

    let mut inner = pair.into_inner();

    // Name.
    let name_pair = inner.next().unwrap();
    assert_eq!(name_pair.as_rule(), Rule::word);
    let name = name_pair.as_str();

    // Value.
    let value = inner.next().map(parse_expression);

    Arg { name, value }
}

fn parse_args_opt<'a>(pair: Pair<'a>) -> Vec<Arg<'a>> {
    assert_eq!(pair.as_rule(), Rule::args_opt);

    if let Some(args_pair) = pair.into_inner().next() {
        assert_eq!(args_pair.as_rule(), Rule::args);
        args_pair.into_inner().map(parse_arg).collect()
    } else {
        Vec::new()
    }
}

fn parse_tagname<'a>(pair: Pair<'a>) -> &'a str {
    match pair.as_rule() {
        Rule::tagname_regular => pair.as_str(),
        Rule::tagname_component => pair.as_str(),
        rule => {
            panic!("Invalid rule for tagname: {:?}", rule);
        }
    }
}

fn parse_tag<'a>(pair: Pair<'a>) -> Tag<'a> {
    assert_eq!(pair.as_rule(), Rule::tag);
    let inner = pair.into_inner().next().unwrap();

    let is_selfclosing = match inner.as_rule() {
        Rule::tag_short => true,
        Rule::tag_full => false,
        rule => {
            panic!("Invalid tag rule: {:?}", rule);
        }
    };

    let mut tag_inner = inner.into_inner();

    let tagname = tag_inner.next().unwrap();
    let name = parse_tagname(tagname);

    let args = parse_args_opt(tag_inner.next().unwrap());

    let children = if is_selfclosing {
        Vec::new()
    } else {
        parse_nodes_opt(tag_inner.next().unwrap())
    };

    Tag {
        name,
        args,
        children,
    }
}

fn parse_node<'a>(pair: Pair<'a>) -> Node<'a> {
    assert_eq!(pair.as_rule(), Rule::node);
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::tag => Node::Tag(parse_tag(inner)),
        Rule::block_hashed | Rule::literal_string => Node::Expression(parse_expression(inner)),
        Rule::literal => Node::Expression(Expression::Str(inner.as_str().trim())),
        rule => {
            panic!("Invalid rule for node: {:?}", rule);
        }
    }
}

pub fn parse_nodes_opt<'a>(pair: Pair<'a>) -> Vec<Node<'a>> {
    assert_eq!(pair.as_rule(), Rule::nodes_opt);
    pair.into_inner().map(parse_node).collect::<Vec<_>>()
}

pub fn parse_doc<'a>(input: &'a str) -> Result<Node<'a>, pest::error::Error<Rule>> {
    let pairs = HtmlParser::parse(Rule::html, input)?;

    let nodes_pair = pairs.into_iter().next().unwrap();
    assert_eq!(nodes_pair.as_rule(), Rule::nodes);

    let mut nodes = nodes_pair.into_inner().map(parse_node).collect::<Vec<_>>();
    let node = if nodes.len() > 1 {
        Node::List(nodes)
    } else {
        nodes.pop().unwrap()
    };
    Ok(node)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_args() {
        fn parse<'a>(input: &'a str) -> Vec<Arg<'a>> {
            let pairs = HtmlParser::parse(Rule::args_opt, input).unwrap();
            parse_args_opt(pairs.into_iter().next().unwrap())
        };

        let arg_a = Arg {
            name: "a",
            value: None,
        };

        let arg_b = Arg {
            name: "b",
            value: Some(Expression::Str("lit")),
        };

        let arg_c = Arg {
            name: "c",
            value: Some(Expression::Block("{true}", false)),
        };

        assert_eq!(
            parse("a b=\"lit\" c={true}"),
            vec![arg_a.clone(), arg_b.clone(), arg_c.clone(),]
        );
        assert_eq!(
            parse("c={true} b=\"lit\" a"),
            vec![arg_c.clone(), arg_b.clone(), arg_a.clone(),]
        );
    }

    #[test]
    fn test_parse_tag_selfclosing_no_args() {
        let ast = parse_doc(
            r#"
            <div />
        "#,
        )
        .unwrap();

        assert_eq!(
            ast,
            Node::Tag(Tag {
                name: "div",
                args: Vec::new(),
                children: Vec::new(),
            })
        )
    }

    #[test]
    fn test_parse_tag_selfclosing_with_args() {
        let ast = parse_doc(
            r#"
            <div bool lit="x" />
        "#,
        )
        .unwrap();

        assert_eq!(
            ast,
            Node::Tag(Tag {
                name: "div",
                args: vec![
                    Arg {
                        name: "bool",
                        value: None,
                    },
                    Arg {
                        name: "lit",
                        value: Some(Expression::Str("x")),
                    }
                ],
                children: Vec::new(),
            })
        )
    }

    #[test]
    fn test_parse_tag_empty_no_args() {
        let ast = parse_doc("<div></div>").unwrap();
        assert_eq!(
            ast,
            Node::Tag(Tag {
                name: "div",
                args: vec![],
                children: Vec::new(),
            })
        );
    }

    #[test]
    fn test_parse_tag_empty_with_args() {
        let ast = parse_doc("<div a={true} b></div>").unwrap();
        assert_eq!(
            ast,
            Node::Tag(Tag {
                name: "div",
                args: vec![
                    Arg {
                        name: "a",
                        value: Some(Expression::Block("{true}", false)),
                    },
                    Arg {
                        name: "b",
                        value: None,
                    }
                ],
                children: Vec::new(),
            })
        );
    }

    #[test]
    fn test_parse_tag_with_children() {
        let ast = parse_doc(
            r#"
            <div a={true} b>
                "hello"
                <br />
                <ul>
                    <li>inner</li>
                    <li>#{multi}</li>
                </ul>
            </div>
        "#,
        )
        .unwrap();
        assert_eq!(
            ast,
            Node::Tag(Tag {
                name: "div",
                args: vec![
                    Arg {
                        name: "a",
                        value: Some(Expression::Block("{true}", false)),
                    },
                    Arg {
                        name: "b",
                        value: None,
                    }
                ],
                children: vec![
                    Node::Expression(Expression::Str("hello")),
                    Node::Tag(Tag {
                        name: "br",
                        args: Vec::new(),
                        children: Vec::new(),
                    }),
                    Node::Tag(Tag {
                        name: "ul",
                        args: Vec::new(),
                        children: vec![
                            Node::Tag(Tag {
                                name: "li",
                                args: Vec::new(),
                                children: vec![Node::Expression(Expression::Str("inner"))],
                            }),
                            Node::Tag(Tag {
                                name: "li",
                                args: Vec::new(),
                                children: vec![Node::Expression(Expression::Block("{multi}", true))],
                            })
                        ],
                    })
                ],
            })
        );
    }
}
