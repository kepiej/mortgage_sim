use crate::mortgage::Mortgage;
use crate::mortgagepayments::MonthlyPayment;

use std::{f64, fmt, str};

#[derive(Debug, PartialEq, Clone)]
pub enum PaymentScheme {
    FixedCapital,
    FixedMensualities,
    VariableLinearCapital(f64),
}

impl PaymentScheme {
    pub fn monthly_payments(&self, m: Mortgage) -> Vec<MonthlyPayment> {
        let payments: Vec<MonthlyPayment> = match self {
            PaymentScheme::FixedCapital => fixed_capital_payments(m),
            PaymentScheme::FixedMensualities => fixed_mensualities(m),
            PaymentScheme::VariableLinearCapital(init_pay) => {
                variable_linear_capital_payments(m, *init_pay)
            }
        };
        return payments;
    }
}

impl str::FromStr for PaymentScheme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        match parts.as_slice() {
            ["FixedCapital"] | ["VasteKapitaalaflossing"] => Ok(PaymentScheme::FixedCapital),
            ["FixedMensualities"] | ["VasteMensualiteiten"] => Ok(PaymentScheme::FixedMensualities),
            ["VariableLinearCapital", init_pay]
            | ["VariabeleLineaireKapitaalaflossing", init_pay] => {
                match init_pay.trim().parse::<f64>() {
                    Ok(value) => Ok(PaymentScheme::VariableLinearCapital(value)),
                    Err(_) => Err(format!("Invalid initial payment: '{}'", init_pay)),
                }
            }
            _ => Err("Invalid input".to_string()),
        }
    }
}

impl fmt::Display for PaymentScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FixedCapital => write!(f, "{}", "FixedCapital"),
            Self::FixedMensualities => write!(f, "{}", "FixedMensualities"),
            Self::VariableLinearCapital(init_pay) => {
                write!(f, "VariableLinearCapital {}", init_pay)
            }
        }
    }
}

fn fixed_capital_payments(mort: Mortgage) -> Vec<MonthlyPayment> {
    let principal: f64 = mort.principal();
    let nperiods: i64 = mort.nperiods();
    let m_interest_rate: Vec<f64> = mort.monthly_interest_rate();
    let mut payments: Vec<MonthlyPayment> = Vec::new();
    let principal_portion: f64 = principal / nperiods as f64;
    let mut balance: f64 = principal;
    let mut mens: f64;
    for period in 0usize..nperiods as usize {
        mens = principal_portion + m_interest_rate[period] * balance;
        payments.push(MonthlyPayment::new(
            period + 1,
            mort.yearly_interest_rate()[period],
            mens,
            Some(principal_portion),
            Some(balance),
        ));
        balance -= principal_portion;
    }

    return payments;
}

fn fixed_mensualities(mort: Mortgage) -> Vec<MonthlyPayment> {
    let principal: f64 = mort.principal();
    let nperiods: i64 = mort.nperiods();
    let m_interest_rate: Vec<f64> = mort.monthly_interest_rate();
    let mut payments: Vec<MonthlyPayment> = Vec::new();
    let mut mens: f64;
    let mut principal_portion: f64;
    let mut balance: f64 = principal;
    for period in 0usize..nperiods as usize {
        mens = (principal
            * m_interest_rate[period]
            * (1.0 + m_interest_rate[period]).powf(nperiods as f64))
            / ((1.0 + m_interest_rate[period]).powf(nperiods as f64) - 1.0);
        principal_portion = mens - (balance * m_interest_rate[period]);
        payments.push(MonthlyPayment::new(
            period + 1,
            mort.yearly_interest_rate()[period],
            mens,
            Some(principal_portion),
            Some(balance),
        ));
        balance -= principal_portion;
    }
    return payments;
}

fn variable_linear_capital_payments(mort: Mortgage, initial_payment: f64) -> Vec<MonthlyPayment> {
    let m_interest_rate: Vec<f64> = mort.monthly_interest_rate();
    let principal: f64 = mort.principal();
    let nperiods: i64 = mort.nperiods();
    let delta: f64 = 2.0
        * (principal - (nperiods as f64) * (initial_payment - (m_interest_rate[0] * principal)))
        / ((nperiods - 1) * nperiods) as f64;
    println!("{delta}");
    let mut payments: Vec<MonthlyPayment> = Vec::new();
    let mut principal_portion: f64 = initial_payment - (m_interest_rate[0] * principal);
    let mut mens: f64;
    let mut balance: f64 = principal;
    for period in 0usize..nperiods as usize {
        mens = principal_portion + m_interest_rate[period] * balance;
        payments.push(MonthlyPayment::new(
            period + 1,
            mort.yearly_interest_rate()[period],
            mens,
            Some(principal_portion),
            Some(balance),
        ));
        balance -= principal_portion;
        principal_portion += delta;
    }
    return payments;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mortgage::year_to_monthly_interest;
    use crate::mortgagepayments::MortgagePayments;

    #[test]
    fn test_parse_paymentscheme() {
        let mut pay: PaymentScheme = "FixedCapital".parse::<PaymentScheme>().unwrap();
        assert_eq!(PaymentScheme::FixedCapital, pay);

        pay = "FixedMensualities".parse::<PaymentScheme>().unwrap();
        assert_eq!(PaymentScheme::FixedMensualities, pay);

        pay = "VariableLinearCapital 500.0"
            .parse::<PaymentScheme>()
            .unwrap();
        assert_eq!(PaymentScheme::VariableLinearCapital(500.0), pay);
    }

    #[test]
    fn test_fixed_capital_payments() {
        let m: Mortgage = Mortgage::new(92000.0, 1, [1.8 / 100.0; 1].to_vec());
        assert_eq!(
            PaymentScheme::FixedCapital
                .monthly_payments(m)
                .total_repaid(),
            92000.0 * (1.0 + year_to_monthly_interest(&(1.8 / 100.0)))
        )
    }

    #[test]
    fn test_fixed_mensualities() {
        let m: Mortgage = Mortgage::new(92000.0, 1, [1.8 / 100.0; 1].to_vec());
        assert!(
            PaymentScheme::FixedMensualities
                .monthly_payments(m)
                .total_repaid()
                - 92000.0 * (1.0 + year_to_monthly_interest(&(1.8 / 100.0)))
                <= 1e-6
        )
    }

    #[test]
    fn test_variable_linear_capital_payments() {
        //TODO
        assert_eq!(true, false);
    }
}
