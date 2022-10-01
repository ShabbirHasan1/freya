use dioxus::core::ElementId;
use dioxus::prelude::*;
use dioxus_core::Scope;
use dioxus_native_core::real_dom::{NodeType, RealDom};
use dioxus_router::*;
use freya_components::*;
use freya_elements as dioxus_elements;
use freya_node_state::node::NodeState;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::sleep;

#[derive(PartialEq, Eq, Clone)]
struct TreeNode {
    tag: String,
    id: ElementId,
    height: u16,
    text: Option<String>,
}

#[derive(Props)]
pub struct DevToolsProps {
    rdom: Arc<Mutex<RealDom<NodeState>>>,
}

// Hacky stuff over here
impl PartialEq for DevToolsProps {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

#[allow(non_snake_case)]
pub fn DevTools(cx: Scope<DevToolsProps>) -> Element {
    let children = use_state(&cx, || Vec::<TreeNode>::new());
    let setter = children.setter();

    use_effect(&cx, (), move |_| {
        let rdom = cx.props.rdom.clone();
        async move {
            loop {
                sleep(Duration::from_millis(25)).await;

                let rdom = rdom.lock().unwrap();
                let mut children = Vec::new();

                let mut root_found = false;
                let mut devtools_found = false;

                rdom.traverse_depth_first(|n| {
                    if n.height == 2 {
                        if root_found == false {
                            root_found = true;
                        } else {
                            devtools_found = true;
                        }
                    }

                    if !devtools_found {
                        let mut maybe_text = None;
                        let tag = match &n.node_type {
                            NodeType::Text { text, .. } => {
                                maybe_text = Some(text.clone());
                                "text"
                            }
                            NodeType::Element { tag, .. } => tag,
                            NodeType::Placeholder => "placeholder",
                        }
                        .to_string();

                        children.push(TreeNode {
                            height: n.height,
                            id: n.id,
                            tag,
                            text: maybe_text,
                        });
                    }
                });
                setter(children);
            }
        }
    });

    let children = children.get().iter().map(|node| {
        rsx! {
            NodeElement {
                node: node
            }
        }
    });

    cx.render(rsx! {
        Router {
             container {
                width: "100%",
                direction: "horizontal",
                height: "50",
                FreyaLink {
                    to: "/",
                    label {
                        width: "100",
                        "Elements"
                    }
                }
                FreyaLink {
                    to: "/settings",
                    label {
                        width: "100",
                        "Settings"
                    }
                }
             }
            Route { to: "/",
                ScrollView {
                    width: "100%",
                    height: "calc(100% - 50)",
                    padding: "30",
                    show_scrollbar: true,
                    children
                }
            }
            Route { to: "/settings",
                label {
                    "Settings would be here."
                }
            }
        }
    })
}

#[derive(Props)]
struct FreyaLinkProps<'a> {
    pub to: &'a str,
    pub children: Element<'a>,
}

#[allow(non_snake_case)]
fn FreyaLink<'a>(cx: Scope<'a, FreyaLinkProps<'a>>) -> Element<'a> {
    let svc = cx.use_hook(|| cx.consume_context::<Arc<RouterCore>>());

    render!(
        Button {
            on_click: move |_| {
                if let Some(service) = svc {
                    service.push_route(cx.props.to, None, None);
                } else {
                    println!("Not in router");
                }
            },
            &cx.props.children
        }
    )
}

#[allow(non_snake_case)]
#[inline_props]
fn NodeElement<'a>(cx: Scope<'a>, node: &'a TreeNode) -> Element<'a> {
    let text = node
        .text
        .as_ref()
        .map(|v| format!("({v})"))
        .unwrap_or_default();

    render!(
        rect {
            width: "100%",
            height: "25",
            scroll_x: "{node.height * 10}",
            label {
                "{node.tag} #{node.id} {text}"
            }
        }
    )
}
