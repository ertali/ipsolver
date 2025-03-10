use crate::interior::InteriorPointIteration;
use nalgebra::{DMatrix, DVector};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub iteration: usize,

    #[prop_or_default]
    pub iteration_data: Option<InteriorPointIteration>,
}

pub struct InteriorPointView;

impl Component for InteriorPointView {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let it = props.iteration_data.as_ref();

        let d_matrix = it.map(|iter| &iter.d_matrix);
        let a_tilde = it.map(|iter| &iter.a_tilde_matrix);
        let c_tilde = it.map(|iter| &iter.c_tilde_vector);
        let p_matrix = it.map(|iter| &iter.p_matrix);
        let cp_vector = it.map(|iter| &iter.cp_vector);
        let current_x = it.map(|iter| &iter.current_x);

        html! {
            <div class="interior-point-view">
                <h3>{ format!("Iteration {}", props.iteration) }</h3>

                <div class="matrix-container">
                    <div class="matrix-box">
                        <h4>{"D = diag(x)"}</h4>
                        { Self::render_matrix(d_matrix) }
                    </div>

                    <div class="matrix-box">
                        <h4>{"A~ = A * D"}</h4>
                        { Self::render_matrix(a_tilde) }
                    </div>

                    <div class="matrix-box">
                        <h4>{"c~ = D * c"}</h4>
                        { Self::render_vector(c_tilde) }
                    </div>

                    <div class="matrix-box">
                        <h4>{"P = I - A~^T (A~ A~^T)^{-1} A~"}</h4>
                        { Self::render_matrix(p_matrix) }
                    </div>

                    <div class="matrix-box">
                        <h4>{"P c~"}</h4>
                        { Self::render_vector(cp_vector) }
                    </div>

                    <div class="matrix-box">
                        <h4>{"Current x"}</h4>
                        { Self::render_vector(current_x) }
                    </div>
                </div>
            </div>
        }
    }
}

impl InteriorPointView {
    fn render_matrix(matrix_opt: Option<&DMatrix<f64>>) -> Html {
        if let Some(mat) = matrix_opt {
            let (rows, cols) = mat.shape();
            html! {
                <table class="matrix">
                    <tbody>
                    {
                        for (0..rows).map(|r| html!{
                            <tr>
                            {
                                for (0..cols).map(|c| html! {
                                    <td>{ format!("{:.4}", mat[(r, c)]) }</td>
                                })
                            }
                            </tr>
                        })
                    }
                    </tbody>
                </table>
            }
        } else {
            html! { <p>{"(Not available)"}</p> }
        }
    }

    fn render_vector(vec_opt: Option<&DVector<f64>>) -> Html {
        if let Some(v) = vec_opt {
            html! {
                <table class="vector">
                    <tbody>
                    {
                        for (0..v.len()).map(|i| html!{
                            <tr>
                                <td>{ format!("{:.4}", v[i]) }</td>
                            </tr>
                        })
                    }
                    </tbody>
                </table>
            }
        } else {
            html! { <p>{"(Not available)"}</p> }
        }
    }
}
