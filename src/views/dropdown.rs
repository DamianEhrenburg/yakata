use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub struct DropdownOption {
    pub value: String,
    pub label: String,
    pub sublabel: Option<String>,
}

impl DropdownOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            sublabel: None,
        }
    }

    pub fn with_sublabel(
        value: impl Into<String>,
        label: impl Into<String>,
        sublabel: impl Into<String>,
    ) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            sublabel: Some(sublabel.into()),
        }
    }
}

#[component]
pub fn Dropdown(
    #[props(default)] class: String,
    options: Vec<DropdownOption>,
    selected_value: String,
    on_select: EventHandler<String>,
) -> Element {
    let mut is_open = use_signal(|| false);

    let selected_opt = options.iter().find(|opt| opt.value == selected_value);
    let current_label = selected_opt
        .map(|opt| opt.label.clone())
        .unwrap_or_else(|| selected_value.clone());
    let current_sublabel = selected_opt.and_then(|opt| opt.sublabel.clone());

    rsx! {
        div {
            class: "custom-dropdown {class}",
            tabindex: "-1",
            onkeydown: move |evt| {
                if evt.key().to_string() == "Escape" && *is_open.read() {
                    is_open.set(false);
                }
            },
            button {
                r#type: "button",
                class: if *is_open.read() { "dropdown-trigger open" } else { "dropdown-trigger" },
                onclick: move |e| {
                    e.stop_propagation();
                    is_open.toggle();
                },
                span { class: "dropdown-label",
                    span { class: "dropdown-label-main", "{current_label}" }
                    if let Some(sub) = current_sublabel {
                        span { class: "dropdown-label-sub", "{sub}" }
                    }
                }
                svg {
                    class: if *is_open.read() { "dropdown-chevron open" } else { "dropdown-chevron" },
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2.2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    path { d: "m6 9 6 6 6-6" }
                }
            }
            if *is_open.read() {
                div {
                    class: "dropdown-backdrop",
                    onclick: move |e| {
                        e.stop_propagation();
                        is_open.set(false);
                    },
                    onwheel: move |_| {
                        is_open.set(false);
                    },
                }
                div { class: "dropdown-menu",
                    for opt in options {
                        {
                            let val = opt.value.clone();
                            let is_selected = val == selected_value;
                            rsx! {
                                div {
                                    class: if is_selected { "dropdown-item selected" } else { "dropdown-item" },
                                    onclick: {
                                        let val = val.clone();
                                        move |e| {
                                            e.stop_propagation();
                                            on_select.call(val.clone());
                                            is_open.set(false);
                                        }
                                    },
                                    span { class: "dropdown-item-main", "{opt.label}" }
                                    if let Some(sub) = opt.sublabel.as_ref() {
                                        span { class: "dropdown-item-sub", "{sub}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
