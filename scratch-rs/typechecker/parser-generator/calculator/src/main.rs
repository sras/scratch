mod types;
mod calculator;

fn main() {
    println!("{:?}", calculator::ExprParser::new().parse("82"));
}


