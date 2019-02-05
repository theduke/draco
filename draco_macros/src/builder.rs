use super::parser;

fn build_expr<'a>(buffer: &mut String, expr: parser::Expression<'a>) {
    use parser::Expression;
    match expr {
        Expression::Block(block, _is_hashed) => {
            buffer.push_str(block);
        }
        Expression::Str(val) => {
            buffer.push('"');
            buffer.push_str(val);
            buffer.push('"');
        }
    }
}

fn build_node<'a>(buffer: &mut String, node: parser::Node<'a>) {
    use parser::{Expression, Node};

    match node {
        Node::Expression(expr) => {
            buffer.push_str("draco::Node::from(");
            build_expr(buffer, expr);
            buffer.push(')');
        }
        Node::Tag(tag) => {
            buffer.push('{');
            buffer.push_str("let mut node = draco::html::");
            buffer.push_str(tag.name);
            buffer.push_str("();\n");

            for attr in tag.args {
                match (attr.name, &attr.value) {
                    (attr_name, Some(Expression::Block(ref block, _is_hashed))) if attr_name.starts_with("on") => {
                        let event_name = &attr_name[2..];
                        if event_name == "input" || event_name == "checked" {
                            buffer.push_str("node = node.on_");
                            buffer.push_str(event_name);
                            buffer.push('(');
                            buffer.push_str(block);
                            buffer.push_str(");\n");
                        } else {
                            buffer.push_str("node = node.on(\"");
                            buffer.push_str(event_name);
                            buffer.push_str("\",");
                            buffer.push_str(block);
                            buffer.push_str(");\n");
                        }
                    }
                    (attr_name, Some(Expression::Block(ref block, _is_hashed))) if attr_name == "disabled" || attr_name == "checked" => {
                        buffer.push_str("if ");
                        buffer.push_str(block);
                        buffer.push_str(" { node = node.attr(\"");
                        buffer.push_str(attr_name);
                        buffer.push_str("\",\"\")};\n");
                    }
                    (attr_name, None) => {
                        buffer.push_str("node = node.attr(\"");
                        buffer.push_str(attr_name);
                        buffer.push_str("\",");
                        buffer.push_str("\"\"");
                        buffer.push_str(");\n");
                    }
                    (name, Some(ref value)) => {
                        buffer.push_str("node = node.attr(\"");
                        buffer.push_str(name);
                        buffer.push_str("\",");
                        build_expr(buffer, value.clone());
                        buffer.push_str(");\n");
                    }
                }
            }
            buffer.push_str("node");
            for child in tag.children {
                match child {
                    Node::Expression(Expression::Block(code, is_hashed)) if is_hashed => {
                        buffer.push_str(".append(");
                        buffer.push_str(code);
                        buffer.push_str(")\n");
                    }
                    Node::List(items) => {
                        for item in items {
                            buffer.push_str(".push(");
                            build_node(buffer, item);
                            buffer.push_str(")\n");
                        }
                    }
                    other => {
                        buffer.push_str(".push(");
                        build_node(buffer, other);
                        buffer.push_str(")\n");
                    }
                }
            }

            buffer.push_str("}");
        }
        Node::List(items) => {
            buffer.push_str("vec![");
            for item in items {
                build_node(buffer, item);
                buffer.push_str(", ");
            }
            buffer.push_str("]\n");
        }
    }
}

pub fn build<'a>(doc: parser::Node<'a>) -> String {
    let mut buffer = String::new();
    build_node(&mut buffer, doc);
    buffer
}

#[cfg(test)]
mod test {
    use crate::{parser::*, builder::*};

    macro_rules! make_tests {
        {
            $(
                $name:ident {
                    $input:expr =>
                    ( $( $output:expr ),* )
                }
            )*
        } => {
            $(
                #[test]
                fn $name() {
                    let node = parse_doc($input).unwrap();
                    let output = build(node);
                    let expected = &[ $( $output ),* ].join("\n");
                    assert_eq!(&output, expected);
                }
            )*
        };
    }

    make_tests!{
        plain_str { "\"val\"" => ("draco::Node::from(\"val\")") }
        expr { "{true}" => ("draco::Node::from({true})") }
        tag_selfclosing_simple { "<div/>" => ("{let mut node = draco::html::div();\nnode}") }
        tag_simple { "<div></div>" => ("{let mut node = draco::html::div();\nnode}") }

        tag_with_attr_regular { "<div name=\"name\"></div>" => (
            "{let mut node = draco::html::div();",
            "node = node.attr(\"name\",\"name\");",
            "node}"
         ) }
        tag_with_attr_novalue { "<div disabled></div>" => (
            "{let mut node = draco::html::div();",
            "node = node.attr(\"disabled\",\"\");",
            "node}" 
        ) }
        tag_with_attr_disabled_cond { "<div disabled={true}></div>" => (
            "{let mut node = draco::html::div();",
            "if {true} { node = node.attr(\"disabled\",\"\")};",
            "node}" 
        )}
        tag_with_attr_oninput { "<div oninput={true}></div>" => (
            "{let mut node = draco::html::div();",
            "node = node.on_input({true});",
            "node}"
        )}
        tag_with_attr_onchecked { "<div onchecked={true}></div>" => (
            "{let mut node = draco::html::div();",
            "node = node.on_checked({true});",
            "node}" 
        )}
        tag_with_attr_onevent { "<div onclick={true}></div>" => (
            "{let mut node = draco::html::div();",
            "node = node.on(\"click\",{true});",
            "node}" 
        )}
        tag_with_child_str { "<div>hello</div>" => (
            "{let mut node = draco::html::div();",
            "node.push(draco::Node::from(\"hello\"))",
            "}" 
        )}
        tag_with_child_block { "<div>{true}</div>" => (
            "{let mut node = draco::html::div();",
            "node.push(draco::Node::from({true}))",
            "}" 
        )}
        tag_with_child_hashblock { "<div>#{true}</div>" => (
            "{let mut node = draco::html::div();",
            "node.append({true})",
            "}" 
        )}
        tag_with_children { "<div>hello <p>p</p></div>" => (
            "{let mut node = draco::html::div();",
            "node.push(draco::Node::from(\"hello\"))",
            ".push({let mut node = draco::html::p();\nnode.push(draco::Node::from(\"p\"))\n})",
            "}" 
        )}

       
    }
}
