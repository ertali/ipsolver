use log;
use nalgebra::{DMatrix, DVector};
use yew::prelude::*;

use crate::interior::{
    perform_interior_point_iteration, InteriorPointError, InteriorPointIteration,
    InteriorPointProblem,
};

// Import the two child components
mod input_form;
mod interior_view;

use input_form::{InputForm, InputFormData};
use interior_view::InteriorPointView;

/// The main (only) solver “mode” we support now.
pub struct App {
    // How many variables / constraints the user typed
    problem_size: Option<(usize, usize)>,

    // We'll store the interior‐point “problem” once the user presses “Solve.”
    current_problem: Option<InteriorPointProblem>,

    // We keep track of each iteration's data for display
    interior_iterations: Vec<InteriorPointIteration>,

    // For convenience, store whether user wants to maximize or not
    maximize: bool,

    // Keep track if we've encountered an error or are “done.”
    done: bool,
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
                // Log the raw user input
                log::info!("User pressed 'Solve' with:");
                log::info!("  A = {:?}", a);
                log::info!("  b = {:?}", b);
                log::info!("  c = {:?}", c);
                log::info!("  alpha = {}", alpha);
                log::info!("  initial = {:?}", initial);
                log::info!("  maximize = {}", maximize);

                let sign = if maximize { 1.0 } else { -1.0 };
                let new_c = c.map(|val| val * sign);
                let n = a.ncols();

                let feasible_x = if initial.len() == n {
                    DVector::from_vec(initial.clone())
                } else {
                    DVector::from_element(n, 1.0)
                };

                // Create the interior point problem
                let problem = InteriorPointProblem {
                    a_matrix: a,
                    b_vector: b,
                    c_vector: new_c,
                    x_vector: feasible_x,
                    alpha,
                    constraint_types: vec![],
                };

                self.current_problem = Some(problem);
                self.interior_iterations.clear();
                self.done = false;
                self.maximize = maximize;
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
                            // Log the iteration data so you can see the results
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
                    // “Reset” button
                    <button class="back-button" onclick={link.callback(|_| Msg::Reset)}>
                        { "Reset / Clear" }
                    </button>

                    // The input form:
                    <InputForm
                        on_submit={
                            link.callback(
                                |input: InputFormData| match input {
                                    InputFormData::InteriorPointInput(a, b, c, alpha, initial, maximize) => {
                                        Msg::StartInteriorPoint {
                                            a, b, c, alpha, initial, maximize
                                        }
                                    }
                                    // We removed the other solver variants, so if they appear, ignore them:
                                    _ => Msg::Reset,
                                }
                            )
                        }
                        on_size_change={link.callback(|(vars, cons)| Msg::SetProblemSize(vars, cons))}
                    />

                    // Next iteration
                    <button class="next-step-button" onclick={link.callback(|_| Msg::NextStep)}>
                        { "Next Interior-Point Step" }
                    </button>
                </div>

                // If we have an interior-point problem, show the iteration snapshots
                {
                    if let Some(_prob) = &self.current_problem {
                        // Show all iteration snapshots
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
                            <p>{ "No interior-point problem started yet." }</p>
                        }
                    }
                }
            </div>
        }
    }
}
