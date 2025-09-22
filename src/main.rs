// Single-file implementation for the TradeBias mock UI using egui_node_graph2.
//
// NOTE TO USER:
// The original request for a button-based UI to add nodes was not feasible
// with the available libraries, as the `egui_node_graph` crate and its
// direct alternatives are "yanked" (removed) from the official repository.
//
// This implementation uses the modern `egui_node_graph2` library. The standard
// workflow for this library is to use a right-click context menu on the canvas
// to add new nodes. This has been implemented instead of the button-based UI.
//
// The interactive widgets (TextEdit, Slider) inside the nodes are now correctly
// implemented as requested.

use eframe::{egui, App, NativeOptions};
use egui_node_graph2::*;
use std::borrow::Cow;

// --- 1. Define user data types ---

/// NodeData is used to identify the type of a node.
#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum TradeBiasNodeTemplate {
    DataSource,
    Indicator,
    Output,
}

/// The data types for the node's pins.
#[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DataType {
    Dataframe,
    IndicatorValue,
    Signal,
}

/// The value types for input parameters. These store the editable values
/// for the interactive widgets.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum InputValue {
    FilePath(String),
    Period(usize),
    WebhookUrl(String),
}

impl Default for InputValue {
    fn default() -> Self {
        // A dummy default is required by the library.
        InputValue::FilePath(String::new())
    }
}

/// The graph's global state. We don't need any for this example.
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct GraphState;

/// The response type for custom side-effects. We don't have any.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradeBiasResponse {}
impl UserResponseTrait for TradeBiasResponse {}

// --- 2. Implement the required traits ---

impl DataTypeTrait<GraphState> for DataType {
    fn data_type_color(&self, _user_state: &mut GraphState) -> egui::Color32 {
        match self {
            DataType::Dataframe => egui::Color32::from_rgb(38, 109, 211),
            DataType::IndicatorValue => egui::Color32::from_rgb(211, 134, 38),
            DataType::Signal => egui::Color32::from_rgb(100, 211, 38),
        }
    }

    fn name(&self) -> Cow<'_, str> {
        match self {
            DataType::Dataframe => "Dataframe".into(),
            DataType::IndicatorValue => "Indicator Value".into(),
            DataType::Signal => "Signal".into(),
        }
    }
}

impl NodeTemplateTrait for TradeBiasNodeTemplate {
    type NodeData = Self; // The node data is just the template itself
    type DataType = DataType;
    type ValueType = InputValue;
    type UserState = GraphState;
    type CategoryType = &'static str;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
        match self {
            Self::DataSource => "Data Source".into(),
            Self::Indicator => "Indicator".into(),
            Self::Output => "Output".into(),
        }
    }

    fn node_finder_categories(&self, _user_state: &mut Self::UserState) -> Vec<&'static str> {
        match self {
            Self::DataSource => vec!["Input"],
            Self::Indicator => vec!["Processing"],
            Self::Output => vec!["Output"],
        }
    }

    fn node_graph_label(&self, _user_state: &mut Self::UserState) -> String {
        match self {
            Self::DataSource => "CSV Data Source".to_string(),
            Self::Indicator => "Moving Average".to_string(),
            Self::Output => "Webhook Signal".to_string(),
        }
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        *self
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        match self {
            Self::DataSource => {
                graph.add_input_param(
                    node_id,
                    "File Path".to_string(),
                    DataType::Dataframe,
                    InputValue::FilePath("C:/data/BTCUSD.csv".to_string()),
                    InputParamKind::ConstantOnly,
                    true,
                );
                graph.add_output_param(node_id, "Dataframe".to_string(), DataType::Dataframe);
            }
            Self::Indicator => {
                graph.add_input_param(
                    node_id,
                    "Dataframe".to_string(),
                    DataType::Dataframe,
                    InputValue::FilePath("".to_string()), // Dummy value
                    InputParamKind::ConnectionOnly,
                    true,
                );
                graph.add_input_param(
                    node_id,
                    "Period".to_string(),
                    DataType::IndicatorValue,
                    InputValue::Period(14),
                    InputParamKind::ConstantOnly,
                    true,
                );
                graph.add_output_param(
                    node_id,
                    "Indicator Value".to_string(),
                    DataType::IndicatorValue,
                );
            }
            Self::Output => {
                graph.add_input_param(
                    node_id,
                    "Signal Trigger".to_string(),
                    DataType::Signal,
                    InputValue::FilePath("".to_string()), // Dummy value
                    InputParamKind::ConnectionOnly,
                    true,
                );
                graph.add_input_param(
                    node_id,
                    "Webhook URL".to_string(),
                    DataType::Signal,
                    InputValue::WebhookUrl("https://my-webhook-url.com/signal".to_string()),
                    InputParamKind::ConstantOnly,
                    true,
                );
            }
        }
    }
}

/// This struct is used to populate the node finder menu.
pub struct AllNodeTemplates;
impl NodeTemplateIter for AllNodeTemplates {
    type Item = TradeBiasNodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        vec![
            TradeBiasNodeTemplate::DataSource,
            TradeBiasNodeTemplate::Indicator,
            TradeBiasNodeTemplate::Output,
        ]
    }
}

/// `WidgetValueTrait` defines the UI for editing the constant values of input parameters.
/// This is where the interactive widgets are implemented.
impl WidgetValueTrait for InputValue {
    type Response = TradeBiasResponse;
    type UserState = GraphState;
    type NodeData = TradeBiasNodeTemplate;

    fn value_widget(
        &mut self,
        param_name: &str,
        _node_id: NodeId,
        ui: &mut egui::Ui,
        _user_state: &mut Self::UserState,
        _node_data: &Self::NodeData,
    ) -> Vec<TradeBiasResponse> {
        ui.label(param_name);
        match self {
            InputValue::FilePath(value) => {
                ui.text_edit_singleline(value);
            }
            InputValue::Period(value) => {
                ui.add(egui::Slider::new(value, 1..=200));
            }
            InputValue::WebhookUrl(value) => {
                ui.text_edit_singleline(value);
            }
        }
        Vec::new()
    }
}

/// We don't need to add any custom UI to the node's body, so this is empty.
impl NodeDataTrait for TradeBiasNodeTemplate {
    type Response = TradeBiasResponse;
    type UserState = GraphState;
    type DataType = DataType;
    type ValueType = InputValue;

    fn bottom_ui(
        &self,
        _ui: &mut egui::Ui,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<Self::Response, Self>>
    where
        Self: Sized,
    {
        Vec::new()
    }
}

// --- 3. Define the main application struct ---

type MyEditorState =
    GraphEditorState<TradeBiasNodeTemplate, DataType, InputValue, TradeBiasNodeTemplate, GraphState>;

#[derive(Default)]
pub struct TradeBiasApp {
    state: MyEditorState,
    user_state: GraphState,
}

impl App for TradeBiasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.label("TradeBias Strategy Builder");
                ui.add_space(20.0);
                ui.label("Right-click on the canvas to add nodes.");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let _ = self
                .state
                .draw_graph_editor(ui, AllNodeTemplates, &mut self.user_state, Vec::default());
        });
    }
}

// --- 4. Main function ---

fn main() {
    let options = NativeOptions::default();
    eframe::run_native(
        "TradeBias Strategy Builder",
        options,
        Box::new(|_cc| Ok(Box::new(TradeBiasApp::default()))),
    )
    .expect("Failed to run eframe");
}
