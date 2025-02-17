use nalgebra::{DMatrix, DVector};

/// Stores a snapshot of each iteration for the interior point method.
#[derive(Clone, PartialEq)]
pub struct InteriorPointIteration {
    pub d_matrix: DMatrix<f64>,
    pub a_tilde_matrix: DMatrix<f64>,
    pub c_tilde_vector: DVector<f64>,
    pub p_matrix: DMatrix<f64>,
    pub cp_vector: DVector<f64>,
    pub current_x: DVector<f64>,
}

/// The interior point problem (for the simplified method, we no longer
/// store or update x_tilde across iterations).
pub struct InteriorPointProblem {
    /// A matrix (m x n) in standard form (Ax = b).
    pub a_matrix: DMatrix<f64>,
    /// b vector (m).
    pub b_vector: DVector<f64>,
    /// c vector (n).
    pub c_vector: DVector<f64>,
    /// current x (n), must be strictly > 0
    pub x_vector: DVector<f64>,
    /// Step size parameter (0 < alpha < 1).
    pub alpha: f64,
    /// (Unused in the current implementation.)
    pub constraint_types: Vec<String>,
}

#[derive(Debug)]
pub enum InteriorPointError {
    NoImprovement,
    NotFeasible,
    SingularMatrix(String),
}

/// Creates D = diag(x), ensuring each diagonal entry is at least 1e-8.
pub fn create_d_matrix(x: &DVector<f64>) -> DMatrix<f64> {
    let n = x.len();
    let mut d = DMatrix::zeros(n, n);
    for i in 0..n {
        d[(i, i)] = x[i].max(1e-8);
    }
    d
}

/// Computes A_tilde = A * D.
pub fn calculate_a_tilde(a: &DMatrix<f64>, d: &DMatrix<f64>) -> DMatrix<f64> {
    a * d
}

/// Computes c_tilde = D * c.
pub fn calculate_c_tilde(c: &DVector<f64>, d: &DMatrix<f64>) -> DVector<f64> {
    d * c
}

/// Computes P = I - A_tilde^T (A_tilde * A_tilde^T)^(-1) A_tilde.
pub fn calculate_p_matrix(a_tilde: &DMatrix<f64>) -> Result<DMatrix<f64>, InteriorPointError> {
    let n = a_tilde.ncols();
    let i_n = DMatrix::identity(n, n);

    let a_tilde_t = a_tilde.transpose();
    // Slight regularization: + 1e-8 on the diagonal
    let mtx = a_tilde * &a_tilde_t + DMatrix::identity(a_tilde.nrows(), a_tilde.nrows()) * 1e-8;

    let mtx_inv = mtx.try_inverse().ok_or_else(|| {
        InteriorPointError::SingularMatrix("Cannot invert (A_tilde * A_tilde^T)".to_string())
    })?;

    let p = i_n - a_tilde_t * mtx_inv * a_tilde;
    Ok(p)
}

/// Computes cp = P * c_tilde.
pub fn calculate_cp_vector(p: &DMatrix<f64>, c_tilde: &DVector<f64>) -> DVector<f64> {
    p * c_tilde
}

/// Performs one iteration of the *textbook's simplified* interior-point update:
///
///  1) D = diag(x)   (clamp small values to 1e-8)
///  2) A~ = A * D,  c~ = D * c
///  3) P = I - A~^T (A~ A~^T)^{-1} A~
///  4) cp = P * c~
///  5) Identify v = max_{val < 0} |val| in cp
///  6) factor = alpha / v  (clamped to [1e-3, 0.5])
///  7) x_tilde = 1 + factor * cp   <-- note we do NOT keep x_tilde from previous iteration
///  8) x_new = D * x_tilde
///  9) update x in the problem state
pub fn perform_interior_point_iteration(
    problem: &mut InteriorPointProblem,
) -> Result<InteriorPointIteration, InteriorPointError> {
    log::info!("Iteration start: x = {:?}", problem.x_vector);

    // 1) Build D = diag(x).
    let d = create_d_matrix(&problem.x_vector);

    // 2) A_tilde, c_tilde
    let a_tilde = calculate_a_tilde(&problem.a_matrix, &d);
    let c_tilde = calculate_c_tilde(&problem.c_vector, &d);

    // 3) P
    let p = calculate_p_matrix(&a_tilde)?;

    // 4) cp = P c_tilde
    let cp = calculate_cp_vector(&p, &c_tilde);

    // 5) find v = largest absolute value among negative cp components
    let mut v = 0.0_f64;
    for &val in cp.iter() {
        if val < 0.0 && val.abs() > v {
            v = val.abs();
        }
    }
    if v < 1e-8 {
        // No negative component left (or it's too small) => no improvement
        log::warn!("Step size too small or no negative direction: v = {}", v);
        return Err(InteriorPointError::NoImprovement);
    }

    // 6) factor = alpha / v, clamped
    let factor = (problem.alpha / v).min(0.5).max(1e-3);

    // 7) According to the textbook, each iteration re-initializes x_tilde = 1 + (alpha/v)*cp
    let ones = DVector::from_element(problem.x_vector.len(), 1.0);
    let new_x_tilde = &ones + factor * &cp;

    // 8) x_new = D * x_tilde
    let new_x = (&d * &new_x_tilde).column(0).into_owned();

    // 9) Update the problem state
    problem.x_vector = new_x.clone();

    log::info!("Updated x: {:?}", new_x);

    // Return iteration snapshot
    Ok(InteriorPointIteration {
        d_matrix: d,
        a_tilde_matrix: a_tilde,
        c_tilde_vector: c_tilde,
        p_matrix: p,
        cp_vector: cp,
        current_x: new_x,
    })
}
