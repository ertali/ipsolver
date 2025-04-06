use nalgebra::{DMatrix, DVector};

#[derive(Clone, PartialEq)]
pub struct InteriorPointIteration {
    pub d_matrix: DMatrix<f64>,
    pub a_tilde_matrix: DMatrix<f64>,
    pub c_tilde_vector: DVector<f64>,
    pub p_matrix: DMatrix<f64>,
    pub cp_vector: DVector<f64>,
    pub current_x: DVector<f64>,
}

pub struct InteriorPointProblem {
    pub a_matrix: DMatrix<f64>,
    pub b_vector: DVector<f64>,
    pub c_vector: DVector<f64>,
    pub x_vector: DVector<f64>,
    pub alpha: f64,
    pub constraint_types: Vec<String>,
    pub is_augmented: bool,
}

#[derive(Debug)]
pub enum InteriorPointError {
    NoImprovement,
    NotFeasible,
    SingularMatrix(String),
}

pub fn create_d_matrix(x: &DVector<f64>) -> DMatrix<f64> {
    let n = x.len();
    let mut d = DMatrix::zeros(n, n);
    for i in 0..n {
        d[(i, i)] = x[i].max(1e-8);
    }
    d
}

pub fn calculate_a_tilde(a: &DMatrix<f64>, d: &DMatrix<f64>) -> DMatrix<f64> {
    a * d
}

pub fn calculate_c_tilde(c: &DVector<f64>, d: &DMatrix<f64>) -> DVector<f64> {
    d * c
}

pub fn calculate_p_matrix(a_tilde: &DMatrix<f64>) -> Result<DMatrix<f64>, InteriorPointError> {
    let n = a_tilde.ncols();
    let i_n = DMatrix::identity(n, n);

    let a_tilde_t = a_tilde.transpose();
    let mtx = a_tilde * &a_tilde_t + DMatrix::identity(a_tilde.nrows(), a_tilde.nrows()) * 1e-8;

    let mtx_inv = mtx.try_inverse().ok_or_else(|| {
        InteriorPointError::SingularMatrix("Cannot invert (A_tilde * A_tilde^T)".to_string())
    })?;

    let p = i_n - a_tilde_t * mtx_inv * a_tilde;
    Ok(p)
}

pub fn calculate_cp_vector(p: &DMatrix<f64>, c_tilde: &DVector<f64>) -> DVector<f64> {
    p * c_tilde
}

pub fn perform_interior_point_iteration(
    problem: &mut InteriorPointProblem,
) -> Result<InteriorPointIteration, InteriorPointError> {
    log::info!("Iteration start: x = {:?}", problem.x_vector);

    let d = create_d_matrix(&problem.x_vector);

    let a_tilde = calculate_a_tilde(&problem.a_matrix, &d);
    let c_tilde = calculate_c_tilde(&problem.c_vector, &d);

    let p = calculate_p_matrix(&a_tilde)?;

    let cp = calculate_cp_vector(&p, &c_tilde);

    let mut v = 0.0_f64;
    for &val in cp.iter() {
        if val < 0.0 && val.abs() > v {
            v = val.abs();
        }
    }
    if v < 1e-8 {
        log::warn!("Step size too small or no negative direction: v = {}", v);
        return Err(InteriorPointError::NoImprovement);
    }

    let factor = (problem.alpha / v).min(0.5).max(1e-3);

    let ones = DVector::from_element(problem.x_vector.len(), 1.0);
    let new_x_tilde = &ones + factor * &cp;

    let new_x = (&d * &new_x_tilde).column(0).into_owned();

    problem.x_vector = new_x.clone();

    log::info!("Updated x: {:?}", new_x);

    Ok(InteriorPointIteration {
        d_matrix: d,
        a_tilde_matrix: a_tilde,
        c_tilde_vector: c_tilde,
        p_matrix: p,
        cp_vector: cp,
        current_x: new_x,
    })
}
