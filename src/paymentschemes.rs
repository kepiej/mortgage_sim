use crate::mortgage::Mortgage;
use csv::Writer;
use serde::Serialize;
use std::path::Path;
use std::{f64, fmt, io, str};

#[derive(Debug, PartialEq)]
pub enum PaymentScheme {
    FixedCapital,
    FixedMensualities,
    VariableLinearCapital(f64),
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

#[derive(Serialize)]
pub struct MonthlyPayment {
    month: usize,
    yearly_interest_rate: f64,
    payment: f64,
    capital: f64,
    interest: f64,
    balance: f64,
}

impl MonthlyPayment {
    pub fn new(
        month: usize,
        yearly_interest_rate: f64,
        payment: f64,
        capital: Option<f64>,
        balance: Option<f64>,
    ) -> Self {
        let capital_init: f64 = match capital {
            Some(capital) => capital,
            None => f64::NAN,
        };

        return Self {
            month: month,
            yearly_interest_rate: yearly_interest_rate,
            payment: payment,
            capital: capital_init,
            interest: payment - capital_init,
            balance: match balance {
                Some(balance) => balance,
                None => f64::NAN,
            },
        };
    }
}

pub struct MortgagePayments {
    payments: Vec<MonthlyPayment>,
}

impl MortgagePayments {
    pub fn new(m: Mortgage, p: PaymentScheme) -> Self {
        let payments: Vec<MonthlyPayment> = match p {
            PaymentScheme::FixedCapital => fixed_capital_payments(m),
            PaymentScheme::FixedMensualities => fixed_mensualities(m),
            PaymentScheme::VariableLinearCapital(init_pay) => {
                variable_linear_capital_payments(m, init_pay)
            }
        };

        return Self { payments: payments };
    }

    pub fn payments(&self) -> Vec<f64> {
        return self.payments.iter().map(|x| x.payment).collect();
    }

    pub fn capital_paid(&self) -> Vec<f64> {
        return self.payments.iter().map(|x| x.capital).collect();
    }

    pub fn total_repaid(&self) -> f64 {
        return self.payments().iter().sum();
    }

    pub fn to_csv(&self, filename: &Path) -> io::Result<()> {
        let mut wtr = Writer::from_path(filename).expect("Bestand kon niet gemaakt worden!");

        for row in self.payments.iter() {
            wtr.serialize(row)?
        }

        wtr.flush()
    }
}

fn fixed_capital_payments(mort: Mortgage) -> Vec<MonthlyPayment> {
    let principal: f64 = mort.principal();
    let nperiods: i64 = mort.nperiods();
    let mut payments: Vec<MonthlyPayment> = Vec::new();
    let fixedcap: f64 = principal / nperiods as f64;

    let m_interest_rate: Vec<f64> = mort.monthly_interest_rate();
    let mut saldo: f64;
    let mut mens: f64;
    for period in 0usize..nperiods as usize {
        saldo = principal - period as f64 * fixedcap;
        mens = fixedcap + m_interest_rate[period] * saldo;
        payments.push(MonthlyPayment::new(
            period + 1,
            mort.yearly_interest_rate()[period],
            mens,
            Some(fixedcap),
            Some(saldo),
        ));
    }

    return payments;
}

fn fixed_mensualities(mort: Mortgage) -> Vec<MonthlyPayment> {
    let principal: f64 = mort.principal();
    let nperiods: i64 = mort.nperiods();
    let mut payments: Vec<MonthlyPayment> = Vec::new();
    let m_interest_rate: Vec<f64> = mort.monthly_interest_rate();
    let mut mens: f64;
    let mut principal_portion: f64;
    let mut balance: f64 = principal;
    for period in 0usize..nperiods as usize {
        mens = (principal
            * m_interest_rate[period]
            * (1.0 + m_interest_rate[period]).powf(nperiods as f64))
            / ((1.0 + m_interest_rate[period]).powf(nperiods as f64) - 1.0);
        //TODO: Verify this is correct
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
    let mut xt: Vec<f64> = Vec::new();
    xt.push(initial_payment - (m_interest_rate[0] * principal));
    let mut payments: Vec<MonthlyPayment> = Vec::new();
    let mut mens: f64;
    let mut saldo: f64 = principal;
    for period in 1usize..(nperiods + 1) as usize {
        mens = xt[period - 1] + m_interest_rate[period - 1] * saldo;
        payments.push(MonthlyPayment::new(
            period,
            mort.yearly_interest_rate()[period - 1],
            mens,
            Some(xt[period - 1]),
            Some(saldo),
        ));
        saldo = principal - xt[..period].iter().sum::<f64>();
        xt.push(xt[period - 1] + delta);
    }
    return payments;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mortgage::year_to_monthly_interest;

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
        assert_eq!(
            fixed_capital_payments(Mortgage::new(92000.0, 1, [1.8 / 100.0; 1].to_vec()))[0].payment,
            92000.0 * (1.0 + year_to_monthly_interest(&(1.8 / 100.0)))
        )
    }

    #[test]
    fn test_fixed_mensualities() {
        assert!(
            fixed_mensualities(Mortgage::new(92000.0, 1, [1.8 / 100.0; 1].to_vec()))[0].payment
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
