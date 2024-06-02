use std::collections::HashMap;
mod lexical;
mod parser;

fn main() {
    // let simple_json = r#"
    //     {
    //         "name": "Alice",
    //         "age": 30,
    //         "is_student": false
    //     }
    //     jfdiosfjds: fdsoi  ,
    //     fjdsoifdsfk,
    //     fjdiofidosfjj fjdiso
    // "#;

    // println!("source: {simple_json}");
    // println!("parser: {:?}", parser::parse(simple_json));
    let json = r#"
        {
            "a": null,
            "b": {
                "c": null
            }
        }
    "#;
    let mut obj = HashMap::<String, parser::Value>::new();
    obj.insert("a".to_string(), parser::Value::Null);
    let mut b = HashMap::<String, parser::Value>::new();
    b.insert("c".to_string(), parser::Value::Null);
    obj.insert("b".to_string(), parser::Value::Object(b));
    assert_eq!(Ok(parser::Value::Object(obj)), parser::parse(json))
}
