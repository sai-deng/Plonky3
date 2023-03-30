use p3_field::packed::PackedField;

#[derive(Debug, Copy, Clone)]
pub struct StarkEvaluationVars<'a, P: PackedField> {
    pub local_values: &'a [P],
    pub next_values: &'a [P],
}
