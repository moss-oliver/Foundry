
use proc_macro_hack::proc_macro_hack;
extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro::TokenTree;
use proc_macro::TokenTree::*;

// div>
fn html_end_tag(item_iter: &mut dyn Iterator<Item=TokenTree>, tag: String, output: &mut String) {
    output.push_str(")))");
    let token_get = item_iter.next();
    match token_get {
        Some(Ident(tag_name)) => {
            if tag == tag_name.to_string() {

                let token_get = item_iter.next();
                match token_get {
                    Some(Punct(tag_end)) => {
                        if tag_end.as_char() != '>' {
                            panic!("Unexpected token 1");
                        }
                    }
                    _ => { panic!("Unexpected token 2."); }
                }
            } else {
                panic!("Tags don't match: {}", tag);
            }
        }
        _ => {panic!("Unexpected token 3.");}
    }
}

//"..."</ OR </
fn html_tag_content(item_iter: &mut dyn Iterator<Item=TokenTree>, tag: String, output: &mut String) {
    let mut looping = true;
    let mut first_child = true;
    let mut in_string = false;

    while looping {
        let token_get = item_iter.next();
        match token_get {
            Some(Punct(tag_start)) => {
                if tag_start.as_char() == '<' {
                    if in_string {
                        in_string = false;
                        output.push_str(" \".to_string())");
                    }
                    //Check if closing tag or new tag.
                    let token_get = item_iter.next();
                    match token_get {
                        Some(Punct(tag_end)) => {
                            if tag_end.as_char() == '/' {
                                html_end_tag(item_iter, tag.clone(), output);
                                looping = false;
                            }
                        },
                        Some(Ident(tag_name)) => {
                            if !first_child {
                                output.push_str(",");
                            }
                            html_open_tag(item_iter, tag_name.to_string(), output);
                        }
                        _ => { panic!("Unexpected token 4.") }
                    }
                } else if tag_start.as_char() == '*' {
                    if let Some(Ident(lit)) = item_iter.next() {
                        if !first_child {
                            output.push_str(",");
                        }
                        output.push_str(&lit.to_string());
                    } else {
                        panic!("Unexpected token.")
                    }
                } else {
                    if in_string {
                        output.push_str(&format!("{}", tag_start.as_char()));
                    }
                }
            },
            Some(Ident(lit)) => {
                if in_string == false {
                    in_string = true;

                    if !first_child {
                        output.push_str(",");
                    }
                    output.push_str("Box::new(\"");
                }
                output.push_str(" ");
                output.push_str(&lit.to_string());
            },
            Some(Literal(lit)) => {
                if !first_child {
                    output.push_str(",");
                }
                output.push_str(&format!("Box::new({}.to_string())", lit.to_string()));
            },
            Some(Group(group)) => {
                if !first_child {
                    output.push_str(",");
                }

                /*output.push_str("Box::new(format!(\"{}\", ");
                output.push_str(&group.stream().to_string());
                output.push_str("))");*/
                output.push_str("foundry_web::Boxer::box_it(");
                output.push_str(&group.stream().to_string());
                output.push_str(")");
            },
            _ => {
                panic!("Unexpected token 5.")
                }
        }
        first_child = false;
    }
}

fn html_attr_parse(item_iter: &mut dyn Iterator<Item=TokenTree>, attr_name: String, output: &mut String) {
    let token_attr = item_iter.next();
    
    match token_attr {
        Some(Ident(id)) => {
            output.push_str(&format!("(\"{}\", {}.into())", attr_name, id.to_string()));
        },
        Some(Literal(id)) => {
            output.push_str(&format!("(\"{}\", {}.into())", attr_name, id.to_string()));
        },
        Some(Group(group)) => {
            output.push_str(&format!("(\"{}\", Value::as_func(move || {}))", attr_name, group.to_string()));
        },
        Some(Punct(punct)) => {
            if punct.as_char() == '@' {
                let lit = item_iter.next();
                if let Some(Ident(lit_val)) = lit {
                    output.push_str(&format!("(\"{}\", {}.clone().into())", attr_name, lit_val.to_string()));
                }
                else {
                    panic!("Unexpected token: {:?}", lit)
                }
            } else {
                panic!("Unexpected token: {:?}", punct)
            }
        }
        _ => {

        }
    }
}

fn html_open_tag(item_iter: &mut dyn Iterator<Item=TokenTree>, tag: String, output: &mut String) {
    output.push_str(&format!("Box::new(HtmlNode::new(\"{}\", vec!(", tag));
    let mut looping = true;
    let mut first_attribute = true;
    while looping {
        let token_get = item_iter.next();
        match token_get {
            Some(Ident(id)) => {
                if first_attribute == false {
                    output.push_str(", ");
                }
                if let Some(Punct(eq)) = item_iter.next() {
                    if eq.as_char() == '=' {
                        html_attr_parse(item_iter, id.to_string(), output);
                        first_attribute = false;
                    }
                    else {
                        panic!("Unexpected token 7.");
                    }
                } else {
                    panic!("Unexpected token 8.");
                }
            },
            Some(Punct(tag_end)) => {
                if tag_end.as_char() == '>' {
                    output.push_str("), vec!("); //TODO: include attributes.
                    looping = false;
                    html_tag_content(item_iter, tag.clone(), output);
                }
            }
            _ => { panic!("Unexpected token 9."); }
        }
    }
}

fn html_parse(item_iter: &mut dyn Iterator<Item=TokenTree>, output: &mut String) {
    let token_get = item_iter.next();
    match token_get {
        Some(Punct(tag_start)) => {
            if tag_start.as_char() == '<' {
                //ensure new tag.
                {
                    let token_get = item_iter.next();
                    match token_get {
                        Some(Ident(tag_name)) => {
                            html_open_tag(item_iter, tag_name.to_string(), output);
                        }
                        _ => {panic!("Unexpected token 10.")}
                    }
                }
            }
        }
        _ => {panic!("Unexpected token 11.")}
    }
    // Debugging code:
    //println!("OUTPUT: {}", output);
}

//#[proc_macro]
#[proc_macro_hack]
pub fn html(_item: TokenStream) -> TokenStream {
    let mut output = "".to_string();
    let mut item_iter = _item.into_iter();
    html_parse(&mut item_iter, &mut output);

    output.parse().unwrap()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
