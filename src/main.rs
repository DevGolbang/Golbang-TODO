use state::{Entry, Filter, State};
use strum::IntoEnumIterator;
use yew::format::Json;
use yew::web_sys::HtmlInputElement as InputElement;
use yew::{classes, html, Component, ComponentLink, Html, InputData, NodeRef, ShouldRender};
use yew::{events::KeyboardEvent, Classes};
use yew_services::storage::{Area, StorageService};

mod state;

const KEY: &str = "yew.todomvc.self";


pub enum Msg{
    Add,
    Edit(usize),
    Update(String),
    UpdateEdit(String),
    Remove(usize),
    SetFilter(Filter),
    ToggleAll,
    ToggleEdit(usize),
    Toggle(usize),
    ClearCompleted,
    Focus,
}

pub struct Model{
    link: ComponentLink<Self>,
    storage: StorageService,
    state: State,
    focus_ref: NodeRef,
}
impl Component for Model{
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self{
        let storage = StorageService::new(Area::Local).expect("storage was disabled by the user");
        let entries = {
            if let Json(Ok(restored_model)) = storage.restore(KEY){
                restored_model
            }else{
                Vec::new()
            }
        };

        let state = State{
            entries,
            filter: Filter::All,
            value:"".into(),
            edit_value: "".into(),
        };
        let focus_ref = NodeRef::default();
        Self{
            link,
            storage,
            state,
            focus_ref,

        }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender{
        match msg{
            Msg::Add => {
                let description = self.state.value.trim();
                if !description.is_empty(){
                    let entry = Entry{
                        description: description.to_string(),
                        completed: false,
                        editing: false,
                    };
                    self.state.entries.push(entry);
                }
                self.state.value = "".to_string();
            }
            Msg::Edit(idx) => {
                let edit_value = self.state.edit_value.trim().to_string();
                self.state.complete_edit(idx, edit_value);
                self.state.edit_value = "".to_string();
            }
            Msg::Update(val) => {
                println!("Input: {}", val);
                self.state.value = val;
            } 
            Msg::UpdateEdit(val)=>{
                println!("Input: {}", val);
                self.state.edit_value = val;
            }
            Msg::Remove(idx) => {
                self.state.remove(idx);
            }
            Msg::SetFilter(filter) => {
                self.state.filter = filter;
            }
            Msg::ToggleEdit(idx) => {
                self.state.edit_value = self.state.entries[idx].description.clone();
                self.state.clear_all_edit();
                self.state.toggle_edit(idx);
            }
            Msg::ToggleAll =>{
                let status = !self.state.is_all_completed();
                self.state.toggle_all(status);
            }
            Msg::Toggle(idx) => {
                self.state.toggle(idx);
            }
            Msg::ClearCompleted=>{
                self.state.clear_completed();
            }
            Msg::Focus =>{
                if let Some(input) = self.focus_ref.cast::<InputElement>(){
                    input.focus().unwrap();
                }
            }
        }
        self.storage.store(KEY, Json(&self.state.entries));
        true
    }
    fn change(&mut self, _: Self::Properties) -> ShouldRender{
        false
    }
    fn view(&self)-> Html{
        let hidden_class = if self.state.entries.is_empty(){
            "hidden"
        }else{
            ""
        };
      html!{
            <div class="todomvc-wrapper">
                <section class="todoapp">
                    <header class ="header">
                        <h1>{"TODO(할일)"}</h1>
                        {self.view_input()}
                    </header>
                    <section class=classes!("main", hidden_class)>
                        <input
                            type="checkbox"
                            class="toggle-all"
                            id="toggle-all"
                            checked=self.state.is_all_completed()
                            onclick=self.link.callback(|_| Msg::ToggleAll)
                        />
                        <label for="toggle-all"/>
                        <u1 class ="todo-list">
                            {for self.state.entries.iter().filter(|e| self.state.filter.fits(e)). enumerate().map(|e| self.view_entry(e))}
                        </u1>
                    </section>
                    <footer class=classes!("footer", hidden_class)>
                        <span class="todo-count">
                            <strong>{self.state.total()}</strong>
                            {"개 남았습니다."}
                        </span>
                        <ul class="filters">
                            {for Filter::iter().map(|flt| self.view_filter(flt))}
                        </ul>
                        <button class="clear-completed" onclick=self.link.callback(|_| Msg::ClearCompleted)>
                            {format!("완벽히 끝낸것들 ({})", self.state.total_completed())}
                        </button>
                    </footer>
                </section>
                <footer class="info">
                    <strong>{"원본 출처: "}<a href="https://github.com/yewstack/yew/tree/master/examples/todomvc">{"Yew(Rust(WASM) Frontend Library)"}</a></strong>
                    <p>{ "더블 클릭하여 할 일을 적으세요!" }</p>
                    <p>{ "작성자: " }<a href="https://github.com/DevGolbang/" target="_blank">{ "송주현" }</a></p>
                    <p>{ "출처: " }<a href="http://todomvc.com/" target="_blank">{ "TodoMVC" }</a></p>
                </footer>
            </div>
            }
    }
}
impl Model{
    fn view_filter(&self, filter: Filter) -> Html{
        let cls = if self.state.filter == filter{
            "selected"
        } else{
            "not-selected"
        };
        let filter_text = match filter{
            Filter::All => "모두",
            Filter::Active => "하는중",
            Filter::Completed => "완료",
        };
        html!{
            <li>
                <a class=cls
                    href=filter.as_href()
                    onclick=self.link.callback(move |_| Msg::SetFilter(filter))
                >
                    {filter_text}
                </a>
            </li>
        }
    }
    fn view_input(&self) -> Html{
        html!{
            <input
                class="new-todo"
                placeholder="할 일을 적으세요!"
                value=self.state.value.clone()
                oninput = self.link.callback(|e: InputData| Msg::Update(e.value))
                onkeypress=self.link.batch_callback(|e: KeyboardEvent|{
                    if e.key() == "Enter"{Some(Msg::Add)} else {None}
                })
            />
        }
    }

    fn view_entry(&self, (idx, entry): (usize, &Entry)) -> Html{
        let mut class = Classes::from("todo");
        if entry.editing{
            class.push(" editing");
        }
        if entry.completed{
            class.push(" completed");
        }
        html!{
            <li class=class>
                <div class="view">
                    <input
                        type="checkbox"
                        class="toggle"
                        checked=entry.completed
                        onclick=self.link.callback(move |_| Msg::Toggle(idx))
                    />
                    <label  ondblclick=self.link.callback(move |_| Msg::ToggleEdit(idx))>{&entry.description}</label>
                    <button class="destroy" onclick=self.link.callback(move |_| Msg::Remove(idx)) />
                </div>
                {self.view_entry_edit_input((idx, &entry))}
            </li>
        }
    }

    fn view_entry_edit_input(&self, (idx, entry): (usize, &Entry)) -> Html{
        if entry.editing{
         
            html!{
                <input
                    class="edit"
                    type="text"
                    ref=self.focus_ref.clone()
                    value=self.state.edit_value.clone()
                    onmouseover=self.link.callback(|_| Msg::Focus)
                    oninput=self.link.callback(|e: InputData| Msg::UpdateEdit(e.value))
                    onblur=self.link.callback(move |_| Msg::Edit(idx))
                    onkeypress=self.link.batch_callback(move |e: KeyboardEvent|{
                        if e.key() == "Enter" {Some(Msg::Edit(idx))} else { None}
                    })
                />

            }

        }else{
            html!{<input type= "hidden"/>}
        }
    }
}
fn main(){
    yew::start_app::<Model>();
}