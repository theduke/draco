 #![feature(proc_macro_hygiene)]

extern crate draco_macros;

use draco_macros::html;

struct Message;

type Elem = draco::NonKeyedElement<Message>;
type Node = draco::Node<Message>;

#[test]
fn div_nested_block_list() {
    let items = (1u32..2u32).map(|_| html!(<div />));
    let _: Elem = html!(
        <div>
            #{items}
        </div>
    );
}
