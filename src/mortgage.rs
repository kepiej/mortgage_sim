use std::fmt;

pub struct Mortgage {
    principal: f64,
    nperiods: i64,
    year_interest_rate: Vec<f64>,
}

impl fmt::Display for Mortgage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Mortgage(capital={}, nperiods={})",
            self.principal, self.nperiods
        )
    }
}

impl Mortgage {
    pub fn new(principal: f64, nperiods: i64, year_interest_rate: Vec<f64>) -> Self {
        return Self {
            principal: principal,
            nperiods: nperiods,
            year_interest_rate: year_interest_rate.to_vec(),
        };
    }

    pub fn principal(&self) -> f64 {
        return self.principal;
    }

    pub fn nperiods(&self) -> i64 {
        return self.nperiods;
    }

    pub fn yearly_interest_rate(&self) -> &Vec<f64> {
        return &self.year_interest_rate;
    }

    pub fn monthly_interest_rate(&self) -> Vec<f64> {
        return self
            .year_interest_rate
            .iter()
            .map(|x| year_to_monthly_interest(x))
            .collect();
    }
}

pub fn year_to_monthly_interest(yearly_i: &f64) -> f64 {
    return (1.0 + yearly_i).powf(1.0 / 12.0) - 1.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_year_to_monthly() {
        assert!(year_to_monthly_interest(&(1.8 / 100.0)) - 0.0014877 <= 1e-6);
    }
}
