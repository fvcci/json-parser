mod lexical;
mod parser;

fn main() {
    let simple_json = r#"
        {
            "name": "Alice",
            "age": 30,
            "is_student": false
        }
        jfdiosfjds: fdsoi  ,
        fjdsoifdsfk,
        fjdiofidosfjj fjdiso
    "#;

    println!("source: {simple_json}");
    println!("parser: {:?}", parser::parse(simple_json));
}
