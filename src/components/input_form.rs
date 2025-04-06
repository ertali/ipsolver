use nalgebra::{DMatrix, DVector};
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

#[derive(Clone)]
pub enum InputFormData {
    InteriorPointInput(
        DMatrix<f64>,
        DVector<f64>,
        DVector<f64>,
        f64,
        Vec<f64>,
        bool,
        bool,
    ),
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub on_submit: Callback<InputFormData>,
    pub on_size_change: Callback<(usize, usize)>,
    #[prop_or(10)]
    pub max_variables: usize,
}

pub struct InputForm {
    variables: usize,
    constraints: usize,

    objective_coeffs: Vec<f64>,

    constraint_coeffs: Vec<Vec<f64>>,
    constraint_signs: Vec<String>,
    rhs_values: Vec<f64>,

    maximization: bool,

    alpha: f64,
    initial_feasible: Vec<f64>,

    augmented_model: bool,
}

pub enum Msg {
    SetVariables(usize),
    SetConstraints(usize),
    UpdateObjectiveCoeff(usize, f64),
    UpdateConstraintCoeff(usize, usize, f64),
    UpdateRHSValue(usize, f64),
    ToggleOptimizationType,
    UpdateAlpha(f64),
    UpdateInitialPoint(usize, f64),
    Submit,
    SetAugmentedModel(bool),
    UpdateConstraintSign(usize, String),
}

impl Component for InputForm {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        let variables = 2;
        let constraints = 2;
        Self {
            variables,
            constraints,
            objective_coeffs: vec![0.0; variables],
            constraint_coeffs: vec![vec![0.0; variables]; constraints],
            constraint_signs: vec!["<=".to_string(); constraints],
            rhs_values: vec![0.0; constraints],
            maximization: true,
            alpha: 0.5,
            initial_feasible: vec![1.0; variables],
            augmented_model: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetVariables(v) => {
                let v = v.min(ctx.props().max_variables);
                self.variables = v;
                self.resize();
                ctx.props()
                    .on_size_change
                    .emit((self.variables, self.constraints));
                true
            }
            Msg::SetConstraints(c) => {
                self.constraints = c;
                self.resize();
                ctx.props()
                    .on_size_change
                    .emit((self.variables, self.constraints));
                true
            }
            Msg::UpdateObjectiveCoeff(j, val) => {
                if j < self.objective_coeffs.len() {
                    self.objective_coeffs[j] = val;
                    true
                } else {
                    false
                }
            }
            Msg::UpdateConstraintCoeff(i, j, val) => {
                if i < self.constraint_coeffs.len() && j < self.constraint_coeffs[i].len() {
                    self.constraint_coeffs[i][j] = val;
                    true
                } else {
                    false
                }
            }
            Msg::UpdateRHSValue(i, val) => {
                if i < self.rhs_values.len() {
                    self.rhs_values[i] = val;
                    true
                } else {
                    false
                }
            }
            Msg::ToggleOptimizationType => {
                self.maximization = !self.maximization;
                true
            }
            Msg::UpdateAlpha(a) => {
                self.alpha = a.max(0.0).min(1.0);
                true
            }
            Msg::UpdateInitialPoint(idx, val) => {
                if idx < self.initial_feasible.len() {
                    self.initial_feasible[idx] = val;
                    true
                } else {
                    false
                }
            }
            Msg::Submit => {
                let (a, b, c) = self.create_matrix_form();
                let signs = self.constraint_signs.clone();
                let data = InputFormData::InteriorPointInput(
                    a,
                    b,
                    c,
                    self.alpha,
                    self.initial_feasible.clone(),
                    self.maximization,
                    self.augmented_model,
                );
                ctx.props().on_submit.emit(data);
                true
            }
            Msg::SetAugmentedModel(val) => {
                self.augmented_model = val;
                true
            }
            Msg::UpdateConstraintSign(i, sign) => {
                if i < self.constraint_signs.len() {
                    self.constraint_signs[i] = sign;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        log::info!("Loaded new InputForm!");
        html! {
            <div class="input-form">

            <div class="model-type-selector">
                <label>
                    <input
                        type="radio"
                        name="model_mode"
                        value="augmented"
                        checked={self.augmented_model}
                        oninput={link.callback(|_| Msg::SetAugmentedModel(true))}
                    />
                    { "Already Augmented (A x = b)" }
                </label>

                <label>
                    <input
                        type="radio"
                        name="model_mode"
                        value="autoaugment"
                        checked={!self.augmented_model}
                        oninput={link.callback(|_| Msg::SetAugmentedModel(false))}
                    />
                    { "Auto-Augment (<=, >=, =)" }
                </label>
            </div>

                <div class="optimization-type">
                    <select
                        value={if self.maximization { "max" } else { "min" }}
                        onchange={link.callback(|_e: Event| {
                            Msg::ToggleOptimizationType
                        })}>
                        <option value="min">{"Minimize"}</option>
                        <option value="max">{"Maximize"}</option>
                    </select>
                    <span>{" Z = "}</span>
                </div>

                <div class="size-selectors">
                    <div>
                        <label>{"Variables: "}
                            <input
                                type="number"
                                min="1"
                                max={ctx.props().max_variables.to_string()}
                                value={self.variables.to_string()}
                                oninput={link.callback(|e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    Msg::SetVariables(input.value().parse().unwrap_or(2))
                                })}
                            />
                        </label>
                    </div>
                    <div>
                        <label>{"Constraints: "}
                            <input
                                type="number"
                                min="1"
                                max="10"
                                value={self.constraints.to_string()}
                                oninput={link.callback(|e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    Msg::SetConstraints(input.value().parse().unwrap_or(2))
                                })}
                            />
                        </label>
                    </div>
                </div>

                <div class="objective-function">
                {
                    for (0..self.variables).map(|j| {
                        html! {
                            <span>
                                {if j > 0 { " + " } else { "" }}
                                <input
                                    type="number"
                                    step="0.1"
                                    value={self.objective_coeffs[j].to_string()}
                                    oninput={link.callback(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        Msg::UpdateObjectiveCoeff(
                                            j,
                                            input.value().parse().unwrap_or(0.0)
                                        )
                                    })}
                                />
                                { format!("x{}", j + 1) }
                            </span>
                        }
                    })
                }
                </div>

                <div class="constraints">
                                    {
                                        for (0..self.constraints).map(|i| {
                                            html! {
                                                <div class="constraint-row">
                                                    {
                                                        for (0..self.variables).map(|j| {
                                                            html! {
                                                                <span>
                                                                    { if j > 0 { " + " } else { "" } }
                                                                    <input
                                                                        type="number"
                                                                        step="0.1"
                                                                        value={self.constraint_coeffs[i][j].to_string()}
                                                                        oninput={link.callback(move |e: InputEvent| {
                                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                                            Msg::UpdateConstraintCoeff(i, j, input.value().parse().unwrap_or(0.0))
                                                                        })}
                                                                    />
                                                                    { format!("x{}", j+1) }
                                                                </span>
                                                            }
                                                        })
                                                    }
                                                    // Insert your sign dropdown here:
                                                    <select
                                                        value={self.constraint_signs[i].clone()}
                                                        oninput={link.callback(move |e: InputEvent| {
                                                            let select: HtmlSelectElement = e.target_unchecked_into();
                                                            Msg::UpdateConstraintSign(i, select.value())
                                                        })}
                                                    >
                                                        <option value="<=">{"<="}</option>
                                                        <option value="=">{"="}</option>
                                                        <option value=">=">{">="}</option>
                                                    </select>
                                                    <input
                                                        type="number"
                                                        step="0.1"
                                                        value={self.rhs_values[i].to_string()}
                                                        oninput={link.callback(move |e: InputEvent| {
                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                            Msg::UpdateRHSValue(i, input.value().parse().unwrap_or(0.0))
                                                        })}
                                                    />
                                                </div>
                                            }
                                        })
                                    }
                                </div>

                <div class="alpha-selector">
                    <label>{"Step Size (α): "}
                        <input
                            type="number"
                            min="0"
                            max="1"
                            step="0.1"
                            value={self.alpha.to_string()}
                            oninput={link.callback(move |e: InputEvent| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                Msg::UpdateAlpha(input.value().parse().unwrap_or(0.5))
                            })}
                        />
                    </label>
                </div>

                <div class="initial-point-input">
                    <h4>{"Initial Feasible Point (x > 0)"}</h4>
                    {
                        for (0..self.variables).map(|idx| {
                            html! {
                                <label>
                                    {format!("x{} = ", idx+1)}
                                    <input
                                        type="number"
                                        step="0.1"
                                        value={self.initial_feasible[idx].to_string()}
                                        oninput={link.callback(move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            Msg::UpdateInitialPoint(
                                                idx,
                                                input.value().parse().unwrap_or(1.0)
                                            )
                                        })}
                                    />
                                </label>
                            }
                        })
                    }
                </div>

                <button onclick={link.callback(|_| Msg::Submit)}>
                    {"Solve"}
                </button>
            </div>
        }
    }
}

impl InputForm {
    fn resize(&mut self) {
        self.objective_coeffs.resize(self.variables, 0.0);

        self.constraint_coeffs
            .resize(self.constraints, vec![0.0; self.variables]);
        for row in self.constraint_coeffs.iter_mut() {
            row.resize(self.variables, 0.0);
        }
        self.constraint_signs
            .resize(self.constraints, "<=".to_string());
        self.rhs_values.resize(self.constraints, 0.0);

        self.initial_feasible.resize(self.variables, 1.0);
    }

    fn create_matrix_form(&self) -> (DMatrix<f64>, DVector<f64>, DVector<f64>) {
        if self.augmented_model {
            let m = self.constraints;
            let n = self.variables;

            let mut a_data = Vec::with_capacity(m * n);
            for i in 0..m {
                for j in 0..n {
                    a_data.push(self.constraint_coeffs[i][j]);
                }
            }
            let a_matrix = DMatrix::from_row_slice(m, n, &a_data);

            let b_vector = DVector::from_iterator(m, self.rhs_values.iter().copied());

            let c_vector = DVector::from_vec(self.objective_coeffs.clone());

            (a_matrix, b_vector, c_vector)
        } else {
            let m = self.constraints;
            let mut slack_count = 0;

            let mut big_a_data: Vec<f64> = Vec::new();
            let mut big_b_data: Vec<f64> = Vec::new();

            for i in 0..m {
                let sign = &self.constraint_signs[i];
                let mut multiplier = 1.0;
                if sign == ">=" {
                    multiplier = -1.0;
                }

                let mut row_data = Vec::with_capacity(self.variables);
                for j in 0..self.variables {
                    row_data.push(multiplier * self.constraint_coeffs[i][j]);
                }

                if sign == "<=" || sign == ">=" {
                    row_data.push(1.0);
                    slack_count += 1;
                }
            }

            let mut needed_slacks = 0;
            for i in 0..m {
                let sign = &self.constraint_signs[i];
                if sign == "<=" || sign == ">=" {
                    needed_slacks += 1;
                }
            }

            let mut all_rows: Vec<Vec<f64>> = Vec::with_capacity(m);

            for i in 0..m {
                let sign = &self.constraint_signs[i];
                let mut multiplier = 1.0;
                if sign == ">=" {
                    multiplier = -1.0;
                }

                let mut row_data = Vec::with_capacity(self.variables + needed_slacks);
                for j in 0..self.variables {
                    row_data.push(multiplier * self.constraint_coeffs[i][j]);
                }

                for _ in 0..needed_slacks {
                    row_data.push(0.0);
                }

                if sign == "<=" || sign == ">=" {
                    let slack_index_for_this_row = {
                        let mut count_before = 0;
                        for r in 0..i {
                            let s = &self.constraint_signs[r];
                            if s == "<=" || s == ">=" {
                                count_before += 1;
                            }
                        }
                        count_before
                    };
                    row_data[self.variables + slack_index_for_this_row] = 1.0;
                }

                all_rows.push(row_data);

                let rhs_val = multiplier * self.rhs_values[i];
                big_b_data.push(rhs_val);
            }

            let m = self.constraints;
            let n = self.variables + needed_slacks;
            let mut big_a_data = Vec::with_capacity(m * n);
            for row_vec in all_rows {
                big_a_data.extend_from_slice(&row_vec);
            }

            let a_matrix = DMatrix::from_row_slice(m, n, &big_a_data);
            let b_vector = DVector::from_iterator(m, big_b_data.into_iter());

            let mut c_vec = self.objective_coeffs.clone();
            c_vec.resize(n, 0.0);
            let c_vector = DVector::from_vec(c_vec);

            (a_matrix, b_vector, c_vector)
        }
    }
}
