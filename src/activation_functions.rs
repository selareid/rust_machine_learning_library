pub(crate) struct ActivationFunctions {}

struct DerivativeFunctions {}

impl ActivationFunctions {
    pub fn identity(input: f64) -> f64 {
        input
    }

    //0 if less than 0, 1 if greater or equal to 0
    pub fn binary_step(input: f64) -> f64 {
        if input >= 0.0 {
            return 1.0;
        }

        0.0
    }

    //just sigmoid actually
    pub fn scuffed_sigmoid(input: f64) -> f64 {
        1.0 / (1.0 + (0.0-input).exp())
    }

    pub fn ReLU(input: f64) -> f64 {
        if input > 0.0 {
            return input;
        }

        0.0
    }

    pub fn softplus(input: f64) -> f64 {
        (1.0 + input.exp()).ln()
    }
}