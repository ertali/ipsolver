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
                
                // Prepare initial feasible point based on mode
                let initial_point = if self.augmented_model {
                    // Already augmented - use user input as is
                    self.initial_feasible.clone()
                } else {
                    // Auto-augment mode - extend initial point for slack variables
                    let slack_count = self.constraint_signs.iter()
                        .filter(|&sign| sign == "<=" || sign == ">=")
                        .count();
                    
                    let mut extended_initial = self.initial_feasible.clone();
                    // Add positive initial values for slack variables
                    for _ in 0..slack_count {
                        extended_initial.push(1.0);
                    }
                    extended_initial
                };
                
                let data = InputFormData::InteriorPointInput(
                    a,
                    b,
                    c,
                    self.alpha,
                    initial_point,
                    self.maximization,
                    self.augmented_model,
                );
                ctx.props().on_submit.emit(data);
                true
            }
            Msg::SetAugmentedModel(val) => {
                self.augmented_model = val;
                // When switching to augmented mode, set all constraint signs to "="
                if val {
                    for i in 0..self.constraint_signs.len() {
                        self.constraint_signs[i] = "=".to_string();
                    }
                } else {
                    // When switching back to auto-augment, ensure signs are not "="
                    for i in 0..self.constraint_signs.len() {
                        if self.constraint_signs[i] == "=" {
                            self.constraint_signs[i] = "<=".to_string();
                        }
                    }
                }
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
                                                        key={format!("constraint-{}-{}", i, self.augmented_model)}
                                                        value={self.constraint_signs[i].clone()}
                                                        disabled={self.augmented_model}
                                                        oninput={link.callback(move |e: InputEvent| {
                                                            let select: HtmlSelectElement = e.target_unchecked_into();
                                                            Msg::UpdateConstraintSign(i, select.value())
                                                        })}
                                                    >
                                                        <option value="<=">{"<="}</option>
                                                        <option value=">=">{">="}</option>
                                                        <option value="=">{"="}</option>
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
        let default_sign = if self.augmented_model { "=".to_string() } else { "<=".to_string() };
        self.constraint_signs
            .resize(self.constraints, default_sign);
        self.rhs_values.resize(self.constraints, 0.0);

        self.initial_feasible.resize(self.variables, 1.0);
    }

    fn create_matrix_form(&self) -> (DMatrix<f64>, DVector<f64>, DVector<f64>) {
        if self.augmented_model {
            // Already augmented - just create matrices directly
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
            // Auto-augment: convert inequalities to equalities by adding slack variables
            let m = self.constraints;
            
            // Count how many slack variables we need
            let slack_count = self.constraint_signs.iter()
                .filter(|&sign| sign == "<=" || sign == ">=")
                .count();
            
            let n = self.variables + slack_count;
            
            // Build the augmented matrix A and vector b
            let mut a_data = Vec::with_capacity(m * n);
            let mut b_data = Vec::with_capacity(m);
            
            let mut slack_index = 0;
            
            for i in 0..m {
                let sign = &self.constraint_signs[i];
                
                // Determine multiplier for >= constraints
                let multiplier = if sign == ">=" { -1.0 } else { 1.0 };
                
                // Add original variable coefficients
                for j in 0..self.variables {
                    a_data.push(multiplier * self.constraint_coeffs[i][j]);
                }
                
                // Add slack variable coefficients
                for s in 0..slack_count {
                    if (sign == "<=" || sign == ">=") && s == slack_index {
                        a_data.push(1.0);  // This slack variable belongs to this constraint
                    } else {
                        a_data.push(0.0);  // Other slack variables are 0 for this constraint
                    }
                }
                
                // Advance slack index if we used a slack variable
                if sign == "<=" || sign == ">=" {
                    slack_index += 1;
                }
                
                // Add RHS value
                b_data.push(multiplier * self.rhs_values[i]);
            }
            
            let a_matrix = DMatrix::from_row_slice(m, n, &a_data);
            let b_vector = DVector::from_vec(b_data);
            
            // Extend objective function with zeros for slack variables
            let mut c_vec = self.objective_coeffs.clone();
            c_vec.resize(n, 0.0);
            let c_vector = DVector::from_vec(c_vec);
            
            (a_matrix, b_vector, c_vector)
        }
    }
}
