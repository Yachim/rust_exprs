pub struct Expression<'a> {
    str_expr: &'a str,
}

impl<'a> Expression<'a> {
    pub fn from_str(expr: &'a str) -> Self {
        Self { str_expr: expr }
    }
}

#[cfg(test)]
mod tests {}
