use log;
use nalgebra::{DMatrix, DVector};
use yew::prelude::*;

use crate::interior::{
    perform_interior_point_iteration, InteriorPointError, InteriorPointIteration,
    InteriorPointProblem,
};

mod input_form;
mod interior_view;

use input_form::{InputForm, InputFormData};
use interior_view::InteriorPointView;

pub struct App {
    problem_size: Option<(usize, usize)>,

    current_problem: Option<InteriorPointProblem>,

    interior_iterations: Vec<InteriorPointIteration>,

    maximize: bool,

    done: bool,

    error_message: Option<String>,
}

pub enum Msg {
    SetProblemSize(usize, usize),
    StartInteriorPoint {
        a: DMatrix<f64>,
        b: DVector<f64>,
        c: DVector<f64>,
        alpha: f64,
        initial: Vec<f64>,
        maximize: bool,
    },
    NextStep,
    Reset,
    SetInitialPoint(DVector<f64>),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            problem_size: None,
            current_problem: None,
            interior_iterations: vec![],
            maximize: true, // default
            done: false,
            error_message: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetProblemSize(vars, cons) => {
                log::info!(
                    "User changed problem size: {} variables, {} constraints",
                    vars,
                    cons
                );
                self.problem_size = Some((vars, cons));
                true
            }
            Msg::StartInteriorPoint {
                a,
                b,
                c,
                alpha,
                initial,
                maximize,
            } => {
                let final_n = a.ncols();

                let feasible_x = if initial.len() == final_n {
                    DVector::from_vec(initial.clone())
                } else {
                    let mut new_init = vec![1.0; final_n];
                    for (i, val) in initial.iter().enumerate() {
                        if i < final_n {
                            new_init[i] = val.max(1e-4);
                        }
                    }
                    DVector::from_vec(new_init)
                };

                let sign = if maximize { 1.0 } else { -1.0 };
                let new_c = c.map(|val| val * sign);

                let problem = InteriorPointProblem {
                    a_matrix: a,
                    b_vector: b,
                    c_vector: new_c,
                    x_vector: feasible_x,
                    alpha,
                    constraint_types: vec![],
                    is_augmented: false,
                };

                self.current_problem = Some(problem);
                self.interior_iterations.clear();
                self.done = false;
                self.maximize = maximize;
                self.error_message = None; // Clear any previous errors

                // Automatically perform the first iteration (Iteration 0)
                if let Some(problem) = &mut self.current_problem {
                    match perform_interior_point_iteration(problem) {
                        Ok(iter_data) => {
                            self.interior_iterations.push(iter_data);
                        }
                        Err(InteriorPointError::NoImprovement) => {
                            self.done = true;
                            self.error_message = Some("The algorithm converged immediately or found no improvement direction. This might indicate the initial point is already optimal, or the problem constraints are inconsistent.".to_string());
                        }
                        Err(InteriorPointError::NotFeasible) => {
                            self.done = true;
                            self.error_message = Some("The problem appears to be infeasible. Please check your constraints and initial point to ensure they form a valid feasible region.".to_string());
                        }
                        Err(InteriorPointError::SingularMatrix(msg)) => {
                            self.done = true;
                            self.error_message = Some(format!("Mathematical error: {}. This usually means the constraint matrix is ill-conditioned or the problem is degenerate. Try adjusting your constraints or initial point.", msg));
                        }
                    }
                }

                true
            }
            Msg::NextStep => {
                if let Some(problem) = &mut self.current_problem {
                    if self.done {
                        log::info!(
                            "User clicked NextStep but solver is marked done (no improvement)."
                        );
                        return false;
                    }

                    log::info!(
                        "Performing next step with current x = {:?}",
                        problem.x_vector
                    );

                    match perform_interior_point_iteration(problem) {
                        Ok(iter_data) => {
                            log::info!(
                                "Iteration snapshot => D = diag(x) =>\n{:?}",
                                iter_data.d_matrix
                            );
                            log::info!("A~ =>\n{:?}", iter_data.a_tilde_matrix);
                            log::info!("c~ => {:?}", iter_data.c_tilde_vector);
                            log::info!("P =>\n{:?}", iter_data.p_matrix);
                            log::info!("P c~ => {:?}", iter_data.cp_vector);
                            log::info!("Updated x => {:?}", iter_data.current_x);

                            self.interior_iterations.push(iter_data);
                            true
                        }
                        Err(InteriorPointError::NoImprovement) => {
                            log::info!("No improvement => probably at optimum.");
                            self.done = true;
                            true
                        }
                        Err(e) => {
                            log::error!("Interior point iteration error: {:?}", e);
                            self.done = true;
                            true
                        }
                    }
                } else {
                    false
                }
            }
            Msg::Reset => {
                log::info!("User clicked Reset.");
                self.problem_size = None;
                self.current_problem = None;
                self.interior_iterations.clear();
                self.done = false;
                self.error_message = None;
                true
            }
            Msg::SetInitialPoint(x) => {
                log::info!("User set initial x manually to {:?}", x);
                if let Some(prob) = &mut self.current_problem {
                    prob.x_vector = x;
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        html! {
            <div class="app-container">
                <h1>{ "Interior-Point Solver" }</h1>

                <div>
                    <button class="back-button" onclick={link.callback(|_| Msg::Reset)}>
                        { "Reset / Clear" }
                    </button>

                    <InputForm
                        on_submit={
                            link.callback(
                                |input: InputFormData| match input {
                                    InputFormData::InteriorPointInput(a, b, c, alpha, initial, maximize, is_augmented) => {
                                        Msg::StartInteriorPoint {
                                            a, b, c, alpha, initial, maximize
                                        }
                                    }
                                    _ => Msg::Reset,
                                }
                            )
                        }
                        on_size_change={link.callback(|(vars, cons)| Msg::SetProblemSize(vars, cons))}
                    />

                    <button class="next-step-button" onclick={link.callback(|_| Msg::NextStep)}>
                        { "Next Interior-Point Step" }
                    </button>
                </div>

                {
                    if let Some(error) = &self.error_message {
                        html! {
                            <div class="error-message">
                                <div class="error-icon">{ "⚠️" }</div>
                                <h3>{ "Problem Detected" }</h3>
                                <p>{ error }</p>
                                <div class="error-actions">
                                    <p><strong>{ "What to try:" }</strong></p>
                                    <ul>
                                        <li>{ "Check that your constraints are consistent and don't contradict each other" }</li>
                                        <li>{ "Ensure your initial point satisfies all constraints and is positive" }</li>
                                        <li>{ "Verify your constraint matrix is well-formed" }</li>
                                        <li>{ "Try different initial values or adjust the step size (α)" }</li>
                                    </ul>
                                    <button onclick={link.callback(|_| Msg::Reset)}>
                                        { "← Go Back and Try Again" }
                                    </button>
                                </div>
                            </div>
                        }
                    } else if let Some(_prob) = &self.current_problem {
                        html! {
                            <div class="iterations">
                                {
                                    for self.interior_iterations.iter().enumerate().map(|(i, iteration_data)| {
                                                    html! {
                                                        <InteriorPointView
                                                            iteration={i}
                                                            iteration_data={Some(iteration_data.clone())}
                                                        />
                                                    }
                                                })
                                }
                            </div>
                        }
                    } else {
                        html! {
                            <div class="no-problem-message">
                                <div class="message-icon">{ "📊" }</div>
                                <h3>{ "Ready to Solve" }</h3>
                                <p>{ "Configure your linear programming problem above and press \"Solve\" to begin the interior-point algorithm visualization." }</p>
                            </div>
                        }
                    }
                }
            </div>
        }
    }
}
