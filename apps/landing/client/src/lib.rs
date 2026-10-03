use serde::Deserialize;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{
    Document, Element, Event, HtmlButtonElement, HtmlElement, HtmlFormElement, HtmlInputElement,
    HtmlTextAreaElement, KeyboardEvent, Request, RequestInit, Response,
};

fn element(document: &Document, id: &str) -> Result<Element, JsValue> {
    document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(id))
}

fn listen(
    target: &Element,
    kind: &str,
    callback: impl FnMut(Event) + 'static,
) -> Result<(), JsValue> {
    let handler = Closure::<dyn FnMut(Event)>::new(callback);
    target.add_event_listener_with_callback(kind, handler.as_ref().unchecked_ref())?;
    // These listeners live for the lifetime of this single-page document.
    handler.forget();
    Ok(())
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let document = web_sys::window()
        .and_then(|w| w.document())
        .ok_or_else(|| JsValue::from_str("document"))?;
    preview(&document)?;
    waitlist(&document)
}

struct Mode {
    id: &'static str,
    caption: &'static str,
    symbol: &'static str,
    title: &'static str,
    description: &'static str,
    placeholder: &'static str,
    prompts: [&'static str; 3],
}

const MODES: [Mode; 3] = [
    Mode {
        id: "chat",
        caption: "Chat with AI.",
        symbol: "✳",
        title: "What would you like to ask?",
        description: "Ask a question, explore an idea, or learn something new.",
        placeholder: "What’s on your mind?",
        prompts: [
            "Think through an idea",
            "Explain something",
            "Explore a document",
        ],
    },
    Mode {
        id: "work",
        caption: "Get help with a task.",
        symbol: "▧",
        title: "What would you like to get done?",
        description: "Research a topic, make a file, or plan a project.",
        placeholder: "Describe your task…",
        prompts: ["Research a topic", "Draft a report", "Plan a project"],
    },
    Mode {
        id: "code",
        caption: "Work on code.",
        symbol: "⌘",
        title: "Let’s get into the code.",
        description: "Read code, make changes, and check the results.",
        placeholder: "What are you building?",
        prompts: [
            "Understand a codebase",
            "Review a change",
            "Investigate a bug",
        ],
    },
];

struct Preview {
    document: Document,
    tabs: Vec<Element>,
    panel: Element,
    caption: Element,
    symbol: Element,
    title: Element,
    description: Element,
    input: HtmlTextAreaElement,
    feedback: Element,
    chips: Element,
}

impl Preview {
    fn select(&self, index: usize) -> Result<(), JsValue> {
        let mode = &MODES[index];
        for (i, tab) in self.tabs.iter().enumerate() {
            let selected = i == index;
            tab.set_attribute("aria-selected", if selected { "true" } else { "false" })?;
            tab.set_attribute("tabindex", if selected { "0" } else { "-1" })?;
            tab.set_class_name(if selected {
                "mode-tab active"
            } else {
                "mode-tab"
            });
        }
        self.panel
            .set_attribute("aria-labelledby", &format!("tab-{}", mode.id))?;
        self.caption.set_text_content(Some(mode.caption));
        self.symbol.set_text_content(Some(mode.symbol));
        self.title.set_text_content(Some(mode.title));
        self.description.set_text_content(Some(mode.description));
        self.input.set_placeholder(mode.placeholder);
        self.input.set_value("");
        self.feedback.set_text_content(None);
        self.chips.set_text_content(None);
        for prompt in mode.prompts {
            let button = self.document.create_element("button")?;
            button.set_attribute("type", "button")?;
            button.set_text_content(Some(prompt));
            self.chips.append_child(&button)?;
        }
        Ok(())
    }
}

fn preview(document: &Document) -> Result<(), JsValue> {
    let preview = Rc::new(Preview {
        document: document.clone(),
        tabs: MODES
            .iter()
            .map(|mode| element(document, &format!("tab-{}", mode.id)))
            .collect::<Result<_, _>>()?,
        panel: element(document, "preview-panel")?,
        caption: element(document, "mode-caption")?,
        symbol: element(document, "preview-symbol")?,
        title: element(document, "preview-title")?,
        description: element(document, "preview-description")?,
        input: element(document, "preview-input")?.dyn_into()?,
        feedback: element(document, "preview-feedback")?,
        chips: element(document, "prompt-chips")?,
    });
    for (index, tab) in preview.tabs.iter().enumerate() {
        let state = preview.clone();
        listen(tab, "click", move |_| {
            let _ = state.select(index);
        })?;
        let state = preview.clone();
        listen(tab, "keydown", move |event| {
            let Some(keyboard) = event.dyn_ref::<KeyboardEvent>() else {
                return;
            };
            let next = match keyboard.key().as_str() {
                "ArrowRight" | "ArrowDown" => (index + 1) % 3,
                "ArrowLeft" | "ArrowUp" => (index + 2) % 3,
                "Home" => 0,
                "End" => 2,
                _ => return,
            };
            event.prevent_default();
            let _ = state.select(next);
            if let Some(tab) = state.tabs[next].dyn_ref::<HtmlElement>() {
                let _ = tab.focus();
            }
        })?;
    }
    let state = preview.clone();
    listen(&preview.chips, "click", move |event| {
        let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
            return;
        };
        if let Ok(Some(button)) = target.closest("button") {
            state
                .input
                .set_value(&button.text_content().unwrap_or_default());
            state.feedback.set_text_content(None);
            let _ = state.input.focus();
        }
    })?;
    listen(
        &element(document, "preview-form")?,
        "submit",
        move |event| {
            event.prevent_default();
            let message = if preview.input.value().trim().is_empty() {
                let _ = preview.input.focus();
                "Add a prompt, or choose one of the examples above."
            } else {
                "This is a preview. Your prompt is not sent to an AI or saved."
            };
            preview.feedback.set_text_content(Some(message));
        },
    )
}

struct Waitlist {
    form: HtmlFormElement,
    input: HtmlInputElement,
    button: HtmlButtonElement,
    status: Element,
    submitting: Cell<bool>,
}

#[derive(Deserialize)]
struct ApiReply {
    message: String,
}

impl Waitlist {
    fn message(&self, message: &str, success: bool) {
        self.status.set_text_content(Some(message));
        let _ = self
            .status
            .set_attribute("data-state", if success { "success" } else { "error" });
    }

    async fn send(&self) -> Result<(), JsValue> {
        let init = RequestInit::new();
        init.set_method("POST");
        init.set_body(&JsValue::from_str(
            &serde_json::json!({ "email": self.input.value() }).to_string(),
        ));
        init.set_signal(Some(&web_sys::AbortSignal::timeout_with_u32(15_000)));
        let request = Request::new_with_str_and_init(&self.form.action(), &init)?;
        request.headers().set("Content-Type", "application/json")?;
        request.headers().set("Accept", "application/json")?;
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("window"))?;
        let response: Response = JsFuture::from(window.fetch_with_request(&request))
            .await?
            .dyn_into()?;
        let reply: ApiReply =
            serde_wasm_bindgen::from_value(JsFuture::from(response.json()?).await?)?;
        self.message(&reply.message, response.ok());
        if response.status() == 400 {
            self.input.set_attribute("aria-invalid", "true")?;
            self.input.focus()?;
        }
        if response.ok() {
            self.input.set_value("");
        }
        Ok(())
    }
}

fn waitlist(document: &Document) -> Result<(), JsValue> {
    let form: HtmlFormElement = element(document, "waitlist")?.dyn_into()?;
    let button = form
        .query_selector("button[type=submit]")?
        .ok_or_else(|| JsValue::from_str("join button"))?
        .dyn_into()?;
    let state = Rc::new(Waitlist {
        form,
        input: element(document, "waitlist-email")?.dyn_into()?,
        button,
        status: element(document, "waitlist-status")?,
        submitting: Cell::new(false),
    });
    let clear = state.clone();
    listen(state.input.as_ref(), "input", move |_| {
        let _ = clear.input.remove_attribute("aria-invalid");
        clear.status.set_text_content(None);
        let _ = clear.status.remove_attribute("data-state");
    })?;
    let submit = state.clone();
    listen(state.form.as_ref(), "submit", move |event| {
        event.prevent_default();
        if submit.submitting.get() || !submit.form.report_validity() {
            return;
        }
        submit.submitting.set(true);
        submit.button.set_disabled(true);
        submit.button.set_text_content(Some("Joining…"));
        let _ = submit.form.set_attribute("aria-busy", "true");
        submit.status.set_text_content(None);
        let state = submit.clone();
        spawn_local(async move {
            if state.send().await.is_err() {
                state.message(
                    "Could not join the waitlist. Check your connection and try again.",
                    false,
                );
            }
            state.submitting.set(false);
            state.button.set_disabled(false);
            state.button.set_text_content(Some("Join waitlist"));
            let _ = state.form.remove_attribute("aria-busy");
        });
    })
}
