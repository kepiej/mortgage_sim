use std::fmt;

pub struct Mortgage {
    pub capital: f64,
    pub nperiods: i64,
    pub year_interest_rate: Vec<f64>,
}

impl fmt::Display for Mortgage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Mortgage(capital={}, nperiods={})",
            self.capital, self.nperiods
        )
    }
}

impl Mortgage {
    pub fn new(capital: f64, nperiods: i64, year_interest_rate: Vec<f64>) -> Self {
        return Self {
            capital: capital,
            nperiods: nperiods,
            year_interest_rate: year_interest_rate.to_vec(),
        };
    }

    pub fn nperiods(&self) -> i64 {
        return self.nperiods;
    }

    pub fn monthly_interest_rate(&self) -> Vec<f64> {
        return self
            .year_interest_rate
            .iter()
            .map(|x| year_to_monthly_interest(x))
            .collect();
    }
}

fn year_to_monthly_interest(yearly_i: &f64) -> f64 {
    return (1.0 + yearly_i).powf(1.0 / 12.0) - 1.0;
}

pub fn month_interest_rate(year_interest_rate: &[f64]) -> Vec<f64> {
    return year_interest_rate
        .iter()
        .map(|x| year_to_monthly_interest(x))
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_year_to_monthly() {
        assert!(year_to_monthly_interest(&(1.8 / 100.0)) - 0.0014877 <= 1e-6);
    }

    #[test]
    fn test_period_interest_rate() {
        assert_eq!(
            month_interest_rate(&[1.8 / 100.0; 12 * 20].to_vec()),
            [year_to_monthly_interest(&(1.8 / 100.0)); 12 * 20].to_vec()
        );
    }
}
