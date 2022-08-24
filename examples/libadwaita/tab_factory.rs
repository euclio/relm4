use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use relm4::{
    adw,
    factory::{DynamicIndex, FactoryComponent, FactoryComponentSender, FactoryVecDeque},
    gtk, ComponentParts, ComponentSender, RelmApp, SimpleComponent,
};

#[derive(Debug)]
struct Counter {
    value: u8,
}

#[derive(Debug)]
enum CounterMsg {
    Increment,
    Decrement,
}

#[derive(Debug)]
enum CounterOutput {
    SendFront(DynamicIndex),
    MoveUp(DynamicIndex),
    MoveDown(DynamicIndex),
}

struct CounterWidgets {
    label: gtk::Label,
}

impl FactoryComponent for Counter {
    type ParentWidget = adw::TabView;
    type ParentMsg = AppMsg;

    type Widgets = CounterWidgets;

    type Init = u8;

    type Input = CounterMsg;
    type Output = CounterOutput;

    type Root = gtk::Box;
    type CommandOutput = ();

    fn output_to_parent_msg(output: Self::Output) -> Option<AppMsg> {
        Some(match output {
            CounterOutput::SendFront(index) => AppMsg::SendFront(index),
            CounterOutput::MoveUp(index) => AppMsg::MoveUp(index),
            CounterOutput::MoveDown(index) => AppMsg::MoveDown(index),
        })
    }

    fn init_root(&self) -> Self::Root {
        relm4::view! {
            root = gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
            }
        }
        root
    }

    fn init_model(
        value: Self::Init,
        _index: &DynamicIndex,
        _sender: FactoryComponentSender<Self>,
    ) -> Self {
        Self { value }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: &Self::Root,
        returned_widget: &adw::TabPage,
        sender: FactoryComponentSender<Self>,
    ) -> Self::Widgets {
        relm4::view! {
            label = gtk::Label {
                set_label: &self.value.to_string(),
                set_width_chars: 3,
            }
        }

        relm4::view! {
            add_button = gtk::Button {
                set_label: "+",
                connect_clicked[sender] => move |_| {
                    sender.input(CounterMsg::Increment)
                }
            }
        }

        relm4::view! {
            remove_button = gtk::Button {
                set_label: "-",
                connect_clicked[sender] => move |_| {
                    sender.input(CounterMsg::Decrement)
                }
            }
        }

        relm4::view! {
            move_up_button = gtk::Button {
                set_label: "Up",
                connect_clicked[sender, index] => move |_| {
                    sender.output(CounterOutput::MoveUp(index.clone()))
                }
            }
        }

        relm4::view! {
            move_down_button = gtk::Button {
                set_label: "Down",
                connect_clicked[sender, index] => move |_| {
                    sender.output(CounterOutput::MoveDown(index.clone()))
                }
            }
        }

        relm4::view! {
            to_front_button = gtk::Button {
                set_label: "To start",
                connect_clicked[sender, index] => move |_| {
                    sender.output(CounterOutput::SendFront(index.clone()))
                }
            }
        }

        root.append(&label);
        root.append(&add_button);
        root.append(&remove_button);
        root.append(&move_up_button);
        root.append(&move_down_button);
        root.append(&to_front_button);

        returned_widget.set_title(&format!("Page {}", self.value));

        CounterWidgets { label }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactoryComponentSender<Self>) {
        match msg {
            CounterMsg::Increment => {
                self.value = self.value.wrapping_add(1);
            }
            CounterMsg::Decrement => {
                self.value = self.value.wrapping_sub(1);
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: FactoryComponentSender<Self>) {
        widgets.label.set_label(&self.value.to_string());
    }
}

struct AppModel {
    created_widgets: u8,
    counters: FactoryVecDeque<Counter>,
}

#[derive(Debug)]
enum AppMsg {
    AddCounter,
    RemoveCounter,
    SendFront(DynamicIndex),
    MoveUp(DynamicIndex),
    MoveDown(DynamicIndex),
}

#[relm4::component]
impl SimpleComponent for AppModel {
    // AppWidgets is generated by the macro
    type Widgets = AppWidgets;

    type Init = u8;

    type Input = AppMsg;
    type Output = ();

    view! {
        adw::Window {
            set_title: Some("Tab factory example"),
            set_default_width: 300,
            set_default_height: 100,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,

                adw::HeaderBar {},

                adw::TabBar {
                    set_view: Some(&tabs),
                    set_autohide: false,
                },

                gtk::Button {
                    set_label: "Add counter",
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::AddCounter);
                    }
                },

                gtk::Button {
                    set_label: "Remove counter",
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::RemoveCounter);
                    }
                },

                append: tabs = &adw::TabView {}
            }
        }
    }

    // Initialize the UI.
    fn init(
        counter: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Insert the macro codegen here
        let widgets = view_output!();

        let model = AppModel {
            created_widgets: counter,
            counters: FactoryVecDeque::new(widgets.tabs.clone(), &sender.input),
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        let mut counters_guard = self.counters.guard();

        match msg {
            AppMsg::AddCounter => {
                counters_guard.push_back(self.created_widgets);
                self.created_widgets = self.created_widgets.wrapping_add(1);
            }
            AppMsg::RemoveCounter => {
                counters_guard.pop_back();
            }
            AppMsg::SendFront(index) => {
                counters_guard.move_front(index.current_index());
            }
            AppMsg::MoveDown(index) => {
                let index = index.current_index();
                let new_index = index + 1;
                // Already at the end?
                if new_index < counters_guard.len() {
                    counters_guard.move_to(index, new_index);
                }
            }
            AppMsg::MoveUp(index) => {
                let index = index.current_index();
                // Already at the start?
                if index != 0 {
                    counters_guard.move_to(index, index - 1);
                }
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.tabFactory");
    app.run::<AppModel>(0);
}
