use std::collections::HashMap;

use nom::{
    alpha, alphanumeric, alt, complete, delimited, is_not, many1, map, named, opt, recognize,
    separated_list, separated_pair, tag, ws,
};

use crate::BibItem;

named!(
    quoted_string<&str, &str>,
    complete!(delimited!(
        tag!("\""), 
        string,
        tag!("\"")
    ))
);

named!(
    braced_string<&str, &str>,
    complete!(delimited!(
        tag!("{"), 
        string,
        tag!("}")
    ))
);

named!(
    any_string<&str, &str>,
    is_not!("\"{}")
);

named!(
    string<&str, &str>,
    recognize!(many1!(
        alt!(
            recognize!(delimited_string) |
            any_string
        )
    ))
);

named!(
    delimited_string<&str, &str>,
    alt!( braced_string | quoted_string )
);

named!(
    tag_pair<&str, (&str, &str)>,
    ws!(
        separated_pair!(
            alpha,
            tag!("="),
            alt!(
                alphanumeric |
                delimited_string
            )
        )
    )
);

named!(
    tag_list<&str, HashMap<String, String>>,
    map!(
        separated_list!(tag!(","), complete!(tag_pair)),
        |tpl_vec| {
            let mut hm = HashMap::new();
            for (k, v) in tpl_vec.into_iter() {
                hm.insert(k.to_lowercase(), String::from(v));
            }
            hm
        }
    )
);

named!(
    bib_entry<&str, BibItem>,
    ws!(alt!(
        // STRING
        do_parse!(
            tag!("@STRING") >>
            tags: delimited!(tag!("{"), tag_list, tag!("}")) >>
            (BibItem::String(tags))
        ) |

        // PREAMBLE
        do_parse!(
            tag!("@PREAMBLE") >>
            (BibItem::Preamble)
        ) |

        // COMMENT
        do_parse!(
            tag!("@COMMENT") >>
            (BibItem::Comment)
        ) |

        // Entry type
        do_parse!(
            tag!("@") >>
            typ: alpha >>
            tag!("{") >>
            label: alphanumeric >>
            tag!(",") >>
            tags: tag_list >>
            opt!(tag!(",")) >>
            tag!("}") >>
            (BibItem::Entry {
                entry_type: typ.to_uppercase(),
                label: String::from(label),
                tags
            })
        )
    ))
);

named!(
    pub bibfile<&str, Vec<BibItem>>,
    ws!(many0!(complete!(bib_entry)))
);

#[cfg(test)]
mod tests {
    use super::*;

    fn get_expected() -> BibItem {
        let mut expected_props = HashMap::new();
        expected_props.insert(String::from("author"), String::from("Some Body"));
        expected_props.insert(String::from("title"), String::from("Some Thing"));
        expected_props.insert(String::from("date"), String::from("2000"));
        BibItem::Entry {
            entry_type: String::from("ARTICLE"),
            label: String::from("label"),
            tags: expected_props,
        }
    }

    #[test]
    fn test_quoted_string() {
        let line = "\"This is a string\"\0";
        let (i, o) = quoted_string(line).unwrap();

        assert_eq!(o, "This is a string");
    }

    #[test]
    fn test_braced_string() {
        let line = "{test string}\0";
        let (_, o) = braced_string(line).unwrap();

        assert_eq!(o, "test string");
    }

    #[test]
    fn test_kv_pair() {
        let line = "key=\"value\"\0";
        let r = tag_pair(line);

        if let Err(ref inc) = r {
            println!("{:?}", inc);
        } else {
            let (_, (k, v)) = r.unwrap();

            println!("{:?} = {:?}", k, v);
            assert_eq!(k, "key");
            assert_eq!(v, "value");
        }
    }

    #[test]
    fn test_kv_list() {
        let line = "keyone=\"value1\",\nkeytwo={value2}\0";
        let mut expected = HashMap::new();
        expected.insert(String::from("keyone"), String::from("value1"));
        expected.insert(String::from("keytwo"), String::from("value2"));
        let (_, o) = tag_list(line).unwrap();
        assert_eq!(o, expected);
    }

    #[test]
    fn test_full_bib_item() {
        let line = "\
@article{label,
    author = \"Some Body\",
    title = \"Some Thing\",
    date = \"2000\"
}\0";
        let expected = get_expected();
        let (_, o) = bib_entry(line).unwrap();
        assert_eq!(o, expected);
    }

    #[test]
    fn test_full_bib_item_no_delim() {
        let line = "\
@article{label,
    author = \"Some Body\",
    title = \"Some Thing\",
    date = 2000
}\0";
        let expected = get_expected();
        let (_, o) = bib_entry(line).unwrap();
        assert_eq!(o, expected);
    }

    #[test]
    fn test_full_bib_item_with_whitespce() {
        let line = "\
@article { label,
    author = \"Some Body\",
    title = \"Some Thing\",
    date = \"2000\"
}\0";
        let expected = get_expected();
        let (_, o) = bib_entry(line).unwrap();
        assert_eq!(o, expected);
    }

    #[test]
    fn test_full_bib_item_trailing_comma() {
        let line = "\
@article{label,
    author = \"Some Body\",
    title = \"Some Thing\",
    date = \"2000\",
}\0";
        let expected = get_expected();
        let (_, o) = bib_entry(line).unwrap();
        assert_eq!(o, expected);
    }

    fn get_expected_strings() -> BibItem {
        let mut hm = HashMap::new();
        hm.insert(String::from("key"), String::from("value"));
        hm.insert(String::from("another"), String::from("value"));

        BibItem::String(hm)
    }

    #[test]
    fn test_replacement_strings() {
        let line = "@STRING { key = \"value\", another = \"value\" }\0";
        let expected = get_expected_strings();
        let (_, o) = bib_entry(line).unwrap();

        assert_eq!(o, expected);
    }

    #[test]
    fn test_bib_file() {
        let line = "\
@article {label,
    title = \"article\",
    author = {somebody},
    date = 2000,
}

@book {labeltwo,
    title = \"book\",
    author = {somebody else},
    date = 2000,
}\0";

        let (_, r) = bibfile(line).unwrap();
        println!("{:?}", r);
        assert_eq!(r.len(), 2);
    }

}
