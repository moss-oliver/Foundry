
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
                            panic!("Unexpected token 1.");
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

    while looping {
        let token_get = item_iter.next();
        match token_get {
            Some(Punct(tag_start)) => {
                if tag_start.as_char() == '<' {
                    //Check if closing tag or new tag.
                    let token_get = item_iter.next();
                    match token_get {
                        Some(Punct(tag_end)) => {
                            if tag_end.as_char() == '/' {
                                html_end_tag(item_iter, tag.clone(), output);
                                looping = false;
                            }
                        }
                        Some(Ident(tag_name)) => {
                            if !first_child {
                                output.push_str(",");
                            }
                            html_open_tag(item_iter, tag_name.to_string(), output);
                        }
                        _ => { panic!("Unexpected token 4.") }
                    }
                }
            },
            Some(Literal(lit)) => { //TODO: parse strings with out "" chars.
                if !first_child {
                    output.push_str(",");
                }
                output.push_str(&format!("Box::new({}.to_string())", lit.to_string()));
            },
            Some(Group(group)) => {
                if !first_child {
                    output.push_str(",");
                }
                output.push_str(&group.stream().to_string());
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
            output.push_str(&format!("(\"{}\", Str({}.to_string()))", attr_name, id.to_string()));
        },
        Some(Literal(id)) => {
            output.push_str(&format!("(\"{}\", Str({}.to_string()))", attr_name, id.to_string()));
        },
        Some(Group(group)) => {
            output.push_str(&format!("(\"{}\", Func(Rc::new(move || {})))", attr_name, group.to_string()));
        },
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
