use yew::prelude::*;
use yew_functional::function_component;

#[derive(Clone, Properties, PartialEq)]
struct Props {
    a: usize,
}

#[function_component(Comp)]
fn comp<'a>(props: &'a Props) -> Html {

    html! {
        <p>
            { props.a }
        </p>
    }
}

fn main() {}
