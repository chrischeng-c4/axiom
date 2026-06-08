// Tuple fields are not in the supported field-type set.
use agent::AgentSchema;

#[derive(AgentSchema)]
struct BadTuple {
    pair: (i32, i32),
}

fn main() {}
