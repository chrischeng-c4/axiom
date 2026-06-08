// Reference fields are not in the supported field-type set.
use agent::AgentSchema;

#[derive(AgentSchema)]
struct BadRef<'a> {
    borrowed: &'a str,
}

fn main() {}
