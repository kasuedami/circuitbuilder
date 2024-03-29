use std::fs;

use eframe::{egui::{self, TopBottomPanel, SidePanel, Label, Sense, ViewportBuilder, Slider, ComboBox}, epaint::Vec2};
use simulator::{Circuit, simulator::Simulator, function::Function};

const EXAMPLE: &str = r#"{"inputs":[{"value_index":0},{"value_index":1}],"outputs":[{"value_index":2}],"components":[{"input_value_indices":[0,1],"output_value_indices":[2],"owned_value_indices":[],"function":"And"}],"value_list_len":3}"#;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder {
            inner_size: Some(Vec2::new(1024.0, 840.0)),
            ..Default::default()
        },
        ..Default::default()
    };

    eframe::run_native(
        "circuitbuilder",
        options,
        Box::new(|_cc| Box::<CircuitBuilder>::default())
    )
}

#[derive(Default)]
struct CircuitBuilder {
    circuit: Option<Circuit>,
    selected_element: SelectedElement,
    adding_element: AddingElement,
    simulator: Option<Simulator>,
    occupied_sides: OccupiedSides,
}

#[derive(Default)]
struct OccupiedSides {
    top: f32,
    left: f32,
    right: f32,
}

#[derive(Default)]
enum SelectedElement {
    #[default]
    None,
    Input(usize),
    Output(usize),
    Component(usize),
}

#[derive(Default)]
enum AddingElement {
    #[default]
    None,
    Input,
    Output(usize),
    Component(AddComponentData),
}

struct AddComponentData {
    function: Function,
    input_value_indices: Vec<usize>,
}

impl eframe::App for CircuitBuilder {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {

        if let Some(simulator) = &mut self.simulator {
            simulator.simulate();
        }

        self.menu_bar(ctx);
        self.explorer(ctx);
        self.inspector(ctx);
    }
}

impl CircuitBuilder {
    fn menu_bar(&mut self, ctx: &egui::Context) {
        self.occupied_sides.top = TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("New").clicked() {
                    self.circuit = Some(Circuit::new());
                }

                if ui.button("Load").clicked() {
                    if let Some(file) = rfd::FileDialog::new().pick_file() {
                        let contents = fs::read_to_string(file).unwrap();

                        self.circuit = Some(serde_json::from_str(&contents).unwrap());
                    }
                }

                if ui.button("Example").clicked() {
                    self.circuit = Some(serde_json::from_str(EXAMPLE).unwrap());
                }

                if let Some(circuit) = &self.circuit {

                    if self.simulator.is_none() {
                        if ui.button("Start simulation").clicked() {
                            self.simulator = Some(Simulator::new(circuit.clone()));
                        }
                    } else {
                        if ui.button("Stop simulation").clicked() {
                            self.simulator = None;
                        }
                    }
                }
            })
        }).response.rect.height();
    }

    fn explorer(&mut self, ctx: &egui::Context) {
        self.occupied_sides.left = SidePanel::left("explorer").show(ctx, |ui| {
            if let Some(circuit) = &self.circuit {

                ui.label("Circuit");
                ui.separator();

                ui.collapsing("Inputs", |ui| {
                        for i in 0..circuit.all_inputs().len() {
                            let value_index = circuit.all_inputs()[i].value_index();

                            if ui.add(Label::new(format!("{i}: value index: {value_index}")).sense(Sense::click())).clicked() {
                                self.selected_element = SelectedElement::Input(i);
                            }
                        }
                    });

                ui.collapsing("Outputs", |ui| {
                        for i in 0..circuit.all_outputs().len() {
                            let value_index = circuit.all_outputs()[i].value_index();

                            if ui.add(Label::new(format!("{i}: value index: {value_index}")).sense(Sense::click())).clicked() {
                                self.selected_element = SelectedElement::Output(i);
                            }
                        }
                    });

                ui.collapsing("Components", |ui| {
                        for i in 0..circuit.all_components().len() {
                            let component = &circuit.all_components()[i];
                            let function = component.function();

                            ui.collapsing(format!("{i}: {function}"), |ui| {
                                if ui.add(Label::new(format!("Select")).sense(Sense::click())).clicked() {
                                    self.selected_element = SelectedElement::Component(i);
                                }

                                ui.collapsing("Inputs", |ui| {
                                        for i in 0..component.input_value_indices().len() {
                                            let value_index = component.input_value_indices()[i];
                                            ui.label(format!("{i}: value index: {value_index}"));
                                        }
                                    });

                                ui.collapsing("Outputs", |ui| {
                                        for i in 0..component.output_value_indices().len() {
                                            let value_index = component.output_value_indices()[i];
                                            ui.label(format!("{i}: value index: {value_index}"));
                                        }
                                    });
                                });
                        }
                    });
            }

            if self.circuit.is_some() && self.simulator.is_none() {

                ui.separator();
                ui.label("Add elements");

                ui.horizontal(|ui| {
                    if ui.button("Input").clicked() {
                        self.adding_element = AddingElement::Input;
                    }

                    if ui.button("Output").clicked() {
                        self.adding_element = AddingElement::Output(0);
                    }

                    if ui.button("Component").clicked() {
                        self.adding_element = AddingElement::Component(AddComponentData::default());
                    }
                });

                match &mut self.adding_element {
                    AddingElement::None => (),
                    AddingElement::Input => {
                        ui.label("Adding new input");

                        if ui.button("Confirm").clicked() {
                            self.circuit.as_mut().unwrap().add_input();
                            self.adding_element = AddingElement::None;
                        }
                    },
                    AddingElement::Output(index) => {
                        ui.label("Adding new output");

                        let mut value = *index;
                        let value_range = 0..=self.circuit.as_ref().unwrap().value_list_len() - 1;
                        ui.add(Slider::new(&mut value, value_range));

                        *index = value;

                        if ui.button("Confirm").clicked() {
                            self.circuit.as_mut().unwrap().add_output(*index);
                            self.adding_element = AddingElement::None;
                        }
                    },
                    AddingElement::Component(component_data) => {
                        ui.label("Adding new component");

                        let options = &[
                            Function::And,
                            Function::Or,
                            Function::Not,
                            Function::Nand,
                            Function::Nor,
                            Function::Circuit(Circuit::new()),
                            Function::FlipFlopRS,
                            Function::FlipFlopJK,
                            Function::FlipFlopD,
                            Function::FlipFlopT,
                        ];

                        let current_discriminatn = std::mem::discriminant(&component_data.function);
                        let mut value = options.iter().position(|value| current_discriminatn == std::mem::discriminant(value)).unwrap();

                        ComboBox::from_label("Choose function")
                            .show_index(ui, &mut value, options.len(), |i| format!("{}", options[i]));

                        if current_discriminatn != std::mem::discriminant(&options[value]) {
                            component_data.function = options[value].clone();

                            if component_data.input_value_indices.len() != component_data.function.input_value_count() {
                                component_data.input_value_indices.resize(component_data.function.input_value_count(), 0);
                            }
                        }

                        if let Function::Circuit(ref mut circuit) = &mut component_data.function {
                            if ui.button("Choose circuit").clicked() {
                                if let Some(file) = rfd::FileDialog::new().pick_file() {
                                    let contents = fs::read_to_string(file).unwrap();
                                    let loaded: Circuit = serde_json::from_str(&contents).unwrap();

                                    *circuit = loaded;
                                }
                            }
                        }

                        for i in 0..component_data.input_value_indices.len() {
                            ui.add(Slider::new(&mut component_data.input_value_indices[i], 0..=self.circuit.as_ref().unwrap().value_list_len() - 1));
                        }

                        if ui.button("Confirm").clicked() {
                            self.circuit.as_mut().unwrap().add_component(component_data.function.clone(), component_data.input_value_indices.clone());
                            self.adding_element = AddingElement::None;
                        }
                    },
                }
            }

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());

        }).response.rect.height();
    }

    fn inspector(&mut self, ctx: &egui::Context) {
        self.occupied_sides.right = SidePanel::right("inspector").show(ctx, |ui| {
            if let Some(circuit) = &self.circuit {

                match self.selected_element {
                    SelectedElement::None => { ui.label("No element selected"); },
                    SelectedElement::Input(index) => {
                        ui.label(format!("Input {index}"));
                        ui.separator();

                        let value_index = circuit.input(index).value_index();
                        ui.label(format!("Value index: {value_index}"));

                        if let Some(simulator) = &mut self.simulator {
                            let current_value = simulator.value_for_index(value_index);
                            if ui.button(current_value.to_string()).clicked() {
                                simulator.set_input(index, !current_value);
                            }
                        }
                    },
                    SelectedElement::Output(index) => {
                        ui.label(format!("Output {index}"));
                        ui.separator();

                        let value_index = circuit.output(index).value_index();
                        ui.label(format!("Value index: {value_index}"));

                        if let Some(simulator) = &mut self.simulator {
                            let current_value = simulator.value_for_index(value_index);
                            ui.label(current_value.to_string());
                        }
                    },
                    SelectedElement::Component(index) => {
                        let component = circuit.component(index);
                        let function = component.function();

                        ui.label(format!("Component {index} {function}"));
                        ui.separator();

                        ui.label("Inputs");

                        for i in 0..component.input_value_indices().len() {
                            let value_index = component.input_value_indices()[i];

                            ui.horizontal(|ui| {
                                ui.label(format!("{i}: Value index {value_index}"));

                                if let Some(simulator) = &self.simulator {
                                    let value = simulator.value_for_index(value_index);
                                    ui.label(value.to_string());
                                }
                            });
                        }

                        ui.separator();
                        ui.label("Outputs");

                        for i in 0..component.output_value_indices().len() {
                            let value_index = component.output_value_indices()[i];

                            ui.horizontal(|ui| {
                                ui.label(format!("{i}: Value index {value_index}"));

                                if let Some(simulator) = &self.simulator {
                                    let value = simulator.value_for_index(value_index);
                                    ui.label(value.to_string());
                                }
                            });
                        }

                    },
                }

            }

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());

        }).response.rect.height();
    }
}

impl Default for AddComponentData {
    fn default() -> Self {
        Self {
            function: Function::And,
            input_value_indices: Default::default()
        }
    }
}